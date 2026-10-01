//! T14、T15：Workbook 仓库。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_core::testkit::Fixture;
use sheltie_runtime::workbook_repo::VerifyStatus;
use sheltie_runtime::{Error, WorkbookRepo};

fn is_readonly(p: &Path) -> bool {
    p.metadata().unwrap().permissions().readonly()
}

// ── T14 ───────────────────────────────────────────────────────

// Task: T14
#[test]
fn add_two_step_example_copies_and_marks_readonly() {
    let (_d, home) = temp_home();
    let added = repo(&home)
        .add(&abs(&example_dir("two-step")), None)
        .unwrap();
    assert_eq!(
        (
            added.data["id"].as_str().unwrap(),
            added.data["version"].as_str().unwrap()
        ),
        ("two-step", "1.0.0")
    );
    let dir =
        std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).to_path_buf();
    assert!(dir.join("workbook.toml").exists());
    assert!(dir.join("instructions/outline.md").exists());
    assert!(is_readonly(&dir.join("workbook.toml")));
    assert_eq!(
        added.data["digest"].as_str().unwrap(),
        WorkbookRepo::digest_dir(&abs(&dir)).unwrap().as_str()
    );
}

// Task: T14
#[test]
fn add_rejects_duplicate_id_version_with_workbook_exists() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    assert!(matches!(
        r.add(&abs(&example_dir("two-step")), None),
        Err(Error::WorkbookExists { .. })
    ));
}

// Task: T14
#[test]
fn add_rejects_symlink_inside_workbook() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    std::os::unix::fs::symlink("/etc/hosts", src.join("instructions/evil.md")).unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
    assert!(!std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T14
#[test]
fn add_rejects_file_over_32mib() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    let big = std::fs::File::create(src.join("instructions/big.bin")).unwrap();
    big.set_len(33_554_433).unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

// Task: T14
#[test]
fn add_rejects_non_regular_file() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    let status = std::process::Command::new("mkfifo")
        .arg(src.join("instructions/pipe"))
        .status()
        .unwrap();
    assert!(status.success());
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

// Task: T14
#[test]
fn add_failure_leaves_no_staging_and_no_row() {
    let (d, home) = temp_home();
    let repo = repo(&home);
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    let before_rows = repo.list().unwrap().len();
    let src = copy_example("two-step", d.path());
    std::fs::write(src.join("workbook.toml"), "schema = \"workbook/v9\"\n").unwrap();
    assert!(repo.add(&abs(&src), None).is_err());
    let staging = std::path::PathBuf::from(home.staging_dir().as_str());
    assert!(!staging.exists() || std::fs::read_dir(staging).unwrap().next().is_none());
    assert_eq!(repo.list().unwrap().len(), before_rows, "失败不得插入新行");
}

// T31按GF-17/GF-30替换T24的内容错误不建库断言；原始T24证据保留。
// Task: C002-T31
#[test]
fn invalid_source_structure_on_new_home_does_not_create_store_or_lock() {
    let (d, home) = temp_home();
    let source = copy_example("two-step", d.path());
    std::fs::remove_file(source.join("workbook.toml")).unwrap();
    assert!(repo(&home).add(&abs(&source), None).is_err());
    assert!(!home.store_path().as_path().exists());
    assert!(!home.lock_path().as_path().exists());
    assert!(!home.staging_dir().as_path().exists());
}

// Task: C002-T31
#[test]
fn invalid_private_copy_has_no_business_rows_and_the_same_request_can_retry() {
    let (directory, home) = temp_home();
    let source = copy_example("two-step", directory.path());
    let manifest = std::fs::read(source.join("workbook.toml")).unwrap();
    std::fs::write(source.join("workbook.toml"), "schema = \"workbook/v9\"\n").unwrap();
    let repository = repo(&home);
    let request = "invalid-content";
    assert!(repository.add(&abs(&source), Some(request.into())).is_err());
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    for table in ["workbooks", "works", "requests", "audit", "work_sequence"] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0,
            "{table}"
        );
    }
    assert!(!home.workbook_dir("two-step", "1.0.0").as_path().exists());
    std::fs::write(source.join("workbook.toml"), manifest).unwrap();
    let added = repository.add(&abs(&source), Some(request.into())).unwrap();
    assert_eq!(added.data["id"], "two-step");
    assert!(!added.replayed);
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM requests WHERE request_id=?1",
                [request],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(
        repository
            .add(&abs(&source), Some(request.into()))
            .unwrap()
            .replayed
    );
}

