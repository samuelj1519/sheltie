//! 管理根内的受限文件操作（`INV-3`、GF-32、存储合同 §4）。
//!
//! 统一入口：祖先链软链检查、句柄身份核对、限额读取与同一句柄封存、
//! 独占临时名原子写、不跟随软链的删除。观察、限额、摘要与封存作用在
//! 同一个打开的文件对象上；不能「检查路径、重开另一对象、再 chmod」。

use std::io::{Read, Seek, Write};
use std::os::unix::fs::MetadataExt;

use rustix::fs::{
    AtFlags, Dir, FileType, Mode, OFlags, RenameFlags, fchmod, fstat, fsync, mkdirat, openat,
    renameat_with, statat, unlinkat,
};

use sheltie_core::digest::Sha256Hex;
use sheltie_core::path::{AbsPath, RelPath};

use crate::error::{Error, Result};

/// 单文件上限 32 MiB（存储合同 §5.2，与 workbook_repo 一致）。
pub const MAX_FILE_BYTES: u64 = 33_554_432;
/// 目录总量上限 256 MiB。
pub const MAX_TOTAL_BYTES: u64 = 268_435_456;
/// 明确列举的宿主元数据文件（存储合同 §5.3）：Finder 一类工具生成，不属于 Workbook
/// 字节。复制与读取都**准确拒绝并点名文件**，不静默忽略字节。
pub const HOST_METADATA_FILES: &[&str] = &[".DS_Store"];

/// 已验证的管理根内路径。每段都独立校验，构造后不再接受裸绝对路径。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManagedRelPath {
    value: String,
    segments: Vec<String>,
}

impl ManagedRelPath {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.as_bytes().contains(&0) {
            return Err(Error::InvalidRequest {
                reason: "管理路径不能包含NUL".to_string(),
            });
        }
        let parsed = sheltie_core::path::RelPath::new(value.clone()).map_err(Error::Core)?;
        let segments = parsed
            .as_str()
            .split('/')
            .map(str::to_string)
            .collect::<Vec<_>>();
        Ok(Self { value, segments })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    fn leaf(&self) -> Result<&str> {
        self.segments
            .last()
            .map(String::as_str)
            .ok_or_else(|| Error::InvalidRequest {
                reason: "管理路径不能是空路径".to_string(),
            })
    }

    fn parent(&self) -> Option<Self> {
        (self.segments.len() > 1).then(|| {
            let segments = self.segments[..self.segments.len() - 1].to_vec();
            Self {
                value: segments.join("/"),
                segments,
            }
        })
    }
}

/// 根目录句柄锚定的管理文件系统。根内路径只按目录句柄逐段解析。
#[derive(Debug)]
pub struct ManagedFs {
    root: AbsPath,
    root_dir: std::fs::File,
    root_ident: (u64, u64),
}

/// 从已核验的根目录句柄逐段打开的子目录。
#[derive(Debug)]
pub struct ManagedDir {
    file: std::fs::File,
    root: AbsPath,
    root_ident: (u64, u64),
    path: ManagedRelPath,
}

/// 一个待发布目录的受管句柄，保持同步与原子移动绑定到同一 inode。
#[derive(Debug)]
pub(crate) struct ManagedTree {
    file: std::fs::File,
    root: AbsPath,
    root_ident: (u64, u64),
    path: ManagedRelPath,
}

impl ManagedDir {
    pub fn path(&self) -> &ManagedRelPath {
        &self.path
    }

    pub fn write_new(&self, lock: &crate::home::HomeLock, leaf: &str, bytes: &[u8]) -> Result<()> {
        self.check_lock(lock)?;
        let leaf = self.leaf(leaf)?;
        let display = format!("{}/{}", self.path.as_str(), leaf);
        let fd = openat(
            &self.file,
            &leaf,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|e| map_fs_error(&display, e))?;
        let mut file = std::fs::File::from(fd);
        file.write_all(bytes).map_err(|e| Error::io(&display, e))?;
        file.sync_all().map_err(|e| Error::io(&display, e))?;
        fsync(&self.file).map_err(|e| map_fs_error(&display, e))
    }

    pub fn rename_new(&self, lock: &crate::home::HomeLock, from: &str, to: &str) -> Result<()> {
        self.check_lock(lock)?;
        let from = self.leaf(from)?;
        let to = self.leaf(to)?;
        let display = format!("{}/{}", self.path.as_str(), to);
        check_rename_source(&self.file, &from, &display)?;
        renameat_with(&self.file, &from, &self.file, &to, RenameFlags::NOREPLACE)
            .map_err(|e| map_fs_error(&display, e))?;
        fsync(&self.file).map_err(|e| map_fs_error(&display, e))
    }

    fn leaf(&self, value: &str) -> Result<String> {
        let path = ManagedRelPath::new(value)?;
        if path.segments.len() != 1 {
            return Err(Error::InvalidRequest {
                reason: "ManagedDir操作只接受单个叶名称".to_string(),
            });
        }
        Ok(path.value)
    }

    fn check_lock(&self, lock: &crate::home::HomeLock) -> Result<()> {
        if !lock.matches_root(&self.root, self.root_ident) {
            return Err(Error::InvalidRequest {
                reason: "写锁与目录句柄的管理根不是同一对象".to_string(),
            });
        }
        Ok(())
    }
}

impl ManagedFs {
    /// 打开已存在的真实管理根。每一段都拒绝符号链接；Home.resolve已处理系统根别名。
    pub fn open_existing(home: &crate::home::Home) -> Result<Self> {
        Self::open_root(home.root())
    }

    pub(crate) fn open_root(root: &AbsPath) -> Result<Self> {
        let slash = std::fs::File::open("/").map_err(|e| Error::io("/", e))?;
        Self::traverse_root(root, slash, false)
    }

    pub(crate) fn create_root(root: &AbsPath) -> Result<Self> {
        let slash = std::fs::File::open("/").map_err(|e| Error::io("/", e))?;
        Self::traverse_root(root, slash, true)
    }

    fn traverse_root(root: &AbsPath, slash: std::fs::File, create_missing: bool) -> Result<Self> {
        if root.as_str().as_bytes().contains(&0) {
            return Err(Error::InvalidRequest {
                reason: "管理根不能包含NUL".to_string(),
            });
        }
        let mut current = slash;
        let absolute = root.as_str().trim_start_matches('/');
        let mut path_segments = Vec::new();
        for segment in absolute.split('/') {
            if segment.is_empty() {
                continue;
            }
            if segment == "." || segment == ".." {
                return Err(Error::InvalidRequest {
                    reason: format!("管理根包含未规范化路径段 {segment}"),
                });
            }
            path_segments.push(segment.to_string());
            let display = format!("/{}", path_segments.join("/"));
            let next = match statat(&current, segment, AtFlags::SYMLINK_NOFOLLOW) {
                Ok(stat) if FileType::from_raw_mode(stat.st_mode) == FileType::Symlink => {
                    return Err(Error::InvalidRequest {
                        reason: format!("管理根段 {display} 是符号链接"),
                    });
                }
                Ok(stat) if FileType::from_raw_mode(stat.st_mode) != FileType::Directory => {
                    return Err(Error::InvalidRequest {
                        reason: format!("管理根段 {display} 不是目录"),
                    });
                }
                Ok(_) => open_directory_at(&current, segment, &display)?,
                Err(rustix::io::Errno::NOENT) if create_missing => {
                    match mkdirat(&current, segment, Mode::from_raw_mode(0o700)) {
                        Ok(()) | Err(rustix::io::Errno::EXIST) => {
                            fsync(&current).map_err(|e| map_fs_error(&display, e))?;
                        }
                        Err(error) => return Err(map_fs_error(&display, error)),
                    }
                    open_directory_at(&current, segment, &display)?
                }
                Err(error) => return Err(map_fs_error(&display, error)),
            };
            current = next;
        }
        let meta = current
            .metadata()
            .map_err(|e| Error::io(root.as_str(), e))?;
        if !meta.is_dir() {
            return Err(Error::InvalidRequest {
                reason: format!("管理根 {root} 不是目录"),
            });
        }
        Ok(Self {
            root: root.clone(),
            root_ident: (meta.dev(), meta.ino()),
            root_dir: current,
        })
    }

    pub fn root(&self) -> &AbsPath {
        &self.root
    }

    pub(crate) fn open_lock_file(&self) -> Result<std::fs::File> {
        let lock_path = self.root.join_segment(".lock");
        let fd = openat(
            &self.root_dir,
            ".lock",
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|e| map_fs_error(lock_path.as_str(), e))?;
        let file = std::fs::File::from(fd);
        let stat = fstat(&file).map_err(|e| map_fs_error(lock_path.as_str(), e))?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile || stat.st_nlink != 1 {
            return Err(Error::InvalidRequest {
                reason: format!("{} 必须是普通单链接锁文件", lock_path),
            });
        }
        fsync(&file).map_err(|e| map_fs_error(lock_path.as_str(), e))?;
        fsync(&self.root_dir).map_err(|e| map_fs_error(self.root.as_str(), e))?;
        Ok(file)
    }

    /// Open only an existing, trusted `.lock` file. Unlike `open_lock_file`, this path never
    /// creates the root or lock entry and is used only while retrying a concurrent Store init.
    pub(crate) fn open_existing_lock_file(&self) -> Result<Option<std::fs::File>> {
        let lock_path = self.root.join_segment(".lock");
        let observed = match statat(&self.root_dir, ".lock", AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => stat,
            Err(error) if error == rustix::io::Errno::NOENT => return Ok(None),
            Err(error) => return Err(map_fs_error(lock_path.as_str(), error)),
        };
        if FileType::from_raw_mode(observed.st_mode) != FileType::RegularFile
            || observed.st_nlink != 1
        {
            return Err(Error::InvalidRequest {
                reason: format!("{} 必须是普通单链接锁文件", lock_path),
            });
        }
        let fd = openat(
            &self.root_dir,
            ".lock",
            OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::from_raw_mode(0),
        )
        .map_err(|error| map_fs_error(lock_path.as_str(), error))?;
        let file = std::fs::File::from(fd);
        let opened = fstat(&file).map_err(|error| map_fs_error(lock_path.as_str(), error))?;
        if FileType::from_raw_mode(opened.st_mode) != FileType::RegularFile
            || opened.st_nlink != 1
            || opened.st_dev != observed.st_dev
            || opened.st_ino != observed.st_ino
        {
            return Err(Error::InvalidRequest {
                reason: format!("{} 在安全打开期间被替换", lock_path),
            });
        }
        Ok(Some(file))
    }

    pub(crate) fn identity(&self) -> (u64, u64) {
        self.root_ident
    }

    fn absolute(&self, rel: &ManagedRelPath) -> Result<AbsPath> {
        AbsPath::new(format!(
            "{}/{}",
            self.root.as_str().trim_end_matches('/'),
            rel.as_str()
        ))
        .map_err(Error::Core)
    }

    fn check_lock(&self, lock: &crate::home::HomeLock) -> Result<()> {
        if !lock.matches_root(&self.root, self.root_ident) {
            return Err(Error::InvalidRequest {
                reason: "写锁与管理根句柄不是同一对象".to_string(),
            });
        }
        Ok(())
    }

    fn open_dir(&self, rel: Option<&ManagedRelPath>) -> Result<std::fs::File> {
        let Some(rel) = rel else {
            return self
                .root_dir
                .try_clone()
                .map_err(|e| Error::io(self.root.as_str(), e));
        };
        let mut current = self
            .root_dir
            .try_clone()
            .map_err(|e| Error::io(self.root.as_str(), e))?;
        for segment in &rel.segments {
            current = open_directory_at(&current, segment, &self.display_path(rel))?;
        }
        Ok(current)
    }

