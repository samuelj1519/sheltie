use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use rustix::fs::{
    AtFlags, Dir, FileType, Mode, OFlags, RenameFlags, fchmod, fstat, fsync, mkdirat, openat,
    renameat_with, statat,
};
use sha2::{Digest, Sha256};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::ids::WorkId;
use sheltie_core::path::RelPath;

use crate::model::{
    Artifact, CopiedFile, MAX_FILE_BYTES, MAX_TOTAL_BYTES, Manifest, SelectedResult,
};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Identity {
    device: u64,
    inode: u64,
}

fn identity(file: &File, path: &Path) -> Result<Identity> {
    let stat = fstat(file).map_err(|error| io_error(path, "fstat", error.into()))?;
    Ok(Identity {
        device: stat.st_dev as u64,
        inode: stat.st_ino as u64,
    })
}

fn io_error(path: &Path, operation: &'static str, source: std::io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        operation,
        source,
    }
}

fn integrity(path: &Path, message: impl Into<String>) -> Error {
    Error::Integrity {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

fn reject(message: impl Into<String>) -> Error {
    Error::Rejected {
        code: "INVALID_TARGET",
        message: message.into(),
    }
}

#[derive(Debug)]
struct Directory {
    file: File,
    path: PathBuf,
    ancestors: Vec<Identity>,
}

impl Directory {
    fn open(path: &Path) -> Result<Self> {
        let text = path
            .to_str()
            .ok_or_else(|| reject("目标与管理根路径必须能用UTF-8表示"))?;
        if !path.is_absolute()
            || text.as_bytes().contains(&0)
            || text.split('/').any(|part| part == "." || part == "..")
        {
            return Err(reject("需要真实无链接的绝对目录路径"));
        }
        let mut file = File::open("/").map_err(|error| io_error(path, "open_root", error))?;
        let mut ancestors = vec![identity(&file, Path::new("/"))?];
        let mut current = PathBuf::from("/");
        for part in text.split('/').filter(|part| !part.is_empty()) {
            current.push(part);
            file = open_directory(&file, part, &current)?;
            ancestors.push(identity(&file, &current)?);
        }
        Ok(Self {
            file,
            path: path.to_path_buf(),
            ancestors,
        })
    }

    fn verify(&self) -> Result<()> {
        let current = Self::open(&self.path)?;
        if current.ancestors != self.ancestors
            || identity(&self.file, &self.path)? != identity(&current.file, &self.path)?
        {
            return Err(integrity(&self.path, "目录路径或祖先对象已改变"));
        }
        Ok(())
    }

    fn duplicate(&self) -> Result<Self> {
        Ok(Self {
            file: self
                .file
                .try_clone()
                .map_err(|error| io_error(&self.path, "clone_directory", error))?,
            path: self.path.clone(),
            ancestors: self.ancestors.clone(),
        })
    }
}

fn open_directory(parent: &File, name: &str, path: &Path) -> Result<File> {
    let before = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| io_error(path, "stat_directory", error.into()))?;
    if FileType::from_raw_mode(before.st_mode) != FileType::Directory {
        return Err(reject(format!("{path:?} 不是无链接普通目录")));
    }
    let fd = openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| io_error(path, "open_directory", error.into()))?;
    let file = File::from(fd);
    let after = identity(&file, path)?;
    if after
        != (Identity {
            device: before.st_dev as u64,
            inode: before.st_ino as u64,
        })
    {
        return Err(integrity(path, "目录在打开时被替换"));
    }
    verify_entry(parent, name, &file, path, true)?;
    Ok(file)
}

fn new_directory(parent: &File, name: &str, path: &Path) -> Result<File> {
    mkdirat(parent, name, Mode::from_raw_mode(0o700))
        .map_err(|error| io_error(path, "mkdir_exclusive", error.into()))?;
    let file = open_directory(parent, name, path)?;
    fchmod(&file, Mode::from_raw_mode(0o700))
        .map_err(|error| io_error(path, "chmod_directory", error.into()))?;
    verify_entry(parent, name, &file, path, true)?;
    Ok(file)
}

