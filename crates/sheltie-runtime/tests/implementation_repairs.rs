//! C002-T31：历史文件和Attempt目录落位后的同步恢复。
#![cfg(feature = "failpoint")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use rusqlite::Connection;
use sheltie_core::ids::NodeId;
use sheltie_runtime::Error;

static SYNC_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct SyncGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        sheltie_runtime::failpoint::disarm_sync_error().unwrap();
    }
}

fn sync_guard() -> SyncGuard {
    SyncGuard {
        _lock: SYNC_TEST_LOCK.lock().unwrap(),
    }
}

fn assert_sync_pending(error: Error, request: &str) {
    let Error::EffectPending {
        committed,
        request_id,
        cause,
        cause_detail,
        original,
        ..
    } = error
    else {
        panic!("同步失败必须保留已提交请求：{error:?}");
    };
    assert!(committed);
    assert_eq!(request_id, request);
    assert_eq!(cause, sheltie_core::ErrorCode::Io);
    assert!(cause_detail.contains("injected sync failure"));
    assert!(original.is_some());
}

fn published(connection: &Connection, request: &str) -> i64 {
    connection
        .query_row(
            "SELECT published FROM requests WHERE request_id=?1",
            [request],
            |row| row.get(0),
        )
        .unwrap()
}

// Task: C002-T31
#[test]
fn matching_history_after_failed_parent_sync_stays_pending_until_sync_succeeds() {
    let _guard = sync_guard();
    let (_dir, home, svc) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&svc));
    let node = NodeId::new("outline").unwrap();
    let request = "history-sync";
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    sheltie_runtime::failpoint::arm_sync_error(home.root().as_str(), "managed_file_parent_sync")
        .unwrap();
    assert_sync_pending(
        svc.begin(&work, &node, Some(request.into())).unwrap_err(),
        request,
    );
    assert_eq!(published(&connection, request), 0);
    let effects: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id=?1",
            [request],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let history = effects
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == "write_file")
        .unwrap();
    let path = home
        .root()
        .as_path()
        .join(history["path"].as_str().unwrap());
    assert_eq!(
        std::fs::read(&path).unwrap(),
        history["content"].as_str().unwrap().as_bytes()
    );
    sheltie_runtime::failpoint::arm_sync_error(home.root().as_str(), "managed_file_parent_sync")
        .unwrap();
    assert_sync_pending(
        svc.begin(&work, &node, Some(request.into())).unwrap_err(),
        request,
    );
    assert_eq!(published(&connection, request), 0);
    assert_eq!(
        std::fs::read(&path).unwrap(),
        history["content"].as_str().unwrap().as_bytes()
    );
    sheltie_runtime::failpoint::disarm_sync_error().unwrap();
    assert!(
        svc.begin(&work, &node, Some(request.into()))
            .unwrap()
            .replayed
    );
    assert_eq!(published(&connection, request), 1);
}

// Task: C002-T31
#[test]
fn existing_output_directory_after_failed_parent_sync_stays_pending_until_sync_succeeds() {
    let _guard = sync_guard();
    let (_dir, home, svc) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&svc));
    let node = NodeId::new("outline").unwrap();
    let relative = format!("works/{work}/attempts/outline/occurrence-001/attempt-000/outputs");
    let point = format!("directory_parent_sync:{relative}");
    let request = "directory-sync";
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    sheltie_runtime::failpoint::arm_sync_error(home.root().as_str(), &point).unwrap();
    assert_sync_pending(
        svc.begin(&work, &node, Some(request.into())).unwrap_err(),
        request,
    );
    assert!(home.root().as_path().join(&relative).is_dir());
    assert_eq!(published(&connection, request), 0);
    sheltie_runtime::failpoint::arm_sync_error(home.root().as_str(), &point).unwrap();
    assert_sync_pending(
        svc.begin(&work, &node, Some(request.into())).unwrap_err(),
        request,
    );
    assert_eq!(published(&connection, request), 0);
    sheltie_runtime::failpoint::disarm_sync_error().unwrap();
    assert!(
        svc.begin(&work, &node, Some(request.into()))
            .unwrap()
            .replayed
    );
    assert_eq!(published(&connection, request), 1);
}

// Task: C002-T31
#[test]
fn completed_history_repair_retries_a_failed_parent_sync_without_changing_the_snapshot() {
    let _guard = sync_guard();
    let (_dir, home, svc) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&svc));
    let node = NodeId::new("outline").unwrap();
    let request = "completed-history-sync";
    let response = svc.begin(&work, &node, Some(request.into())).unwrap();
    let path = response.data["brief_path"].as_str().unwrap();
    let original = std::fs::read(path).unwrap();
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let snapshot: String = connection
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id=?1",
            [request],
            |row| row.get(0),
        )
        .unwrap();
    std::fs::remove_file(path).unwrap();
    for _ in 0..2 {
        sheltie_runtime::failpoint::arm_sync_error(
            home.root().as_str(),
            "managed_file_parent_sync",
        )
        .unwrap();
        assert_sync_pending(
            svc.begin(&work, &node, Some(request.into())).unwrap_err(),
            request,
        );
        assert_eq!(published(&connection, request), 1);
        assert_eq!(std::fs::read(path).unwrap(), original);
        assert_eq!(
            connection
                .query_row(
                    "SELECT reply_json FROM requests WHERE request_id=?1",
                    [request],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            snapshot
        );
    }
    sheltie_runtime::failpoint::disarm_sync_error().unwrap();
    let replay = svc.begin(&work, &node, Some(request.into())).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.data, response.data);
    assert_eq!(replay.next, response.next);
    assert_eq!(replay.revision, response.revision);
}