    fn open_tree_root(&self, rel: &ManagedRelPath) -> Result<std::fs::File> {
        self.open_dir(Some(rel))
    }

    fn open_parent(&self, rel: &ManagedRelPath) -> Result<(std::fs::File, String)> {
        let leaf = rel.leaf()?.to_string();
        let parent = rel.parent();
        Ok((self.open_dir(parent.as_ref())?, leaf))
    }

    fn display_path(&self, rel: &ManagedRelPath) -> String {
        format!(
            "{}/{}",
            self.root.as_str().trim_end_matches('/'),
            rel.as_str()
        )
    }

    /// 持锁安全建立路径中的所有目录段，并对新目录项同步父目录。
    pub fn ensure_dir(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<ManagedDir> {
        self.check_lock(lock)?;
        self.ensure_dir_unlocked(path)?;
        Ok(ManagedDir {
            file: self.open_dir(Some(path))?,
            root: self.root.clone(),
            root_ident: self.root_ident,
            path: path.clone(),
        })
    }

    fn ensure_dir_unlocked(&self, path: &ManagedRelPath) -> Result<()> {
        let mut current = self
            .root_dir
            .try_clone()
            .map_err(|e| Error::io(self.root.as_str(), e))?;
        let mut prefix = Vec::new();
        for segment in &path.segments {
            prefix.push(segment.clone());
            let rel = ManagedRelPath {
                value: prefix.join("/"),
                segments: prefix.clone(),
            };
            match statat(&current, segment, AtFlags::SYMLINK_NOFOLLOW) {
                Ok(stat) if FileType::from_raw_mode(stat.st_mode) == FileType::Symlink => {
                    return Err(Error::InvalidRequest {
                        reason: format!("{} 是符号链接，不能作为管理目录", self.display_path(&rel)),
                    });
                }
                Ok(stat) if FileType::from_raw_mode(stat.st_mode) != FileType::Directory => {
                    return Err(Error::InvalidRequest {
                        reason: format!("{} 不是目录", self.display_path(&rel)),
                    });
                }
                Ok(stat) => {
                    let opened = open_directory_at(&current, segment, &self.display_path(&rel))?;
                    let after =
                        fstat(&opened).map_err(|e| map_fs_error(&self.display_path(&rel), e))?;
                    if after.st_dev != stat.st_dev || after.st_ino != stat.st_ino {
                        return Err(Error::InvalidRequest {
                            reason: format!("{} 在建立期间被替换", self.display_path(&rel)),
                        });
                    }
                    current = opened;
                }
                Err(rustix::io::Errno::NOENT) => {
                    mkdirat(&current, segment, Mode::from_raw_mode(0o700))
                        .map_err(|e| map_fs_error(&self.display_path(&rel), e))?;
                    fsync(&current).map_err(|e| map_fs_error(&self.display_path(&rel), e))?;
                    let fd = openat(
                        &current,
                        segment,
                        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                        Mode::empty(),
                    )
                    .map_err(|e| map_fs_error(&self.display_path(&rel), e))?;
                    current = std::fs::File::from(fd);
                }
                Err(e) => return Err(map_fs_error(&self.display_path(&rel), e)),
            }
        }
        Ok(())
    }

    /// 以目录句柄逐段打开普通、单链接文件，并核对打开前后的dev/inode。
    pub fn open_regular(&self, path: &ManagedRelPath) -> Result<SafeFile> {
        let (parent, leaf) = self.open_parent(path)?;
        let mut file = open_regular_at(
            &parent,
            &leaf,
            &self.display_path(path),
            self.absolute(path)?,
        )?;
        file.managed = Some(ManagedOrigin {
            root: self.root.clone(),
            root_ident: self.root_ident,
            rel: path.clone(),
        });
        Ok(file)
    }

    pub fn open_optional(&self, path: &ManagedRelPath) -> Result<Option<SafeFile>> {
        match self.open_regular(path) {
            Ok(file) => Ok(Some(file)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn open_optional_locked(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<Option<SafeFile>> {
        self.check_lock(lock)?;
        self.open_optional(path)
    }

    pub(crate) fn directory_exists_locked(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<bool> {
        self.check_lock(lock)?;
        let (parent, leaf) = match self.open_parent(path) {
            Ok(parent) => parent,
            Err(Error::NotFound { .. }) => return Ok(false),
            Err(error) => return Err(error),
        };
        match statat(&parent, &leaf, AtFlags::SYMLINK_NOFOLLOW) {
            Err(rustix::io::Errno::NOENT) => Ok(false),
            Err(error) => Err(map_fs_error(&self.display_path(path), error)),
            Ok(stat) if FileType::from_raw_mode(stat.st_mode) == FileType::Directory => {
                open_directory_at(&parent, &leaf, &self.display_path(path))?;
                Ok(true)
            }
            Ok(_) => Err(Error::StoreCorrupt {
                detail: format!("{} 不是受管目录", self.display_path(path)),
            }),
        }
    }

    pub(crate) fn open_tree_locked(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<ManagedTree> {
        self.check_lock(lock)?;
        let file = self.open_dir(Some(path))?;
        Ok(ManagedTree {
            file,
            root: self.root.clone(),
            root_ident: self.root_ident,
            path: path.clone(),
        })
    }

    pub(crate) fn verify_tree_at(&self, tree: &ManagedTree, path: &ManagedRelPath) -> Result<()> {
        self.check_tree_root(tree)?;
        let (parent, leaf) = self.open_parent(path)?;
        let stat = statat(&parent, &leaf, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| map_fs_error(&self.display_path(path), error))?;
        let held =
            fstat(&tree.file).map_err(|error| map_fs_error(&self.display_path(path), error))?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::Directory
            || stat.st_dev != held.st_dev
            || stat.st_ino != held.st_ino
        {
            return Err(Error::StoreCorrupt {
                detail: format!("{} 不再指向已核验的发布目录对象", self.display_path(path)),
            });
        }
        Ok(())
    }

    pub(crate) fn sync_managed_tree(
        &self,
        lock: &crate::home::HomeLock,
        tree: &ManagedTree,
    ) -> Result<()> {
        self.check_lock(lock)?;
        self.check_tree_root(tree)?;
        sync_dir_tree(
            &tree.file,
            &self.display_path(&tree.path),
            self.root.as_str(),
            true,
        )
    }

    pub(crate) fn rename_tree_new(
        &self,
        lock: &crate::home::HomeLock,
        tree: &ManagedTree,
        to: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        self.check_tree_root(tree)?;
        let from = &tree.path;
        let (source_parent, source_leaf) = self.open_parent(from)?;
        let (target_parent, target_leaf) = self.open_parent(to)?;
        let source = statat(&source_parent, &source_leaf, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| map_fs_error(&self.display_path(from), error))?;
        let held =
            fstat(&tree.file).map_err(|error| map_fs_error(&self.display_path(from), error))?;
        if FileType::from_raw_mode(source.st_mode) != FileType::Directory
            || source.st_dev != held.st_dev
            || source.st_ino != held.st_ino
        {
            return Err(Error::StoreCorrupt {
                detail: format!("{} 在发布前已被替换", self.display_path(from)),
            });
        }
        renameat_with(
            &source_parent,
            &source_leaf,
            &target_parent,
            &target_leaf,
            RenameFlags::NOREPLACE,
        )
        .map_err(|error| map_fs_error(&self.display_path(to), error))?;
        self.verify_tree_at(tree, to)
            .map_err(|error| Error::RecoveryRequired {
                path: self.display_path(to),
                detail: format!("目录已移动但目标不再是已核验对象：{error}"),
            })?;
        sync_rename_parents(
            &source_parent,
            &target_parent,
            &self.display_path(from),
            &self.display_path(to),
            self.root.as_str(),
        )
    }

    pub(crate) fn sync_publish_parents_locked(
        &self,
        lock: &crate::home::HomeLock,
        from: &ManagedRelPath,
        to: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let source_parent = from.parent().ok_or_else(|| Error::StoreCorrupt {
            detail: "pending目录不能是管理根".to_string(),
        })?;
        let target_parent = to.parent().ok_or_else(|| Error::StoreCorrupt {
            detail: "final目录不能是管理根".to_string(),
        })?;
        let source = self.open_dir(Some(&source_parent))?;
        let target = self.open_dir(Some(&target_parent))?;
        sync_rename_parents(
            &source,
            &target,
            &self.display_path(from),
            &self.display_path(to),
            self.root.as_str(),
        )
    }

    pub(crate) fn sync_publish_final_root_locked(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let directory = self.open_dir(Some(path))?;
        crate::failpoint::sync_error(self.root.as_str(), "publish_final_root_sync")
            .map_err(|error| Error::io(self.display_path(path), error))?;
        fsync(&directory).map_err(|error| map_fs_error(&self.display_path(path), error))
    }

    fn check_tree_root(&self, tree: &ManagedTree) -> Result<()> {
        if tree.root != self.root || tree.root_ident != self.root_ident {
            return Err(Error::InvalidRequest {
                reason: "发布目录句柄不属于此管理根".to_string(),
            });
        }
        Ok(())
    }

    pub(crate) fn validate_sqlite_control_file(&self, path: &ManagedRelPath) -> Result<bool> {
        let display = self.display_path(path);
        let (parent, leaf) = self.open_parent(path)?;
        let stat = match statat(&parent, &leaf, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => stat,
            Err(rustix::io::Errno::NOENT) => return Ok(false),
            Err(error) => return Err(map_fs_error(&display, error)),
        };
        if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile
            || !matches!(stat.st_nlink, 0 | 1)
        {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "SQLite控制文件 {display} 必须是普通单链接文件（nlink={}）",
                    stat.st_nlink
                ),
            });
        }
        Ok(stat.st_nlink == 1)
    }

    /// 持锁独占创建普通文件，写完并同步文件及其父目录。
    pub fn write_new(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
        bytes: &[u8],
    ) -> Result<()> {
        self.check_lock(lock)?;
        drop(self.write_new_observed_unlocked(path, bytes)?);
        Ok(())
    }

    pub fn write_new_observed(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
        bytes: &[u8],
    ) -> Result<SafeFile> {
        self.check_lock(lock)?;
        self.write_new_observed_unlocked(path, bytes)
    }

    fn write_new_observed_unlocked(&self, path: &ManagedRelPath, bytes: &[u8]) -> Result<SafeFile> {
        let (parent, leaf) = self.open_parent(path)?;
        let display = self.display_path(path);
        let fd = openat(
            &parent,
            &leaf,
            OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|e| map_fs_error(&display, e))?;
        let mut file = std::fs::File::from(fd);
        let created_metadata = file.metadata().map_err(|e| Error::io(&display, e))?;
        if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            return match unlink_created_at(&parent, &leaf, &display, &created_metadata) {
                Ok(()) => Err(Error::io(&display, error)),
                Err(cleanup) => Err(Error::RecoveryRequired {
                    path: display,
                    detail: format!("新文件写入失败：{error}；本次创建inode清理失败：{cleanup}"),
                }),
            };
        }
        let metadata = file.metadata().map_err(|error| Error::RecoveryRequired {
            path: display.clone(),
            detail: format!("新文件内容已写入，但无法复核原句柄元数据：{error}"),
        })?;
        if !metadata.is_file()
            || metadata.nlink() != 1
            || (metadata.dev(), metadata.ino()) != (created_metadata.dev(), created_metadata.ino())
        {
            return Err(Error::RecoveryRequired {
                path: display,
                detail: "新文件句柄在写入后身份、类型或链接数改变；保留该对象".into(),
            });
        }
        if let Err(error) = fsync(&parent) {
            return Err(Error::RecoveryRequired {
                path: display,
                detail: format!("新文件已同步但父目录同步失败，保留创建inode：{error}"),
            });
        }
        Ok(SafeFile {
            file,
            path: self.absolute(path)?,
            meta: metadata,
            managed: Some(ManagedOrigin {
                root: self.root.clone(),
                root_ident: self.root_ident,
                rel: path.clone(),
            }),
        })
    }

    /// 持锁以同目录临时对象原子替换投影文件；目标叶不跟随符号链接。
    pub fn write_atomic(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
        bytes: &[u8],
    ) -> Result<()> {
        self.check_lock(lock)?;
        self.write_atomic_unlocked(path, bytes)
    }

    fn write_atomic_unlocked(&self, path: &ManagedRelPath, bytes: &[u8]) -> Result<()> {
        if let Some(parent_path) = path.parent() {
            self.ensure_dir_unlocked(&parent_path)?;
        }
        let (parent, leaf) = self.open_parent(path)?;
        let display = self.display_path(path);
        let tmp = format!(".{leaf}.tmp-{}", uuid::Uuid::now_v7().simple());
        let fd = openat(
            &parent,
            &tmp,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(|e| map_fs_error(&display, e))?;
        let mut file = std::fs::File::from(fd);
        if let Err(e) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            let _ = unlinkat(&parent, &tmp, AtFlags::empty());
            return Err(Error::io(&display, e));
        }
        drop(file);
        if let Err(e) = renameat_with(&parent, &tmp, &parent, &leaf, RenameFlags::empty()) {
            let _ = unlinkat(&parent, &tmp, AtFlags::empty());
            return Err(map_fs_error(&display, e));
        }
        fsync(&parent).map_err(|e| map_fs_error(&display, e))
    }

    /// 持锁将对象移动到不存在的新目标，两个端点都以父目录句柄定位。
    pub fn rename_new(
        &self,
        lock: &crate::home::HomeLock,
        from: &ManagedRelPath,
        to: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        self.rename_new_unlocked(from, to)
    }

    pub fn rename_verified_new(
        &self,
        lock: &crate::home::HomeLock,
        file: &SafeFile,
        to: &ManagedRelPath,
        expected: &[u8],
    ) -> Result<()> {
        self.check_lock(lock)?;
        let origin = file.managed.as_ref().ok_or_else(|| Error::InvalidRequest {
            reason: format!("{} 不是管理根内的候选文件", file.path),
        })?;
        if origin.root != self.root || origin.root_ident != self.root_ident {
            return Err(Error::InvalidRequest {
                reason: format!("{} 的候选句柄属于另一管理根", file.path),
            });
        }
        verify_candidate_handle(file, expected)?;
        let (source_parent, source_leaf) = self.open_parent(&origin.rel)?;
        let (target_parent, target_leaf) = self.open_parent(to)?;
        verify_path_matches_handle(&source_parent, &source_leaf, file)?;
        renameat_with(
            &source_parent,
            &source_leaf,
            &target_parent,
            &target_leaf,
            RenameFlags::NOREPLACE,
        )
        .map_err(|e| map_fs_error(&self.display_path(to), e))?;
        if let Err(error) = verify_path_matches_handle(&target_parent, &target_leaf, file) {
            return Err(Error::RecoveryRequired {
                path: self.display_path(to),
                detail: format!("候选移动后身份核验失败，目标和其他端点已保留：{error}"),
            });
        }
        verify_candidate_handle(file, expected).map_err(|error| Error::RecoveryRequired {
            path: self.display_path(to),
            detail: format!("候选移动后字节复核失败，目标和其他端点已保留：{error}"),
        })?;
        fsync(&source_parent).map_err(|e| Error::RecoveryRequired {
            path: self.display_path(&origin.rel),
            detail: format!("候选已移动但源目录同步失败：{e}"),
        })?;
        fsync(&target_parent).map_err(|e| Error::RecoveryRequired {
            path: self.display_path(to),
            detail: format!("候选已移动但目标目录同步失败：{e}"),
        })
    }

    pub fn replace_verified_regular_file(
        &self,
        lock: &crate::home::HomeLock,
        file: &SafeFile,
        to: &ManagedRelPath,
        expected: &[u8],
    ) -> Result<()> {
        self.check_lock(lock)?;
        let origin = file.managed.as_ref().ok_or_else(|| Error::InvalidRequest {
            reason: format!("{} 不是管理根内的候选文件", file.path),
        })?;
        if origin.root != self.root || origin.root_ident != self.root_ident {
            return Err(Error::InvalidRequest {
                reason: format!("{} 的候选句柄属于另一管理根", file.path),
            });
        }
        verify_candidate_handle(file, expected)?;
        let (source_parent, source_leaf) = self.open_parent(&origin.rel)?;
        let (target_parent, target_leaf) = self.open_parent(to)?;
        verify_path_matches_handle(&source_parent, &source_leaf, file)?;
        let old_target = statat(&target_parent, &target_leaf, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| map_fs_error(&self.display_path(to), e))?;
        check_regular_stat(&self.display_path(to), &old_target)?;
        renameat_with(
            &source_parent,
            &source_leaf,
            &target_parent,
            &target_leaf,
            RenameFlags::EXCHANGE,
        )
        .map_err(|e| map_fs_error(&self.display_path(to), e))?;
        let source_after = statat(&source_parent, &source_leaf, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| Error::RecoveryRequired {
                path: self.display_path(&origin.rel),
                detail: format!("候选交换后源端点无法观察：{e}"),
            })?;
        let target_after = statat(&target_parent, &target_leaf, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| Error::RecoveryRequired {
                path: self.display_path(to),
                detail: format!("候选交换后目标端点无法观察：{e}"),
            })?;
        if !stat_matches_handle(&target_after, file)
            || source_after.st_dev != old_target.st_dev
            || source_after.st_ino != old_target.st_ino
        {
            return Err(Error::RecoveryRequired {
                path: self.display_path(to),
                detail: "候选交换后端点身份不符；候选、原文件和目标均保留".into(),
            });
        }
        verify_candidate_handle(file, expected).map_err(|error| Error::RecoveryRequired {
            path: self.display_path(to),
            detail: format!("候选交换后字节复核失败；两个端点均保留：{error}"),
        })?;
        fsync(&source_parent).map_err(|e| Error::RecoveryRequired {
            path: self.display_path(&origin.rel),
            detail: format!("候选交换已生效但源目录同步失败：{e}"),
        })?;
        fsync(&target_parent).map_err(|e| Error::RecoveryRequired {
            path: self.display_path(to),
            detail: format!("候选交换已生效但目标目录同步失败：{e}"),
        })
    }

    /// Atomically exchange two already-owned regular files. This is used to replace the current
    /// executable without opening a path-based overwrite window; the old object lands at `from`.
    pub fn replace_regular_file(
        &self,
        lock: &crate::home::HomeLock,
        from: &ManagedRelPath,
        to: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let (src_parent, src_leaf) = self.open_parent(from)?;
        let (dst_parent, dst_leaf) = self.open_parent(to)?;
        let src_path = self.display_path(from);
        let dst_path = self.display_path(to);
        let src_before = statat(&src_parent, &src_leaf, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| map_fs_error(&src_path, e))?;
        check_regular_stat(&src_path, &src_before)?;
        let dst_before = statat(&dst_parent, &dst_leaf, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| map_fs_error(&dst_path, e))?;
        check_regular_stat(&dst_path, &dst_before)?;
        renameat_with(
            &src_parent,
            &src_leaf,
            &dst_parent,
            &dst_leaf,
            RenameFlags::EXCHANGE,
        )
        .map_err(|e| map_fs_error(&dst_path, e))?;
        let source_after =
            statat(&src_parent, &src_leaf, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| {
                Error::RecoveryRequired {
                    path: src_path.clone(),
                    detail: format!("交换已执行但无法观察源端点；请保留两个端点：{e}"),
                }
            })?;
        let target_after =
            statat(&dst_parent, &dst_leaf, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| {
                Error::RecoveryRequired {
                    path: dst_path.clone(),
                    detail: format!("交换已执行但无法观察目标端点；请保留两个端点：{e}"),
                }
            })?;
        if FileType::from_raw_mode(source_after.st_mode) != FileType::RegularFile
            || source_after.st_nlink != 1
            || source_after.st_dev != dst_before.st_dev
            || source_after.st_ino != dst_before.st_ino
            || FileType::from_raw_mode(target_after.st_mode) != FileType::RegularFile
            || target_after.st_nlink != 1
            || target_after.st_dev != src_before.st_dev
            || target_after.st_ino != src_before.st_ino
        {
            return Err(Error::RecoveryRequired {
                path: dst_path,
                detail: "原子交换后端点身份或链接数不符；保留两个端点以便恢复".into(),
            });
        }
        fsync(&src_parent).map_err(|e| Error::RecoveryRequired {
            path: src_path.clone(),
            detail: format!("原子交换已生效但目录同步失败：{e}"),
        })?;
        fsync(&dst_parent).map_err(|e| Error::RecoveryRequired {
            path: dst_path.clone(),
            detail: format!("原子交换已生效但目录同步失败：{e}"),
        })?;
        Ok(())
    }

    fn rename_new_unlocked(&self, from: &ManagedRelPath, to: &ManagedRelPath) -> Result<()> {
        let (src_parent, src_leaf) = self.open_parent(from)?;
        let (dst_parent, dst_leaf) = self.open_parent(to)?;
        check_rename_source(&src_parent, &src_leaf, &self.display_path(from))?;
        renameat_with(
            &src_parent,
            &src_leaf,
            &dst_parent,
            &dst_leaf,
            RenameFlags::NOREPLACE,
        )
        .map_err(|e| map_fs_error(&self.display_path(to), e))?;
        sync_rename_parents(
            &src_parent,
            &dst_parent,
            &self.display_path(from),
            &self.display_path(to),
            self.root.as_str(),
        )
    }

    /// 持锁安全删除本管理根下的对象；目录逐层由句柄枚举，叶链接只unlink自身。
    pub fn remove_owned_tree(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let (parent, leaf) = self.open_parent(path)?;
        remove_at(&parent, &leaf, &self.display_path(path))?;
        fsync(&parent).map_err(|e| map_fs_error(&self.display_path(path), e))
    }

    pub(crate) fn remove_regular_file_if_same(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
        observed: &SafeFile,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let origin = observed
            .managed
            .as_ref()
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{} 不是管理根内的文件句柄", observed.path),
            })?;
        if origin.root != self.root || origin.root_ident != self.root_ident || origin.rel != *path {
            return Err(Error::InvalidRequest {
                reason: format!("{} 的文件句柄身份不匹配", observed.path),
            });
        }
        let (parent, leaf) = self.open_parent(path)?;
        verify_path_matches_handle(&parent, &leaf, observed)?;
        unlinkat(&parent, &leaf, AtFlags::empty())
            .map_err(|error| map_fs_error(&self.display_path(path), error))?;
        fsync(&parent).map_err(|error| map_fs_error(&self.display_path(path), error))
    }

    /// Purge explicitly requested data while retaining the management root and its lock inode.
    pub fn purge_contents(&self, lock: &crate::home::HomeLock) -> Result<()> {
        self.check_lock(lock)?;
        let locked_ident = lock.locked_identity();
        let mut names = directory_entry_names(&self.root_dir, self.root.as_str())?
            .into_iter()
            .filter(|name| name != "." && name != "..")
            .filter_map(
                |name| match statat(&self.root_dir, &name, AtFlags::SYMLINK_NOFOLLOW) {
                    Ok(stat) if (stat.st_dev as u64, stat.st_ino) == locked_ident => None,
                    Ok(_) => Some(Ok(name)),
                    Err(error) => Some(Err(map_fs_error(&format!("{}/{name}", self.root), error))),
                },
            )
            .collect::<Result<Vec<_>>>()?;
        names.sort_by(|a, b| {
            purge_priority(a)
                .cmp(&purge_priority(b))
                .then_with(|| a.as_bytes().cmp(b.as_bytes()))
        });

        // Validate every top-level tree before changing permissions or deleting any data.
        for name in &names {
            preflight_remove_at(&self.root_dir, name, &format!("{}/{name}", self.root))?;
        }
        fchmod(&self.root_dir, Mode::from_raw_mode(0o700))
            .map_err(|e| map_fs_error(self.root.as_str(), e))?;
        if let Err(error) = names.iter().try_for_each(|name| {
            let stat = statat(&self.root_dir, name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|e| map_fs_error(&format!("{}/{name}", self.root), e))?;
            if FileType::from_raw_mode(stat.st_mode) == FileType::Directory {
                make_directories_writable(&self.root_dir, name, &format!("{}/{name}", self.root))?;
            }
            Ok::<_, Error>(())
        }) {
            return Err(Error::io(
                self.root.as_str(),
                std::io::Error::other(format!(
                    "purge删除前权限预检未完成，尚未删除数据；请检查并可重试：{error}"
                )),
            ));
        }

        let mut removed_roots = Vec::new();
        for name in &names {
            let display = format!("{}/{name}", self.root);
            if let Err(error) = remove_at(&self.root_dir, name, &display) {
                return Err(partial_purge_error(&self.root, name, &removed_roots, error));
            }
            removed_roots.push(name.clone());
            if let Err(error) =
                crate::failpoint::rendezvous("purge_after_top_level_delete", self.root.as_str())
            {
                return Err(partial_purge_error(
                    &self.root,
                    name,
                    &removed_roots,
                    Error::io(self.root.as_str(), error),
                ));
            }
        }
        crate::failpoint::rendezvous("purge_before_final_rescan", self.root.as_str()).map_err(
            |error| {
                partial_purge_error(
                    &self.root,
                    ".",
                    &removed_roots,
                    Error::io(self.root.as_str(), error),
                )
            },
        )?;
        // Read-only SQLite may create store.db-shm without HomeLock (D-039). It can appear
        // after the initial listing, so re-scan once the main Store was removed. No other new
        // root entry is expected; preserve it and report a partial purge instead of guessing.
        for _ in 0..3 {
            let extras = directory_entry_names(&self.root_dir, self.root.as_str())
                .map_err(|error| partial_purge_error(&self.root, ".", &removed_roots, error))?
                .into_iter()
                .filter(|name| name != "." && name != "..")
                .filter_map(
                    |name| match statat(&self.root_dir, &name, AtFlags::SYMLINK_NOFOLLOW) {
                        Ok(stat) if (stat.st_dev as u64, stat.st_ino) == locked_ident => None,
                        Ok(_) => Some(Ok(name)),
                        Err(error) => {
                            Some(Err(map_fs_error(&format!("{}/{name}", self.root), error)))
                        }
                    },
                )
                .collect::<Result<Vec<_>>>()
                .map_err(|error| partial_purge_error(&self.root, ".", &removed_roots, error))?;
            if extras.is_empty() {
                break;
            }
            for name in extras {
                if !name.eq_ignore_ascii_case("store.db-shm") {
                    return Err(partial_purge_error(
                        &self.root,
                        &name,
                        &removed_roots,
                        Error::StoreCorrupt {
                            detail: format!("purge最终复核发现未登记对象 {name}"),
                        },
                    ));
                }
                let display = format!("{}/{name}", self.root);
                preflight_remove_at(&self.root_dir, &name, &display).map_err(|error| {
                    partial_purge_error(&self.root, &name, &removed_roots, error)
                })?;
                if let Err(error) = remove_at(&self.root_dir, &name, &display) {
                    return Err(partial_purge_error(
                        &self.root,
                        &name,
                        &removed_roots,
                        error,
                    ));
                }
                removed_roots.push(name);
                fsync(&self.root_dir).map_err(|error| {
                    partial_purge_error(
                        &self.root,
                        "store.db-shm",
                        &removed_roots,
                        map_fs_error(self.root.as_str(), error),
                    )
                })?;
            }
        }
        let remaining = directory_entry_names(&self.root_dir, self.root.as_str())
            .map_err(|error| partial_purge_error(&self.root, ".", &removed_roots, error))?
            .into_iter()
            .filter(|name| name != "." && name != "..")
            .filter_map(
                |name| match statat(&self.root_dir, &name, AtFlags::SYMLINK_NOFOLLOW) {
                    Ok(stat) if (stat.st_dev as u64, stat.st_ino) == locked_ident => None,
                    Ok(_) => Some(Ok(name)),
                    Err(error) => Some(Err(map_fs_error(&format!("{}/{name}", self.root), error))),
                },
            )
            .collect::<Result<Vec<_>>>()
            .map_err(|error| partial_purge_error(&self.root, ".", &removed_roots, error))?;
        self.check_lock(lock)
            .map_err(|error| partial_purge_error(&self.root, ".lock", &removed_roots, error))?;
        if let Some(name) = remaining.first() {
            return Err(partial_purge_error(
                &self.root,
                name,
                &removed_roots,
                Error::StoreCorrupt {
                    detail: format!("purge完成后根内仍有对象 {name}"),
                },
            ));
        }
        fsync(&self.root_dir).map_err(|e| {
            partial_purge_error(
                &self.root,
                ".",
                &removed_roots,
                map_fs_error(self.root.as_str(), e),
            )
        })?;
        self.check_lock(lock)
            .map_err(|error| partial_purge_error(&self.root, ".lock", &removed_roots, error))
    }

    pub fn sync_dir_locked(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let dir = self.open_dir(Some(path))?;
        fsync(&dir).map_err(|e| map_fs_error(&self.display_path(path), e))
    }

    pub fn set_readonly(&self, lock: &crate::home::HomeLock, file: &SafeFile) -> Result<()> {
        self.set_mode(lock, file, 0o444, "封存")
    }

    pub fn set_executable(&self, lock: &crate::home::HomeLock, file: &SafeFile) -> Result<()> {
        self.set_mode(lock, file, 0o755, "设为可执行")
    }

    fn set_mode(
        &self,
        lock: &crate::home::HomeLock,
        file: &SafeFile,
        mode: u16,
        operation: &str,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let Some(origin) = &file.managed else {
            return Err(Error::InvalidRequest {
                reason: "不能修改外部只读文件的权限".to_string(),
            });
        };
        if origin.root != self.root || origin.root_ident != self.root_ident {
            return Err(Error::InvalidRequest {
                reason: "文件句柄不属于此管理根".to_string(),
            });
        }
        let current = fstat(&file.file).map_err(|e| map_fs_error(file.path.as_str(), e))?;
        check_regular_stat(file.path.as_str(), &current)?;
        if current.st_dev as u64 != file.meta.dev() || current.st_ino as u64 != file.meta.ino() {
            return Err(Error::InvalidRequest {
                reason: format!("{} 的句柄身份已改变", file.path),
            });
        }
        file.set_mode(mode)?;
        let current = self.open_regular(&origin.rel)?;
        if file.meta.dev() != current.meta.dev() || file.meta.ino() != current.meta.ino() {
            return Err(Error::InvalidRequest {
                reason: format!("{} 在{operation}期间被替换", file.path),
            });
        }
        Ok(())
    }

    pub fn set_tree_readonly(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let dir = self.open_dir(Some(path))?;
        set_dir_tree_mode(
            &dir,
            &self.display_path(path),
            self.root.as_str(),
            0o444,
            0o555,
        )?;
        fchmod(&dir, Mode::from_raw_mode(0o555))
            .map_err(|e| map_fs_error(&self.display_path(path), e))?;
        crate::failpoint::sync_error(self.root.as_str(), "publish_readonly_root_sync").map_err(
            |error| Error::RecoveryRequired {
                path: self.display_path(path),
                detail: format!("只读权限已设置但目录同步失败：{error}"),
            },
        )?;
        fsync(&dir).map_err(|e| map_fs_error(&self.display_path(path), e))
    }

    pub fn make_tree_writable(
        &self,
        lock: &crate::home::HomeLock,
        path: &ManagedRelPath,
    ) -> Result<()> {
        self.check_lock(lock)?;
        let dir = self.open_dir(Some(path))?;
        set_dir_tree_mode(
            &dir,
            &self.display_path(path),
            self.root.as_str(),
            0o644,
            0o755,
        )?;
        fchmod(&dir, Mode::from_raw_mode(0o755))
            .map_err(|e| map_fs_error(&self.display_path(path), e))?;
        fsync(&dir).map_err(|e| map_fs_error(&self.display_path(path), e))
    }
}

fn directory_entry_names(directory: &std::fs::File, display: &str) -> Result<Vec<String>> {
    Dir::read_from(directory)
        .map_err(|e| map_fs_error(display, e))?
        .map(|entry| {
            let entry = entry.map_err(|e| Error::io(display, e.into()))?;
            std::str::from_utf8(entry.file_name().to_bytes())
                .map(str::to_string)
                .map_err(|_| Error::InvalidRequest {
                    reason: format!("{display} 下有非UTF-8名称，拒绝清理"),
                })
        })
        .collect()
}

fn sync_dir_tree(
    directory: &std::fs::File,
    display: &str,
    root: &str,
    is_root: bool,
) -> Result<()> {
    for name in directory_entry_names(directory, display)? {
        if name == "." || name == ".." {
            continue;
        }
        let child_display = format!("{display}/{name}");
        let stat = statat(directory, &name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| map_fs_error(&child_display, error))?;
        match FileType::from_raw_mode(stat.st_mode) {
            FileType::Directory => {
                let child = open_directory_at(directory, &name, &child_display)?;
                sync_dir_tree(&child, &child_display, root, false)?;
            }
            FileType::RegularFile => {
                check_regular_stat(&child_display, &stat)?;
                let path = AbsPath::new(child_display.clone()).map_err(Error::Core)?;
                let file = open_regular_at(directory, &name, &child_display, path)?;
                crate::failpoint::sync_error(root, "publish_file_sync")
                    .map_err(|error| Error::io(&child_display, error))?;
                fsync(&file.file).map_err(|error| map_fs_error(&child_display, error))?;
            }
            FileType::Symlink => {
                return Err(Error::InvalidRequest {
                    reason: format!("{child_display} 是符号链接"),
                });
            }
            _ => {
                return Err(Error::InvalidRequest {
                    reason: format!("{child_display} 是特殊文件"),
                });
            }
        }
    }
    let point = if is_root {
        "publish_tree_root_sync"
    } else {
        "publish_tree_nested_dir_sync"
    };
    crate::failpoint::sync_error(root, point).map_err(|error| Error::io(display, error))?;
    fsync(directory).map_err(|error| map_fs_error(display, error))
}

fn check_rename_source(parent: &std::fs::File, leaf: &str, display: &str) -> Result<()> {
    let stat =
        statat(parent, leaf, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| map_fs_error(display, e))?;
    let kind = FileType::from_raw_mode(stat.st_mode);
    if kind == FileType::Symlink
        || (kind == FileType::RegularFile && stat.st_nlink != 1)
        || !matches!(kind, FileType::RegularFile | FileType::Directory)
    {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 不是可受管移动对象"),
        });
    }
    Ok(())
}

