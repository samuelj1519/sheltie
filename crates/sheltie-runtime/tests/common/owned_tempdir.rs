//! 临时目录持有原根fd；收尾只放开目录权限，不跟随链接、不修改文件权限。
use rustix::fs::{AtFlags, Dir, FileType, Mode, OFlags, fchmod, open, openat, statat};
use std::fs::File;
use std::ops::Deref;

pub struct OwnedTempDir {
    directory: tempfile::TempDir,
    root: File,
}
impl OwnedTempDir {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = File::from(
            open(
                directory.path(),
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .unwrap(),
        );
        Self { directory, root }
    }
}
impl Deref for OwnedTempDir {
    type Target = tempfile::TempDir;
    fn deref(&self) -> &Self::Target {
        &self.directory
    }
}
impl Drop for OwnedTempDir {
    fn drop(&mut self) {
        fn writable(directory: &File) -> rustix::io::Result<()> {
            fchmod(directory, Mode::from_raw_mode(0o700))?;
            for entry in Dir::read_from(directory)? {
                let entry = entry?;
                let name = entry.file_name();
                if name.to_bytes() == b"." || name.to_bytes() == b".." {
                    continue;
                }
                let stat = statat(directory, name, AtFlags::SYMLINK_NOFOLLOW)?;
                if FileType::from_raw_mode(stat.st_mode) == FileType::Directory {
                    let child = File::from(openat(
                        directory,
                        name,
                        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                        Mode::empty(),
                    )?);
                    writable(&child)?;
                }
            }
            Ok(())
        }
        let _ = writable(&self.root);
    }
}
