//! C002-T08：Workbook 事务、幂等与发布生命周期（N02/O08/§5.2）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_runtime::request::InputValue;
use sheltie_runtime::{Error, StartArgs, WorkbookRepo};

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

    let err = WorkbookRepo::new(home.clone())
        .remove("two-step", "1.0.0", None)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookInUse, "{err:?}");

    // 行未删：只读库仍能列出，Work 仍可推进。
    assert_eq!(WorkbookRepo::new(home.clone()).list().unwrap().len(), 1);
    let (_, card) = svc.status(&wid).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Active);
}

/// 损坏的 works 行让 remove 停止（STORE_CORRUPT），不得按冗余列预筛后跳过。
// Task: C002-T08
#[test]
fn remove_stops_on_corrupt_reference_row() {
    let (_d, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
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
    assert_eq!(WorkbookRepo::new(home.clone()).list().unwrap().len(), 1);
}

/// 并行 add 各自 staging：互不删除，两个都成功（N02 反面）。
// Task: C002-T08
#[test]
fn parallel_adds_do_not_delete_each_others_staging() {
    let (_d, home) = temp_home();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let a = {
        let home = home.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            WorkbookRepo::new(home.clone()).add(&abs(&example_dir("two-step")), None)
        })
    };
    let b = {
        let home = home.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            WorkbookRepo::new(home.clone()).add(&abs(&example_dir("gated-release")), None)
        })
    };
    barrier.wait();
    a.join().unwrap().unwrap();
    b.join().unwrap().unwrap();
    let rows = WorkbookRepo::new(home.clone()).list().unwrap();
    assert_eq!(rows.len(), 2, "两个 Workbook 都在：{rows:?}");
    assert_eq!(
        std::fs::read(
            home.workbook_dir("two-step", "1.0.0")
                .join_segment("workbook.toml")
                .as_path()
        )
        .unwrap(),
        std::fs::read(example_dir("two-step").join("workbook.toml")).unwrap()
    );
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
    let repo = WorkbookRepo::new(home.clone());
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
    WorkbookRepo::new(home.clone())
        .add(&abs(&example_dir("gated-release")), None)
        .unwrap();
    assert!(final_dir.as_path().exists(), "恢复发布了原件");
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
    let wb = WorkbookRepo::new(home.clone())
        .load("two-step", Some("1.0.0"))
        .unwrap();
    assert_eq!(wb.manifest.id().as_str(), "two-step");
}

// Task: C002-T26
#[test]
fn workbook_publication_recovery_checks_owner_and_manifest_identity() {
    for (mutation, final_only) in [
        ("owner_request_id", false),
        ("owner_op", false),
        ("manifest_id_pending", false),
        ("manifest_id_final", true),
        ("manifest_content_pending", false),
        ("manifest_permission_pending", false),
    ] {
        let (_dir, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        let request_id = format!("t26-wb-{mutation}");
        repo.add(&abs(&example_dir("two-step")), Some(request_id.clone()))
            .unwrap();
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let effects_raw: String = conn
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get(0),
            )
            .unwrap();
        let effects: serde_json::Value = serde_json::from_str(&effects_raw).unwrap();
        let pending_rel = effects[0]["pending"].as_str().unwrap();
        let internal_id = pending_rel.split('/').nth(1).unwrap();
        let payload = home
            .rel(pending_rel)
            .unwrap()
            .as_path()
            .to_path_buf()
            .into_std_path_buf();
        let final_dir = home.workbook_dir("two-step", "1.0.0");
        if !final_only {
            make_writable(final_dir.as_path().parent().unwrap().as_std_path());
            std::fs::rename(final_dir.as_path(), &payload).unwrap();
        }
        conn.execute(
            "UPDATE requests SET published = 0 WHERE request_id = ?1",
            [&request_id],
        )
        .unwrap();

        match mutation {
            "owner_request_id" | "owner_op" => {
                let owner_path = home
                    .pending_dir()
                    .join_segment(&format!("{internal_id}.owner"));
                let mut owner: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(owner_path.as_path()).unwrap()).unwrap();
                if mutation == "owner_request_id" {
                    owner["request_id"] = serde_json::json!("t26-different-request");
                } else {
                    owner["op"] = serde_json::json!("remove_workbook");
                }
                let mut bytes = serde_json::to_vec(&owner).unwrap();
                bytes.push(b'\n');
                std::fs::write(owner_path.as_path(), bytes).unwrap();
            }
            "manifest_id_pending" | "manifest_id_final" => {
                let root = if final_only {
                    std::path::PathBuf::from(final_dir.as_str())
                } else {
                    payload.clone()
                };
                let manifest = root.join("workbook.toml");
                make_writable(&manifest);
                let before = std::fs::read_to_string(&manifest).unwrap();
                assert!(before.contains("id = \"two-step\""));
                std::fs::write(
                    &manifest,
                    before.replacen("id = \"two-step\"", "id = \"other\"", 1),
                )
                .unwrap();
            }
            "manifest_content_pending" => {
                let instruction = payload.join("instructions/outline.md");
                make_writable(&instruction);
                let mut bytes = std::fs::read(&instruction).unwrap();
                let first = bytes.first_mut().unwrap();
                *first ^= 1;
                std::fs::write(instruction, bytes).unwrap();
            }
            "manifest_permission_pending" => {
                use std::os::unix::fs::PermissionsExt as _;
                let manifest = payload.join("workbook.toml");
                std::fs::set_permissions(&manifest, std::fs::Permissions::from_mode(0o0)).unwrap();
            }
            _ => unreachable!(),
        }

        let error = repo
            .add(
                &abs(&example_dir("gated-release")),
                Some("t26-wb-blocked-write".to_string()),
            )
            .unwrap_err();
        let Error::EffectPending {
            committed,
            pending_request_id,
            cause,
            ..
        } = error
        else {
            panic!("Workbook发布归属不符必须保留pending身份：{error:?}");
        };
        assert!(!committed);
        assert_eq!(pending_request_id.as_deref(), Some(request_id.as_str()));
        let expected_cause = if mutation == "manifest_permission_pending" {
            ErrorCode::Io
        } else {
            ErrorCode::StoreCorrupt
        };
        assert_eq!(cause, expected_cause);
        assert_eq!(payload.exists(), !final_only);
        assert_eq!(final_dir.as_path().exists(), final_only);
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM requests WHERE request_id = 't26-wb-blocked-write'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
    }
}