fn sync_rename_parents(
    source_parent: &std::fs::File,
    target_parent: &std::fs::File,
    source_display: &str,
    target_display: &str,
    root: &str,
) -> Result<()> {
    crate::failpoint::sync_error(root, "publish_source_parent_sync").map_err(|error| {
        Error::RecoveryRequired {
            path: source_display.to_string(),
            detail: format!("rename已生效但源目录同步失败：{error}"),
        }
    })?;
    fsync(source_parent).map_err(|error| Error::RecoveryRequired {
        path: source_display.to_string(),
        detail: format!("rename已生效但源目录同步失败：{error}"),
    })?;
    crate::failpoint::sync_error(root, "publish_target_parent_sync").map_err(|error| {
        Error::RecoveryRequired {
            path: target_display.to_string(),
            detail: format!("rename已生效但目标目录同步失败：{error}"),
        }
    })?;
    fsync(target_parent).map_err(|error| Error::RecoveryRequired {
        path: target_display.to_string(),
        detail: format!("rename已生效但目标目录同步失败：{error}"),
    })
}

/// 用户显式来源路径的只读句柄。它不提供写入、删除或改权限的方法。
#[derive(Debug)]
pub struct ExternalReadFile(SafeFile);

impl ExternalReadFile {
    pub fn open_regular(path: &AbsPath) -> Result<Self> {
        let parent = path
            .as_path()
            .parent()
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{path} 没有父目录"),
            })?;
        let canonical_parent =
            std::fs::canonicalize(parent).map_err(|e| map_std_error(path.as_str(), e))?;
        let parent =
            AbsPath::new(canonical_parent.to_string_lossy().into_owned()).map_err(Error::Core)?;
        let fs = ManagedFs::open_root(&parent)?;
        let leaf = path
            .as_path()
            .file_name()
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{path} 没有文件名"),
            })?;
        Ok(Self(open_regular_at(
            &fs.root_dir,
            leaf,
            path.as_str(),
            path.clone(),
        )?))
    }

    pub(crate) fn open_regular_no_follow(path: &AbsPath) -> Result<Self> {
        let parent = path
            .as_path()
            .parent()
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{path} 没有父目录"),
            })?;
        let parent = AbsPath::new(parent.to_string()).map_err(Error::Core)?;
        let parent = canonical_external_root(&parent)?;
        let fs = ManagedFs::open_root(&parent)?;
        let leaf = path
            .as_path()
            .file_name()
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{path} 没有文件名"),
            })?;
        Ok(Self(open_regular_at(
            &fs.root_dir,
            leaf,
            path.as_str(),
            path.clone(),
        )?))
    }

    pub fn open_optional(path: &AbsPath) -> Result<Option<Self>> {
        match Self::open_regular(path) {
            Ok(file) => Ok(Some(file)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn read_bounded(&self, max_bytes: u64) -> Result<Vec<u8>> {
        self.0.read_bounded(max_bytes)
    }

    pub fn sha256_bounded(&self, max_bytes: u64) -> Result<(Sha256Hex, u64)> {
        self.0.sha256_bounded(max_bytes)
    }

    pub fn metadata(&self) -> &std::fs::Metadata {
        self.0.metadata()
    }
}

/// A directory tree pinned to a no-follow root handle. The first pass records only names and
/// object identities; content is opened and read only when a caller consumes each entry.
#[derive(Debug)]
pub(crate) struct ExternalReadTree {
    root: AbsPath,
    root_dir: std::fs::File,
    directories: std::collections::BTreeMap<String, (u64, u64)>,
    files: Vec<ExternalTreeFile>,
}

#[derive(Debug)]
pub(crate) struct ExternalTreeFileHandle(std::fs::File);

impl Read for ExternalTreeFileHandle {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buffer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalTreeFile {
    pub(crate) relative: RelPath,
    pub(crate) bytes: u64,
    parent: String,
    name: String,
    device: u64,
    inode: u64,
}

impl ExternalReadTree {
    pub(crate) fn open(root: &AbsPath) -> Result<Self> {
        let canonical_root = canonical_external_root(root)?;
        let fs = ManagedFs::open_root(&canonical_root)?;
        Self::from_root(canonical_root, fs.root_dir)
    }

    pub(crate) fn open_managed(home: &crate::home::Home, root: &AbsPath) -> Result<Self> {
        let relative = ManagedRelPath::new(home.to_rel(root)?)?;
        let fs = ManagedFs::open_existing(home)?;
        let root_dir = fs.open_tree_root(&relative)?;
        Self::from_root(root.clone(), root_dir)
    }

    fn from_root(root: AbsPath, root_dir: std::fs::File) -> Result<Self> {
        let meta = root_dir
            .metadata()
            .map_err(|e| Error::io(root.as_str(), e))?;
        let mut tree = Self {
            root: root.clone(),
            root_dir,
            directories: std::collections::BTreeMap::from([(
                "".to_string(),
                (meta.dev(), meta.ino()),
            )]),
            files: Vec::new(),
        };
        let root_fd = tree
            .root_dir
            .try_clone()
            .map_err(|e| Error::io(root.as_str(), e))?;
        tree.collect(&root_fd, "")?;
        tree.files.sort_by(|a, b| {
            a.relative
                .as_str()
                .as_bytes()
                .cmp(b.relative.as_str().as_bytes())
        });
        Ok(tree)
    }

    pub(crate) fn files(&self) -> &[ExternalTreeFile] {
        &self.files
    }

    pub(crate) fn validate_sizes(&self) -> Result<u64> {
        let mut total = 0u64;
        for file in &self.files {
            if file.bytes > MAX_FILE_BYTES {
                return Err(Error::InvalidRequest {
                    reason: format!(
                        "{} 下 {} 超过 {MAX_FILE_BYTES} 字节",
                        self.root, file.relative
                    ),
                });
            }
            total = total
                .checked_add(file.bytes)
                .filter(|sum| *sum <= MAX_TOTAL_BYTES)
                .ok_or_else(|| Error::InvalidRequest {
                    reason: format!("{} 总量超过 {MAX_TOTAL_BYTES} 字节", self.root),
                })?;
        }
        Ok(total)
    }

    pub(crate) fn root(&self) -> &AbsPath {
        &self.root
    }

    pub(crate) fn directories(&self) -> impl Iterator<Item = &str> {
        self.directories
            .keys()
            .filter(|path| !path.is_empty())
            .map(String::as_str)
    }

    pub(crate) fn validate_unchanged(&self) -> Result<()> {
        let root_dir = self
            .root_dir
            .try_clone()
            .map_err(|e| Error::io(self.root.as_str(), e))?;
        let mut current = Self {
            root: self.root.clone(),
            root_dir,
            directories: std::collections::BTreeMap::new(),
            files: Vec::new(),
        };
        let root_meta = current
            .root_dir
            .metadata()
            .map_err(|e| Error::io(self.root.as_str(), e))?;
        current
            .directories
            .insert(String::new(), (root_meta.dev(), root_meta.ino()));
        let current_root = current
            .root_dir
            .try_clone()
            .map_err(|e| Error::io(self.root.as_str(), e))?;
        current.collect(&current_root, "")?;
        current.files.sort_by(|a, b| {
            a.relative
                .as_str()
                .as_bytes()
                .cmp(b.relative.as_str().as_bytes())
        });
        if current.directories != self.directories || current.files != self.files {
            return Err(Error::InvalidRequest {
                reason: format!("{} 在读取期间目录项或对象身份改变", self.root),
            });
        }
        Ok(())
    }

    pub(crate) fn open_file(&self, entry: &ExternalTreeFile) -> Result<ExternalTreeFileHandle> {
        let index = self.files.binary_search_by(|file| {
            file.relative
                .as_str()
                .as_bytes()
                .cmp(entry.relative.as_str().as_bytes())
        });
        if !index.is_ok_and(|index| self.files[index] == *entry) {
            return Err(Error::InvalidRequest {
                reason: "目录树条目不属于此快照".to_string(),
            });
        }
        let mut current = self
            .root_dir
            .try_clone()
            .map_err(|e| Error::io(self.root.as_str(), e))?;
        let mut prefix = String::new();
        if !entry.parent.is_empty() {
            for segment in entry.parent.split('/') {
                prefix = if prefix.is_empty() {
                    segment.to_string()
                } else {
                    format!("{prefix}/{segment}")
                };
                let child =
                    open_directory_at(&current, segment, &format!("{}/{prefix}", self.root))?;
                let meta = child
                    .metadata()
                    .map_err(|e| Error::io(format!("{}/{prefix}", self.root), e))?;
                if self.directories.get(&prefix) != Some(&(meta.dev(), meta.ino())) {
                    return Err(Error::InvalidRequest {
                        reason: format!("{}/{prefix} 在遍历期间被替换", self.root),
                    });
                }
                current = child;
            }
        }
        let display = format!("{}/{}", self.root, entry.relative);
        let stat = statat(&current, &entry.name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| map_fs_error(&display, e))?;
        check_regular_stat(&display, &stat)?;
        if stat.st_dev as u64 != entry.device
            || stat.st_ino != entry.inode
            || stat.st_size as u64 != entry.bytes
        {
            return Err(Error::InvalidRequest {
                reason: format!("{display} 在读取前被替换或改变"),
            });
        }
        let fd = openat(
            &current,
            &entry.name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|e| map_fs_error(&display, e))?;
        let opened = fstat(&fd).map_err(|e| map_fs_error(&display, e))?;
        check_regular_stat(&display, &opened)?;
        if opened.st_dev as u64 != entry.device
            || opened.st_ino != entry.inode
            || opened.st_size as u64 != entry.bytes
        {
            return Err(Error::InvalidRequest {
                reason: format!("{display} 在打开期间被替换或改变"),
            });
        }
        Ok(ExternalTreeFileHandle(std::fs::File::from(fd)))
    }

    pub(crate) fn read_file(&self, relative: &RelPath, max_bytes: u64) -> Result<Vec<u8>> {
        let index = self.files.binary_search_by(|file| {
            file.relative
                .as_str()
                .as_bytes()
                .cmp(relative.as_str().as_bytes())
        });
        let entry = index
            .ok()
            .and_then(|index| self.files.get(index))
            .filter(|entry| entry.relative == *relative)
            .ok_or_else(|| Error::NotFound {
                what: format!("{}/{}", self.root, relative),
            })?;
        if entry.bytes > max_bytes {
            return Err(Error::InvalidRequest {
                reason: format!("{}/{} 超过 {max_bytes} 字节", self.root, relative),
            });
        }
        let file = self.open_file(entry)?;
        let mut bytes = Vec::with_capacity(entry.bytes.min(1 << 20) as usize);
        file.take(max_bytes + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| Error::io(relative.as_str(), e))?;
        if bytes.len() as u64 != entry.bytes {
            return Err(Error::InvalidRequest {
                reason: format!("{}/{} 在读取期间改变", self.root, relative),
            });
        }
        Ok(bytes)
    }

    fn collect(&mut self, dir: &std::fs::File, relative: &str) -> Result<()> {
        let display = if relative.is_empty() {
            self.root.to_string()
        } else {
            format!("{}/{relative}", self.root)
        };
        let entries = Dir::read_from(dir).map_err(|e| map_fs_error(&display, e))?;
        for item in entries {
            let item = item.map_err(|e| Error::io(&display, e.into()))?;
            let raw_name = item.file_name().to_bytes();
            if raw_name == b"." || raw_name == b".." {
                continue;
            }
            let name = tree_entry_name(raw_name, &display)?;
            let child_rel = if relative.is_empty() {
                name.clone()
            } else {
                format!("{relative}/{name}")
            };
            let child_display = format!("{}/{child_rel}", self.root);
            if HOST_METADATA_FILES.contains(&name.as_str()) {
                return Err(Error::InvalidRequest {
                    reason: format!(
                        "{child_display} 是宿主元数据文件（如 Finder 生成），先清理再装"
                    ),
                });
            }
            let stat = statat(dir, &name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|e| map_fs_error(&child_display, e))?;
            match FileType::from_raw_mode(stat.st_mode) {
                FileType::Directory => {
                    let child = open_directory_at(dir, &name, &child_display)?;
                    let meta = child.metadata().map_err(|e| Error::io(&child_display, e))?;
                    self.directories
                        .insert(child_rel.clone(), (meta.dev(), meta.ino()));
                    self.collect(&child, &child_rel)?;
                }
                FileType::RegularFile => {
                    check_regular_stat(&child_display, &stat)?;
                    let relative_path = RelPath::new(child_rel.clone()).map_err(Error::Core)?;
                    let parent = if relative.is_empty() {
                        String::new()
                    } else {
                        relative.to_string()
                    };
                    self.files.push(ExternalTreeFile {
                        relative: relative_path,
                        bytes: stat.st_size as u64,
                        parent,
                        name,
                        device: stat.st_dev as u64,
                        inode: stat.st_ino as u64,
                    });
                }
                FileType::Symlink => {
                    return Err(Error::InvalidRequest {
                        reason: format!("{child_display} 是符号链接"),
                    });
                }
                _ => {
                    return Err(Error::InvalidRequest {
                        reason: format!("{child_display} 不是普通文件或目录"),
                    });
                }
            }
        }
        Ok(())
    }
}

fn tree_entry_name(raw: &[u8], display: &str) -> Result<String> {
    std::str::from_utf8(raw)
        .map(str::to_string)
        .map_err(|_| Error::InvalidRequest {
            reason: format!("{display} 下有非UTF-8名称"),
        })
}

fn canonical_external_root(root: &AbsPath) -> Result<AbsPath> {
    let mut value = crate::request::lexical_abs(root.as_str());
    for (alias, target) in [("/var/", "/private/var/"), ("/tmp/", "/private/tmp/")] {
        if !value.starts_with(alias) {
            continue;
        }
        let alias_root = alias.trim_end_matches('/');
        let target_root = target.trim_end_matches('/');
        if std::fs::read_link(alias_root).is_ok_and(|link| {
            let resolved = if link.is_absolute() {
                link
            } else {
                std::path::Path::new("/").join(link)
            };
            resolved == std::path::Path::new(target_root)
        }) {
            value = format!("{target}{}", &value[alias.len()..]);
        }
        break;
    }
    AbsPath::new(value).map_err(Error::Core)
}

#[cfg(test)]
mod tree_tests {
    use super::*;

    // Task: C002-T21
    #[test]
    fn directory_reader_rejects_non_utf8_entry_names() {
        assert!(
            matches!(tree_entry_name(b"bad-\xff", "/source"), Err(Error::InvalidRequest { reason }) if reason.contains("非UTF-8"))
        );
        assert_eq!(
            tree_entry_name(b"valid-name", "/source").unwrap(),
            "valid-name"
        );
    }

    // Task: C002-T21
    #[test]
    fn alias_normalization_is_limited_to_os_temp_roots() {
        let arbitrary = AbsPath::new("/var-link/tree".to_string()).unwrap();
        assert_eq!(canonical_external_root(&arbitrary).unwrap(), arbitrary);
    }
}

#[cfg(test)]
mod seal_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    fn managed_file() -> (
        tempfile::TempDir,
        crate::home::Home,
        crate::home::HomeLock,
        ManagedFs,
        ManagedRelPath,
        SafeFile,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let root = AbsPath::new(dir.path().to_str().unwrap()).unwrap();
        let home = crate::home::Home::at(root);
        let lock = home.acquire_lock().unwrap();
        let fs = ManagedFs::open_existing(&home).unwrap();
        let path = ManagedRelPath::new("works/w1/output.md").unwrap();
        fs.ensure_dir(&lock, &ManagedRelPath::new("works/w1").unwrap())
            .unwrap();
        fs.write_new(&lock, &path, b"committed bytes").unwrap();
        let file = fs.open_regular(&path).unwrap();
        (dir, home, lock, fs, path, file)
    }

    // Task: C002-T22
    #[test]
    fn seal_refuses_same_inode_content_change_before_chmod() {
        let (_dir, home, _lock, _fs, path, held) = managed_file();
        let (expected, bytes) = held.sha256_bounded(MAX_FILE_BYTES).unwrap();
        let absolute = home.root().as_path().join(path.as_str());
        std::fs::write(&absolute, b"tampered bytes!").unwrap();
        let changed = std::fs::metadata(&absolute).unwrap();
        assert_eq!(changed.ino(), held.meta.ino(), "仍是原inode");
        assert_eq!(changed.len(), bytes, "仍是原长度");

        assert!(
            held.verify_seal_reference(expected.as_str(), bytes)
                .is_err()
        );
        assert_ne!(
            std::fs::metadata(absolute).unwrap().permissions().mode() & 0o222,
            0,
            "内容不符必须在fchmod前停止"
        );
    }

    // Task: C002-T22
    #[test]
    fn seal_refuses_hardlink_added_after_observation() {
        let (_dir, home, _lock, _fs, path, held) = managed_file();
        let (expected, bytes) = held.sha256_bounded(MAX_FILE_BYTES).unwrap();
        let outside = tempfile::tempdir().unwrap();
        let alias = outside.path().join("alias");
        std::fs::hard_link(home.root().as_path().join(path.as_str()), &alias).unwrap();
        let before_mode = std::fs::metadata(&alias).unwrap().permissions().mode();

        assert!(
            held.verify_seal_reference(expected.as_str(), bytes)
                .is_err()
        );
        assert_eq!(
            std::fs::metadata(&alias).unwrap().permissions().mode(),
            before_mode,
            "发现硬链接后不得改变任何别名的权限"
        );
    }

    // Task: C002-T22
    #[test]
    fn path_replacement_seals_observed_object_and_leaves_replacement_untouched() {
        let (_dir, home, lock, fs, path, held) = managed_file();
        let (expected, bytes) = held.sha256_bounded(MAX_FILE_BYTES).unwrap();
        let target = home.root().as_path().join(path.as_str());
        let outside = tempfile::tempdir().unwrap();
        let replacement = outside.path().join("replacement");
        let moved_original = outside.path().join("observed-original");
        std::fs::rename(&target, &moved_original).unwrap();
        std::fs::write(&replacement, b"outside sentinel").unwrap();
        let replacement_before = replacement.metadata().unwrap().permissions().mode();
        std::os::unix::fs::symlink(&replacement, &target).unwrap();

        assert!(fs.set_readonly(&lock, &held).is_err());
        assert_eq!(
            moved_original.metadata().unwrap().permissions().mode() & 0o777,
            0o444
        );
        assert_eq!(std::fs::read(&replacement).unwrap(), b"outside sentinel");
        assert_eq!(
            replacement.metadata().unwrap().permissions().mode(),
            replacement_before
        );
        assert_eq!(
            held.sha256_bounded(MAX_FILE_BYTES).unwrap(),
            (expected, bytes)
        );
    }
}

pub(crate) fn open_managed_regular(home: &crate::home::Home, path: &AbsPath) -> Result<SafeFile> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    ManagedFs::open_existing(home)?.open_regular(&rel)
}

pub(crate) fn open_managed_optional(
    home: &crate::home::Home,
    path: &AbsPath,
) -> Result<Option<SafeFile>> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    ManagedFs::open_existing(home)?.open_optional(&rel)
}