fn verify_entry(
    parent: &File,
    name: &str,
    held: &File,
    path: &Path,
    directory: bool,
) -> Result<()> {
    let entry = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| io_error(path, "stat_entry", error.into()))?;
    let actual = fstat(held).map_err(|error| io_error(path, "stat_handle", error.into()))?;
    let expected_type = if directory {
        FileType::Directory
    } else {
        FileType::RegularFile
    };
    if FileType::from_raw_mode(entry.st_mode) != expected_type
        || FileType::from_raw_mode(actual.st_mode) != expected_type
        || entry.st_dev != actual.st_dev
        || entry.st_ino != actual.st_ino
        || (!directory && actual.st_nlink != 1)
    {
        return Err(integrity(path, "路径、类型或链接数不再对应持有对象"));
    }
    Ok(())
}

fn inventory(directory: &File, path: &Path, expected: BTreeSet<String>) -> Result<()> {
    let entries = Dir::read_from(directory)
        .map_err(|error| io_error(path, "read_directory", error.into()))?;
    let mut found = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| io_error(path, "read_entry", error.into()))?;
        let bytes = entry.file_name().to_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        let name =
            std::str::from_utf8(bytes).map_err(|_| integrity(path, "出现不明非UTF-8目录项"))?;
        found.insert(name.to_string());
    }
    if found != expected {
        return Err(integrity(path, "目录项与本次创建清单不一致"));
    }
    Ok(())
}

fn verify_private_mode(file: &File, path: &Path, expected: u32) -> Result<()> {
    let stat = fstat(file).map_err(|error| io_error(path, "stat_permissions", error.into()))?;
    if stat.st_mode as u32 & 0o777 != expected {
        return Err(integrity(path, "自有对象权限已改变"));
    }
    Ok(())
}

fn leaf(artifact: &Artifact) -> Result<String> {
    let name = artifact
        .path
        .as_path()
        .file_name()
        .ok_or_else(|| reject("Artifact路径没有安全叶名"))?;
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.as_bytes().contains(&0)
        || name.contains('/')
        || name.contains('\\')
    {
        return Err(reject("Artifact叶名不是单个安全文件段"));
    }
    Ok(name.to_string())
}

#[derive(Debug)]
pub struct Destination {
    home: Directory,
    parent: Directory,
}

impl Destination {
    pub fn open(home: &Path, parent: &Path) -> Result<Self> {
        let home = Directory::open(home)?;
        let parent = Directory::open(parent)?;
        let home_id = identity(&home.file, &home.path)?;
        let parent_id = identity(&parent.file, &parent.path)?;
        if home.ancestors.contains(&parent_id) || parent.ancestors.contains(&home_id) {
            return Err(reject("目标父目录与管理根对象或祖先重叠"));
        }
        Ok(Self { home, parent })
    }

    fn verify(&self) -> Result<()> {
        self.home.verify()?;
        self.parent.verify()
    }

    pub fn stage(&self, work: &WorkId) -> Result<Staging> {
        self.verify()?;
        let random = uuid::Uuid::now_v7();
        let name = format!(".sheltie-export-{random}");
        let final_name = format!("{work}-{random}");
        let path = self.parent.path.join(&name);
        let root = new_directory(&self.parent.file, &name, &path)?;
        let artifacts = new_directory(&root, "artifacts", &path.join("artifacts"))?;
        Ok(Staging {
            destination: Self {
                home: self.home.duplicate()?,
                parent: self.parent.duplicate()?,
            },
            work: work.clone(),
            name,
            final_name,
            root,
            artifacts,
            entries: Vec::new(),
            manifest: None,
            moved: false,
        })
    }
}

#[derive(Debug)]
struct Entry {
    file: File,
    directory: File,
    directory_name: String,
    name: String,
    relative: RelPath,
    artifact: Artifact,
    finished: bool,
}

#[derive(Debug)]
pub struct Staging {
    destination: Destination,
    work: WorkId,
    name: String,
    final_name: String,
    root: File,
    artifacts: File,
    entries: Vec<Entry>,
    manifest: Option<(File, Vec<u8>)>,
    moved: bool,
}

#[derive(Debug)]
pub struct ArtifactWriter {
    file: File,
    staging: Identity,
    index: usize,
    remaining: u64,
    witness: WriterWitness,
}