// Task: C002-T28
#[test]
fn committed_pending_workbook_is_readable_without_recovery_and_is_marked_pending() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t28-pending-wb".into()),
    )
    .unwrap();
    let final_dir = home.workbook_dir("two-step", "1.0.0");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-pending-wb'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let payload = home.rel(pending).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = 't28-pending-wb'",
        [],
    )
    .unwrap();
    drop(conn);
    make_writable(std::path::Path::new(final_dir.as_str()));
    std::fs::rename(final_dir.as_path(), payload.as_path()).unwrap();

    let rows = repo.list().unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].pending_publish);
    let loaded = repo.load("two-step", Some("1.0.0")).unwrap();
    assert!(loaded.pending_publish);
    assert_eq!(loaded.manifest.id().as_str(), "two-step");
    let verified = repo.verify(Some(("two-step", "1.0.0"))).unwrap();
    assert_eq!(verified[0].status, sheltie_runtime::VerifyStatus::Ok);
    assert!(verified[0].pending_publish);
    assert!(repo.cleanup_pending().unwrap().is_empty());
    assert!(payload.as_path().is_dir(), "published=0原件不能被清理");
    assert!(!final_dir.as_path().exists(), "只读操作不恢复发布目录");
    let started = sheltie_runtime::WorkService::new(home.clone())
        .start(start_args(), Some("t28-start-from-pending".into()))
        .unwrap();
    assert!(matches!(
        started.reply,
        sheltie_core::work::Reply::Started { .. }
    ));
    assert!(final_dir.as_path().is_dir(), "写锁内恢复后从final重新核验");
}

// Task: C002-T28
#[test]
fn cleanup_removes_a_valid_precommit_orphan_without_age_checks() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    let orphan_id = "0198f01a7f0070008000000000000002";
    let container = home.pending_dir().join_segment(orphan_id);
    std::fs::create_dir_all(container.as_path().join("payload")).unwrap();
    std::fs::write(container.as_path().join("payload/private"), b"uncommitted").unwrap();
    let owner = serde_json::json!({
        "format": "pending/v1",
        "internal_id": orphan_id,
        "request_id": "never-committed-request",
        "op": "add_workbook",
    });
    let sidecar = home
        .pending_dir()
        .join_segment(&format!("{orphan_id}.owner"));
    std::fs::write(
        sidecar.as_path(),
        format!("{}\n", serde_json::to_string(&owner).unwrap()),
    )
    .unwrap();
    let incomplete_id = "0198f01a7f0070008000000000000003";
    let incomplete_sidecar = home
        .pending_dir()
        .join_segment(&format!("{incomplete_id}.owner"));
    std::fs::write(incomplete_sidecar.as_path(), b"{\"format\":\n").unwrap();

    let warnings = repo.cleanup_pending().unwrap();
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains(incomplete_id))
    );
    assert!(!container.as_path().exists());
    assert!(!sidecar.as_path().exists());
    assert!(incomplete_sidecar.as_path().is_file());
}

// Task: C002-T28
#[test]
fn cleanup_preserves_nonempty_payload_after_request_completion() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t28-nonempty-completed".into()),
    )
    .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-nonempty-completed'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    drop(conn);
    let store_before = std::fs::read(home.store_path().as_path()).unwrap();
    let payload = home.rel(pending).unwrap();
    std::fs::create_dir_all(payload.as_path()).unwrap();
    let extra = payload.as_path().join("retained-private-bytes");
    std::fs::write(&extra, b"retain").unwrap();

    let warnings = repo.cleanup_pending().unwrap();
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("container仍有内容"))
    );
    assert_eq!(std::fs::read(extra).unwrap(), b"retain");
    assert!(payload.as_path().is_dir());
    assert_eq!(
        std::fs::read(home.store_path().as_path()).unwrap(),
        store_before
    );
    assert!(repo.load("two-step", Some("1.0.0")).is_ok());
}

// Task: C002-T28
#[test]
fn pending_workbook_with_missing_owner_does_not_fall_back_to_another_version() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("t28-v27-old".into()))
        .unwrap();
    let newer_source_root = tempfile::tempdir().unwrap();
    let newer_source = copy_example("two-step", newer_source_root.path());
    let manifest = std::fs::read_to_string(newer_source.join("workbook.toml")).unwrap();
    std::fs::write(
        newer_source.join("workbook.toml"),
        manifest.replace("version = \"1.0.0\"", "version = \"2.0.0\""),
    )
    .unwrap();
    repo.add(&abs(&newer_source), Some("t28-v27-new".into()))
        .unwrap();

    let old_final = home.workbook_dir("two-step", "1.0.0");
    let newer_final = home.workbook_dir("two-step", "2.0.0");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-v27-old'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let internal_id = pending.split('/').nth(1).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = 't28-v27-old'",
        [],
    )
    .unwrap();
    drop(conn);
    make_writable(std::path::Path::new(old_final.as_str()));
    std::fs::rename(old_final.as_path(), home.rel(pending).unwrap().as_path()).unwrap();
    std::fs::remove_file(
        home.pending_dir()
            .join_segment(&format!("{internal_id}.owner"))
            .as_path(),
    )
    .unwrap();

    let error = match repo.load("two-step", Some("1.0.0")) {
        Ok(_) => panic!("缺少owner的pending Workbook不能读取"),
        Err(error) => error,
    };
    assert_eq!(error.code(), ErrorCode::StoreCorrupt);
    assert!(newer_final.as_path().is_dir(), "另一版本原件保持不变");
    assert_eq!(
        repo.load("two-step", Some("2.0.0"))
            .unwrap()
            .manifest
            .version(),
        "2.0.0"
    );
}