pub(crate) fn open_managed_optional_locked(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
) -> Result<Option<SafeFile>> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    ManagedFs::open_existing(home)?.open_optional_locked(lock, &rel)
}

pub(crate) fn remove_managed_file_if_same(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
    observed: &SafeFile,
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    ManagedFs::open_existing(home)?.remove_regular_file_if_same(lock, &rel, observed)
}

pub(crate) fn verify_managed_file_bound(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
    observed: &SafeFile,
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    let fs = ManagedFs::open_existing(home)?;
    fs.check_lock(lock)?;
    let origin = observed
        .managed
        .as_ref()
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("{} 不是管理根内文件", observed.path),
        })?;
    if origin.root != fs.root || origin.root_ident != fs.root_ident || origin.rel != rel {
        return Err(Error::InvalidRequest {
            reason: format!("{} 的受管来源身份不匹配", observed.path),
        });
    }
    let (parent, leaf) = fs.open_parent(&rel)?;
    verify_path_matches_handle(&parent, &leaf, observed)
}

pub(crate) fn validate_store_files(home: &crate::home::Home) -> Result<()> {
    let fs = ManagedFs::open_existing(home)?;
    let main = ManagedRelPath::new("store.db")?;
    let main_exists = fs.open_optional(&main)?.is_some();
    let mut sidecar_exists = false;
    for name in ["store.db-wal", "store.db-shm", "store.db-journal"] {
        sidecar_exists |= fs.validate_sqlite_control_file(&ManagedRelPath::new(name)?)?;
    }
    if !main_exists && sidecar_exists {
        return Err(Error::StoreCorrupt {
            detail: "store.db不存在但仍有SQLite控制文件；拒绝建立新Store".into(),
        });
    }
    Ok(())
}

