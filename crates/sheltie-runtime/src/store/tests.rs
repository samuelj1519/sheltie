//! T13：SQLite 存储的结构校验、事务、去重、CAS、序号。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::Store;
use crate::store::{CommitInput, CommitOutcome, OpenMode, SCHEMA_VERSION};
use crate::{Error, Home};
use sheltie_core::path::AbsPath;
use sheltie_core::testkit::{self, Fixture};
use sheltie_core::work::{Principal, Timestamp};

fn temp_home() -> (tempfile::TempDir, Home) {
    let dir = tempfile::tempdir().unwrap();
    let home = Home::resolve(Some(
        (AbsPath::new(dir.path().to_string_lossy().into_owned()).unwrap()).as_str(),
    ))
    .unwrap();
    (dir, home)
}

fn open_rw(home: &Home) -> Store {
    Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap()
}

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
        workbook_insert: None,
        workbook_delete: None,
        workbook_in_use_check: None,
        expected_revision: expected,
        state: Some(work.clone()),
        request_id: request_id.to_string(),
        intent_hash: sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes())
            .as_str()
            .to_string(),
        reply_json: format!("{{\"reply\":\"{payload}\"}}"),
        effects_json: "[]".to_string(),
        principal: Principal("tester".into()),
        command_json: "{}".into(),
        at: Timestamp::parse("2026-09-24T03:00:00Z").unwrap(),
    }
}

// 手写合同schema2；不使用production TABLES/create_script作为自己的期望。
const HANDWRITTEN_WORKS: &str = "CREATE TABLE works (
  work_id     TEXT PRIMARY KEY,
  revision    INTEGER NOT NULL,
  status      TEXT NOT NULL,
  state_json  TEXT NOT NULL,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
)";

fn handwritten_schema2(works: &str) -> (tempfile::TempDir, Home) {
    let (dir, home) = temp_home();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert!(
        connection
            .set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,
                true
            )
            .unwrap()
    );
    connection
        .execute_batch("PRAGMA journal_mode = WAL;")
        .unwrap();
    connection
        .execute_batch(&format!(
            "CREATE TABLE workbooks (id TEXT NOT NULL, version TEXT NOT NULL, digest TEXT NOT NULL, dir TEXT NOT NULL, added_at TEXT NOT NULL, PRIMARY KEY (id, version));
             {works};
             CREATE TABLE work_sequence (day TEXT PRIMARY KEY, last INTEGER NOT NULL);
             CREATE TABLE requests (request_id TEXT PRIMARY KEY, intent_hash TEXT NOT NULL, work_id TEXT, reply_json TEXT NOT NULL, effects_json TEXT NOT NULL, published INTEGER NOT NULL, at TEXT NOT NULL);
             CREATE TABLE audit (seq INTEGER PRIMARY KEY AUTOINCREMENT, work_id TEXT NOT NULL, revision INTEGER NOT NULL, request_id TEXT NOT NULL, principal TEXT NOT NULL, command_json TEXT NOT NULL, at TEXT NOT NULL);
             INSERT INTO work_sequence VALUES ('2026-10-02', 7);
             PRAGMA user_version = 2;"
        ))
        .unwrap();
    drop(connection);
    (dir, home)
}

fn assert_schema2_shape_rejected(works: String) {
    let (dir, home) = handwritten_schema2(&works);
    let main_before = std::fs::read(home.store_path().as_path()).unwrap();
    let wal = dir.path().join("store.db-wal");
    let wal_before = std::fs::read(&wal).unwrap();
    assert!(!wal_before.is_empty());
    for mode in [OpenMode::ReadOnly, OpenMode::ReadWrite] {
        let error = Store::open(&home.store_path(), mode).unwrap_err();
        let Error::StoreSchemaMismatch { detail } = error else {
            panic!("same-version形状错误须准确定位：{error:?}");
        };
        assert_eq!(detail, "表 works 的建表语句与 SCHEMA_VERSION 不符");
        assert_eq!(
            std::fs::read(home.store_path().as_path()).unwrap(),
            main_before
        );
        assert_eq!(std::fs::read(&wal).unwrap(), wal_before);
    }
}