#[derive(Debug)]
struct WriterWitness {
    destination: Destination,
    root: File,
    artifacts: File,
    directory: File,
    staging_name: String,
    directory_name: String,
    name: String,
}

impl WriterWitness {
    fn verify(&self, file: &File) -> Result<()> {
        self.destination.verify()?;
        let path = self.destination.parent.path.join(&self.staging_name);
        verify_entry(
            &self.destination.parent.file,
            &self.staging_name,
            &self.root,
            &path,
            true,
        )?;
        verify_entry(
            &self.root,
            "artifacts",
            &self.artifacts,
            &path.join("artifacts"),
            true,
        )?;
        let directory = path.join("artifacts").join(&self.directory_name);
        verify_private_mode(&self.root, &path, 0o700)?;
        verify_private_mode(&self.artifacts, &path.join("artifacts"), 0o700)?;
        verify_private_mode(&self.directory, &directory, 0o700)?;
        verify_private_mode(file, &directory.join(&self.name), 0o600)?;
        verify_entry(
            &self.artifacts,
            &self.directory_name,
            &self.directory,
            &directory,
            true,
        )?;
        verify_entry(
            &self.directory,
            &self.name,
            file,
            &directory.join(&self.name),
            false,
        )
    }
}

impl Write for ArtifactWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() as u64 > self.remaining {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Artifact超过声明字节数",
            ));
        }
        self.witness
            .verify(&self.file)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let written = self.file.write(bytes)?;
        self.remaining -= written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Staging {
    fn path(&self) -> PathBuf {
        self.destination.parent.path.join(&self.name)
    }

    pub fn staging_path(&self) -> Option<PathBuf> {
        if self.moved || self.destination.parent.verify().is_err() {
            return None;
        }
        let path = self.path();
        verify_entry(
            &self.destination.parent.file,
            &self.name,
            &self.root,
            &path,
            true,
        )
        .ok()?;
        Some(path)
    }

    fn verify(&self) -> Result<()> {
        if self.moved {
            return Err(reject("本次暂存已移动，不能重新发布或接管"));
        }
        self.destination.verify()?;
        verify_entry(
            &self.destination.parent.file,
            &self.name,
            &self.root,
            &self.path(),
            true,
        )?;
        verify_entry(
            &self.root,
            "artifacts",
            &self.artifacts,
            &self.path().join("artifacts"),
            true,
        )?;
        verify_private_mode(&self.root, &self.path(), 0o700)?;
        verify_private_mode(&self.artifacts, &self.path().join("artifacts"), 0o700)?;
        Ok(())
    }

    pub fn create_artifact(&mut self, index: usize, artifact: &Artifact) -> Result<ArtifactWriter> {
        let name = leaf(artifact)?;
        if artifact.bytes > MAX_FILE_BYTES
            || index != self.entries.len()
            || self
                .entries
                .iter()
                .any(|entry| entry.artifact.key == artifact.key)
        {
            return Err(reject("Artifact大小、创建索引或key不合规"));
        }
        let total = self
            .entries
            .iter()
            .try_fold(artifact.bytes, |total, entry| {
                total.checked_add(entry.artifact.bytes)
            });
        if total.is_none_or(|total| total > MAX_TOTAL_BYTES) {
            return Err(reject("Artifact总量超过256MiB或溢出"));
        }
        self.verify()?;
        let ordinal = index
            .checked_add(1)
            .ok_or_else(|| reject("Artifact索引溢出"))?;
        let directory_name = format!("{ordinal:04}");
        let directory_path = self.path().join("artifacts").join(&directory_name);
        let directory = new_directory(&self.artifacts, &directory_name, &directory_path)?;
        let path = directory_path.join(&name);
        let fd = openat(
            &directory,
            name.as_str(),
            OFlags::CREATE | OFlags::EXCL | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|error| io_error(&path, "create_file_exclusive", error.into()))?;
        let file = File::from(fd);
        verify_entry(&directory, &name, &file, &path, false)?;
        fchmod(&file, Mode::from_raw_mode(0o600))
            .map_err(|error| io_error(&path, "chmod_file", error.into()))?;
        let writer = ArtifactWriter {
            file: file
                .try_clone()
                .map_err(|error| io_error(&path, "clone_file", error))?,
            staging: identity(&self.root, &self.path())?,
            index,
            remaining: artifact.bytes,
            witness: WriterWitness {
                destination: Destination {
                    home: self.destination.home.duplicate()?,
                    parent: self.destination.parent.duplicate()?,
                },
                root: self
                    .root
                    .try_clone()
                    .map_err(|error| io_error(&self.path(), "clone_staging", error))?,
                artifacts: self
                    .artifacts
                    .try_clone()
                    .map_err(|error| io_error(&self.path(), "clone_artifacts", error))?,
                directory: directory
                    .try_clone()
                    .map_err(|error| io_error(&directory_path, "clone_index", error))?,
                staging_name: self.name.clone(),
                directory_name: directory_name.clone(),
                name: name.clone(),
            },
        };
        let relative = RelPath::new(format!("artifacts/{directory_name}/{name}"))
            .map_err(|error| reject(error.to_string()))?;
        self.entries.push(Entry {
            file,
            directory,
            directory_name,
            name,
            relative,
            artifact: artifact.clone(),
            finished: false,
        });
        Ok(writer)
    }

    pub fn finish_artifact(
        &mut self,
        mut writer: ArtifactWriter,
        artifact: &Artifact,
    ) -> Result<CopiedFile> {
        self.verify()?;
        if writer.staging != identity(&self.root, &self.path())? {
            return Err(reject("文件句柄不属于本次暂存"));
        }
        let entry = self
            .entries
            .get(writer.index)
            .ok_or_else(|| reject("未知文件句柄索引"))?;
        if entry.finished || entry.artifact != *artifact {
            return Err(reject("文件来源或完成资格与本次声明不符"));
        }
        writer.flush().map_err(|error| {
            io_error(
                &self.path().join(entry.relative.as_str()),
                "flush_file",
                error,
            )
        })?;
        self.verify_artifact(entry)?;
        let path = self.path().join(entry.relative.as_str());
        sync(&entry.file, &path)?;
        verify_entry(&entry.directory, &entry.name, &entry.file, &path, false)?;
        checkpoint(TargetPoint::AfterFileSync, &path)?;
        self.verify()?;
        let copied = copied(entry);
        self.entries[writer.index].finished = true;
        Ok(copied)
    }

    fn verify_artifact(&self, entry: &Entry) -> Result<()> {
        let path = self.path().join(entry.relative.as_str());
        verify_entry(
            &self.artifacts,
            &entry.directory_name,
            &entry.directory,
            path.parent().unwrap_or(&path),
            true,
        )?;
        verify_entry(&entry.directory, &entry.name, &entry.file, &path, false)?;
        verify_private_mode(&entry.directory, path.parent().unwrap_or(&path), 0o700)?;
        verify_private_mode(&entry.file, &path, 0o600)?;
        inventory(
            &entry.directory,
            path.parent().unwrap_or(&path),
            BTreeSet::from([entry.name.clone()]),
        )?;
        read_back(
            &entry.file,
            &path,
            &entry.artifact.sha256,
            entry.artifact.bytes,
        )?;
        verify_entry(&entry.directory, &entry.name, &entry.file, &path, false)
    }

    fn verify_contents(&self) -> Result<()> {
        self.verify()?;
        let mut root_names = BTreeSet::from(["artifacts".to_string()]);
        if self.manifest.is_some() {
            root_names.insert("manifest.json".to_string());
        }
        inventory(&self.root, &self.path(), root_names)?;
        inventory(
            &self.artifacts,
            &self.path().join("artifacts"),
            self.entries
                .iter()
                .map(|entry| entry.directory_name.clone())
                .collect(),
        )?;
        for entry in &self.entries {
            self.verify_artifact(entry)?;
        }
        if let Some((file, bytes)) = &self.manifest {
            let path = self.path().join("manifest.json");
            verify_entry(&self.root, "manifest.json", file, &path, false)?;
            verify_private_mode(file, &path, 0o600)?;
            read_back(file, &path, &Sha256Hex::of_bytes(bytes), bytes.len() as u64)?;
        }
        Ok(())
    }

    pub fn publish(&mut self, result: &SelectedResult, files: &[CopiedFile]) -> Result<PathBuf> {
        self.verify_contents()?;
        checkpoint(TargetPoint::AfterReadback, &self.path())?;
        result.validate(&self.work)?;
        if files.len() != result.artifacts.len() || files.len() != self.entries.len() {
            return Err(reject("副本集合与明确结果集合不一致"));
        }
        for ((entry, expected), supplied) in self.entries.iter().zip(&result.artifacts).zip(files) {
            if !entry.finished || entry.artifact != *expected || copied(entry) != *supplied {
                return Err(reject("副本映射、顺序或完成事实不符"));
            }
        }
        let manifest = Manifest {
            format: "work-export-manifest/v1".to_string(),
            result: result.clone(),
            files: files.to_vec(),
        };
        let bytes = serde_json::to_vec(&manifest).map_err(|error| reject(error.to_string()))?;
        let manifest_path = self.path().join("manifest.json");
        self.verify()?;
        let fd = openat(
            &self.root,
            "manifest.json",
            OFlags::CREATE | OFlags::EXCL | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|error| io_error(&manifest_path, "create_manifest_exclusive", error.into()))?;
        let mut file = File::from(fd);
        verify_entry(&self.root, "manifest.json", &file, &manifest_path, false)?;
        fchmod(&file, Mode::from_raw_mode(0o600))
            .map_err(|error| io_error(&manifest_path, "chmod_manifest", error.into()))?;
        file.write_all(&bytes)
            .map_err(|error| io_error(&manifest_path, "write_manifest", error))?;
        self.manifest = Some((file, bytes));
        checkpoint(TargetPoint::AfterManifestWrite, &manifest_path)?;
        checkpoint(TargetPoint::BeforeTreeSync, &self.path())?;
        self.verify_contents()?;
        for entry in &self.entries {
            sync(&entry.file, &self.path().join(entry.relative.as_str()))?;
            sync(
                &entry.directory,
                &self.path().join("artifacts").join(&entry.directory_name),
            )?;
        }
        if let Some((file, _)) = &self.manifest {
            sync(file, &manifest_path)?;
        }
        sync(&self.artifacts, &self.path().join("artifacts"))?;
        sync(&self.root, &self.path())?;
        checkpoint(TargetPoint::BeforeRename, &self.path())?;
        self.verify_contents()?;
        let final_path = self.destination.parent.path.join(&self.final_name);
        renameat_with(
            &self.destination.parent.file,
            self.name.as_str(),
            &self.destination.parent.file,
            self.final_name.as_str(),
            RenameFlags::NOREPLACE,
        )
        .map_err(|error| io_error(&final_path, "publish_noreplace", error.into()))?;
        self.moved = true;
        let confirmation = (|| {
            checkpoint(TargetPoint::AfterRenameBeforeParentSync, &final_path)?;
            self.confirm_final()?;
            sync(&self.destination.parent.file, &self.destination.parent.path)?;
            checkpoint(TargetPoint::AfterParentSync, &final_path)?;
            self.confirm_final()
        })();
        if let Err(error) = confirmation {
            return Err(Error::PublicationUnconfirmed {
                target_path: self.confirm_final().ok().map(|()| final_path),
                message: error.to_string(),
            });
        }
        Ok(final_path)
    }

    fn confirm_final(&self) -> Result<()> {
        self.destination.parent.verify()?;
        let path = self.destination.parent.path.join(&self.final_name);
        verify_entry(
            &self.destination.parent.file,
            &self.final_name,
            &self.root,
            &path,
            true,
        )
    }
}