pub(crate) fn validate_store_files_locked(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
) -> Result<()> {
    let fs = ManagedFs::open_existing(home)?;
    let main = ManagedRelPath::new("store.db")?;
    let main_exists = fs.open_optional_locked(lock, &main)?.is_some();
    let mut sidecar_exists = false;
    for name in ["store.db-wal", "store.db-shm", "store.db-journal"] {
        fs.check_lock(lock)?;
        sidecar_exists |= fs.validate_sqlite_control_file(&ManagedRelPath::new(name)?)?;
    }
    if !main_exists && sidecar_exists {
        return Err(Error::StoreCorrupt {
            detail: "store.db不存在但仍有SQLite控制文件；拒绝建立新Store".into(),
        });
    }
    Ok(())
}

fn map_fs_error(path: &str, errno: rustix::io::Errno) -> Error {
    let error = std::io::Error::from(errno);
    if error.kind() == std::io::ErrorKind::NotFound {
        Error::NotFound {
            what: path.to_string(),
        }
    } else {
        Error::io(path, error)
    }
}

fn check_regular_stat(path: &str, stat: &rustix::fs::Stat) -> Result<()> {
    let file_type = FileType::from_raw_mode(stat.st_mode);
    if file_type == FileType::Symlink {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 是符号链接"),
        });
    }
    if file_type != FileType::RegularFile {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 不是普通文件"),
        });
    }
    if stat.st_nlink != 1 {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 是硬链接（nlink = {}）", stat.st_nlink),
        });
    }
    Ok(())
}

