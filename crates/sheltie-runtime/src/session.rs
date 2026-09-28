//! 锁内读写 Store 会话。所有普通写入口先持有 HomeLock，再获得 RW Store。

use crate::error::{Error, Result};
use crate::fsx::{
    open_managed_optional_locked, remove_managed_file_if_same, write_new_file_observed,
};
use crate::home::{Home, HomeLock};
use crate::store::{OpenMode, Store};
use std::sync::Arc;

pub(crate) struct WriteSession {
    pub(crate) lock: Arc<HomeLock>,
    pub(crate) store: Store,
}

impl WriteSession {
    pub(crate) fn open_existing(home: &Home) -> Result<Self> {
        drop(Store::open_for_home(home, OpenMode::ReadOnly)?);
        let lock = Arc::new(home.acquire_lock()?);
        let path = home.store_path();
        crate::fsx::validate_store_files_locked(home, &lock)?;
        let store_file = crate::fsx::open_managed_optional_locked(home, &lock, &path)?;
        if store_file.is_none() {
            return Err(Error::NotFound {
                what: path.to_string(),
            });
        }
        let store = Store::open_existing_locked(home, Arc::clone(&lock), OpenMode::ReadWrite)?;
        Ok(Self { lock, store })
    }

    pub(crate) fn open_or_create(home: &Home) -> Result<Self> {
        match Store::open_for_home(home, OpenMode::ReadOnly) {
            Ok(store) => drop(store),
            Err(Error::NotFound { .. }) => {}
            Err(error) => return Err(error),
        }
        let lock = Arc::new(home.acquire_lock()?);
        let path = home.store_path();
        crate::fsx::validate_store_files_locked(home, &lock)?;
        if open_managed_optional_locked(home, &lock, &path)?.is_none() {
            let created = write_new_file_observed(home, &lock, &path, b"")?;
            crate::failpoint::rendezvous("write_session_after_store_create", home.root().as_str())
                .map_err(|error| Error::io(path.as_str(), error))?;
            if let Err(error) = Store::initialize_existing(home, &lock, &path, &created) {
                return match remove_managed_file_if_same(home, &lock, &path, &created) {
                    Ok(()) => Err(error),
                    Err(cleanup) => Err(Error::RecoveryRequired {
                        path: path.to_string(),
                        detail: format!(
                            "Store初始化失败：{error}；失败清理仅能删除本次创建inode，现有端点已保留：{cleanup}"
                        ),
                    }),
                };
            }
            let current = open_managed_optional_locked(home, &lock, &path)?.ok_or_else(|| {
                Error::RecoveryRequired {
                    path: path.to_string(),
                    detail: "Store初始化后创建对象消失；保留当前根内其他对象".into(),
                }
            })?;
            if file_identity(&current) != file_identity(&created) {
                return Err(Error::RecoveryRequired {
                    path: path.to_string(),
                    detail: "Store初始化期间其路径被换绑；不覆盖或清理新端点".into(),
                });
            }
        }
        let store = Store::open_existing_locked(home, Arc::clone(&lock), OpenMode::ReadWrite)?;
        Ok(Self { lock, store })
    }
}

fn file_identity(file: &crate::fsx::SafeFile) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt as _;
    (file.metadata().dev(), file.metadata().ino())
}

#[cfg(test)]
mod tests {
    use super::WriteSession;
    use crate::error::Error;
    use crate::home::Home;
    use sheltie_core::path::AbsPath;

    // Task: C002-T24
    #[test]
    fn session_store_refuses_a_replaced_lock_on_the_next_connection() {
        let directory = tempfile::tempdir().unwrap();
        let home_root = directory.path().join("home");
        std::fs::create_dir(&home_root).unwrap();
        let home = Home::at(AbsPath::new(home_root.to_string_lossy().into_owned()).unwrap());
        let session = WriteSession::open_or_create(&home).unwrap();
        session.store.list_workbooks().unwrap();
        let db_before = std::fs::read(home.store_path().as_path()).unwrap();

        std::fs::remove_file(home.lock_path().as_path()).unwrap();
        std::fs::write(home.lock_path().as_path(), b"replacement lock").unwrap();
        assert!(matches!(
            session.store.list_workbooks(),
            Err(Error::InvalidRequest { .. })
        ));
        assert_eq!(
            std::fs::read(home.store_path().as_path()).unwrap(),
            db_before
        );
    }

    // Task: C002-T24
    #[test]
    fn session_store_refuses_a_replaced_root_on_the_next_connection() {
        let directory = tempfile::tempdir().unwrap();
        let home_root = directory.path().join("home");
        let moved_root = directory.path().join("moved-home");
        std::fs::create_dir(&home_root).unwrap();
        let home = Home::at(AbsPath::new(home_root.to_string_lossy().into_owned()).unwrap());
        let session = WriteSession::open_or_create(&home).unwrap();
        session.store.list_workbooks().unwrap();
        std::fs::rename(&home_root, &moved_root).unwrap();
        std::fs::create_dir(&home_root).unwrap();
        std::fs::write(home_root.join(".lock"), b"replacement lock").unwrap();
        std::fs::write(home_root.join("store.db"), b"external store sentinel").unwrap();

        assert!(matches!(
            session.store.list_workbooks(),
            Err(Error::InvalidRequest { .. })
        ));
        assert_eq!(
            std::fs::read(home_root.join("store.db")).unwrap(),
            b"external store sentinel"
        );
        drop(session);
        std::fs::remove_dir_all(moved_root).unwrap();
    }
}
