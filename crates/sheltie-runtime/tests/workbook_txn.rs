//! C002-T08：Workbook 事务、幂等与发布生命周期（N02/O08/§5.2）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_runtime::request::InputValue;
use sheltie_runtime::store::OpenMode;
use sheltie_runtime::{StartArgs, Store, WorkbookRepo};

fn make_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    let mode = if meta.is_dir() { 0o755 } else { 0o644 };
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
    if meta.is_dir() {
        for entry in std::fs::read_dir(path).unwrap().flatten() {
            make_writable(&entry.path());
        }
    }
}

fn lit(s: &str) -> InputValue {
    InputValue::Literal {
        text: s.to_string(),
    }
}

fn start_args() -> StartArgs {
    StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: [("topic".to_string(), lit("t"))].into_iter().collect(),
    }
}

/// remove 的引用检查在同一个事务里：有非终态 Work 时删行回滚（§5.2）。
// Task: C002-T08
#[test]
fn remove_rejects_active_reference_and_rolls_back_row() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());

    let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    let err = WorkbookRepo::new(home.clone(), store)
        .remove("two-step", "1.0.0", None)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookInUse, "{err:?}");

    // 行未删：只读库仍能列出，Work 仍可推进。
    let ro = Store::open(&home.store_path(), OpenMode::ReadOnly).unwrap();
    assert_eq!(WorkbookRepo::new(home.clone(), ro).list().unwrap().len(), 1);
    let (_, card) = svc.status(&wid).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Active);
}

/// 损坏的 works 行让 remove 停止（STORE_CORRUPT），不得按冗余列预筛后跳过。
// Task: C002-T08
#[test]
fn remove_stops_on_corrupt_reference_row() {
    let (_d, home) = temp_home();
    let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    let repo = WorkbookRepo::new(home.clone(), store);
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    // 直写一行损坏的 state_json。
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "INSERT INTO works (work_id, revision, status, state_json, created_at, updated_at)
         VALUES ('2026-09-24-001-x', 1, 'active', '不是 JSON', '2026-09-24T00:00:00Z', '2026-09-24T00:00:00Z')",
        [],
    )
    .unwrap();
    drop(conn);

    let err = repo.remove("two-step", "1.0.0", None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err:?}");
    let ro = Store::open(&home.store_path(), OpenMode::ReadOnly).unwrap();
    assert_eq!(WorkbookRepo::new(home.clone(), ro).list().unwrap().len(), 1);
}

/// 并行 add 各自 staging：互不删除，两个都成功（N02 反面）。
// Task: C002-T08
#[test]
fn parallel_adds_do_not_delete_each_others_staging() {
    let (_d, home) = temp_home();
    // 先串行建库（建库窗口本身由管理根写锁串行化；这里避免测试代码在锁外抢建）。
    drop(Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap());
    let a = {
        let home = home.clone();
        std::thread::spawn(move || {
            let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
            WorkbookRepo::new(home.clone(), store).add(&abs(&example_dir("two-step")), None)
        })
    };
    let b = {
        let home = home.clone();
        std::thread::spawn(move || {
            let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
            WorkbookRepo::new(home.clone(), store).add(&abs(&example_dir("gated-release")), None)
        })
    };
    a.join().unwrap().unwrap();
    b.join().unwrap().unwrap();
    let ro = Store::open(&home.store_path(), OpenMode::ReadOnly).unwrap();
    let rows = WorkbookRepo::new(home.clone(), ro).list().unwrap();
    assert_eq!(rows.len(), 2, "两个 Workbook 都在：{rows:?}");
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().exists());
    assert!(
        home.workbook_dir("gated-release", "1.0.0")
            .as_path()
            .exists()
    );
}

