//! T13：SQLite 存储的结构校验、事务、去重、CAS、序号。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use sheltie_core::testkit::{self, Fixture};
use sheltie_core::work::{Principal, Timestamp};
use sheltie_runtime::store::{CommitInput, CommitOutcome, OpenMode, SCHEMA_VERSION};
use sheltie_runtime::{Error, Store};

fn state_fixture() -> sheltie_core::work::WorkState {
    Fixture::two_step()
        .started_with(&[("topic", "t")])
        .state()
        .clone()
}

fn input(
    work: &sheltie_core::work::WorkState,
    request_id: &str,
    payload: &str,
    expected: Option<u64>,
) -> CommitInput {
    CommitInput {
        work_id: Some(work.work_id.clone()),
        expected_revision: expected,
        state: Some(work.clone()),
        request_id: request_id.to_string(),
        payload_hash: sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes())
            .as_str()
            .to_string(),
        reply_json: format!("{{\"reply\":\"{payload}\"}}"),
        principal: Principal("tester".into()),
        command_json: "{}".into(),
        at: Timestamp::parse("2026-09-24T03:00:00Z").unwrap(),
    }
}

// Task: T13
#[test]
#[ignore = "T13"]
fn open_creates_schema_with_user_version_1() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    let v: i64 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(v, SCHEMA_VERSION);
    let n: i64 = conn.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('workbooks','works','work_sequence','requests','audit')", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 5);
}

// Task: T13
#[test]
#[ignore = "T13"]
fn open_readonly_on_missing_db_is_not_found() {
    let (_d, home) = temp_home();
    assert!(matches!(
        Store::open(&home.store_path(), OpenMode::ReadOnly),
        Err(Error::NotFound { .. })
    ));
    assert!(!std::path::PathBuf::from(home.store_path().as_str()).exists());
}

// Task: T13
#[test]
#[ignore = "T13"]
fn open_rejects_wrong_user_version() {
    let (_d, home) = temp_home();
    open_rw(&home);
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.pragma_update(None, "user_version", 2).unwrap();
    drop(conn);
    assert!(matches!(
        Store::open(&home.store_path(), OpenMode::ReadWrite),
        Err(Error::StoreSchemaMismatch { .. })
    ));
}

// Task: T13
#[test]
#[ignore = "T13"]
fn open_rejects_same_version_different_table_shape() {
    // 手工造一个「旧形状」库：user_version = 1，但 works 表少一列。
    let (d, home) = temp_home();
    let conn = rusqlite::Connection::open(d.path().join("store.db")).unwrap();
    conn.execute_batch(
        "CREATE TABLE workbooks (id TEXT NOT NULL, version TEXT NOT NULL, digest TEXT NOT NULL, dir TEXT NOT NULL, added_at TEXT NOT NULL, PRIMARY KEY (id, version));
         CREATE TABLE works (work_id TEXT PRIMARY KEY, revision INTEGER NOT NULL, state_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
         CREATE TABLE work_sequence (day TEXT PRIMARY KEY, last INTEGER NOT NULL);
         CREATE TABLE requests (request_id TEXT PRIMARY KEY, work_id TEXT, payload_hash TEXT NOT NULL, reply_json TEXT NOT NULL, at TEXT NOT NULL);
         CREATE TABLE audit (seq INTEGER PRIMARY KEY AUTOINCREMENT, work_id TEXT NOT NULL, revision INTEGER NOT NULL, request_id TEXT NOT NULL, principal TEXT NOT NULL, command_json TEXT NOT NULL, at TEXT NOT NULL);
         PRAGMA user_version = 1;",
    )
    .unwrap();
    drop(conn);
    assert!(matches!(
        Store::open(&home.store_path(), OpenMode::ReadWrite),
        Err(Error::StoreSchemaMismatch { .. })
    ));
}

// Task: T13
#[test]
#[ignore = "T13"]
fn commit_inserts_state_audit_and_request_atomically() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let st = state_fixture();
    let out = store.commit(input(&st, "r1", "p1", None)).unwrap();
    assert_eq!(out, CommitOutcome::Committed { revision: 1 });
    let row = store.load_work(&st.work_id).unwrap();
    assert_eq!(row.revision, 1);
    assert_eq!(row.state, st);
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    let audit: i64 = conn
        .query_row("SELECT count(*) FROM audit", [], |r| r.get(0))
        .unwrap();
    let req: i64 = conn
        .query_row("SELECT count(*) FROM requests", [], |r| r.get(0))
        .unwrap();
    assert_eq!((audit, req), (1, 1));
}