// Task: T14
#[test]
fn add_rejects_hard_link_inside_workbook() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    std::fs::hard_link(src.join("workbook.toml"), src.join("instructions/hard.md")).unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

/// 一个最小的合法 Workbook，用来凑文件大小边界。
fn write_minimal_workbook(dir: &Path) {
    std::fs::create_dir_all(dir.join("flows")).unwrap();
    std::fs::write(
        dir.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"big\"\nversion = \"1.0.0\"\nname = \"大小边界\"\ndescription = \"凑上限用。\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("flows/default.toml"),
        "schema = \"flow/v1\"\nid = \"default\"\nentry = \"a\"\n\n[[nodes]]\nid = \"a\"\ntitle = \"甲\"\nexecutor = \"agent\"\ninstruction = { text = \"做。\" }\noutputs = [{ name = \"x\", path = \"x.md\", max_bytes = 65536 }]\n",
    )
    .unwrap();
}

/// 目录现有小文件的总字节数（只算普通文件，与 `copy_confined` 的口径一致）。
fn small_files_bytes(dir: &Path) -> u64 {
    walk(dir)
        .iter()
        .filter(|p| p.is_file())
        .map(|p| p.metadata().unwrap().len())
        .sum()
}

// Task: T14
#[test]
fn add_accepts_files_at_exact_limits() {
    // 7 个恰好 32 MiB 的文件加一个凑数文件，总量恰好 256 MiB：两个上限都顶到且接受。
    let (d, home) = temp_home();
    let src = d.path().join("big");
    write_minimal_workbook(&src);
    let small = small_files_bytes(&src);
    let max_file = sheltie_runtime::workbook_repo::MAX_FILE_BYTES;
    let max_total = sheltie_runtime::workbook_repo::MAX_TOTAL_BYTES;
    for i in 0..7 {
        std::fs::File::create(src.join(format!("big{i}.bin")))
            .unwrap()
            .set_len(max_file)
            .unwrap();
    }
    let last = max_total - 7 * max_file - small;
    assert!(last > 0 && last <= max_file);
    std::fs::File::create(src.join("big7.bin"))
        .unwrap()
        .set_len(last)
        .unwrap();
    repo(&home).add(&abs(&src), None).unwrap();
}

// Task: T14
#[test]
fn add_rejects_when_total_over_256mib() {
    // 总量上限多 1 字节。
    let (d, home) = temp_home();
    let src = d.path().join("big");
    write_minimal_workbook(&src);
    let small = small_files_bytes(&src);
    let max_file = sheltie_runtime::workbook_repo::MAX_FILE_BYTES;
    let max_total = sheltie_runtime::workbook_repo::MAX_TOTAL_BYTES;
    for i in 0..7 {
        std::fs::File::create(src.join(format!("big{i}.bin")))
            .unwrap()
            .set_len(max_file)
            .unwrap();
    }
    std::fs::File::create(src.join("big7.bin"))
        .unwrap()
        .set_len(max_total - 7 * max_file - small + 1)
        .unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

// Task: C002-T21
#[test]
fn total_limit_rejects_before_staging_or_registering_source_files() {
    let (d, home) = temp_home();
    let src = d.path().join("over-limit");
    write_minimal_workbook(&src);
    let small = small_files_bytes(&src);
    let max_file = sheltie_runtime::workbook_repo::MAX_FILE_BYTES;
    let max_total = sheltie_runtime::workbook_repo::MAX_TOTAL_BYTES;
    for i in 0..7 {
        std::fs::File::create(src.join(format!("large-{i}.bin")))
            .unwrap()
            .set_len(max_file)
            .unwrap();
    }
    let unreadable = src.join("large-0.bin");
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();
    std::fs::File::create(src.join("last.bin"))
        .unwrap()
        .set_len(max_total - 7 * max_file - small + 1)
        .unwrap();

    let repo = repo(&home);
    let result = repo.add(&abs(&src), None);
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(result, Err(Error::InvalidRequest { reason }) if reason.contains("总量")));
    assert!(!home.store_path().as_path().exists());
    assert!(!home.lock_path().as_path().exists());
    assert!(!home.pending_dir().as_path().exists());
}