// Task: C002-T28
#[test]
fn pending_cleanup_removes_only_completed_empty_metadata() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("t28-clean-wb".into()))
        .unwrap();
    let final_dir = home.workbook_dir("two-step", "1.0.0");
    let before = WorkbookRepo::digest_dir(&final_dir).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-clean-wb'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let internal_id = effects[0]["pending"]
        .as_str()
        .unwrap()
        .split('/')
        .nth(1)
        .unwrap();
    drop(conn);

    assert!(repo.cleanup_pending().unwrap().is_empty());
    assert!(final_dir.as_path().is_dir());
    assert_eq!(WorkbookRepo::digest_dir(&final_dir).unwrap(), before);
    assert!(
        !home
            .pending_dir()
            .join_segment(&format!("{internal_id}.owner"))
            .as_path()
            .exists()
    );
    assert_eq!(repo.load("two-step", Some("1.0.0")).unwrap().digest, before);
}

// Task: C002-T28
#[test]
fn malformed_request_effects_stop_cleanup_before_any_unreferenced_object_is_removed() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("t28-bad-index".into()))
        .unwrap();
    let orphan_id = "0198f01a-7f00-7000-8000-000000000001";
    let orphan = home.pending_dir().join_segment(orphan_id);
    std::fs::create_dir_all(orphan.as_path().join("payload")).unwrap();
    let owner = serde_json::json!({
        "format": "pending/v1",
        "internal_id": orphan_id,
        "request_id": "orphan-request",
        "op": "add_workbook",
    });
    std::fs::write(
        home.pending_dir()
            .join_segment(&format!("{orphan_id}.owner"))
            .as_path(),
        format!("{}\n", serde_json::to_string(&owner).unwrap()),
    )
    .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET effects_json = 'not-json' WHERE request_id = 't28-bad-index'",
        [],
    )
    .unwrap();
    drop(conn);

    assert_eq!(
        repo.cleanup_pending().unwrap_err().code(),
        ErrorCode::StoreCorrupt
    );
    assert!(orphan.as_path().is_dir(), "坏引用索引不能触发任何清理");
    assert!(
        home.pending_dir()
            .join_segment(&format!("{orphan_id}.owner"))
            .as_path()
            .is_file()
    );
}

// Task: C002-T28
#[test]
fn work_status_loads_committed_pending_frozen_graph_without_locking_or_recovery() {
    let (_dir, home, svc) = home_with_example("two-step");
    let response = svc
        .start(start_args(), Some("t28-pending-work".into()))
        .unwrap();
    let wid = work_id_of(&response);
    let final_dir = home.work_dir(&wid);
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-pending-work'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let payload = home.rel(pending).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = 't28-pending-work'",
        [],
    )
    .unwrap();
    drop(conn);
    make_writable(std::path::Path::new(final_dir.as_str()));
    std::fs::rename(final_dir.as_path(), payload.as_path()).unwrap();

    let (text, card, pending_publish) = svc.status_with_publication(&wid).unwrap();
    assert!(pending_publish);
    assert!(text.contains(wid.as_str()));
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Active);
    assert!(!final_dir.as_path().exists(), "只读status不能执行恢复");
    assert!(payload.as_path().join("workbook").is_dir());
}

// Task: C002-T28
#[test]
fn cleanup_index_rejects_a_decodable_effect_path_that_points_into_an_orphan() {
    let (_dir, home, svc) = home_with_example("two-step");
    let started = svc
        .start(start_args(), Some("t28-index-work".into()))
        .unwrap();
    let work = work_id_of(&started);
    svc.begin(
        &work,
        &sheltie_core::ids::NodeId::new("outline").unwrap(),
        Some("t28-index-begin".into()),
    )
    .unwrap();
    let orphan_id = "0198f01a7f0070008000000000000010";
    let container = home.pending_dir().join_segment(orphan_id);
    let payload = container.as_path().join("payload");
    std::fs::create_dir_all(&payload).unwrap();
    std::fs::write(payload.join("private"), b"must-retain").unwrap();
    let owner = serde_json::json!({
        "format": "pending/v1",
        "internal_id": orphan_id,
        "request_id": "t28-unreferenced-owner",
        "op": "add_workbook",
    });
    std::fs::write(
        home.pending_dir()
            .join_segment(&format!("{orphan_id}.owner"))
            .as_path(),
        format!("{}\n", serde_json::to_string(&owner).unwrap()),
    )
    .unwrap();

    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let mut effects: serde_json::Value = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-index-begin'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|raw| serde_json::from_str(&raw).unwrap())
        .unwrap();
    let write = effects
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|effect| effect["kind"] == "write_file")
        .unwrap();
    write["path"] = serde_json::json!(format!("pending/{orphan_id}/payload/private"));
    connection
        .execute(
            "UPDATE requests SET effects_json = ?1 WHERE request_id = 't28-index-begin'",
            [serde_json::to_string(&effects).unwrap()],
        )
        .unwrap();
    drop(connection);

    assert_eq!(
        repo(&home).cleanup_pending().unwrap_err().code(),
        ErrorCode::StoreCorrupt
    );
    assert_eq!(
        std::fs::read(payload.join("private")).unwrap(),
        b"must-retain"
    );
    assert!(container.as_path().is_dir());
}