// Task: C002-T40
#[test]
fn handwritten_schema2_control_accepts_both_open_modes() {
    let (_dir, home) = handwritten_schema2(HANDWRITTEN_WORKS);
    for mode in [OpenMode::ReadOnly, OpenMode::ReadWrite] {
        let store = Store::open(&home.store_path(), mode).unwrap();
        assert!(store.list_works().unwrap().is_empty());
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let last: i64 = connection
            .query_row(
                "SELECT last FROM work_sequence WHERE day='2026-10-02'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(last, 7);
    }
}

// Task: T13
#[test]
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
fn open_rejects_wrong_user_version() {
    let (_d, home) = temp_home();
    open_rw(&home);
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    // schema 2 是当前版本：模拟 schema 1 旧库（拒绝且文件字节不变）。
    conn.pragma_update(None, "user_version", 1).unwrap();
    drop(conn);
    assert!(matches!(
        Store::open(&home.store_path(), OpenMode::ReadWrite),
        Err(Error::StoreSchemaMismatch { .. })
    ));
}

// Task: C002-T40
#[test]
fn open_rejects_same_version_different_table_shape() {
    assert_schema2_shape_rejected(HANDWRITTEN_WORKS.replace("  status      TEXT NOT NULL,\n", ""));
}

// Task: T13
#[test]
fn open_readonly_on_existing_db_succeeds() {
    let (_d, home) = temp_home();
    open_rw(&home);
    let ro = Store::open(&home.store_path(), OpenMode::ReadOnly).unwrap();
    assert!(ro.list_works().unwrap().is_empty());
}

// Task: C002-T24
#[test]
fn readonly_store_open_rejects_a_symlink_without_touching_its_target() {
    use std::os::unix::fs::PermissionsExt as _;

    let (_dir, home) = temp_home();
    open_rw(&home);
    let external = tempfile::tempdir().unwrap();
    let sentinel = external.path().join("store-target.db");
    std::fs::write(&sentinel, b"external store sentinel").unwrap();
    let mode = std::fs::metadata(&sentinel).unwrap().permissions().mode();
    std::fs::remove_file(home.store_path().as_path()).unwrap();
    std::os::unix::fs::symlink(&sentinel, home.store_path().as_path()).unwrap();

    assert!(matches!(
        Store::open(&home.store_path(), OpenMode::ReadOnly),
        Err(Error::InvalidRequest { .. })
    ));
    assert_eq!(
        std::fs::read(&sentinel).unwrap(),
        b"external store sentinel"
    );
    assert_eq!(
        std::fs::metadata(&sentinel).unwrap().permissions().mode(),
        mode
    );
}

// Task: C002-T24
#[test]
fn lazy_readonly_store_rejects_a_hardlinked_database() {
    use std::os::unix::fs::PermissionsExt as _;

    let (_dir, home) = temp_home();
    open_rw(&home);
    let bytes_before = std::fs::read(home.store_path().as_path()).unwrap();
    let external = tempfile::tempdir().unwrap();
    let alias = external.path().join("store-alias.db");
    std::fs::hard_link(home.store_path().as_path(), &alias).unwrap();
    let mode = std::fs::metadata(&alias).unwrap().permissions().mode();

    let error = crate::WorkService::new(home.clone()).list().unwrap_err();
    assert!(matches!(error, Error::InvalidRequest { .. }), "{error:?}");
    assert_eq!(std::fs::read(&alias).unwrap(), bytes_before);
    assert_eq!(
        std::fs::metadata(&alias).unwrap().permissions().mode(),
        mode
    );
}

// Task: C002-T24
#[test]
fn lazy_readonly_store_rejects_a_wal_sidecar_symlink_before_sqlite_open() {
    let (_dir, home) = temp_home();
    open_rw(&home);
    let external = tempfile::tempdir().unwrap();
    let sentinel = external.path().join("wal-sentinel");
    std::fs::write(&sentinel, b"external wal sentinel").unwrap();
    std::os::unix::fs::symlink(
        &sentinel,
        home.root().join_segment("store.db-wal").as_path(),
    )
    .unwrap();

    let error = crate::WorkService::new(home.clone()).list().unwrap_err();
    assert!(matches!(error, Error::InvalidRequest { .. }), "{error:?}");
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"external wal sentinel");
}