// Task: T14
#[test]
fn load_with_explicit_version_picks_that_version() {
    let (d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let src = copy_example("two-step", d.path());
    let m = std::fs::read_to_string(src.join("workbook.toml"))
        .unwrap()
        .replace("1.0.0", "1.1.0");
    std::fs::write(src.join("workbook.toml"), m).unwrap();
    r.add(&abs(&src), None).unwrap();
    assert_eq!(
        r.load("two-step", Some("1.0.0"))
            .unwrap()
            .manifest
            .version(),
        "1.0.0"
    );
    assert_eq!(
        r.load("two-step", None).unwrap().manifest.version(),
        "1.1.0"
    );
}

// Task: T14
#[test]
fn load_recompiles_graph_from_installed_copy() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("article-review")), None).unwrap();
    let loaded = r.load("article-review", None).unwrap();
    assert_eq!(loaded.flows.len(), 1);
    assert_eq!(loaded.flow("default").unwrap().1.node_count(), 3);
    assert!(matches!(r.load("nope", None), Err(Error::NotFound { .. })));
}

// Task: T14
#[test]
fn list_orders_by_id_then_version() {
    let (d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let src = copy_example("two-step", d.path());
    let m = std::fs::read_to_string(src.join("workbook.toml"))
        .unwrap()
        .replace("1.0.0", "1.1.0");
    std::fs::write(src.join("workbook.toml"), m).unwrap();
    r.add(&abs(&src), None).unwrap();
    r.add(&abs(&example_dir("article-review")), None).unwrap();
    let rows: Vec<(String, String)> = r
        .list()
        .unwrap()
        .into_iter()
        .map(|w| (w.id, w.version))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("article-review".into(), "1.0.0".into()),
            ("two-step".into(), "1.0.0".into()),
            ("two-step".into(), "1.1.0".into())
        ]
    );
}

// ── T15 ───────────────────────────────────────────────────────

// Task: T15
#[test]
fn remove_deletes_row_and_directory() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    r.remove("two-step", "1.0.0", None).unwrap();
    assert!(r.list().unwrap().is_empty());
    assert!(!std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T15
#[test]
fn remove_requires_explicit_version() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    assert!(matches!(
        r.remove("two-step", "", None),
        Err(Error::InvalidRequest { .. })
    ));
}