// Task: C002-T28
#[cfg(feature = "failpoint")]
#[test]
fn real_work_status_retries_after_start_publish_renames_post_location() {
    use std::time::{Duration, Instant};

    let (_dir, home, svc) = home_with_example("two-step");
    let started = svc
        .start(start_args(), Some("t28-read-race".into()))
        .unwrap();
    let work = work_id_of(&started);
    let final_dir = home.work_dir(&work);
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-read-race'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    connection
        .execute(
            "UPDATE requests SET published = 0 WHERE request_id = 't28-read-race'",
            [],
        )
        .unwrap();
    drop(connection);

    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "pending_read_after_locate",
        &format!("works/{work}"),
        rendezvous.path(),
    )
    .unwrap();
    let reader_service = svc.clone();
    let reader_work = work.clone();
    let reader = std::thread::spawn(move || reader_service.status_with_publication(&reader_work));
    let deadline = Instant::now() + Duration::from_secs(10);
    while !rendezvous.path().join("reached").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !rendezvous.path().join("reached").exists() {
        let _ = std::fs::write(rendezvous.path().join("release"), b"release");
        let _ = reader.join();
        panic!("Work只读装入没有到达定位后的同步点");
    }
    std::fs::rename(final_dir.as_path(), home.rel(pending).unwrap().as_path()).unwrap();
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
    std::fs::write(rendezvous.path().join("release"), b"release").unwrap();

    let (_, card, pending_publish) = reader.join().unwrap().unwrap();
    assert!(pending_publish);
    assert_eq!(card.work_id, work);
    assert!(!final_dir.as_path().exists());
    assert!(home.rel(pending).unwrap().as_path().is_dir());
}

// Task: C002-T28
#[cfg(feature = "failpoint")]
#[test]
fn completed_empty_container_that_changes_before_unlink_is_preserved() {
    use std::time::{Duration, Instant};

    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t28-clean-race".into()),
    )
    .unwrap();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-clean-race'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let internal_id = effects[0]["pending"]
        .as_str()
        .unwrap()
        .split('/')
        .nth(1)
        .unwrap();
    drop(connection);
    let container = home.pending_dir().join_segment(internal_id);
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "pending_cleanup_after_empty_check",
        "t28-clean-race",
        rendezvous.path(),
    )
    .unwrap();
    let cleanup_repo = repo.clone();
    let cleanup = std::thread::spawn(move || cleanup_repo.cleanup_pending());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !rendezvous.path().join("reached").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !rendezvous.path().join("reached").exists() {
        let _ = std::fs::write(rendezvous.path().join("release"), b"release");
        let _ = cleanup.join();
        panic!("cleanup没有到达同一ManagedTree空容器检查后的同步点");
    }
    let sentinel = container.as_path().join("late-content");
    std::fs::write(&sentinel, b"retain").unwrap();
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
    std::fs::write(rendezvous.path().join("release"), b"release").unwrap();

    let warnings = cleanup.join().unwrap().unwrap();
    assert!(warnings.iter().any(|warning| {
        warning.contains("request_id=t28-clean-race")
            && warning.contains("object=pending/")
            && warning.contains("reason=")
    }));
    assert_eq!(std::fs::read(sentinel).unwrap(), b"retain");
    assert!(container.as_path().is_dir());
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
}

/// 旧 remove 的重放不得删除同版本的新对象；新生命周期不被旧请求覆盖（§5.2 第 4 条）。
// Task: C002-T08
#[test]
fn old_remove_replay_does_not_delete_readded_workbook() {
    let (_d, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("r-a".into()))
        .unwrap();
    repo.remove("two-step", "1.0.0", Some("r-r".into()))
        .unwrap();
    // 同版本重新装（新对象、新请求）。
    let repo2 = WorkbookRepo::new(home.clone());
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
    let rows = WorkbookRepo::new(home.clone()).list().unwrap();
    assert_eq!(rows.len(), 1);
}