fn stat_matches_handle(stat: &rustix::fs::Stat, file: &SafeFile) -> bool {
    use std::os::unix::fs::MetadataExt as _;
    FileType::from_raw_mode(stat.st_mode) == FileType::RegularFile
        && stat.st_nlink == 1
        && stat.st_dev as u64 == file.meta.dev()
        && stat.st_ino == file.meta.ino()
}

fn verify_path_matches_handle(parent: &std::fs::File, leaf: &str, file: &SafeFile) -> Result<()> {
    let stat = statat(parent, leaf, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|e| map_fs_error(file.path.as_str(), e))?;
    if !stat_matches_handle(&stat, file) {
        return Err(Error::InvalidRequest {
            reason: format!("{} 与已验证候选句柄不是同一对象", file.path),
        });
    }
    Ok(())
}

fn verify_candidate_handle(file: &SafeFile, expected: &[u8]) -> Result<()> {
    let stat = fstat(&file.file).map_err(|e| map_fs_error(file.path.as_str(), e))?;
    check_regular_stat(file.path.as_str(), &stat)?;
    if !stat_matches_handle(&stat, file) || file.read_bounded(MAX_FILE_BYTES)? != expected {
        return Err(Error::InvalidRequest {
            reason: format!("{} 与摘要校验过的候选内容或身份不符", file.path),
        });
    }
    Ok(())
}