// Task: C002-T24
#[test]
fn lazy_readonly_store_rejects_a_hardlinked_journal_sidecar() {
    let (_dir, home) = temp_home();
    open_rw(&home);
    let external = tempfile::tempdir().unwrap();
    let sentinel = external.path().join("journal-sentinel");
    std::fs::write(&sentinel, b"external journal sentinel").unwrap();
    std::fs::hard_link(
        &sentinel,
        home.root().join_segment("store.db-journal").as_path(),
    )
    .unwrap();

    let error = crate::WorkService::new(home.clone()).list().unwrap_err();
    assert!(matches!(error, Error::InvalidRequest { .. }), "{error:?}");
    assert_eq!(
        std::fs::read(&sentinel).unwrap(),
        b"external journal sentinel"
    );
}

// Task: C002-T40
#[test]
fn open_rejects_same_whitespace_different_column_type() {
    assert_schema2_shape_rejected(
        HANDWRITTEN_WORKS.replace("state_json  TEXT", "state_json  BLOB"),
    );
}

// Task: T13
#[test]
fn insert_workbook_on_readonly_store_is_not_workbook_exists() {
    // 只读连接上插入失败是 SQLITE_READONLY，不得被归成 WorkbookExists。
    let (_d, home) = temp_home();
    open_rw(&home);
    let ro = Store::open(&home.store_path(), OpenMode::ReadOnly).unwrap();
    let row = crate::WorkbookRow {
        id: "x".into(),
        version: "1.0.0".into(),
        digest: "0".repeat(64),
        dir: "workbooks/x/1.0.0".into(),
        added_at: "2026-09-25T00:00:00Z".into(),
    };
    let err = ro.insert_workbook(&row).unwrap_err();
    assert!(!matches!(err, Error::WorkbookExists { .. }), "{err}");
}