// Task: C002-T27
#[test]
fn remove_marker_cannot_be_borrowed_from_another_internal_id() {
    for mutation in ["cross_id", "bad_format"] {
        let (_dir, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        repo.add(
            &abs(&example_dir("two-step")),
            Some("t27-marker-add-a".into()),
        )
        .unwrap();
        repo.add(
            &abs(&example_dir("gated-release")),
            Some("t27-marker-add-b".into()),
        )
        .unwrap();
        let request_a = "t27-marker-remove-a";
        let request_b = "t27-marker-remove-b";
        repo.remove("two-step", "1.0.0", Some(request_a.into()))
            .unwrap();
        repo.remove("gated-release", "1.0.0", Some(request_b.into()))
            .unwrap();
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let effects_a: String = conn
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id = ?1",
                [request_a],
                |row| row.get(0),
            )
            .unwrap();
        let effects_b: String = conn
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id = ?1",
                [request_b],
                |row| row.get(0),
            )
            .unwrap();
        let internal_a =
            serde_json::from_str::<serde_json::Value>(&effects_a).unwrap()[0]["pending"]
                .as_str()
                .unwrap()
                .split('/')
                .nth(1)
                .unwrap()
                .to_string();
        let internal_b =
            serde_json::from_str::<serde_json::Value>(&effects_b).unwrap()[0]["pending"]
                .as_str()
                .unwrap()
                .split('/')
                .nth(1)
                .unwrap()
                .to_string();
        assert_ne!(internal_a, internal_b);
        let marker_a = home
            .pending_dir()
            .join_segment(&format!("{internal_a}.deleted"));
        let marker_b = home
            .pending_dir()
            .join_segment(&format!("{internal_b}.deleted"));
        let bytes_a = std::fs::read(marker_a.as_path()).unwrap();
        let marker_content = if mutation == "cross_id" {
            bytes_a
        } else {
            format!("{{\"format\":\"unknown/v9\",\"internal_id\":\"{internal_b}\"}}\n").into_bytes()
        };
        std::fs::write(marker_b.as_path(), &marker_content).unwrap();
        conn.execute(
            "UPDATE requests SET published = 0 WHERE request_id = ?1",
            [request_b],
        )
        .unwrap();

        let error = repo
            .remove("gated-release", "1.0.0", Some(request_b.to_string()))
            .unwrap_err();
        let Error::EffectPending {
            committed,
            request_id,
            pending_request_id,
            cause,
            ..
        } = error
        else {
            panic!("marker必须绑定自己的internal_id：{error:?}");
        };
        assert!(committed);
        assert_eq!(request_id, request_b);
        assert!(pending_request_id.is_none());
        assert_eq!(cause, ErrorCode::StoreCorrupt);
        assert_eq!(std::fs::read(marker_b.as_path()).unwrap(), marker_content);
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [request_b],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        assert!(
            !home
                .workbook_dir("gated-release", "1.0.0")
                .as_path()
                .exists()
        );
        assert!(
            !home
                .rel(&format!("pending/{internal_b}/payload"))
                .unwrap()
                .as_path()
                .exists()
        );
    }
}