// Task: T13
#[test]
#[ignore = "T13"]
fn commit_replays_same_request_id_and_payload() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let st = state_fixture();
    store.commit(input(&st, "r1", "p1", None)).unwrap();
    let out = store.commit(input(&st, "r1", "p1", None)).unwrap();
    assert_eq!(
        out,
        CommitOutcome::Replayed {
            reply_json: "{\"reply\":\"p1\"}".into()
        }
    );
    assert_eq!(
        store.load_work(&st.work_id).unwrap().revision,
        1,
        "重放不推进 revision"
    );
}

// Task: T13
#[test]
#[ignore = "T13"]
fn commit_rejects_same_request_id_different_payload() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let st = state_fixture();
    store.commit(input(&st, "r1", "p1", None)).unwrap();
    assert!(matches!(
        store.commit(input(&st, "r1", "p2", Some(1))),
        Err(Error::RequestConflict { .. })
    ));
}

// Task: T13
#[test]
#[ignore = "T13"]
fn commit_rejects_stale_revision() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let st = state_fixture();
    store.commit(input(&st, "r1", "p1", None)).unwrap();
    store.commit(input(&st, "r2", "p2", Some(1))).unwrap();
    assert!(matches!(
        store.commit(input(&st, "r3", "p3", Some(1))),
        Err(Error::RevisionConflict {
            expected: 1,
            actual: 2
        })
    ));
}

// Task: T13
#[test]
#[ignore = "T13"]
fn status_column_mirrors_state_json() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let mut fx = Fixture::two_step().started_with(&[("topic", "t")]);
    store.commit(input(fx.state(), "r1", "p1", None)).unwrap();
    fx.cancel().unwrap();
    store
        .commit(input(fx.state(), "r2", "p2", Some(1)))
        .unwrap();
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    let status: String = conn
        .query_row("SELECT status FROM works", [], |r| r.get(0))
        .unwrap();
    assert_eq!(status, "cancelled");
}

// Task: T13
#[test]
#[ignore = "T13"]
fn allocate_seq_starts_at_1_per_day_and_increments() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    assert_eq!(store.allocate_seq("2026-09-24").unwrap(), 1);
    assert_eq!(store.allocate_seq("2026-09-24").unwrap(), 2);
    assert_eq!(store.allocate_seq("2026-09-25").unwrap(), 1);
}

// Task: T13
#[test]
#[ignore = "T13"]
fn allocate_seq_is_not_reused_after_failed_start() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let _ = store.allocate_seq("2026-09-24").unwrap();
    // 假装 start 失败了，什么都没写入 works；下一次仍然是 2。
    assert_eq!(store.allocate_seq("2026-09-24").unwrap(), 2);
}

// Task: T13
#[test]
#[ignore = "T13"]
fn allocate_seq_rejects_1000th_of_day() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    for _ in 0..999 {
        store.allocate_seq("2026-09-24").unwrap();
    }
    assert!(matches!(
        store.allocate_seq("2026-09-24"),
        Err(Error::InvalidRequest { .. })
    ));
}

// Task: T13
#[test]
#[ignore = "T13"]
fn allocate_seq_under_two_threads_yields_distinct_numbers() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let s = store.clone();
            std::thread::spawn(move || {
                (0..50)
                    .map(|_| s.allocate_seq("2026-09-24").unwrap())
                    .collect::<Vec<u32>>()
            })
        })
        .collect();
    let mut all: Vec<u32> = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect();
    all.sort_unstable();
    all.dedup();
    assert_eq!(all.len(), 100);
    let _ = testkit::now();
}