// Task: T13
#[test]
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
fn commit_replays_same_request_id_and_payload() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    let st = state_fixture();
    store.commit(input(&st, "r1", "p1", None)).unwrap();
    let out = store.commit(input(&st, "r1", "p1", None)).unwrap();
    assert_eq!(
        out,
        CommitOutcome::Replayed {
            reply_json: "{\"reply\":\"p1\"}".into(),
            effects_json: "[]".into(),
            published: false,
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
fn allocate_seq_starts_at_1_per_day_and_increments() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    assert_eq!(store.allocate_seq("2026-09-24").unwrap(), 1);
    assert_eq!(store.allocate_seq("2026-09-24").unwrap(), 2);
    assert_eq!(store.allocate_seq("2026-09-25").unwrap(), 1);
}

// Task: T13
#[test]
fn allocate_seq_rejects_1000th_of_day() {
    let (_d, home) = temp_home();
    let store = open_rw(&home);
    // Replacement: C002-T31；历史T13完成事实保留，合法边界夹具加强接受与回滚oracle。
    let connection = rusqlite::Connection::open(store.path().as_str()).unwrap();
    connection
        .execute(
            "INSERT INTO work_sequence (day,last) VALUES ('2026-09-24',997)",
            [],
        )
        .unwrap();
    assert_eq!(store.allocate_seq("2026-09-24").unwrap(), 998);
    assert_eq!(store.allocate_seq("2026-09-24").unwrap(), 999);
    assert!(matches!(
        store.allocate_seq("2026-09-24"),
        Err(Error::InvalidRequest { .. })
    ));
    assert_eq!(
        connection
            .query_row(
                "SELECT last FROM work_sequence WHERE day='2026-09-24'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        999
    );
}

// Task: C002-T31
#[test]
fn readonly_connections_keep_checkpoint_on_close_disabled() {
    let (_dir, home) = temp_home();
    drop(open_rw(&home));
    let store = Store::open_for_home(&home, OpenMode::ReadOnly).unwrap();
    let connection = store.connect().unwrap();
    assert!(
        connection
            .db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE)
            .unwrap()
    );
}

// Task: T13
#[test]
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

// Task: C002-T14
#[test]
fn load_rejects_row_identity_status_and_revision_mismatch() {
    for corruption in ["identity", "status", "revision", "revision_zero"] {
        let (_dir, home) = temp_home();
        let store = open_rw(&home);
        let state = state_fixture();
        store.commit(input(&state, "r1", "p1", None)).unwrap();
        let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
        match corruption {
            "identity" => {
                let mut json = serde_json::to_value(&state).unwrap();
                json["work_id"] = serde_json::json!("2026-09-24-999-other");
                conn.execute("UPDATE works SET state_json = ?1", [json.to_string()])
                    .unwrap();
            }
            "status" => {
                conn.execute("UPDATE works SET status = 'succeeded'", [])
                    .unwrap();
            }
            "revision" => {
                conn.execute("UPDATE works SET revision = -1", []).unwrap();
            }
            "revision_zero" => {
                conn.execute("UPDATE works SET revision = 0", []).unwrap();
            }
            _ => unreachable!(),
        }
        let error = store.load_work(&state.work_id).unwrap_err();
        assert!(
            matches!(&error, Error::StoreCorrupt { .. }),
            "{corruption}: {error}"
        );
        assert!(
            matches!(store.list_works(), Err(Error::StoreCorrupt { .. })),
            "{corruption} must also stop list"
        );
    }
}

// Task: C002-T14
#[test]
fn load_rejects_invalid_state_combination() {
    let (_dir, home) = temp_home();
    let store = open_rw(&home);
    let state = state_fixture();
    store.commit(input(&state, "r1", "p1", None)).unwrap();
    let mut json = serde_json::to_value(&state).unwrap();
    json["status"] = serde_json::json!({"kind": "blocked", "reason": "gate"});
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    conn.execute(
        "UPDATE works SET status = 'blocked', state_json = ?1",
        [json.to_string()],
    )
    .unwrap();
    let error = store.load_work(&state.work_id).unwrap_err();
    assert!(matches!(&error, Error::StoreCorrupt { .. }), "{error}");
    assert!(error.to_string().contains("status 与当前 Attempt"));
}

// Task: C002-T14
#[test]
fn load_rejects_running_attempt_with_end_time() {
    let (_dir, home) = temp_home();
    let store = open_rw(&home);
    let mut fixture = Fixture::two_step().started_with(&[("topic", "t")]);
    store
        .commit(input(fixture.state(), "r1", "start", None))
        .unwrap();
    let node = fixture.state().current.node.as_str().to_string();
    fixture.begin(&node).unwrap();
    store
        .commit(input(fixture.state(), "r2", "begin", Some(1)))
        .unwrap();

    let mut json = serde_json::to_value(fixture.state()).unwrap();
    json["attempts"][0]["ended_at"] = serde_json::json!("2026-09-24T03:00:00Z");
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    conn.execute("UPDATE works SET state_json = ?1", [json.to_string()])
        .unwrap();
    let error = store.load_work(&fixture.state().work_id).unwrap_err();
    assert!(matches!(&error, Error::StoreCorrupt { .. }), "{error}");
    assert!(error.to_string().contains("attempts[0]"));
}

// Task: C002-T14
#[test]
fn load_rejects_running_attempt_on_different_current_occurrence() {
    let (_dir, home) = temp_home();
    let store = open_rw(&home);
    let mut fixture = Fixture::two_step().started_with(&[("topic", "t")]);
    fixture.begin("outline").unwrap();
    fixture.submit_ok("outline#1.0", "完成提纲").unwrap();
    fixture.begin("summary").unwrap();
    let state = fixture.state();
    assert!(state.validate_persisted().is_ok());

    let mut json = serde_json::to_value(state).unwrap();
    json["current"]["node"] = serde_json::json!("outline");
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    conn.execute(
        "INSERT INTO works (work_id, revision, status, state_json, created_at, updated_at)
         VALUES (?1, 4, 'active', ?2, ?3, ?4)",
        rusqlite::params![
            state.work_id.as_str(),
            json.to_string(),
            state.created_at.as_str(),
            state.updated_at.as_str()
        ],
    )
    .unwrap();
    let error = store.load_work(&state.work_id).unwrap_err();
    assert!(matches!(&error, Error::StoreCorrupt { .. }), "{error}");
    assert!(error.to_string().contains("running Attempt 与 current"));
}

// Task: C002-T14
#[test]
fn commit_rejects_mismatched_state_identity_without_writing() {
    let (_dir, home) = temp_home();
    let store = open_rw(&home);
    let state = state_fixture();
    let mut wrong = input(&state, "r1", "p1", None);
    wrong.work_id = Some(sheltie_core::ids::WorkId::parse("2026-09-24-999-other").unwrap());
    assert!(matches!(
        store.commit(wrong),
        Err(Error::StoreCorrupt { .. })
    ));
    assert!(store.list_works().unwrap().is_empty());
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    let requests: i64 = conn
        .query_row("SELECT count(*) FROM requests", [], |row| row.get(0))
        .unwrap();
    assert_eq!(requests, 0);
}

// Task: C002-T14
#[test]
fn commit_rejects_revision_overflow_without_negative_row() {
    let (_dir, home) = temp_home();
    let store = open_rw(&home);
    let state = state_fixture();
    let conn = rusqlite::Connection::open(store.path().as_str()).unwrap();
    conn.execute(
        "INSERT INTO works (work_id, revision, status, state_json, created_at, updated_at)
         VALUES (?1, ?2, 'active', ?3, ?4, ?5)",
        rusqlite::params![
            state.work_id.as_str(),
            i64::MAX,
            serde_json::to_string(&state).unwrap(),
            state.created_at.as_str(),
            state.updated_at.as_str()
        ],
    )
    .unwrap();
    assert_eq!(
        store.load_work(&state.work_id).unwrap().revision,
        i64::MAX as u64
    );
    let error = store
        .commit(input(&state, "r2", "after-max", Some(i64::MAX as u64)))
        .unwrap_err();
    assert!(matches!(&error, Error::StoreCorrupt { .. }), "{error}");
    let revision: i64 = conn
        .query_row("SELECT revision FROM works", [], |row| row.get(0))
        .unwrap();
    let requests: i64 = conn
        .query_row("SELECT count(*) FROM requests", [], |row| row.get(0))
        .unwrap();
    assert_eq!(revision, i64::MAX);
    assert_eq!(requests, 0);
}

// Task: C002-T14
#[test]
fn store_accepts_real_running_and_failed_attempt_states() {
    let (_dir, home) = temp_home();
    let store = open_rw(&home);
    let mut fixture = Fixture::two_step().started_with(&[("topic", "t")]);
    store
        .commit(input(fixture.state(), "r1", "start", None))
        .unwrap();

    let node = fixture.state().current.node.as_str().to_string();
    fixture.begin(&node).unwrap();
    store
        .commit(input(fixture.state(), "r2", "begin", Some(1)))
        .unwrap();
    assert_eq!(
        store.load_work(&fixture.state().work_id).unwrap().revision,
        2
    );

    fixture.fail(&format!("{node}#1.0"), "需要重试").unwrap();
    store
        .commit(input(fixture.state(), "r3", "fail", Some(2)))
        .unwrap();
    let row = store.load_work(&fixture.state().work_id).unwrap();
    assert_eq!(row.revision, 3);
    assert_eq!(&row.state, fixture.state());
}