// Task: C002-T27
#[test]
fn remove_refuses_missing_directory_or_digest_before_commit() {
    for corruption in ["missing_directory", "empty_digest"] {
        let (_dir, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        repo.add(
            &abs(&example_dir("two-step")),
            Some("t27-preflight-add".into()),
        )
        .unwrap();
        let final_dir = home.workbook_dir("two-step", "1.0.0");
        if corruption == "missing_directory" {
            make_writable(final_dir.as_path().as_std_path());
            std::fs::remove_dir_all(final_dir.as_path()).unwrap();
        } else {
            let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
            conn.execute(
                "UPDATE workbooks SET digest = '' WHERE id = 'two-step' AND version = '1.0.0'",
                [],
            )
            .unwrap();
        }

        let request_id = format!("t27-remove-preflight-{corruption}");
        let error = repo
            .remove("two-step", "1.0.0", Some(request_id.clone()))
            .unwrap_err();
        assert_eq!(
            error.code(),
            ErrorCode::StoreCorrupt,
            "{corruption}: {error:?}"
        );
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM audit WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM workbooks", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(final_dir.as_path().exists(), corruption == "empty_digest");
    }
}

// Task: C002-T27
#[test]
fn completed_old_remove_and_add_replays_preserve_new_lifecycle_bytes_and_row() {
    use std::os::unix::fs::MetadataExt as _;

    let (dir, home) = temp_home();
    let source = copy_example("two-step", dir.path());
    let repo = WorkbookRepo::new(home.clone());
    let request_add_old = "t27-lifecycle-add-old";
    let request_remove_old = "t27-lifecycle-remove-old";
    let old_add = repo
        .add(&abs(&source), Some(request_add_old.to_string()))
        .unwrap();
    repo.remove("two-step", "1.0.0", Some(request_remove_old.into()))
        .unwrap();

    std::fs::write(
        source.join("instructions/outline.md"),
        "同身份的新生命周期必须由新请求管理。\n",
    )
    .unwrap();
    let new_add = repo
        .add(&abs(&source), Some("t27-lifecycle-add-new".into()))
        .unwrap();
    assert_ne!(old_add.data["digest"], new_add.data["digest"]);
    let final_dir = home.workbook_dir("two-step", "1.0.0");
    let new_inode = std::fs::metadata(final_dir.as_path()).unwrap().ino();
    let new_bytes =
        std::fs::read(final_dir.join_segment("instructions/outline.md").as_path()).unwrap();
    let row_before: String = rusqlite::Connection::open(home.store_path().as_str())
        .unwrap()
        .query_row(
            "SELECT digest FROM workbooks WHERE id = 'two-step' AND version = '1.0.0'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let old_add_reply: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [request_add_old],
            |row| row.get(0),
        )
        .unwrap();
    let old_remove_reply: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [request_remove_old],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        repo.remove("two-step", "1.0.0", Some(request_remove_old.into()))
            .unwrap()
            .replayed
    );
    assert!(
        repo.add(&abs(&source), Some(request_add_old.to_string()))
            .unwrap()
            .replayed
    );
    assert_eq!(
        std::fs::metadata(final_dir.as_path()).unwrap().ino(),
        new_inode
    );
    assert_eq!(
        std::fs::read(final_dir.join_segment("instructions/outline.md").as_path()).unwrap(),
        new_bytes
    );
    assert_eq!(
        conn.query_row(
            "SELECT digest FROM workbooks WHERE id = 'two-step' AND version = '1.0.0'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        row_before
    );
    assert_eq!(
        conn.query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [request_add_old],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        old_add_reply
    );
    assert_eq!(
        conn.query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [request_remove_old],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        old_remove_reply
    );
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
    let repo = WorkbookRepo::new(home.clone());
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
    WorkbookRepo::new(home.clone())
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
    let repo = WorkbookRepo::new(home.clone());
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

// Task: C002-T25
#[test]
fn workbook_replay_finishes_unpublished_effects() {
    let (source_root, home) = temp_home();
    let source = copy_example("two-step", source_root.path());
    let repo = WorkbookRepo::new(home.clone());
    let add_request = "t25-add-replay";
    let first = repo
        .add(&abs(&source), Some(add_request.to_string()))
        .unwrap();
    let final_dir = home.workbook_dir("two-step", "1.0.0");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects_json: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [add_request],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [add_request],
    )
    .unwrap();
    drop(conn);
    let pending_rel =
        serde_json::from_str::<serde_json::Value>(&effects_json).unwrap()[0]["pending"]
            .as_str()
            .unwrap()
            .to_string();
    make_writable(std::path::Path::new(final_dir.as_str()));
    std::fs::rename(
        final_dir.as_path(),
        home.rel(&pending_rel).unwrap().as_path(),
    )
    .unwrap();
    make_writable(&source);
    std::fs::remove_dir_all(&source).unwrap();

    let replayed_add = repo
        .add(&abs(&source), Some(add_request.to_string()))
        .unwrap();
    assert!(replayed_add.replayed);
    assert_eq!(first.data, replayed_add.data);
    assert!(final_dir.as_path().is_dir());

    let remove_request = "t25-remove-replay";
    let first_remove = repo
        .remove("two-step", "1.0.0", Some(remove_request.to_string()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [remove_request],
    )
    .unwrap();
    drop(conn);
    let replayed_remove = repo
        .remove("two-step", "1.0.0", Some(remove_request.to_string()))
        .unwrap();
    assert!(replayed_remove.replayed);
    assert_eq!(first_remove.data, replayed_remove.data);
    assert!(!final_dir.as_path().exists());

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    for request_id in [add_request, remove_request] {
        let published: i64 = conn
            .query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [request_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(published, 1, "重放必须先完成原效果：{request_id}");
    }
}

// Task: C002-T25
#[test]
fn workbook_write_recovers_work_status_card() {
    let (_dir, home, svc) = home_with_example("two-step");
    let request_id = "t25-start-pending";
    let started = svc
        .start(start_args(), Some(request_id.to_string()))
        .unwrap();
    let wid = work_id_of(&started);
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    let expected_card = std::fs::read(&card).unwrap();
    std::fs::remove_file(&card).unwrap();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [request_id],
    )
    .unwrap();
    drop(conn);

    let added = WorkbookRepo::new(home.clone())
        .add(
            &abs(&example_dir("gated-release")),
            Some("t25-add-after-start".into()),
        )
        .unwrap();
    assert_eq!(added.data["id"], "gated-release");
    assert_eq!(std::fs::read(&card).unwrap(), expected_card);
    assert!(
        home.workbook_dir("gated-release", "1.0.0")
            .as_path()
            .is_dir()
    );

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let published: i64 = conn
        .query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(published, 1, "Workbook写操作结束前必须完成Work卡效果");
}

// Task: C002-T25
#[test]
fn workbook_remove_recovers_latest_work_status_card() {
    let (_dir, home, svc) = home_with_example("two-step");
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("gated-release")), None).unwrap();
    let old_request = "t25-start-before-begin";
    let started = svc
        .start(start_args(), Some(old_request.to_string()))
        .unwrap();
    let wid = work_id_of(&started);
    let begun = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            Some("t25-latest-begin".into()),
        )
        .unwrap();
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    let latest_card = std::fs::read(&card).unwrap();
    assert_eq!(begun.revision, 2);

    std::fs::remove_file(&card).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [old_request],
    )
    .unwrap();
    let begin_reply_before: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = 't25-latest-begin'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    drop(conn);

    let removed = repo
        .remove(
            "gated-release",
            "1.0.0",
            Some("t25-remove-after-work".into()),
        )
        .unwrap();
    assert_eq!(removed.data["id"], "gated-release");
    assert_eq!(std::fs::read(&card).unwrap(), latest_card);
    assert!(
        !home
            .workbook_dir("gated-release", "1.0.0")
            .as_path()
            .exists()
    );

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let published: i64 = conn
        .query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [old_request],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(published, 1);
    let (latest_published, begin_reply_after): (i64, String) = conn
        .query_row(
            "SELECT published, reply_json FROM requests WHERE request_id = 't25-latest-begin'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(latest_published, 1);
    assert_eq!(
        begin_reply_after, begin_reply_before,
        "旧start恢复不能改历史begin响应"
    );
}