// Task: C002-T14
#[test]
fn remove_stops_on_corrupt_row_even_when_redundant_status_looks_terminal() {
    let (_dir, home) = temp_home();
    let repo = repo(&home);
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    let state = Fixture::two_step()
        .started_with(&[("topic", "t")])
        .state()
        .clone();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "INSERT INTO works (work_id, revision, status, state_json, created_at, updated_at)
         VALUES (?1, 1, 'succeeded', ?2, ?3, ?4)",
        rusqlite::params![
            state.work_id.as_str(),
            serde_json::to_string(&state).unwrap(),
            state.created_at.as_str(),
            state.updated_at.as_str()
        ],
    )
    .unwrap();
    let error = repo.remove("two-step", "1.0.0", None).unwrap_err();
    assert!(matches!(&error, Error::StoreCorrupt { .. }), "{error}");
    assert_eq!(repo.list().unwrap().len(), 1);
    assert!(std::path::Path::new(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T15
#[test]
fn remove_rejects_when_active_work_references_version() {
    let (_d, home, svc) = home_with_example("two-step");
    let resp = start_two_step(&svc);
    let err = repo(&home).remove("two-step", "1.0.0", None).unwrap_err();
    match err {
        Error::WorkbookInUse { works, .. } => assert_eq!(works, vec![work_id_of(&resp)]),
        other => panic!("{other:?}"),
    }
}

// Task: T15
#[test]
fn remove_allows_when_only_terminal_works_reference_version() {
    let (_d, home, svc) = home_with_example("two-step");
    let resp = start_two_step(&svc);
    svc.cancel(&work_id_of(&resp), None).unwrap();
    repo(&home).remove("two-step", "1.0.0", None).unwrap();
}

// Task: T15
#[test]
fn remove_allows_when_other_workbook_shares_version() {
    // 活跃 Work 引用 two-step@1.0.0；同号的 article-review@1.0.0 不受牵连。
    let (_d, home, svc) = home_with_example("two-step");
    let r = repo(&home);
    r.add(&abs(&example_dir("article-review")), None).unwrap();
    let _ = start_two_step(&svc);
    r.remove("article-review", "1.0.0", None).unwrap();
    assert!(matches!(
        r.load("article-review", None),
        Err(Error::NotFound { .. })
    ));
}

// Task: T15
#[test]
fn remove_moves_dir_to_tmp_before_delete() {
    // 观察不到中间态就看结果：目录消失、tmp 下无残留。
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    r.remove("two-step", "1.0.0", None).unwrap();
    let tmp = std::path::PathBuf::from(home.tmp_dir().as_str());
    assert!(!tmp.exists() || std::fs::read_dir(tmp).unwrap().next().is_none());
}

// Task: T15
#[test]
fn verify_reports_ok_for_untouched_install() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let rows = r.verify(Some(("two-step", "1.0.0"))).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, VerifyStatus::Ok);
}

// Task: T15
#[test]
fn verify_reports_tampered_after_byte_change() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let f = std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str())
        .join("instructions/outline.md");
    std::fs::set_permissions(&f, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
    std::fs::write(&f, "改了").unwrap();
    assert_eq!(r.verify(None).unwrap()[0].status, VerifyStatus::Tampered);
}

// Task: T15
#[test]
fn verify_reports_missing_when_directory_gone() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let dir = std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str());
    // T04 起整棵含根置只读（0555）；删除前把根本身也放开。
    std::fs::set_permissions(&dir, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    for entry in walk(&dir) {
        // 目录保留可遍历的 0755（同本文件 `make_writable` 的约定），否则 remove_dir_all 进不了子目录。
        let mode = if entry.is_dir() { 0o755 } else { 0o644 };
        std::fs::set_permissions(&entry, std::os::unix::fs::PermissionsExt::from_mode(mode))
            .unwrap();
    }
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(r.verify(None).unwrap()[0].status, VerifyStatus::Missing);
}

// Task: C002-T20
#[test]
fn corrupted_workbook_row_path_is_rejected_before_verify_or_remove_io() {
    use std::os::unix::fs::PermissionsExt;

    let (_d, home) = temp_home();
    let workbooks = repo(&home);
    workbooks.add(&abs(&example_dir("two-step")), None).unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("external-workbook");
    std::fs::write(&sentinel, b"external data").unwrap();
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o640)).unwrap();
    let before = std::fs::read(&sentinel).unwrap();
    let mode_before = std::fs::metadata(&sentinel).unwrap().permissions().mode();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE workbooks SET dir = ?1 WHERE id = 'two-step' AND version = '1.0.0'",
        [outside.path().to_string_lossy().as_ref()],
    )
    .unwrap();
    drop(conn);

    assert_eq!(
        workbooks
            .verify(Some(("two-step", "1.0.0")))
            .unwrap_err()
            .code(),
        sheltie_core::ErrorCode::StoreCorrupt
    );
    assert_eq!(
        workbooks
            .remove("two-step", "1.0.0", Some("remove-corrupt-row".to_string()))
            .unwrap_err()
            .code(),
        sheltie_core::ErrorCode::StoreCorrupt
    );
    assert_eq!(std::fs::read(&sentinel).unwrap(), before);
    assert_eq!(
        std::fs::metadata(&sentinel).unwrap().permissions().mode(),
        mode_before
    );
}