/// add 的发布窗口：COMMIT 后、rename 前被杀——下一次写操作先恢复发布（N02）。
// Task: C002-T08
#[test]
fn add_publish_window_recovered_by_next_write() {
    let (_d, home) = temp_home();
    let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    let repo = WorkbookRepo::new(home.clone(), store);
    repo.add(&abs(&example_dir("two-step")), Some("r-a".into()))
        .unwrap();
    // 模拟发布前被杀：行已提交、published 置 0、最终目录撤回 pending。
    let final_dir = home.workbook_dir("two-step", "1.0.0");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row("SELECT effects_json FROM requests", [], |r| r.get(0))
        .unwrap();
    conn.execute("UPDATE requests SET published = 0", [])
        .unwrap();
    drop(conn);
    let pending_rel = serde_json::from_str::<serde_json::Value>(&effects).unwrap()[0]["pending"]
        .as_str()
        .unwrap()
        .to_string();
    // 已发布的树是只读的（0555/0444）：模拟撤回先放开权限（同生产 delete_dir）。
    make_writable(std::path::Path::new(final_dir.as_str()));
    std::fs::rename(
        final_dir.as_path(),
        home.rel(&pending_rel).unwrap().as_path(),
    )
    .unwrap();
    assert!(!final_dir.as_path().exists());

    // 下一个写操作先恢复：目录回到最终位置、只读、行 published。
    let store2 = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    WorkbookRepo::new(home.clone(), store2)
        .add(&abs(&example_dir("gated-release")), None)
        .unwrap();
    assert!(final_dir.as_path().exists(), "恢复发布了原件");
    let ro = Store::open(&home.store_path(), OpenMode::ReadOnly).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let published: i64 = conn
        .query_row(
            "SELECT published FROM requests WHERE request_id = 'r-a'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(published, 1);
    drop(conn);
    // 恢复后的目录可用（load 核摘要通过）。
    let wb = WorkbookRepo::new(home.clone(), ro)
        .load("two-step", Some("1.0.0"))
        .unwrap();
    assert_eq!(wb.manifest.id().as_str(), "two-step");
}

/// 旧 remove 的重放不得删除同版本的新对象；新生命周期不被旧请求覆盖（§5.2 第 4 条）。
// Task: C002-T08
#[test]
fn old_remove_replay_does_not_delete_readded_workbook() {
    let (_d, home) = temp_home();
    let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    let repo = WorkbookRepo::new(home.clone(), store);
    repo.add(&abs(&example_dir("two-step")), Some("r-a".into()))
        .unwrap();
    repo.remove("two-step", "1.0.0", Some("r-r".into()))
        .unwrap();
    // 同版本重新装（新对象、新请求）。
    let store2 = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    let repo2 = WorkbookRepo::new(home.clone(), store2);
    repo2
        .add(&abs(&example_dir("two-step")), Some("r-a2".into()))
        .unwrap();
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().exists());

    // 重放旧 remove：返回原快照，不删除新对象。
    let again = repo2
        .remove("two-step", "1.0.0", Some("r-r".into()))
        .unwrap();
    assert!(again.replayed);
    assert!(
        home.workbook_dir("two-step", "1.0.0").as_path().exists(),
        "旧 remove 的重放不得删除新生命周期对象"
    );
    // 重放旧 add 同样不覆盖新生命周期：快照原样，目录仍是新对象的字节。
    let readd = repo2
        .add(&abs(&example_dir("two-step")), Some("r-a".into()))
        .unwrap();
    assert!(readd.replayed);
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().exists());
    let ro = Store::open(&home.store_path(), OpenMode::ReadOnly).unwrap();
    let rows = WorkbookRepo::new(home.clone(), ro).list().unwrap();
    assert_eq!(rows.len(), 1);
}

