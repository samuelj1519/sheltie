//! C002-T24：无I/O构造与普通写入口的锁内建库边界。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{abs, example_dir, home_with_example, start_two_step, temp_home, work_id_of};
use sheltie_runtime::request::InputValue;
use sheltie_runtime::{Error, StartArgs, WorkService, WorkbookRepo};

#[cfg(feature = "failpoint")]
static SESSION_INIT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// Task: C002-T24
#[test]
fn service_and_repository_constructors_do_not_create_home_or_store() {
    let (_dir, home) = temp_home();
    let _service = WorkService::new(home.clone());
    let _repo = WorkbookRepo::new(home.clone());
    assert!(!home.store_path().as_path().exists());
    assert!(!home.lock_path().as_path().exists());
}

// Task: C002-T24
#[test]
fn missing_work_and_remove_do_not_initialize_a_store_or_lock() {
    let (_dir, home) = temp_home();
    let service = WorkService::new(home.clone());
    let args = StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: Default::default(),
    };
    assert!(matches!(
        service.start(args, Some("r-missing".into())),
        Err(Error::NotFound { .. })
    ));
    assert!(!home.store_path().as_path().exists());
    assert!(!home.lock_path().as_path().exists());

    let repo = WorkbookRepo::new(home.clone());
    assert!(matches!(
        repo.remove("two-step", "1.0.0", None),
        Err(Error::NotFound { .. })
    ));
    assert!(!home.store_path().as_path().exists());
    assert!(!home.lock_path().as_path().exists());
}

// Task: C002-T24
#[test]
fn orphan_sqlite_sidecar_blocks_new_store_creation_without_lock() {
    let (_dir, home) = temp_home();
    let wal = home.root().join_segment("store.db-wal");
    std::fs::write(wal.as_path(), b"orphan wal sentinel").unwrap();
    let service_error = WorkService::new(home.clone()).list().unwrap_err();
    assert!(matches!(service_error, Error::StoreCorrupt { .. }));
    let add_error = WorkbookRepo::new(home.clone())
        .add(
            &sheltie_core::path::AbsPath::new(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../examples/two-step")
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            )
            .unwrap(),
            None,
        )
        .unwrap_err();
    assert!(matches!(add_error, Error::StoreCorrupt { .. }));
    assert!(!home.store_path().as_path().exists());
    assert!(!home.lock_path().as_path().exists());
    assert_eq!(
        std::fs::read(wal.as_path()).unwrap(),
        b"orphan wal sentinel"
    );
}

// Task: C002-T24
#[test]
fn unreadable_start_input_leaves_requests_sequences_and_works_unchanged() {
    let (_dir, home, service) = home_with_example("two-step");
    let request_id = "r-unreadable-start-input";
    let path = home.root().join_segment("missing-input.txt").to_string();
    let inputs =
        std::collections::BTreeMap::from([("topic".to_string(), InputValue::AtFile { path })]);
    let args = StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs,
    };
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let before = (
        conn.query_row("SELECT COUNT(*) FROM requests", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        conn.query_row("SELECT COUNT(*) FROM work_sequence", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        conn.query_row("SELECT COUNT(*) FROM works", [], |row| row.get::<_, i64>(0))
            .unwrap(),
    );
    drop(conn);

    assert!(matches!(
        service.start(args, Some(request_id.into())),
        Err(Error::InputFileInvalid { .. })
    ));

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let after = (
        conn.query_row("SELECT COUNT(*) FROM requests", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        conn.query_row("SELECT COUNT(*) FROM work_sequence", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        conn.query_row("SELECT COUNT(*) FROM works", [], |row| row.get::<_, i64>(0))
            .unwrap(),
    );
    assert_eq!(after, before);
    assert!(!home.works_dir().as_path().is_dir());
}

// Task: C002-T24
#[cfg(feature = "failpoint")]
#[test]
fn failed_store_initialization_never_deletes_a_replaced_database_leaf() {
    use std::time::{Duration, Instant};

    let _serial = SESSION_INIT_LOCK.lock().unwrap();
    let (_dir, home) = temp_home();
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "write_session_after_store_create",
        home.root().as_str(),
        rendezvous.path(),
    )
    .unwrap();
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        }
    }
    let _guard = Guard;
    let add_home = home.clone();
    let add = std::thread::spawn(move || {
        WorkbookRepo::new(add_home.clone()).add(
            &sheltie_core::path::AbsPath::new(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../examples/two-step")
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            )
            .unwrap(),
            None,
        )
    });
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        let _ = add.join();
        panic!("WriteSession 未到达Store叶创建后的同步点");
    }
    assert!(!home.store_path().as_path().exists());
    let staging = std::fs::read_dir(home.tmp_dir().as_path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("store-init-")
        })
        .unwrap()
        .join("store.db");
    std::fs::remove_file(&staging).unwrap();
    let replacement = rusqlite::Connection::open(&staging).unwrap();
    replacement
        .execute_batch(
            "CREATE TABLE user_records(value TEXT);
             INSERT INTO user_records VALUES ('keep me');
             PRAGMA user_version = 1;",
        )
        .unwrap();
    drop(replacement);
    let old_database = std::fs::read(&staging).unwrap();
    std::fs::write(&release, b"release").unwrap();
    assert!(matches!(
        add.join().unwrap(),
        Err(Error::RecoveryRequired { .. })
    ));
    assert_eq!(std::fs::read(&staging).unwrap(), old_database);
    assert!(!home.store_path().as_path().exists());
    let connection = rusqlite::Connection::open(&staging).unwrap();
    let schema: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    let record: String = connection
        .query_row("SELECT value FROM user_records", [], |row| row.get(0))
        .unwrap();
    assert_eq!(schema, 1);
    assert_eq!(record, "keep me");
    assert!(!home.workbook_dir("two-step", "1.0.0").as_path().exists());
}