// Task: C002-T20
#[test]
fn workbook_row_version_must_be_one_manifest_compatible_path_segment() {
    let (_d, home) = temp_home();
    let workbooks = repo(&home);
    workbooks.add(&abs(&example_dir("two-step")), None).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE workbooks SET version = '1.0.0/nested', dir = 'workbooks/two-step/1.0.0/nested' WHERE id = 'two-step' AND version = '1.0.0'",
        [],
    )
    .unwrap();
    drop(conn);

    let err = workbooks.verify(None).unwrap_err();
    assert_eq!(err.code(), sheltie_core::ErrorCode::StoreCorrupt, "{err:?}");
}

// Task: C002-T25
#[test]
fn empty_remove_digest_blocks_later_workbook_write() {
    let (_d, home) = temp_home();
    let workbooks = repo(&home);
    workbooks.add(&abs(&example_dir("two-step")), None).unwrap();
    let request_id = "remove-before-corruption";
    workbooks
        .remove("two-step", "1.0.0", Some(request_id.to_string()))
        .unwrap();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects_json: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let mut effects: serde_json::Value = serde_json::from_str(&effects_json).unwrap();
    effects[0]["digest"] = serde_json::Value::String(String::new());
    conn.execute(
        "UPDATE requests SET effects_json = ?1, published = 0 WHERE request_id = ?2",
        rusqlite::params![serde_json::to_string(&effects).unwrap(), request_id],
    )
    .unwrap();
    drop(conn);

    let err = workbooks
        .add(
            &abs(&example_dir("two-step")),
            Some("must-not-commit".to_string()),
        )
        .unwrap_err();
    assert_effect_pending(err, false, None, Some(request_id));
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = 'must-not-commit'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

// Task: C002-T25
#[test]
fn workbook_publish_requires_digest_root_before_recovery_io() {
    let (_d, home) = temp_home();
    let workbooks = repo(&home);
    let request_id = "publish-requires-digest-root";
    workbooks
        .add(&abs(&example_dir("two-step")), Some(request_id.to_string()))
        .unwrap();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let intent_hash: String = conn
        .query_row(
            "SELECT intent_hash FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE audit SET command_json = ?1 WHERE request_id = ?2",
        rusqlite::params![
            format!("{{\"intent\":\"add_workbook\",\"source\":\"{intent_hash}\"}}"),
            request_id
        ],
    )
    .unwrap();
    let effects_json: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let mut effects: serde_json::Value = serde_json::from_str(&effects_json).unwrap();
    effects[0].as_object_mut().unwrap().remove("digest_root");
    conn.execute(
        "UPDATE requests SET effects_json = ?1, published = 0 WHERE request_id = ?2",
        rusqlite::params![serde_json::to_string(&effects).unwrap(), request_id],
    )
    .unwrap();
    drop(conn);

    let err = workbooks
        .remove(
            "two-step",
            "1.0.0",
            Some("remove-after-missing-field".to_string()),
        )
        .unwrap_err();
    assert_effect_pending(err, false, None, Some(request_id));
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = 'remove-after-missing-field'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

// Task: C002-T25
#[test]
fn malformed_historical_workbook_snapshot_is_not_projected_as_success() {
    let (_dir, home) = temp_home();
    let workbooks = repo(&home);
    let request_id = "t25-malformed-workbook-snapshot";
    workbooks
        .add(&abs(&example_dir("two-step")), Some(request_id.to_string()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let reply: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let mut snapshot: serde_json::Value = serde_json::from_str(&reply).unwrap();
    snapshot["data"].as_object_mut().unwrap().remove("id");
    conn.execute(
        "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
        rusqlite::params![serde_json::to_string(&snapshot).unwrap(), request_id],
    )
    .unwrap();
    drop(conn);

    let error = workbooks
        .add(&abs(&example_dir("two-step")), Some(request_id.to_string()))
        .unwrap_err();
    assert_effect_pending_without_original(error, true, request_id, None);
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let reply: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(reply, serde_json::to_string(&snapshot).unwrap());
}

// Task: C002-T25
#[test]
fn unpublished_workbook_add_requires_its_owner_sidecar() {
    let (_d, home) = temp_home();
    let workbooks = repo(&home);
    let request_id = "missing-add-owner";
    workbooks
        .add(&abs(&example_dir("two-step")), Some(request_id.to_string()))
        .unwrap();
    let sidecar = std::fs::read_dir(home.pending_dir().as_path())
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".owner"))
                && serde_json::from_slice::<serde_json::Value>(&std::fs::read(path).unwrap())
                    .is_ok_and(|owner| owner["request_id"] == request_id)
        })
        .unwrap();
    std::fs::remove_file(sidecar).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [request_id],
    )
    .unwrap();
    drop(conn);

    let err = workbooks
        .remove(
            "two-step",
            "1.0.0",
            Some("remove-without-owner".to_string()),
        )
        .unwrap_err();
    assert_effect_pending(err, false, None, Some(request_id));
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = 'remove-without-owner'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