fn map_std_error(path: &str, error: std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        Error::NotFound {
            what: path.to_string(),
        }
    } else {
        Error::io(path, error)
    }
}

fn open_regular_at(
    parent: &std::fs::File,
    leaf: &str,
    display: &str,
    path: AbsPath,
) -> Result<SafeFile> {
    let before =
        statat(parent, leaf, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| map_fs_error(display, e))?;
    check_regular_stat(display, &before)?;
    let fd = openat(
        parent,
        leaf,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| map_fs_error(display, e))?;
    let after = fstat(&fd).map_err(|e| map_fs_error(display, e))?;
    check_regular_stat(display, &after)?;
    if before.st_dev != after.st_dev || before.st_ino != after.st_ino {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 在打开期间被替换"),
        });
    }
    let file = std::fs::File::from(fd);
    let meta = file.metadata().map_err(|e| Error::io(display, e))?;
    Ok(SafeFile {
        file,
        path,
        meta,
        managed: None,
    })
}

fn open_directory_at(parent: &std::fs::File, name: &str, display: &str) -> Result<std::fs::File> {
    let before =
        statat(parent, name, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| map_fs_error(display, e))?;
    if FileType::from_raw_mode(before.st_mode) == FileType::Symlink {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 是符号链接"),
        });
    }
    if FileType::from_raw_mode(before.st_mode) != FileType::Directory {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 不是目录"),
        });
    }
    let fd = openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| map_fs_error(display, e))?;
    let after = fstat(&fd).map_err(|e| map_fs_error(display, e))?;
    if after.st_dev != before.st_dev || after.st_ino != before.st_ino {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 在打开期间被替换"),
        });
    }
    Ok(std::fs::File::from(fd))
}

fn remove_at(parent: &std::fs::File, leaf: &str, display: &str) -> Result<()> {
    let stat =
        statat(parent, leaf, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| map_fs_error(display, e))?;
    let ty = FileType::from_raw_mode(stat.st_mode);
    if ty == FileType::Symlink {
        unlinkat(parent, leaf, AtFlags::empty()).map_err(|e| map_fs_error(display, e))?;
        return Ok(());
    }
    if ty == FileType::RegularFile {
        if stat.st_nlink != 1 {
            return Err(Error::InvalidRequest {
                reason: format!("{display} 是硬链接（nlink = {}）", stat.st_nlink),
            });
        }
        unlinkat(parent, leaf, AtFlags::empty()).map_err(|e| map_fs_error(display, e))?;
        return Ok(());
    }
    if ty != FileType::Directory {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 是特殊文件，拒绝删除"),
        });
    }
    let fd = openat(
        parent,
        leaf,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| map_fs_error(display, e))?;
    let dir = std::fs::File::from(fd);
    let opened = fstat(&dir).map_err(|e| map_fs_error(display, e))?;
    if opened.st_dev != stat.st_dev || opened.st_ino != stat.st_ino {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 在删除期间被替换"),
        });
    }
    let entries = Dir::read_from(&dir).map_err(|e| map_fs_error(display, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| map_fs_error(display, e))?;
        let name = entry.file_name().to_bytes();
        if name == b"." || name == b".." {
            continue;
        }
        let name = std::str::from_utf8(name).map_err(|_| Error::InvalidRequest {
            reason: format!("{display} 下有非UTF-8文件名，拒绝删除"),
        })?;
        remove_at(&dir, name, &format!("{display}/{name}"))?;
    }
    drop(dir);
    unlinkat(parent, leaf, AtFlags::REMOVEDIR).map_err(|e| map_fs_error(display, e))?;
    Ok(())
}

fn unlink_created_at(
    parent: &std::fs::File,
    leaf: &str,
    display: &str,
    created: &std::fs::Metadata,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt as _;
    let current = statat(parent, leaf, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| map_fs_error(display, error))?;
    check_regular_stat(display, &current)?;
    if current.st_dev as u64 != created.dev() || current.st_ino as u64 != created.ino() {
        return Err(Error::InvalidRequest {
            reason: format!("{display} 已不再绑定本次创建的inode"),
        });
    }
    unlinkat(parent, leaf, AtFlags::empty()).map_err(|error| map_fs_error(display, error))?;
    fsync(parent).map_err(|error| map_fs_error(display, error))
}

fn preflight_remove_at(parent: &std::fs::File, leaf: &str, display: &str) -> Result<()> {
    let stat =
        statat(parent, leaf, AtFlags::SYMLINK_NOFOLLOW).map_err(|e| map_fs_error(display, e))?;
    match FileType::from_raw_mode(stat.st_mode) {
        FileType::Symlink => Ok(()),
        FileType::RegularFile if stat.st_nlink == 1 => Ok(()),
        FileType::RegularFile => Err(Error::InvalidRequest {
            reason: format!(
                "{display} 是硬链接（nlink = {}），拒绝 purge",
                stat.st_nlink
            ),
        }),
        FileType::Directory => {
            let directory = open_directory_at(parent, leaf, display)?;
            let entries = Dir::read_from(&directory).map_err(|e| map_fs_error(display, e))?;
            for entry in entries {
                let entry = entry.map_err(|e| Error::io(display, e.into()))?;
                let bytes = entry.file_name().to_bytes();
                if bytes == b"." || bytes == b".." {
                    continue;
                }
                let child = std::str::from_utf8(bytes).map_err(|_| Error::InvalidRequest {
                    reason: format!("{display} 下有非UTF-8名称，拒绝 purge"),
                })?;
                preflight_remove_at(&directory, child, &format!("{display}/{child}"))?;
            }
            Ok(())
        }
        _ => Err(Error::InvalidRequest {
            reason: format!("{display} 是特殊文件，拒绝 purge"),
        }),
    }
}

fn purge_priority(name: &str) -> u8 {
    if name.eq_ignore_ascii_case("store.db") {
        5
    } else if ["store.db-wal", "store.db-shm", "store.db-journal"]
        .iter()
        .any(|control| name.eq_ignore_ascii_case(control))
    {
        4
    } else if name == "pending" {
        1
    } else if name == "tmp" {
        2
    } else if name == "bin" {
        3
    } else {
        0
    }
}

fn partial_purge_error(root: &AbsPath, failed: &str, removed: &[String], error: Error) -> Error {
    let removed = if removed.is_empty() {
        "无".to_string()
    } else {
        removed.join(", ")
    };
    Error::io(
        format!("{root}/{failed}"),
        std::io::Error::other(format!(
            "purge部分清理失败；已完整删除顶层对象：{removed}；当前顶层对象 {failed} 可能已部分清理；错误：{error}；修复问题后可重试"
        )),
    )
}

/// 从受管目录句柄打开的普通文件。路径后续替换不会改变此句柄指向的对象。
#[derive(Debug)]
pub struct SafeFile {
    file: std::fs::File,
    path: AbsPath,
    meta: std::fs::Metadata,
    managed: Option<ManagedOrigin>,
}

#[derive(Debug)]
struct ManagedOrigin {
    root: AbsPath,
    root_ident: (u64, u64),
    rel: ManagedRelPath,
}

impl SafeFile {
    /// 句柄元数据（句柄上的事实，不是路径上的）。
    pub fn metadata(&self) -> &std::fs::Metadata {
        &self.meta
    }

    pub fn path(&self) -> &AbsPath {
        &self.path
    }

    /// 流式读取全部内容，先按句柄大小核对上限，读到的字节数必须与句柄元数据一致
    ///（读取间增长或收缩都拒绝）。每次读都从对象起点开始，同一句柄可重复读取。
    pub fn read_bounded(&self, max_bytes: u64) -> Result<Vec<u8>> {
        if self.meta.len() > max_bytes {
            return Err(Error::InvalidRequest {
                reason: format!("{} 超过 {} 字节", self.path, max_bytes),
            });
        }
        let mut buf = Vec::with_capacity(self.meta.len().min(1 << 20) as usize);
        let mut chunk = vec![0u8; 64 * 1024];
        let mut handle = &self.file;
        handle
            .rewind()
            .map_err(|e| Error::io(self.path.as_str(), e))?;
        loop {
            let n = handle
                .read(&mut chunk)
                .map_err(|e| Error::io(self.path.as_str(), e))?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if buf.len() as u64 > max_bytes {
                return Err(Error::InvalidRequest {
                    reason: format!("{} 在读取间增长，超过 {} 字节", self.path, max_bytes),
                });
            }
        }
        if buf.len() as u64 != self.meta.len() {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "{} 实际 {} 字节与句柄元数据 {} 不符（读取间变化）",
                    self.path,
                    buf.len(),
                    self.meta.len()
                ),
            });
        }
        Ok(buf)
    }

    /// 同一对象上的摘要与字节数（先限额再流式）。
    pub fn sha256_bounded(&self, max_bytes: u64) -> Result<(Sha256Hex, u64)> {
        let bytes = self.read_bounded(max_bytes)?;
        let n = bytes.len() as u64;
        Ok((Sha256Hex::of_bytes(&bytes), n))
    }

    pub(crate) fn verify_seal_reference(&self, sha256: &str, bytes: u64) -> Result<()> {
        let before = fstat(&self.file).map_err(|e| map_fs_error(self.path.as_str(), e))?;
        check_regular_stat(self.path.as_str(), &before)?;
        if before.st_dev as u64 != self.meta.dev()
            || before.st_ino as u64 != self.meta.ino()
            || before.st_size as u64 != bytes
        {
            return Err(Error::StoreCorrupt {
                detail: format!("封存句柄 {} 的对象身份或长度已改变", self.path),
            });
        }
        let (actual, actual_bytes) = self.sha256_bounded(MAX_FILE_BYTES)?;
        if actual.as_str() != sha256 || actual_bytes != bytes {
            return Err(Error::StoreCorrupt {
                detail: format!("封存句柄 {} 的内容与提交引用不符", self.path),
            });
        }
        let after = fstat(&self.file).map_err(|e| map_fs_error(self.path.as_str(), e))?;
        check_regular_stat(self.path.as_str(), &after)?;
        if after.st_dev != before.st_dev
            || after.st_ino != before.st_ino
            || after.st_size as u64 != bytes
        {
            return Err(Error::StoreCorrupt {
                detail: format!("封存句柄 {} 在校验期间改变", self.path),
            });
        }
        Ok(())
    }

    /// 在已打开的同一文件对象上置只读并同步。调用方先用同一句柄核验摘要。
    fn set_mode(&self, mode: u16) -> Result<()> {
        fchmod(&self.file, Mode::from_raw_mode(mode))
            .map_err(|e| map_fs_error(self.path.as_str(), e))?;
        fsync(&self.file).map_err(|e| map_fs_error(self.path.as_str(), e))
    }
}

/// 确保根内目录存在。根以目录句柄逐段解析，写入目标不跟随软链。
pub(crate) fn ensure_dirs_under(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    dir: &AbsPath,
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(dir)?)?;
    ManagedFs::open_existing(home)?
        .ensure_dir(lock, &rel)
        .map(|_| ())
}

