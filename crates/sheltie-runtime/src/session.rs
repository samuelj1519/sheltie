//! 锁内读写 Store 会话。所有普通写入口先持有 HomeLock，再获得 RW Store。

use crate::error::{Error, Result};
use crate::fsx::{self, open_managed_optional_locked, write_new_file_observed};
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
            let staging = home
                .tmp_dir()
                .join_segment(&format!("store-init-{}", uuid::Uuid::now_v7().simple()));
            fsx::ensure_dirs_under(home, &lock, &staging)?;
            let source = staging.join_segment("store.db");
            let bytes = Store::empty_database_bytes()?;
            let created = write_new_file_observed(home, &lock, &source, &bytes)?;
            crate::failpoint::rendezvous("write_session_after_store_create", home.root().as_str())
                .map_err(|error| Error::io(source.as_str(), error))?;
            crate::failpoint::maybe_exit("store_initialized_before_publish");
            fsx::rename_verified_managed_file(home, &lock, &created, &path, &bytes).map_err(
                |error| Error::RecoveryRequired {
                    path: path.to_string(),
                    detail: format!("完整暂存Store发布失败；所有现有端点已保留：{error}"),
                },
            )?;
            let relative = home.to_rel(&staging)?;
            let tree = fsx::open_managed_tree(home, &lock, &relative)?;
            fsx::remove_empty_managed_tree(home, &lock, &tree, &relative)?;
        }
        let store = Store::open_existing_locked(home, Arc::clone(&lock), OpenMode::ReadWrite)?;
        Ok(Self { lock, store })
    }
}

#[cfg(test)]
mod tests {
    use super::WriteSession;
    use crate::error::Error;
    use crate::home::Home;
    #[cfg(feature = "failpoint")]
    use crate::store::{OpenMode, Store};
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

    // Task: C002-T24
    #[cfg(feature = "failpoint")]
    #[test]
    fn concurrent_store_initializer_waits_for_schema_before_preflight_rejection() {
        use std::time::{Duration, Instant};

        struct FailpointGuard;
        impl Drop for FailpointGuard {
            fn drop(&mut self) {
                crate::failpoint::disarm_rendezvous().unwrap();
            }
        }

        let _serial = crate::failpoint::RENDEZVOUS_TEST_LOCK.lock().unwrap();
        let _guard = FailpointGuard;
        let directory = tempfile::tempdir().unwrap();
        let home_root = directory.path().join("home");
        std::fs::create_dir(&home_root).unwrap();
        let home = Home::at(AbsPath::new(home_root.to_string_lossy().into_owned()).unwrap());

        let first_sync = tempfile::tempdir().unwrap();
        crate::failpoint::arm_rendezvous(
            "write_session_after_store_create",
            home.root().as_str(),
            first_sync.path(),
        )
        .unwrap();
        let first_home = home.clone();
        let first = std::thread::spawn(move || {
            let result = WriteSession::open_or_create(&first_home);
            drop(result?);
            Ok::<(), Error>(())
        });

        let deadline = Instant::now() + Duration::from_secs(10);
        while !first_sync.path().join("reached").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        if !first_sync.path().join("reached").exists() {
            let _ = std::fs::write(first_sync.path().join("release"), b"release");
            let _ = first.join();
            panic!("首个Store初始化未停在创建后同步点");
        }

        crate::failpoint::disarm_rendezvous().unwrap();
        let second_sync = tempfile::tempdir().unwrap();
        crate::failpoint::arm_rendezvous(
            "home_lock_waiting",
            home.lock_path().as_str(),
            second_sync.path(),
        )
        .unwrap();
        let second_home = home.clone();
        let second = std::thread::spawn(move || {
            let result = WriteSession::open_or_create(&second_home);
            drop(result?);
            Ok::<(), Error>(())
        });

        let deadline = Instant::now() + Duration::from_secs(10);
        while !second_sync.path().join("reached").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        if !second_sync.path().join("reached").exists() {
            let _ = std::fs::write(first_sync.path().join("release"), b"release");
            let _ = std::fs::write(second_sync.path().join("release"), b"release");
            let _ = first.join();
            let _ = second.join();
            panic!("第二个Store初始化没有等待同一根锁");
        }

        std::fs::write(first_sync.path().join("release"), b"release").unwrap();
        first.join().unwrap().unwrap();
        std::fs::write(second_sync.path().join("release"), b"release").unwrap();
        second.join().unwrap().unwrap();

        let store = Store::open_for_home(&home, OpenMode::ReadOnly).unwrap();
        assert!(store.list_workbooks().unwrap().is_empty());
        crate::failpoint::disarm_rendezvous().unwrap();
    }
}