// Task: T15
#[test]
fn verify_all_when_filter_omitted() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    r.add(&abs(&example_dir("gated-release")), None).unwrap();
    assert_eq!(r.verify(None).unwrap().len(), 2);
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walk(&p));
        }
        out.push(p);
    }
    out
}

// Task: C002-T32
#[test]
fn installed_manifest_accepts_exact_file_limit_and_rejects_one_more_byte() {
    const LIMIT: usize = 32 * 1024 * 1024;
    for extra in [0, 1] {
        let (directory, home) = temp_home();
        let source = directory.path().join("source");
        write_minimal_workbook(&source);
        let manifest_path = source.join("workbook.toml");
        let mut expected = std::fs::read(&manifest_path).unwrap();
        expected.extend_from_slice(b"\n#");
        expected.resize(LIMIT + extra, b'x');
        std::fs::write(&manifest_path, &expected).unwrap();
        let repository = repo(&home);
        let result = repository.add(&abs(&source), Some("manifest-limit".to_string()));
        if extra == 0 {
            let response = result.unwrap();
            assert_eq!(response.data["id"], "big");
            assert_eq!(
                std::fs::read(
                    home.workbook_dir("big", "1.0.0")
                        .as_path()
                        .join("workbook.toml")
                )
                .unwrap(),
                expected
            );
            assert_eq!(
                repository.verify(Some(("big", "1.0.0"))).unwrap()[0].status,
                VerifyStatus::Ok
            );
        } else {
            assert!(matches!(result, Err(Error::InvalidRequest { .. })));
            assert!(!home.store_path().as_path().exists());
            assert!(!home.pending_dir().as_path().exists());
        }
    }
}