// Task: C002-T20
#[test]
fn schema2_workbook_recovery_accepts_original_audit_command_bytes() {
    let (_d, home) = temp_home();
    let repo = repo(&home);
    let source = abs(&example_dir("two-step"));
    let add_request = "schema2-add-wire";
    repo.add(&source, Some(add_request.to_string())).unwrap();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let add_hash: String = conn
        .query_row(
            "SELECT intent_hash FROM requests WHERE request_id = ?1",
            [add_request],
            |row| row.get(0),
        )
        .unwrap();
    let original_add_audit = format!("{{\"intent\":\"add_workbook\",\"source\":\"{add_hash}\"}}");
    conn.execute(
        "UPDATE audit SET command_json = ?1 WHERE request_id = ?2",
        rusqlite::params![original_add_audit, add_request],
    )
    .unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [add_request],
    )
    .unwrap();
    drop(conn);

    let remove_request = "schema2-remove-wire";
    repo.remove("two-step", "1.0.0", Some(remove_request.to_string()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE audit SET command_json = '{\"intent\":\"remove_workbook\",\"target\":\"two-step@1.0.0\"}' WHERE request_id = ?1",
        [remove_request],
    )
    .unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [remove_request],
    )
    .unwrap();
    drop(conn);

    let readded = repo
        .add(&source, Some("schema2-readd-after-remove".to_string()))
        .unwrap();
    assert_eq!(readded.data["id"], "two-step");
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    for request in [add_request, remove_request] {
        let published: i64 = conn
            .query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [request],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(published, 1, "旧schema2效果请求 {request} 应被恢复");
    }
}

/// 删除完成标记：remove 完成后 `pending/<id>.deleted` 存在（§3.3）。
// Task: C002-T08
#[test]
fn remove_writes_deleted_marker() {
    let (_d, home) = temp_home();
    let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    let repo = WorkbookRepo::new(home.clone(), store);
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    repo.remove("two-step", "1.0.0", None).unwrap();
    let pending = home.pending_dir();
    let markers: Vec<_> = std::fs::read_dir(pending.as_path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".deleted"))
        .collect();
    assert_eq!(markers.len(), 1, "恰一个完成标记：{markers:?}");
    let content = std::fs::read_to_string(Path::new(pending.as_str()).join(&markers[0])).unwrap();
    assert!(content.contains("delete-complete/v1"), "{content}");
}

/// 请求去重全局共用：同 request-id 用在 add 与 start 之间冲突（REQUEST_CONFLICT）。
// Task: C002-T08
#[test]
fn request_ids_share_one_global_namespace() {
    let (_d, home, svc) = home_with_example("two-step");
    // 先用 r-x 完成一次 add（另一个 Workbook）。
    let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    WorkbookRepo::new(home.clone(), store)
        .add(&abs(&example_dir("gated-release")), Some("r-x".into()))
        .unwrap();
    // 同 id 的 start 意图不同 → 冲突，不产生 Work。
    let err = svc.start(start_args(), Some("r-x".into())).unwrap_err();
    assert_eq!(err.code(), ErrorCode::RequestConflict, "{err:?}");
    assert!(svc.list().unwrap().is_empty());
}

/// add/remove 重放返回原 snapshot（O08 正例，快照字段逐项比较）。
// Task: C002-T08
#[test]
fn add_and_remove_replay_return_original_snapshots() {
    let (_d, home) = temp_home();
    let store = Store::open(&home.store_path(), OpenMode::ReadWrite).unwrap();
    let repo = WorkbookRepo::new(home.clone(), store);
    let add1 = repo
        .add(&abs(&example_dir("two-step")), Some("r-add".into()))
        .unwrap();
    let add2 = repo
        .add(&abs(&example_dir("two-step")), Some("r-add".into()))
        .unwrap();
    assert!(add2.replayed);
    assert_eq!(add1.data, add2.data);
    assert_eq!(add1.request_id, add2.request_id);

    let rm1 = repo
        .remove("two-step", "1.0.0", Some("r-rm".into()))
        .unwrap();
    let rm2 = repo
        .remove("two-step", "1.0.0", Some("r-rm".into()))
        .unwrap();
    assert!(rm2.replayed);
    assert_eq!(rm1.data, rm2.data);
}