fn copied(entry: &Entry) -> CopiedFile {
    CopiedFile {
        key: entry.artifact.key.clone(),
        path: entry.relative.clone(),
        sha256: entry.artifact.sha256.clone(),
        bytes: entry.artifact.bytes,
    }
}

fn read_back(file: &File, path: &Path, expected: &Sha256Hex, bytes: u64) -> Result<()> {
    let before = fstat(file).map_err(|error| io_error(path, "readback_stat", error.into()))?;
    if FileType::from_raw_mode(before.st_mode) != FileType::RegularFile
        || before.st_nlink != 1
        || before.st_size < 0
        || before.st_size as u64 != bytes
        || bytes > MAX_FILE_BYTES
    {
        return Err(integrity(path, "读回对象类型、链接或大小不符"));
    }
    let mut handle = file;
    handle
        .rewind()
        .map_err(|error| io_error(path, "readback_rewind", error))?;
    let mut hash = Sha256::new();
    let mut actual = 0_u64;
    let mut buffer = [0_u8; 65536];
    loop {
        let count = handle
            .read(&mut buffer)
            .map_err(|error| io_error(path, "readback", error))?;
        if count == 0 {
            break;
        }
        actual = actual
            .checked_add(count as u64)
            .ok_or_else(|| integrity(path, "读回大小溢出"))?;
        if actual > bytes {
            return Err(integrity(path, "读回文件增长"));
        }
        hash.update(&buffer[..count]);
    }
    let actual_hash = format!("{:x}", hash.finalize());
    let after = fstat(file).map_err(|error| io_error(path, "readback_stat", error.into()))?;
    if actual != bytes
        || actual_hash != expected.as_str()
        || after.st_dev != before.st_dev
        || after.st_ino != before.st_ino
        || after.st_nlink != 1
        || after.st_size != before.st_size
    {
        return Err(integrity(path, "读回字节、摘要或身份不符"));
    }
    Ok(())
}