/// 原子写：在目标同目录**独占创建**唯一临时名（已存在或软链都直接失败，不跟随），
/// 写入并 `fsync` 临时文件，`rename` 到目标，再 `fsync` 父目录使目录项持久。
/// 临时名是随机 UUID，不在路径上可预测，也不保留固定后缀（O01 的 tmp-pending 缺口）。
/// `base` 是可信基点（管理根或其下已验证的目录）；目标父目录在基点之下逐段核对。
pub(crate) fn write_exclusive_atomic(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
    content: &[u8],
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    ManagedFs::open_existing(home)?.write_atomic(lock, &rel, content)
}

/// 同步目录句柄；失败向caller传播，不能把未持久化的目录项报告为已完成。
pub(crate) fn fsync_dir(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    dir: &AbsPath,
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(dir)?)?;
    ManagedFs::open_existing(home)?.sync_dir_locked(lock, &rel)
}

pub(crate) fn managed_directory_exists(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &str,
) -> Result<bool> {
    let rel = ManagedRelPath::new(path)?;
    ManagedFs::open_existing(home)?.directory_exists_locked(lock, &rel)
}

pub(crate) fn open_managed_tree(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &str,
) -> Result<ManagedTree> {
    let rel = ManagedRelPath::new(path)?;
    ManagedFs::open_existing(home)?.open_tree_locked(lock, &rel)
}

pub(crate) fn verify_managed_tree_at(
    home: &crate::home::Home,
    tree: &ManagedTree,
    path: &str,
) -> Result<()> {
    let rel = ManagedRelPath::new(path)?;
    ManagedFs::open_existing(home)?.verify_tree_at(tree, &rel)
}

pub(crate) fn sync_managed_tree(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    tree: &ManagedTree,
) -> Result<()> {
    ManagedFs::open_existing(home)?.sync_managed_tree(lock, tree)
}

pub(crate) fn rename_managed_tree_new(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    tree: &ManagedTree,
    to: &str,
) -> Result<()> {
    let to = ManagedRelPath::new(to)?;
    ManagedFs::open_existing(home)?.rename_tree_new(lock, tree, &to)
}

pub(crate) fn sync_publish_parents(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    from: &str,
    to: &str,
) -> Result<()> {
    let from = ManagedRelPath::new(from)?;
    let to = ManagedRelPath::new(to)?;
    ManagedFs::open_existing(home)?.sync_publish_parents_locked(lock, &from, &to)
}

pub(crate) fn sync_publish_final_root(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &str,
) -> Result<()> {
    let path = ManagedRelPath::new(path)?;
    ManagedFs::open_existing(home)?.sync_publish_final_root_locked(lock, &path)
}

pub(crate) fn rename_managed_new(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    from: &str,
    to: &str,
) -> Result<()> {
    let from = ManagedRelPath::new(from)?;
    let to = ManagedRelPath::new(to)?;
    ManagedFs::open_existing(home)?.rename_new(lock, &from, &to)
}

pub(crate) fn replace_managed_regular_file(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    from: &str,
    to: &str,
) -> Result<()> {
    let from = ManagedRelPath::new(from)?;
    let to = ManagedRelPath::new(to)?;
    ManagedFs::open_existing(home)?.replace_regular_file(lock, &from, &to)
}

pub(crate) fn verify_and_make_executable_confined(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
    expected: &[u8],
) -> Result<SafeFile> {
    let file = open_managed_regular(home, path)?;
    verify_candidate_handle(&file, expected)?;
    ManagedFs::open_existing(home)?.set_executable(lock, &file)?;
    verify_candidate_handle(&file, expected)?;
    Ok(file)
}

pub(crate) fn rename_verified_managed_file(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    file: &SafeFile,
    to: &AbsPath,
    expected: &[u8],
) -> Result<()> {
    let to = ManagedRelPath::new(home.to_rel(to)?)?;
    ManagedFs::open_existing(home)?.rename_verified_new(lock, file, &to, expected)
}

pub(crate) fn replace_verified_managed_file(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    file: &SafeFile,
    to: &AbsPath,
    expected: &[u8],
) -> Result<()> {
    let to = ManagedRelPath::new(home.to_rel(to)?)?;
    ManagedFs::open_existing(home)?.replace_verified_regular_file(lock, file, &to, expected)
}

/// 复制外部目录到管理根内的目标；源读取只读，目标创建经ManagedFs句柄。
pub(crate) fn copy_tree_confined(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    src: &AbsPath,
    dst: &AbsPath,
) -> Result<u64> {
    let tree = ExternalReadTree::open(src)?;
    let total = tree.validate_sizes()?;
    let fs = ManagedFs::open_existing(home)?;
    let dst = ManagedRelPath::new(home.to_rel(dst)?)?;
    fs.ensure_dir(lock, &dst)?;
    for relative in tree.directories() {
        fs.ensure_dir(
            lock,
            &ManagedRelPath::new(format!("{}/{relative}", dst.as_str()))?,
        )?;
    }
    for entry in tree.files() {
        let source = tree.open_file(entry)?;
        let mut content = Vec::with_capacity(entry.bytes.min(1 << 20) as usize);
        source
            .take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut content)
            .map_err(|e| Error::io(entry.relative.as_str(), e))?;
        if content.len() as u64 != entry.bytes {
            return Err(Error::InvalidRequest {
                reason: format!("{} 在复制期间改变", entry.relative),
            });
        }
        let target = ManagedRelPath::new(format!("{}/{}", dst.as_str(), entry.relative.as_str()))?;
        fs.write_new(lock, &target, &content)?;
    }
    tree.validate_unchanged()?;
    Ok(total)
}

/// 独占创建一个新文件并写入、fsync。目标已存在（含软链占位）即失败。
pub(crate) fn write_new_file(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
    content: &[u8],
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    ManagedFs::open_existing(home)?.write_new(lock, &rel, content)
}

pub(crate) fn write_new_file_observed(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
    content: &[u8],
) -> Result<SafeFile> {
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    ManagedFs::open_existing(home)?.write_new_observed(lock, &rel, content)
}

/// 不跟随软链的删除：叶子是软链时只删链接本身，目录才递归。清理不得沿链接
/// 删到根外（存储合同 §3.3 的 tmp 清理语义）。
pub(crate) fn remove_tree_no_follow(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    path: &AbsPath,
) -> Result<()> {
    if path == home.root() {
        return ManagedFs::open_existing(home)?.purge_contents(lock);
    }
    let rel = ManagedRelPath::new(home.to_rel(path)?)?;
    let fs = ManagedFs::open_existing(home)?;
    match fs.remove_owned_tree(lock, &rel) {
        Ok(()) | Err(Error::NotFound { .. }) => Ok(()),
        Err(error) => Err(error),
    }
}

fn make_directories_writable(parent: &std::fs::File, name: &str, display: &str) -> Result<()> {
    let fd = openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| map_fs_error(display, e))?;
    let directory = std::fs::File::from(fd);
    let entries = Dir::read_from(&directory).map_err(|e| map_fs_error(display, e))?;
    let names = entries
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_bytes().to_vec())
                .map_err(|e| Error::io(display, e.into()))
        })
        .collect::<Result<Vec<_>>>()?;
    for bytes in names {
        if bytes == b"." || bytes == b".." {
            continue;
        }
        let child = std::str::from_utf8(&bytes).map_err(|_| Error::InvalidRequest {
            reason: format!("{display} 下有非UTF-8名称，拒绝清理"),
        })?;
        let stat = statat(&directory, child, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| map_fs_error(display, e))?;
        if FileType::from_raw_mode(stat.st_mode) == FileType::Directory {
            make_directories_writable(&directory, child, &format!("{display}/{child}"))?;
        }
    }
    fchmod(&directory, Mode::from_raw_mode(0o700)).map_err(|e| map_fs_error(display, e))?;
    fsync(&directory).map_err(|e| map_fs_error(display, e))
}

/// 整棵置只读：目录 0555、文件 0444，含传入根本身。先拒软链，再对每个文件用
/// `SafeFile` 句柄核对身份后 fchmod；目录在内容处理完后最后置只读。
pub(crate) fn set_tree_readonly_confined(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    dir: &AbsPath,
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(dir)?)?;
    ManagedFs::open_existing(home)?.set_tree_readonly(lock, &rel)
}

/// 在句柄上整棵放开写权限：文件 0644、目录 0755（含根）。任何检查或权限错误都传播。
pub(crate) fn make_tree_writable(
    home: &crate::home::Home,
    lock: &crate::home::HomeLock,
    dir: &AbsPath,
) -> Result<()> {
    let rel = ManagedRelPath::new(home.to_rel(dir)?)?;
    ManagedFs::open_existing(home)?.make_tree_writable(lock, &rel)
}

fn set_dir_tree_mode(
    directory: &std::fs::File,
    display: &str,
    root: &str,
    file_mode: u16,
    directory_mode: u16,
) -> Result<()> {
    let entries = Dir::read_from(directory).map_err(|e| map_fs_error(display, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| Error::io(display, e.into()))?;
        let bytes = entry.file_name().to_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        let name = std::str::from_utf8(bytes).map_err(|_| Error::InvalidRequest {
            reason: format!("{display} 下有非UTF-8文件名"),
        })?;
        let child_display = format!("{display}/{name}");
        let stat = statat(directory, name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|e| map_fs_error(&child_display, e))?;
        let file_type = FileType::from_raw_mode(stat.st_mode);
        if file_type == FileType::Symlink {
            return Err(Error::InvalidRequest {
                reason: format!("{child_display} 是符号链接"),
            });
        }
        if file_type == FileType::Directory {
            let fd = openat(
                directory,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|e| map_fs_error(&child_display, e))?;
            let child = std::fs::File::from(fd);
            let opened = fstat(&child).map_err(|e| map_fs_error(&child_display, e))?;
            if opened.st_dev != stat.st_dev || opened.st_ino != stat.st_ino {
                return Err(Error::InvalidRequest {
                    reason: format!("{child_display} 在遍历期间被替换"),
                });
            }
            set_dir_tree_mode(&child, &child_display, root, file_mode, directory_mode)?;
            fchmod(&child, Mode::from_raw_mode(directory_mode))
                .map_err(|e| map_fs_error(&child_display, e))?;
            crate::failpoint::sync_error(root, "publish_readonly_nested_dir_sync").map_err(
                |error| Error::RecoveryRequired {
                    path: child_display.clone(),
                    detail: format!("只读权限已设置但嵌套目录同步失败：{error}"),
                },
            )?;
            fsync(&child).map_err(|e| map_fs_error(&child_display, e))?;
        } else if file_type == FileType::RegularFile {
            check_regular_stat(&child_display, &stat)?;
            let fd = openat(
                directory,
                name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|e| map_fs_error(&child_display, e))?;
            let opened = fstat(&fd).map_err(|e| map_fs_error(&child_display, e))?;
            check_regular_stat(&child_display, &opened)?;
            if opened.st_dev != stat.st_dev || opened.st_ino != stat.st_ino {
                return Err(Error::InvalidRequest {
                    reason: format!("{child_display} 在封存期间被替换"),
                });
            }
            let file = std::fs::File::from(fd);
            fchmod(&file, Mode::from_raw_mode(file_mode))
                .map_err(|e| map_fs_error(&child_display, e))?;
            crate::failpoint::sync_error(root, "publish_readonly_file_sync").map_err(|error| {
                Error::RecoveryRequired {
                    path: child_display.clone(),
                    detail: format!("只读权限已设置但文件同步失败：{error}"),
                }
            })?;
            fsync(&file).map_err(|e| map_fs_error(&child_display, e))?;
        } else {
            return Err(Error::InvalidRequest {
                reason: format!("{child_display} 是特殊文件"),
            });
        }
    }
    Ok(())
}
