//! C002-T08：Workbook 事务、幂等与发布生命周期（N02/O08/§5.2）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_runtime::{Error, WorkbookRepo};

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

/// remove 的引用检查在同一个事务里：有非终态 Work 时删行回滚（§5.2）。
// Task: C002-T08
#[test]
fn remove_rejects_active_reference_and_rolls_back_row() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());

    let registered = home.workbook_dir("two-step", "1.0.0");
    let original =
        std::fs::read(registered.join_segment("instructions/outline.md").as_path()).unwrap();
    let err = WorkbookRepo::new(home.clone())
        .remove("two-step", "1.0.0", None)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookInUse, "{err:?}");
    let Error::WorkbookInUse { works, .. } = err else {
        panic!("{err:?}");
    };
    assert_eq!(works, vec![wid.clone()]);
    assert_eq!(
        std::fs::read(registered.join_segment("instructions/outline.md").as_path()).unwrap(),
        original
    );

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
        .start(
            two_step_args(&[("topic", "t")]),
            Some("t28-start-from-pending".into()),
        )
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
        .start(
            two_step_args(&[("topic", "t")]),
            Some("t28-pending-work".into()),
        )
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
        .start(
            two_step_args(&[("topic", "t")]),
            Some("t28-index-work".into()),
        )
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
    let (_dir, home, svc) = home_with_example("two-step");
    let started = svc
        .start(
            two_step_args(&[("topic", "t")]),
            Some("t28-read-race".into()),
        )
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
    let mut worker = RendezvousWorker::single(reader, rendezvous.path());
    worker.wait("Work只读装入没有到达定位后的同步点");
    std::fs::rename(final_dir.as_path(), home.rel(pending).unwrap().as_path()).unwrap();
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
    std::fs::write(rendezvous.path().join("release"), b"release").unwrap();

    let (_, card, pending_publish) = worker.finish().unwrap().unwrap();
    assert!(pending_publish);
    assert_eq!(card.work_id, work);
    assert!(!final_dir.as_path().exists());
    assert!(home.rel(pending).unwrap().as_path().is_dir());
}