// Task: C002-T25
#[test]
fn old_add_replay_does_not_bind_to_a_readded_workbook_lifecycle() {
    let (root, home) = temp_home();
    let source = copy_example("two-step", root.path());
    let repo = WorkbookRepo::new(home.clone());
    let old_request = "t25-old-add-lifecycle";
    let old = repo
        .add(&abs(&source), Some(old_request.to_string()))
        .unwrap();
    repo.remove("two-step", "1.0.0", Some("t25-remove-old-lifecycle".into()))
        .unwrap();
    make_writable(&source);
    std::fs::write(
        source.join("instructions/outline.md"),
        "同一身份的新生命周期使用不同内容。\n",
    )
    .unwrap();
    let new = repo
        .add(&abs(&source), Some("t25-readd-lifecycle".into()))
        .unwrap();
    assert_ne!(old.data["digest"], new.data["digest"]);
    let final_dir = home.workbook_dir("two-step", "1.0.0");
    let new_bytes =
        std::fs::read(std::path::Path::new(final_dir.as_str()).join("instructions/outline.md"))
            .unwrap();

    let replayed_old = repo
        .add(&abs(&source), Some(old_request.to_string()))
        .unwrap();
    assert!(replayed_old.replayed);
    assert_eq!(replayed_old.data, old.data);
    assert_eq!(
        std::fs::read(std::path::Path::new(final_dir.as_str()).join("instructions/outline.md"))
            .unwrap(),
        new_bytes,
        "旧add重放只能回原快照，不能触碰新生命周期对象"
    );
    let row: String = rusqlite::Connection::open(home.store_path().as_str())
        .unwrap()
        .query_row(
            "SELECT digest FROM workbooks WHERE id = 'two-step' AND version = '1.0.0'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(row, new.data["digest"]);
}

// Task: C002-T28
#[test]
fn cleanup_removes_a_valid_orphan_with_a_frozen_readonly_workbook() {
    use std::os::unix::fs::PermissionsExt;
    fn freeze(path: &Path) {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                freeze(&entry.unwrap().path());
            }
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o555)).unwrap();
        } else {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o444)).unwrap();
        }
    }
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    let id = "0198f01a7f0070008000000000000011";
    let container = home.pending_dir().join_segment(id);
    let frozen = container.as_path().join("payload/workbook");
    copy_dir(&example_dir("two-step"), frozen.as_std_path());
    freeze(frozen.as_std_path());
    std::fs::write(
        home.pending_dir()
            .join_segment(&format!("{id}.owner"))
            .as_path(),
        format!(
            "{}\n",
            serde_json::json!({
                "format": "pending/v1", "internal_id": id,
                "request_id": "t28-never-committed-start", "op": "start_work"
            })
        ),
    )
    .unwrap();
    assert!(repo.cleanup_pending().unwrap().is_empty());
    assert!(!container.as_path().exists());
    assert!(
        !home
            .pending_dir()
            .join_segment(&format!("{id}.owner"))
            .as_path()
            .exists()
    );
}

// Task: C002-T28
#[test]
fn completed_workbook_final_requires_its_current_successful_publisher() {
    for mutation in ["snapshot_request_id", "missing_audit"] {
        let (_dir, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        repo.add(&abs(&example_dir("two-step")), Some("t28-final-wb".into()))
            .unwrap();
        repo.cleanup_pending().unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        if mutation == "missing_audit" {
            connection
                .execute("DELETE FROM audit WHERE request_id = 't28-final-wb'", [])
                .unwrap();
        } else {
            let raw: String = connection
                .query_row(
                    "SELECT reply_json FROM requests WHERE request_id = 't28-final-wb'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            let mut snapshot: serde_json::Value = serde_json::from_str(&raw).unwrap();
            snapshot["request_id"] = serde_json::json!("other-request");
            connection
                .execute(
                    "UPDATE requests SET reply_json = ?1 WHERE request_id = 't28-final-wb'",
                    [serde_json::to_string(&snapshot).unwrap()],
                )
                .unwrap();
        }
        drop(connection);
        let error = repo.load("two-step", Some("1.0.0")).err().unwrap();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{mutation}: {error}");
        assert_eq!(repo.list().unwrap_err().code(), ErrorCode::StoreCorrupt);
        assert_eq!(
            repo.verify(None).unwrap_err().code(),
            ErrorCode::StoreCorrupt
        );
        assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
    }
}

// Task: C002-T28
#[test]
fn completed_work_final_requires_its_successful_start_request() {
    for mutation in ["snapshot_request_id", "missing_audit", "unknown_data"] {
        let (_dir, home, svc) = home_with_example("two-step");
        let started = svc
            .start(start_args(), Some("t28-final-work".into()))
            .unwrap();
        let work = work_id_of(&started);
        WorkbookRepo::new(home.clone()).cleanup_pending().unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        if mutation == "missing_audit" {
            connection
                .execute("DELETE FROM audit WHERE request_id = 't28-final-work'", [])
                .unwrap();
        } else {
            let raw: String = connection
                .query_row(
                    "SELECT reply_json FROM requests WHERE request_id = 't28-final-work'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            let mut snapshot: serde_json::Value = serde_json::from_str(&raw).unwrap();
            if mutation == "unknown_data" {
                snapshot["data"]["unknown"] = serde_json::json!(true);
            } else {
                snapshot["request_id"] = serde_json::json!("other-request");
            }
            connection
                .execute(
                    "UPDATE requests SET reply_json = ?1 WHERE request_id = 't28-final-work'",
                    [serde_json::to_string(&snapshot).unwrap()],
                )
                .unwrap();
        }
        drop(connection);
        assert_eq!(
            svc.status_with_publication(&work).unwrap_err().code(),
            ErrorCode::StoreCorrupt
        );
        assert!(home.work_dir(&work).as_path().is_dir());
    }
}

// Task: C002-T28
#[cfg(feature = "failpoint")]
#[test]
fn workbook_reader_accepts_publication_and_cleanup_after_its_reference_index() {
    use std::time::{Duration, Instant};
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    let source = abs(&example_dir("two-step"));
    repo.add(&source, Some("t28-mark-race".into())).unwrap();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    connection
        .execute(
            "UPDATE requests SET published = 0 WHERE request_id = 't28-mark-race'",
            [],
        )
        .unwrap();
    drop(connection);
    let sync = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "pending_after_reference_index",
        "workbooks/two-step/1.0.0",
        sync.path(),
    )
    .unwrap();
    let reader_repo = repo.clone();
    let reader = std::thread::spawn(move || reader_repo.load("two-step", Some("1.0.0")));
    let deadline = Instant::now() + Duration::from_secs(10);
    while !sync.path().join("reached").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !sync.path().join("reached").exists() {
        let _ = std::fs::write(sync.path().join("release"), b"release");
        let _ = reader.join();
        panic!("reader没有到达引用索引后的同步点");
    }
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
    let replay = repo.add(&source, Some("t28-mark-race".into())).unwrap();
    assert!(replay.replayed);
    assert!(repo.cleanup_pending().unwrap().is_empty());
    std::fs::write(sync.path().join("release"), b"release").unwrap();
    let loaded = reader.join().unwrap().unwrap();
    assert!(!loaded.pending_publish);
    assert_eq!(loaded.manifest.id().as_str(), "two-step");
}

// Task: C002-T28
#[test]
fn cleanup_preserves_owner_for_a_valid_id_with_a_symlink_container() {
    use std::os::unix::fs::symlink;
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    let id = "0198f01a7f0070008000000000000012";
    let external = tempfile::tempdir().unwrap();
    let sentinel = external.path().join("sentinel");
    std::fs::write(&sentinel, b"external-bytes").unwrap();
    let container = home.pending_dir().join_segment(id);
    symlink(external.path(), container.as_path()).unwrap();
    let ownerless = home
        .pending_dir()
        .join_segment("0198f01a7f0070008000000000000013");
    std::fs::create_dir(ownerless.as_path()).unwrap();
    std::fs::write(
        ownerless.join_segment("sentinel").as_path(),
        b"unowned-bytes",
    )
    .unwrap();
    let owner = home.pending_dir().join_segment(&format!("{id}.owner"));
    let owner_bytes = format!(
        "{}\n",
        serde_json::json!({
            "format": "pending/v1", "internal_id": id,
            "request_id": "t28-orphan-symlink", "op": "start_work"
        })
    );
    std::fs::write(owner.as_path(), owner_bytes.as_bytes()).unwrap();
    let warnings = repo.cleanup_pending().unwrap();
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("request_id=t28-orphan-symlink")
                && warning.contains("container类型异常"))
    );
    assert!(
        std::fs::symlink_metadata(container.as_path())
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read(owner.as_path()).unwrap(),
        owner_bytes.as_bytes()
    );
    assert_eq!(std::fs::read(sentinel).unwrap(), b"external-bytes");
    assert_eq!(
        std::fs::read(ownerless.join_segment("sentinel").as_path()).unwrap(),
        b"unowned-bytes"
    );
}