fn sync(file: &File, path: &Path) -> Result<()> {
    fsync(file).map_err(|error| io_error(path, "fsync", error.into()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPoint {
    AfterFileSync,
    AfterReadback,
    AfterManifestWrite,
    BeforeTreeSync,
    BeforeRename,
    AfterRenameBeforeParentSync,
    AfterParentSync,
}

#[cfg(feature = "failpoint")]
mod faults {
    use super::*;
    use std::cell::RefCell;
    enum Fault {
        Sync,
        Rendezvous(PathBuf),
    }
    thread_local! {
        static FAULT: RefCell<Option<(TargetPoint, Fault)>> = const { RefCell::new(None) };
        static HOOK: RefCell<Option<fn(TargetPoint)>> = const { RefCell::new(None) };
    }
    pub fn arm_sync_error(point: TargetPoint) {
        FAULT.with(|slot| *slot.borrow_mut() = Some((point, Fault::Sync)));
    }
    pub fn arm_rendezvous(point: TargetPoint, directory: &Path) {
        FAULT.with(|slot| {
            *slot.borrow_mut() = Some((point, Fault::Rendezvous(directory.to_path_buf())))
        });
    }
    pub fn install_boundary_hook(hook: fn(TargetPoint)) {
        HOOK.with(|slot| *slot.borrow_mut() = Some(hook));
    }
    pub(super) fn checkpoint(point: TargetPoint, path: &Path) -> Result<()> {
        HOOK.with(|slot| {
            if let Some(hook) = *slot.borrow() {
                hook(point);
            }
        });
        let fault = FAULT.with(|slot| {
            if slot
                .borrow()
                .as_ref()
                .is_some_and(|(expected, _)| *expected == point)
            {
                slot.borrow_mut().take()
            } else {
                None
            }
        });
        match fault {
            None => Ok(()),
            Some((_, Fault::Sync)) => Err(io_error(
                path,
                "injected_fsync_error",
                std::io::Error::other("受控fsync失败"),
            )),
            Some((_, Fault::Rendezvous(directory))) => {
                std::fs::write(directory.join("ready"), b"ready")
                    .map_err(|error| io_error(&directory, "rendezvous_ready", error))?;
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                while !directory.join("release").exists() {
                    if std::time::Instant::now() >= deadline {
                        return Err(io_error(
                            &directory,
                            "rendezvous_timeout",
                            std::io::Error::new(std::io::ErrorKind::TimedOut, "交错点超时"),
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Ok(())
            }
        }
    }
}

#[cfg(feature = "failpoint")]
pub use faults::{arm_rendezvous, arm_sync_error, install_boundary_hook};

fn checkpoint(point: TargetPoint, path: &Path) -> Result<()> {
    #[cfg(feature = "failpoint")]
    {
        faults::checkpoint(point, path)
    }
    #[cfg(not(feature = "failpoint"))]
    {
        let _ = (point, path);
        Ok(())
    }
}