// Task: C002-T28
#[cfg(feature = "failpoint")]
#[test]
fn completed_empty_container_that_changes_before_unlink_is_preserved() {
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
    let mut worker = RendezvousWorker::single(cleanup, rendezvous.path());
    worker.wait("cleanup没有到达同一ManagedTree空容器检查后的同步点");
    let sentinel = container.as_path().join("late-content");
    std::fs::write(&sentinel, b"retain").unwrap();
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
    std::fs::write(rendezvous.path().join("release"), b"release").unwrap();

    let warnings = worker.finish().unwrap().unwrap();
    assert!(warnings.iter().any(|warning| {
        warning.contains("request_id=t28-clean-race")
            && warning.contains("object=pending/")
            && warning.contains("reason=")
    }));
    assert_eq!(std::fs::read(sentinel).unwrap(), b"retain");
    assert!(container.as_path().is_dir());
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().is_dir());
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

    for replacement in [None, Some("同身份的新生命周期必须由新请求管理。\n")] {
        let (dir, home) = temp_home();
        let source = copy_example("two-step", dir.path());
        let repo = WorkbookRepo::new(home.clone());
        let request_add_old = "t27-lifecycle-add-old";
        let request_remove_old = "t27-lifecycle-remove-old";
        let old_add = repo
            .add(&abs(&source), Some(request_add_old.to_string()))
            .unwrap();
        let old_remove = repo
            .remove("two-step", "1.0.0", Some(request_remove_old.into()))
            .unwrap();

        if let Some(bytes) = replacement {
            std::fs::write(source.join("instructions/outline.md"), bytes).unwrap();
        }
        let new_add = repo
            .add(&abs(&source), Some("t27-lifecycle-add-new".into()))
            .unwrap();
        if replacement.is_some() {
            assert_ne!(old_add.data["digest"], new_add.data["digest"]);
        } else {
            assert_eq!(old_add.data["digest"], new_add.data["digest"]);
        }
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

        let replay_remove = repo
            .remove("two-step", "1.0.0", Some(request_remove_old.into()))
            .unwrap();
        assert!(replay_remove.replayed);
        assert_eq!(replay_remove.data, old_remove.data);
        let replay_add = repo
            .add(&abs(&source), Some(request_add_old.to_string()))
            .unwrap();
        assert!(replay_add.replayed);
        assert_eq!(replay_add.data, old_add.data);
        assert_eq!(repo.list().unwrap().len(), 1);
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
        assert_eq!(row_before, new_add.data["digest"]);
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
    let err = svc
        .start(two_step_args(&[("topic", "t")]), Some("r-x".into()))
        .unwrap_err();
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
        .start(
            two_step_args(&[("topic", "t")]),
            Some(request_id.to_string()),
        )
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
        .start(
            two_step_args(&[("topic", "t")]),
            Some(old_request.to_string()),
        )
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
            .start(
                two_step_args(&[("topic", "t")]),
                Some("t28-final-work".into()),
            )
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
    let mut worker = RendezvousWorker::single(reader, sync.path());
    worker.wait("reader没有到达引用索引后的同步点");
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
    let replay = repo.add(&source, Some("t28-mark-race".into())).unwrap();
    assert!(replay.replayed);
    assert!(repo.cleanup_pending().unwrap().is_empty());
    std::fs::write(sync.path().join("release"), b"release").unwrap();
    let loaded = worker.finish().unwrap().unwrap();
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
        .start(
            two_step_args(&[("topic", "t")]),
            Some("t28-complete-work".into()),
        )
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

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn readers_accept_real_publication_after_their_old_owner_observation() {
    for (reader_kind, change) in ["workbook", "stats", "list"]
        .into_iter()
        .flat_map(|reader| {
            [
                "none",
                "unpublished",
                "intent_hash",
                "reply_json",
                "effects_json",
                "work_id",
                "at",
            ]
            .into_iter()
            .map(move |change| (reader, change))
        })
    {
        let (_directory, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        let source = abs(&example_dir("two-step"));
        let request = if reader_kind == "workbook" {
            "owner-wb"
        } else {
            "owner-work"
        };
        repo.add(&source, Some("owner-method".into())).unwrap();
        let service = sheltie_runtime::WorkService::new(home.clone());
        let work = if reader_kind == "workbook" {
            None
        } else {
            Some(work_id_of(
                &service
                    .start(
                        two_step_args(&[("topic", "owner observation")]),
                        Some(request.into()),
                    )
                    .unwrap(),
            ))
        };
        if reader_kind == "workbook" {
            repo.remove("two-step", "1.0.0", Some("owner-remove".into()))
                .unwrap();
            repo.add(&source, Some(request.into())).unwrap();
        }
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let raw: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id=?1",
                [request],
                |row| row.get(0),
            )
            .unwrap();
        let effects: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let pending = effects[0]["pending"].as_str().unwrap();
        let id = pending.split('/').nth(1).unwrap();
        let owner = home.rel(&format!("pending/{id}.owner")).unwrap();
        assert!(owner.as_path().is_file());
        let expected_stats = work
            .as_ref()
            .map(|work| serde_json::to_value(service.stats(work).unwrap()).unwrap());
        let expected_list = work
            .as_ref()
            .map(|_| serde_json::to_value(service.list().unwrap()).unwrap());
        connection
            .execute(
                "UPDATE requests SET published=0 WHERE request_id=?1",
                [request],
            )
            .unwrap();
        let expected_book = repo.load("two-step", Some("1.0.0")).unwrap();
        let expected_book_identity = (
            expected_book.manifest.id().as_str().to_owned(),
            expected_book.manifest.version().to_string(),
            expected_book.digest.as_str().to_owned(),
        );
        let mut writer_rows = store_rows(&connection);
        if change != "unpublished" {
            let current = writer_rows[3]
                .iter_mut()
                .find(|row| row[0] == rusqlite::types::Value::Text(request.to_owned()))
                .unwrap();
            current[5] = rusqlite::types::Value::Integer(1);
        }
        let writer_files = final_publication_tree(&home);
        let sync = tempfile::tempdir().unwrap();
        sheltie_runtime::failpoint::arm_rendezvous(
            "regular_open_after_stat",
            owner.as_str(),
            sync.path(),
        )
        .unwrap();
        let reader_repo = repo.clone();
        let reader_service = service.clone();
        let reader_work = work.clone();
        let kind = reader_kind.to_owned();
        let reader = std::thread::spawn(move || -> Result<(), Error> {
            match kind.as_str() {
                "workbook" => {
                    let loaded = reader_repo.load("two-step", Some("1.0.0"))?;
                    assert!(!loaded.pending_publish);
                    assert_eq!(
                        (
                            loaded.manifest.id().as_str().to_owned(),
                            loaded.manifest.version().to_string(),
                            loaded.digest.as_str().to_owned()
                        ),
                        expected_book_identity
                    );
                }
                "stats" => {
                    let work = reader_work.unwrap();
                    let stats = reader_service.stats(&work)?;
                    assert_eq!(stats.1.work_id, work);
                    assert_eq!(stats.1.status, sheltie_core::work::WorkStatus::Active);
                    assert_eq!(
                        (
                            stats.1.total_seconds,
                            stats.1.blocked_count,
                            stats.1.approvals
                        ),
                        (0, 0, 0)
                    );
                    assert_eq!(
                        stats
                            .1
                            .nodes
                            .iter()
                            .map(|node| (node.node.as_str(), node.visits, node.attempts))
                            .collect::<Vec<_>>(),
                        vec![("outline", 1, 0), ("summary", 0, 0)]
                    );
                    assert_eq!(
                        serde_json::to_value(stats).unwrap(),
                        expected_stats.unwrap()
                    );
                }
                "list" => {
                    let list = reader_service.list()?;
                    assert_eq!(list.len(), 1);
                    assert_eq!(list[0].work_id, reader_work.unwrap());
                    assert_eq!(list[0].name, "default");
                    assert_eq!(list[0].status, sheltie_core::work::WorkStatus::Active);
                    assert_eq!(list[0].current, "outline#1");
                    assert_eq!(serde_json::to_value(list).unwrap(), expected_list.unwrap());
                }
                _ => unreachable!(),
            }
            Ok(())
        });
        let mut worker = RendezvousWorker::single(reader, sync.path());
        worker.wait("reader never captured the old unpublished owner");
        sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        if change == "unpublished" {
            std::fs::remove_file(owner.as_path()).unwrap();
        } else if let Some(work) = &work {
            let replay = service
                .start(
                    two_step_args(&[("topic", "owner observation")]),
                    Some(request.into()),
                )
                .unwrap();
            assert!(replay.replayed);
            assert_eq!(work_id_of(&replay), *work);
        } else {
            assert!(repo.add(&source, Some(request.into())).unwrap().replayed);
        }
        if change != "unpublished" {
            repo.cleanup_pending().unwrap();
        }
        assert_eq!(
            store_rows(&connection),
            writer_rows,
            "writer {reader_kind}/{change}"
        );
        assert_writer_tree_unchanged(&home, &writer_files, work.as_ref(), change != "unpublished");
        match change {
            "none" | "unpublished" => {}
            "intent_hash" => {
                connection
                    .execute(
                        "UPDATE requests SET intent_hash=?1 WHERE request_id=?2",
                        ["a".repeat(64), request.to_owned()],
                    )
                    .unwrap();
            }
            "reply_json" | "effects_json" => {
                connection
                    .execute(
                        &format!(
                            "UPDATE requests SET {change}={change} || ' ' WHERE request_id=?1"
                        ),
                        [request],
                    )
                    .unwrap();
            }
            "work_id" => {
                connection
                    .execute(
                        "UPDATE requests SET work_id='2026-10-03-999-other' WHERE request_id=?1",
                        [request],
                    )
                    .unwrap();
            }
            "at" => {
                connection
                    .execute(
                        "UPDATE requests SET at='2026-10-03T00:00:00Z' WHERE request_id=?1",
                        [request],
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(!owner.as_path().exists());
        let rows = store_rows(&connection);
        let files = publication_tree(&home);
        std::fs::write(sync.path().join("release"), b"release").unwrap();
        let outcome = worker.finish().unwrap();
        if change == "none" {
            outcome.unwrap();
        } else {
            let error = outcome.unwrap_err();
            assert_eq!(
                error.code(),
                ErrorCode::StoreCorrupt,
                "{reader_kind}/{change}: {error}"
            );
            assert!(
                error.to_string().contains("owner"),
                "{reader_kind}/{change}: {error}"
            );
        }
        assert_eq!(store_rows(&connection), rows, "{reader_kind}/{change}");
        assert_eq!(publication_tree(&home), files, "{reader_kind}/{change}");
        let published: bool = connection
            .query_row(
                "SELECT published FROM requests WHERE request_id=?1",
                [request],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(published, change != "unpublished");
    }
}

#[cfg(feature = "failpoint")]
type PublicationTree = std::collections::BTreeMap<std::path::PathBuf, (u32, u64, Option<Vec<u8>>)>;

#[cfg(feature = "failpoint")]
fn publication_tree(home: &sheltie_runtime::Home) -> PublicationTree {
    use std::os::unix::fs::MetadataExt;
    fn walk(path: &Path, out: &mut PublicationTree) {
        let metadata = std::fs::symlink_metadata(path).unwrap();
        assert!(metadata.is_dir() || metadata.is_file());
        out.insert(
            path.to_owned(),
            (
                metadata.mode(),
                metadata.ino(),
                metadata.is_file().then(|| std::fs::read(path).unwrap()),
            ),
        );
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                walk(&entry.unwrap().path(), out);
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    for namespace in ["works", "workbooks", "pending"] {
        let path = home.rel(namespace).unwrap();
        if path.as_path().exists() {
            walk(path.as_path().as_std_path(), &mut out);
        }
    }
    out
}

#[cfg(feature = "failpoint")]
fn final_publication_tree(home: &sheltie_runtime::Home) -> PublicationTree {
    publication_tree(home)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(home.pending_dir().as_path().as_std_path()))
        .collect()
}

#[cfg(feature = "failpoint")]
fn assert_writer_tree_unchanged(
    home: &sheltie_runtime::Home,
    expected_tree: &PublicationTree,
    work: Option<&sheltie_core::ids::WorkId>,
    published: bool,
) {
    let current_tree = final_publication_tree(home);
    assert_eq!(
        current_tree.keys().collect::<Vec<_>>(),
        expected_tree.keys().collect::<Vec<_>>()
    );
    for (path, expected) in expected_tree {
        let actual = &current_tree[path];
        assert_eq!(
            (&actual.0, &actual.2),
            (&expected.0, &expected.2),
            "writer {}",
            path.display()
        );
        let refreshed_card = published
            && work.is_some_and(|work| {
                path == home
                    .work_dir(work)
                    .join_segment("status-card.md")
                    .as_path()
                    .as_std_path()
            });
        if !refreshed_card {
            assert_eq!(actual.1, expected.1, "writer inode {}", path.display());
        }
    }
}

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn workbook_reader_rejects_audit_drift_after_qualifying_the_publisher() {
    for field in ["work_id", "revision", "at"] {
        let (_directory, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        repo.add(&abs(&example_dir("two-step")), Some("audit-drift".into()))
            .unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let raw: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id='audit-drift'",
                [],
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
        let owner = home.rel(&format!("pending/{id}.owner")).unwrap();
        connection
            .execute(
                "UPDATE requests SET published=0 WHERE request_id='audit-drift'",
                [],
            )
            .unwrap();
        let sync = tempfile::tempdir().unwrap();
        sheltie_runtime::failpoint::arm_rendezvous(
            "regular_open_after_stat",
            owner.as_str(),
            sync.path(),
        )
        .unwrap();
        let reader_repo = repo.clone();
        let reader = std::thread::spawn(move || reader_repo.load("two-step", Some("1.0.0")));
        let mut worker = RendezvousWorker::single(reader, sync.path());
        worker.wait("reader did not qualify the original audit before owner observation");
        sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        let change = match field {
            "work_id" => "work_id='2026-10-03-999-other'",
            "revision" => "revision=1",
            "at" => "at='2026-10-03T00:00:00Z'",
            _ => unreachable!(),
        };
        connection
            .execute(
                &format!("UPDATE audit SET {change} WHERE request_id='audit-drift'"),
                [],
            )
            .unwrap();
        let rows = store_rows(&connection);
        let files = publication_tree(&home);
        let error = worker.finish().unwrap().err().unwrap();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{field}: {error}");
        assert!(
            error.to_string().contains("当前add审计不一致"),
            "{field}: {error}"
        );
        assert_eq!(store_rows(&connection), rows);
        assert_eq!(publication_tree(&home), files);
    }
}

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn readers_reject_start_and_workbook_effect_drift_after_their_reference_index() {
    for field in [
        "extra_effect",
        "final",
        "pending_prefix",
        "pending_leaf",
        "pending_uuid",
        "older_publisher",
    ] {
        let (_directory, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        let source = abs(&example_dir("two-step"));
        repo.add(&source, Some("index-old".into())).unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let old_raw: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id='index-old'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let old_effects: serde_json::Value = serde_json::from_str(&old_raw).unwrap();
        let service = sheltie_runtime::WorkService::new(home.clone());
        let work = if field == "older_publisher" {
            repo.remove("two-step", "1.0.0", Some("index-remove".into()))
                .unwrap();
            repo.add(&source, Some("index-current".into())).unwrap();
            None
        } else {
            Some(work_id_of(
                &service
                    .start(
                        two_step_args(&[("topic", "index observation")]),
                        Some("index-current".into()),
                    )
                    .unwrap(),
            ))
        };
        let other_work = if field == "final" {
            Some(work_id_of(
                &service
                    .start(
                        two_step_args(&[("topic", "other real frozen copy")]),
                        Some("index-other".into()),
                    )
                    .unwrap(),
            ))
        } else {
            None
        };
        let scope = work
            .as_ref()
            .map(|work| format!("works/{work}"))
            .unwrap_or_else(|| "workbooks/two-step/1.0.0".into());
        let sync = tempfile::tempdir().unwrap();
        sheltie_runtime::failpoint::arm_rendezvous(
            "pending_after_reference_index",
            &scope,
            sync.path(),
        )
        .unwrap();
        let reader_repo = repo.clone();
        let reader_service = service.clone();
        let reader_work = work.clone();
        let reader = std::thread::spawn(move || -> Result<(), Error> {
            if let Some(work) = reader_work {
                reader_service.stats(&work)?;
            } else {
                reader_repo.load("two-step", Some("1.0.0"))?;
            }
            Ok(())
        });
        let mut worker = RendezvousWorker::single(reader, sync.path());
        worker.wait("reader did not capture the original reference index");
        sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        let raw: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id='index-current'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut effects: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let pending = effects[0]["pending"].as_str().unwrap().to_owned();
        let id = pending.split('/').nth(1).unwrap();
        match field {
            "extra_effect" => {
                effects.as_array_mut().unwrap().push(serde_json::json!({
                    "kind": "write_file",
                    "path": format!("works/{}/unrequested.txt", work.as_ref().unwrap()),
                    "content": "unrequested",
                    "sha256": "a".repeat(64),
                }));
            }
            "final" => {
                effects[0]["final"] = serde_json::json!(format!("works/{}", other_work.unwrap()));
            }
            "pending_prefix" => {
                effects[0]["pending"] = serde_json::json!(format!("other/{id}/payload"))
            }
            "pending_leaf" => {
                effects[0]["pending"] = serde_json::json!(format!("pending/{id}/other"))
            }
            "pending_uuid" => {
                effects[0]["pending"] = serde_json::json!("pending/not-a-uuid/payload")
            }
            "older_publisher" => effects[0]["pending"] = old_effects[0]["pending"].clone(),
            _ => unreachable!(),
        }
        connection
            .execute(
                "UPDATE requests SET effects_json=?1 WHERE request_id='index-current'",
                [effects.to_string()],
            )
            .unwrap();
        let rows = store_rows(&connection);
        let files = publication_tree(&home);
        let error = worker.finish().unwrap().unwrap_err();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{field}: {error}");
        let detail = if field == "older_publisher" {
            "pending引用索引不一致"
        } else if field.starts_with("pending_") {
            "pending路径无效"
        } else {
            "Start效果归属不一致"
        };
        assert!(error.to_string().contains(detail), "{field}: {error}");
        assert_eq!(store_rows(&connection), rows);
        assert_eq!(publication_tree(&home), files);
    }
}

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn workbook_reader_rejects_an_old_index_even_when_current_effects_are_restored() {
    let (_directory, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("index-restore".into()))
        .unwrap();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let original: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id='index-restore'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let mut changed: serde_json::Value = serde_json::from_str(&original).unwrap();
    assert_ne!(changed[0]["digest"], serde_json::json!("a".repeat(64)));
    changed[0]["digest"] = serde_json::json!("a".repeat(64));
    connection
        .execute(
            "UPDATE requests SET effects_json=?1 WHERE request_id='index-restore'",
            [changed.to_string()],
        )
        .unwrap();
    let sync = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "pending_after_reference_index",
        "workbooks/two-step/1.0.0",
        sync.path(),
    )
    .unwrap();
    let reader_repo = repo.clone();
    let reader = std::thread::spawn(move || reader_repo.load("two-step", Some("1.0.0")));
    let mut worker = RendezvousWorker::single(reader, sync.path());
    worker.wait("reader did not capture the independently altered index digest");
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
    connection
        .execute(
            "UPDATE requests SET effects_json=?1 WHERE request_id='index-restore'",
            [original],
        )
        .unwrap();
    let rows = store_rows(&connection);
    let files = publication_tree(&home);
    let error = worker.finish().unwrap().err().unwrap();
    assert_eq!(error.code(), ErrorCode::StoreCorrupt);
    assert!(
        error.to_string().contains("pending引用索引不一致"),
        "{error}"
    );
    assert_eq!(store_rows(&connection), rows);
    assert_eq!(publication_tree(&home), files);
    assert!(repo.load("two-step", Some("1.0.0")).is_ok());
}

#[cfg(feature = "failpoint")]
fn independent_workbook_digest(home: &sheltie_runtime::Home, directory: &Path) -> String {
    use sha2::{Digest, Sha256};

    let files = publication_tree(home)
        .into_iter()
        .filter_map(|(path, (_, _, bytes))| {
            path.strip_prefix(directory).ok().and_then(|relative| {
                bytes.map(|bytes| (relative.to_str().unwrap().as_bytes().to_vec(), bytes))
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut digest = Sha256::new();
    digest.update(b"sheltie-workbook-digest/v2\0");
    digest.update((files.len() as u64).to_be_bytes());
    for (path, bytes) in files {
        digest.update((path.len() as u64).to_be_bytes());
        digest.update(path);
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(bytes);
    }
    format!("{:x}", digest.finalize())
}

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn installed_manifest_identity_is_bound_even_when_all_digest_records_match() {
    for field in ["id", "version"] {
        let (_directory, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        let response = repo
            .add(
                &abs(&example_dir("two-step")),
                Some("manifest-binding".into()),
            )
            .unwrap();
        let installed = home.workbook_dir("two-step", "1.0.0");
        assert_eq!(
            independent_workbook_digest(&home, installed.as_path().as_std_path()),
            response.data["digest"].as_str().unwrap()
        );
        assert_eq!(
            repo.load("two-step", Some("1.0.0"))
                .unwrap()
                .manifest
                .id()
                .as_str(),
            "two-step"
        );
        let manifest = installed.join_segment("workbook.toml");
        let original = std::fs::read_to_string(manifest.as_path()).unwrap();
        let changed = if field == "id" {
            original.replace("id = \"two-step\"", "id = \"other\"")
        } else {
            original.replace("version = \"1.0.0\"", "version = \"2.0.0\"")
        };
        assert_ne!(changed, original);
        make_writable(manifest.as_path().as_std_path());
        std::fs::write(manifest.as_path(), changed).unwrap();
        let digest = independent_workbook_digest(&home, installed.as_path().as_std_path());
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let (reply, effects): (String, String) = connection
            .query_row(
                "SELECT reply_json,effects_json FROM requests WHERE request_id='manifest-binding'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let mut reply: serde_json::Value = serde_json::from_str(&reply).unwrap();
        let mut effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
        reply["data"]["digest"] = serde_json::json!(digest);
        effects[0]["digest"] = serde_json::json!(digest);
        connection
            .execute("UPDATE workbooks SET digest=?1", [&digest])
            .unwrap();
        connection
            .execute(
                "UPDATE requests SET reply_json=?1,effects_json=?2 WHERE request_id='manifest-binding'",
                rusqlite::params![reply.to_string(), effects.to_string()],
            )
            .unwrap();
        let rows = store_rows(&connection);
        let files = publication_tree(&home);
        let error = repo.load("two-step", Some("1.0.0")).err().unwrap();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{field}: {error}");
        assert!(
            error.to_string().contains("manifest身份与Store行不一致"),
            "{field}: {error}"
        );
        assert_eq!(store_rows(&connection), rows);
        assert_eq!(publication_tree(&home), files);
    }
}

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn work_stats_distinguishes_a_missing_frozen_manifest_before_and_after_publication() {
    for published in [true, false] {
        let (_directory, home, service) = home_with_example("two-step");
        let response = service
            .start(
                two_step_args(&[("topic", "missing declaration")]),
                Some("missing-manifest".into()),
            )
            .unwrap();
        let work = work_id_of(&response);
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let root = if published {
            home.work_dir(&work)
        } else {
            let raw: String = connection
                .query_row(
                    "SELECT effects_json FROM requests WHERE request_id='missing-manifest'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            let effects: serde_json::Value = serde_json::from_str(&raw).unwrap();
            let pending = home.rel(effects[0]["pending"].as_str().unwrap()).unwrap();
            std::fs::rename(home.work_dir(&work).as_path(), pending.as_path()).unwrap();
            connection
                .execute(
                    "UPDATE requests SET published=0 WHERE request_id='missing-manifest'",
                    [],
                )
                .unwrap();
            pending
        };
        assert_eq!(service.stats(&work).unwrap().1.work_id, work);
        let frozen = root.join_segment("workbook");
        assert!(frozen.as_path().is_dir());
        make_writable(frozen.as_path().as_std_path());
        std::fs::remove_file(frozen.join_segment("workbook.toml").as_path()).unwrap();
        let rows = store_rows(&connection);
        let files = publication_tree(&home);
        let error = service.stats(&work).unwrap_err();
        assert_eq!(
            error.code(),
            ErrorCode::StoreCorrupt,
            "published={published}: {error}"
        );
        let detail = if published {
            "已发布冻结副本文件缺失"
        } else {
            "冻结副本缺失声明文件"
        };
        assert!(
            error.to_string().contains(detail),
            "published={published}: {error}"
        );
        assert_eq!(store_rows(&connection), rows);
        assert_eq!(publication_tree(&home), files);
    }
}

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn a_stale_workbook_row_still_requires_the_latest_removal_to_be_qualified() {
    let (_directory, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("stale-add".into()))
        .unwrap();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let registered = store_rows(&connection)[0][0].clone();
    let removal = repo
        .remove("two-step", "1.0.0", Some("stale-remove".into()))
        .unwrap();
    assert!(repo.list().unwrap().is_empty());
    assert_eq!(removal.data["id"], "two-step");
    connection
        .execute(
            "INSERT INTO workbooks (id,version,digest,dir,added_at) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params_from_iter(registered),
        )
        .unwrap();
    let raw: String = connection
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id='stale-remove'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let mut changed: serde_json::Value = serde_json::from_str(&raw).unwrap();
    changed["request_id"] = serde_json::json!("another-request");
    connection
        .execute(
            "UPDATE requests SET reply_json=?1 WHERE request_id='stale-remove'",
            [changed.to_string()],
        )
        .unwrap();
    let rows = store_rows(&connection);
    let files = publication_tree(&home);
    let error = repo.load("two-step", Some("1.0.0")).err().unwrap();
    assert_eq!(error.code(), ErrorCode::StoreCorrupt);
    assert!(
        error
            .to_string()
            .contains("Workbook remove请求 stale-remove snapshot身份无效"),
        "{error}"
    );
    assert_eq!(store_rows(&connection), rows);
    assert_eq!(publication_tree(&home), files);
}

// Task: C002-T49
#[cfg(feature = "failpoint")]
#[test]
fn start_rechecks_manifest_identity_on_the_copy_it_will_freeze() {
    use std::os::unix::fs::MetadataExt;
    for field in ["id", "version"] {
        let (_directory, home, service) = home_with_example("two-step");
        let manifest = home
            .workbook_dir("two-step", "1.0.0")
            .join_segment("workbook.toml");
        make_writable(manifest.as_path().as_std_path());
        let original = std::fs::read_to_string(manifest.as_path()).unwrap();
        let changed = if field == "id" {
            original.replace("id = \"two-step\"", "id = \"new-step\"")
        } else {
            original.replace("version = \"1.0.0\"", "version = \"2.0.0\"")
        };
        assert_ne!(changed, original);
        assert_eq!(changed.len(), original.len());
        let source_inode = std::fs::metadata(manifest.as_path()).unwrap().ino();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let old_rows = store_rows(&connection);
        let before = tempfile::tempdir().unwrap();
        let after = tempfile::tempdir().unwrap();
        sheltie_runtime::failpoint::arm_rendezvous(
            "pending_owner_synced_before_payload",
            "pending_owner_synced_before_payload",
            before.path(),
        )
        .unwrap();
        let writer = std::thread::spawn(move || {
            service.start(
                two_step_args(&[("topic", "copy identity")]),
                Some("copy-identity".into()),
            )
        });
        let mut worker = RendezvousWorker::new(writer, before.path(), after.path());
        worker.wait("start did not finish qualifying the registered workbook before staging");
        sheltie_runtime::failpoint::arm_rendezvous(
            "external_tree_after_stat",
            manifest.as_str(),
            after.path(),
        )
        .unwrap();
        std::fs::write(before.path().join("release"), b"release").unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !after.path().join("reached").exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "start did not reach the source-copy observation"
            );
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        std::fs::write(manifest.as_path(), &changed).unwrap();
        let error = worker.finish().unwrap().unwrap_err();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{field}: {error}");
        assert!(
            error.to_string().contains("冻结副本的 manifest 身份"),
            "{field}: {error}"
        );
        assert_eq!(
            std::fs::read_to_string(manifest.as_path()).unwrap(),
            changed
        );
        assert_eq!(
            std::fs::metadata(manifest.as_path()).unwrap().ino(),
            source_inode
        );
        let rows = store_rows(&connection);
        for table in [0, 1, 3, 4] {
            assert_eq!(rows[table], old_rows[table]);
        }
        assert!(
            std::fs::read_dir(home.works_dir().as_path())
                .unwrap()
                .next()
                .is_none()
        );
    }
}