// Task: C002-T28
#[test]
fn current_workbook_never_uses_an_old_same_second_publisher_when_latest_audit_is_corrupt() {
    for mutation in ["revision", "work_id", "command_other_remove"] {
        let (_dir, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        let source = abs(&example_dir("two-step"));
        repo.add(&source, Some("t28-old-publisher".into())).unwrap();
        repo.remove("two-step", "1.0.0", Some("t28-between-remove".into()))
            .unwrap();
        repo.add(&source, Some("t28-new-publisher".into())).unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let at = "2026-09-29T00:00:00Z";
        connection
            .execute("UPDATE requests SET at = ?1", [at])
            .unwrap();
        connection
            .execute("UPDATE audit SET at = ?1", [at])
            .unwrap();
        connection
            .execute("UPDATE workbooks SET added_at = ?1", [at])
            .unwrap();
        if mutation == "revision" {
            connection
                .execute(
                    "UPDATE audit SET revision = 1 WHERE request_id = 't28-new-publisher'",
                    [],
                )
                .unwrap();
        } else if mutation == "work_id" {
            connection
                .execute(
                    "UPDATE audit SET work_id = 'wrong' WHERE request_id = 't28-new-publisher'",
                    [],
                )
                .unwrap();
        } else {
            connection
                .execute(
                    "UPDATE audit SET command_json = ?1 WHERE request_id = 't28-new-publisher'",
                    [
                        serde_json::json!({"intent":"remove_workbook", "target":"other@1.0.0"})
                            .to_string(),
                    ],
                )
                .unwrap();
        }
        drop(connection);
        let error = repo.load("two-step", Some("1.0.0")).err().unwrap();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{mutation}: {error}");
        assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
    }
}

// Task: C002-T28
#[test]
fn completed_final_views_ignore_preserved_abnormal_pending_metadata() {
    use std::os::unix::fs::symlink;
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t28-complete-wb".into()),
    )
    .unwrap();
    let svc = sheltie_runtime::WorkService::new(home.clone());
    let started = svc
        .start(start_args(), Some("t28-complete-work".into()))
        .unwrap();
    let work = work_id_of(&started);
    repo.cleanup_pending().unwrap();
    let external = tempfile::tempdir().unwrap();
    std::fs::write(external.path().join("sentinel"), b"retain").unwrap();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let mut links = Vec::new();
    for request in ["t28-complete-wb", "t28-complete-work"] {
        let raw: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id = ?1",
                [request],
                |row| row.get(0),
            )
            .unwrap();
        let effects: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let id = effects[0]["pending"]
            .as_str()
            .unwrap()
            .split('/')
            .nth(1)
            .unwrap();
        let link = home.pending_dir().join_segment(id);
        symlink(external.path(), link.as_path()).unwrap();
        links.push(link);
    }
    drop(connection);
    std::fs::remove_file(home.lock_path().as_path()).unwrap();
    assert!(
        !repo
            .load("two-step", Some("1.0.0"))
            .unwrap()
            .pending_publish
    );
    assert!(!repo.list().unwrap()[0].pending_publish);
    assert_eq!(
        repo.verify(None).unwrap()[0].status,
        sheltie_runtime::VerifyStatus::Ok
    );
    assert!(!svc.status_with_publication(&work).unwrap().2);
    assert!(!home.lock_path().as_path().exists());
    for link in links {
        assert!(
            std::fs::symlink_metadata(link.as_path())
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
    assert_eq!(
        std::fs::read(external.path().join("sentinel")).unwrap(),
        b"retain"
    );
}