// Task: C002-T24
#[cfg(feature = "failpoint")]
#[test]
fn old_work_waiting_behind_purge_does_not_recreate_the_removed_store() {
    use std::time::{Duration, Instant};

    let _serial = SESSION_INIT_LOCK.lock().unwrap();
    let (_dir, home, service) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&service));
    let parent_lock = home.acquire_lock().unwrap();
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "home_lock_waiting",
        home.lock_path().as_str(),
        rendezvous.path(),
    )
    .unwrap();
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        }
    }
    let _guard = Guard;
    let child_service = service.clone();
    let child_work = work.clone();
    let child = std::thread::spawn(move || {
        child_service.begin(
            &child_work,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
    });
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        drop(parent_lock);
        let _ = child.join();
        panic!("Work写操作未到达真实锁竞争点");
    }

    for name in [
        "store.db-wal",
        "store.db-shm",
        "store.db-journal",
        "store.db",
    ] {
        let path = home.root().join_segment(name);
        if path.as_path().exists() {
            std::fs::remove_file(path.as_path()).unwrap();
        }
    }
    std::fs::write(&release, b"release").unwrap();
    drop(parent_lock);
    assert!(matches!(child.join().unwrap(), Err(Error::NotFound { .. })));
    assert!(
        !home.store_path().as_path().exists(),
        "等待者不得重建store.db"
    );
    assert!(home.lock_path().as_path().exists(), "同一管理锁仍保留");
}

// Task: C002-T24
#[cfg(feature = "failpoint")]
#[test]
fn real_install_waits_for_a_concurrent_workbook_add_store_initializer() {
    let _serial = SESSION_INIT_LOCK.lock().unwrap();
    let source = abs(&example_dir("two-step"));
    let (_directory, home) = temp_home();
    common::init::concurrent_store_init(
        &home,
        move |home| WorkbookRepo::new(home).add(&source, None).map(|_| ()),
        |home| sheltie_runtime::selfmgmt::install(&home).map(|_| ()),
    );
}

// Task: C002-T24
#[cfg(feature = "failpoint")]
#[test]
fn real_workbook_add_waits_for_a_concurrent_install_store_initializer() {
    let _serial = SESSION_INIT_LOCK.lock().unwrap();
    let source = abs(&example_dir("two-step"));
    let (_directory, home) = temp_home();
    common::init::concurrent_store_init(
        &home,
        |home| sheltie_runtime::selfmgmt::install(&home).map(|_| ()),
        move |home| WorkbookRepo::new(home).add(&source, None).map(|_| ()),
    );
}