// Task: C002-T33
#[test]
fn removing_an_unrelated_workbook_preserves_the_registered_neighbor() {
    let (directory, home) = temp_home();
    let repository = repo(&home);
    repository
        .add(&abs(&example_dir("two-step")), Some("neighbor-a".into()))
        .unwrap();
    let second = copy_example("two-step", directory.path());
    let manifest = std::fs::read_to_string(second.join("workbook.toml")).unwrap();
    std::fs::write(
        second.join("workbook.toml"),
        manifest.replace("id = \"two-step\"", "id = \"other-step\""),
    )
    .unwrap();
    repository
        .add(&abs(&second), Some("neighbor-b".into()))
        .unwrap();
    repository
        .remove("other-step", "1.0.0", Some("remove-neighbor-b".into()))
        .unwrap();
    let rows = repository.list().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "two-step");
    assert_eq!(
        repository
            .load("two-step", Some("1.0.0"))
            .unwrap()
            .manifest
            .id()
            .as_str(),
        "two-step"
    );
    assert_eq!(
        repository.verify(Some(("two-step", "1.0.0"))).unwrap()[0].status,
        VerifyStatus::Ok
    );
}

// Task: C002-T33
#[test]
fn missing_installed_manifest_is_tampered_instead_of_a_missing_workbook() {
    let (_directory, home) = temp_home();
    let repository = repo(&home);
    repository
        .add(&abs(&example_dir("two-step")), None)
        .unwrap();
    let installed = home.workbook_dir("two-step", "1.0.0");
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(installed.as_path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::remove_file(installed.as_path().join("workbook.toml")).unwrap();
    let rows = repository.verify(Some(("two-step", "1.0.0"))).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, VerifyStatus::Tampered);
    assert_eq!(rows[0].id, "two-step");
    assert!(installed.as_path().is_dir());
}

// Task: C002-T33
#[cfg(feature = "failpoint")]
#[test]
fn workbook_copy_rejects_an_observed_replacement_even_if_the_original_is_restored_before_open() {
    use sheltie_runtime::failpoint;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            let _ = failpoint::disarm_rendezvous();
        }
    }
    let _reset = Reset;
    let (directory, home) = temp_home();
    let source = copy_example("two-step", directory.path());
    let manifest = std::fs::canonicalize(source.join("workbook.toml")).unwrap();
    let original = std::fs::read(&manifest).unwrap();
    let before = tempfile::tempdir().unwrap();
    let after = tempfile::tempdir().unwrap();
    failpoint::arm_rendezvous(
        "external_tree_before_stat",
        manifest.to_str().unwrap(),
        before.path(),
    )
    .unwrap();
    let repository = repo(&home);
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        sender
            .send(repository.add(&abs(&source), Some("copy-observed-replacement".into())))
            .unwrap()
    });
    let mut worker = RendezvousWorker::new(worker, before.path(), after.path());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !before.path().join("reached").exists() {
        assert!(Instant::now() < deadline);
        assert!(
            receiver.try_recv().is_err(),
            "copy finished before observing the selected source file"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    let parked = directory.path().join("original-manifest");
    std::fs::rename(&manifest, &parked).unwrap();
    std::fs::write(&manifest, &original).unwrap();
    failpoint::arm_rendezvous(
        "external_tree_after_stat",
        manifest.to_str().unwrap(),
        after.path(),
    )
    .unwrap();
    std::fs::write(before.path().join("release"), b"release").unwrap();
    let result = loop {
        if let Ok(result) = receiver.try_recv() {
            break result;
        }
        if after.path().join("reached").exists() {
            std::fs::remove_file(&manifest).unwrap();
            std::fs::rename(&parked, &manifest).unwrap();
            std::fs::write(after.path().join("release"), b"release").unwrap();
            break receiver.recv_timeout(Duration::from_secs(10)).unwrap();
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    worker.finish().unwrap();
    if parked.exists() {
        std::fs::remove_file(&manifest).unwrap();
        std::fs::rename(&parked, &manifest).unwrap();
    }
    assert!(matches!(result, Err(Error::InvalidRequest { .. })));
    assert_eq!(std::fs::read(&manifest).unwrap(), original);
    if home.store_path().as_path().exists() {
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        for table in ["workbooks", "requests", "audit"] {
            assert_eq!(
                conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row
                    .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
    }
}
