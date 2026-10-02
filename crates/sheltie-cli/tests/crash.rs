//! 真实CLI故障进程；Cargo提供当前候选及feature的二进制，不在测试期间构建。
#![cfg(feature = "failpoint")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "../../sheltie-runtime/tests/common/mod.rs"]
mod runtime_common;
use runtime_common::*;
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::NodeId;
use sheltie_runtime::{Error, WorkbookRepo};
use std::path::Path;
use std::process::Command;

fn sheltie_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sheltie"))
}

fn run_with_failpoint(
    home: &sheltie_runtime::Home,
    failpoint: &str,
    args: &[&str],
) -> std::process::Output {
    Command::new(sheltie_bin())
        .env("SHELTIE_FAILPOINT", failpoint)
        .args(["--home", home.root().as_str(), "--json"])
        .args(args)
        .output()
        .unwrap()
}

fn make_writable_tree(path: &Path) {
    use std::os::unix::fs::PermissionsExt as _;

    let metadata = std::fs::metadata(path).unwrap();
    let mode = if metadata.is_dir() { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    if metadata.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            make_writable_tree(&entry.unwrap().path());
        }
    }
}

// Task: T23
#[test]
fn kill_before_commit_leaves_state_unchanged_and_replay_succeeds() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let out = run_with_failpoint(
        &home,
        "before_commit",
        &[
            "--request-id",
            "r-begin",
            "attempt",
            "begin",
            wid.as_str(),
            "--node",
            "outline",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(sheltie_runtime::failpoint::EXIT_CODE),
        "子进程输出：{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let (_, json) = svc.status(&wid).unwrap();
    assert!(json.last_attempt.is_none(), "提交前被杀，状态不变");
    let again = svc
        .begin(
            &wid,
            &NodeId::new("outline").unwrap(),
            Some("r-begin".into()),
        )
        .unwrap();
    assert!(!again.replayed, "原请求没提交，这次是正常提交");
}

// Task: T23
#[test]
fn kill_after_commit_leaves_state_advanced_and_replay_returns_original_reply_and_rewrites_brief() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let out = run_with_failpoint(
        &home,
        "after_commit_before_effects",
        &[
            "--request-id",
            "r-begin",
            "attempt",
            "begin",
            wid.as_str(),
            "--node",
            "outline",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(sheltie_runtime::failpoint::EXIT_CODE),
        "子进程输出：{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let brief = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/brief.md");
    assert!(!brief.exists(), "效果前被杀，任务书还没写");
    let (_, json) = svc.status(&wid).unwrap();
    assert_eq!(json.last_attempt.as_ref().unwrap().attempt, "outline#1.0");
    let again = svc
        .begin(
            &wid,
            &NodeId::new("outline").unwrap(),
            Some("r-begin".into()),
        )
        .unwrap();
    assert!(again.replayed);
    assert!(brief.exists(), "重放补写了任务书");
}

// Task: C002-T27
#[test]
fn kill_after_delete_before_marker_leaves_result_unknown_and_blocks_next_write() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t27-initial-add".into()),
    )
    .unwrap();
    let request_id = "t27-remove-no-marker";
    let out = run_with_failpoint(
        &home,
        "delete_after_tree_removed_before_marker",
        &[
            "--request-id",
            request_id,
            "workbook",
            "remove",
            "two-step@1.0.0",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(sheltie_runtime::failpoint::EXIT_CODE)
    );

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let internal_id = pending.split('/').nth(1).unwrap();
    let marker = home
        .pending_dir()
        .join_segment(&format!("{internal_id}.deleted"));
    assert!(!home.workbook_dir("two-step", "1.0.0").as_path().exists());
    assert!(!home.rel(pending).unwrap().as_path().exists());
    assert!(!marker.as_path().exists(), "删除后被杀时还没有完成证明");
    assert_eq!(
        conn.query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );

    let error = repo
        .remove("two-step", "1.0.0", Some(request_id.to_string()))
        .unwrap_err();
    let Error::EffectPending {
        committed,
        request_id: actual_request,
        pending_request_id,
        cause,
        original,
        ..
    } = error
    else {
        panic!("缺marker的删除结果必须保留本请求不确定性：{error:?}");
    };
    assert!(committed);
    assert_eq!(actual_request, request_id);
    assert!(pending_request_id.is_none());
    assert_eq!(cause, ErrorCode::StoreCorrupt);
    let original: serde_json::Value = serde_json::from_str(original.as_deref().unwrap()).unwrap();
    assert_eq!(original["ok"], true);
    assert_eq!(original["request_id"], request_id);
    assert_eq!(original["data"]["id"], "two-step");
    assert!(!marker.as_path().exists(), "恢复不得补造完成证明");

    let blocked = repo
        .add(
            &abs(&example_dir("gated-release")),
            Some("t27-write-blocked-by-delete".into()),
        )
        .unwrap_err();
    let Error::EffectPending {
        committed,
        request_id: current,
        pending_request_id,
        cause,
        ..
    } = blocked
    else {
        panic!("结果不明的旧删除必须阻断新写：{blocked:?}");
    };
    assert!(!committed);
    assert_eq!(current, "t27-write-blocked-by-delete");
    assert_eq!(pending_request_id.as_deref(), Some(request_id));
    assert_eq!(cause, ErrorCode::StoreCorrupt);
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = 't27-write-blocked-by-delete'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

// Task: C002-T27
#[test]
fn kill_during_partial_tree_delete_keeps_pending_for_digest_rejection() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t27-partial-add".into()),
    )
    .unwrap();
    let request_id = "t27-partial-delete";
    let out = run_with_failpoint(
        &home,
        "delete_after_first_payload_child",
        &[
            "--request-id",
            request_id,
            "workbook",
            "remove",
            "two-step@1.0.0",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(sheltie_runtime::failpoint::EXIT_CODE)
    );
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects_raw: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects_raw).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let payload = home.rel(pending).unwrap();
    assert!(!home.workbook_dir("two-step", "1.0.0").as_path().exists());
    assert!(payload.as_path().is_dir());
    assert!(
        !std::fs::read_dir(payload.as_path())
            .unwrap()
            .collect::<Vec<_>>()
            .is_empty()
    );

    let error = repo
        .remove("two-step", "1.0.0", Some(request_id.to_string()))
        .unwrap_err();
    let Error::EffectPending {
        committed,
        request_id: actual_request,
        cause,
        ..
    } = error
    else {
        panic!("部分删除后摘要变化必须停止，不得继续猜测：{error:?}");
    };
    assert!(committed);
    assert_eq!(actual_request, request_id);
    assert_eq!(cause, ErrorCode::StoreCorrupt);
    assert!(payload.as_path().is_dir());
    let internal_id = pending.split('/').nth(1).unwrap();
    assert!(
        !home
            .pending_dir()
            .join_segment(&format!("{internal_id}.deleted"))
            .as_path()
            .exists()
    );
    assert_eq!(
        conn.query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

// Task: C002-T27
#[test]
fn delete_refuses_root_replaced_before_unlink() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    use std::time::{Duration, Instant};

    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t27-root-race-add".into()),
    )
    .unwrap();
    let request_id = "t27-root-replaced-before-unlink";
    let out = run_with_failpoint(
        &home,
        "after_commit_before_effects",
        &[
            "--request-id",
            request_id,
            "workbook",
            "remove",
            "two-step@1.0.0",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(sheltie_runtime::failpoint::EXIT_CODE)
    );
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects_raw: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects_raw).unwrap();
    let pending_rel = effects[0]["pending"].as_str().unwrap();
    let payload = home.rel(pending_rel).unwrap().as_path().to_path_buf();
    let preserved_parent = tempfile::tempdir().unwrap();
    let preserved = preserved_parent.path().join("original-empty-payload");
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "delete_before_root_unlink",
        request_id,
        rendezvous.path(),
    )
    .unwrap();
    let recovery_home = home.clone();
    let recovery_id = request_id.to_string();
    let recovery = std::thread::spawn(move || {
        WorkbookRepo::new(recovery_home).remove("two-step", "1.0.0", Some(recovery_id))
    });
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        let _ = recovery.join();
        sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        panic!("删除没有到达root unlink前的句柄身份检查点");
    }
    std::fs::rename(payload.as_path(), &preserved).unwrap();
    std::fs::create_dir(payload.as_path()).unwrap();
    let replacement_inode = std::fs::metadata(payload.as_path()).unwrap().ino();
    let replacement_mode = std::fs::metadata(payload.as_path())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    std::fs::write(&release, b"release").unwrap();
    let error = recovery.join().unwrap().unwrap_err();
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();

    let Error::EffectPending {
        committed,
        request_id: actual_request,
        cause,
        ..
    } = error
    else {
        panic!("root被换绑后不得unlink替代对象：{error:?}");
    };
    assert!(committed);
    assert_eq!(actual_request, request_id);
    assert_eq!(cause, ErrorCode::Io);
    assert!(preserved.is_dir());
    assert!(payload.is_dir());
    assert_eq!(
        std::fs::metadata(payload.as_path()).unwrap().ino(),
        replacement_inode
    );
    assert_eq!(
        std::fs::metadata(payload.as_path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        replacement_mode
    );
    assert!(
        std::fs::read_dir(payload.as_path())
            .unwrap()
            .next()
            .is_none()
    );
    assert!(!home.workbook_dir("two-step", "1.0.0").as_path().exists());
    assert!(
        !home
            .pending_dir()
            .join_segment(&format!(
                "{}.deleted",
                pending_rel.split('/').nth(1).unwrap()
            ))
            .as_path()
            .exists()
    );
    assert_eq!(
        conn.query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

// Task: C002-T27
#[test]
fn remove_refuses_a_different_final_or_pending_tree() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    for endpoint in ["final", "pending"] {
        let (dir, home) = temp_home();
        let repo = WorkbookRepo::new(home.clone());
        repo.add(
            &abs(&example_dir("two-step")),
            Some("t27-add-before-replace".into()),
        )
        .unwrap();
        let request_id = format!("t27-remove-replaced-{endpoint}");
        let out = run_with_failpoint(
            &home,
            "after_commit_before_effects",
            &[
                "--request-id",
                &request_id,
                "workbook",
                "remove",
                "two-step@1.0.0",
            ],
        );
        assert_eq!(
            out.status.code(),
            Some(sheltie_runtime::failpoint::EXIT_CODE)
        );
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let (effects_raw, reply_before): (String, String) = conn
            .query_row(
                "SELECT effects_json, reply_json FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let effects: serde_json::Value = serde_json::from_str(&effects_raw).unwrap();
        let pending_rel = effects[0]["pending"].as_str().unwrap();
        let payload = home.rel(pending_rel).unwrap().as_path().to_path_buf();
        let final_dir = home.workbook_dir("two-step", "1.0.0");
        if endpoint == "pending" {
            std::fs::set_permissions(
                std::path::Path::new(home.root().as_str()).join("workbooks"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            std::fs::set_permissions(
                final_dir.as_path().parent().unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            make_writable_tree(final_dir.as_path().as_std_path());
            std::fs::rename(final_dir.as_path(), payload.as_path()).unwrap();
        }

        let replacement_root = tempfile::tempdir().unwrap();
        let replacement = copy_example("two-step", replacement_root.path());
        std::fs::write(
            replacement.join("instructions/outline.md"),
            "另一个不同摘要的Workbook生命周期。\n",
        )
        .unwrap();
        let original_root = replacement_root.path().join("preserved-original");
        let target = if endpoint == "final" {
            std::fs::set_permissions(
                std::path::Path::new(home.root().as_str()).join("workbooks"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            std::fs::set_permissions(
                final_dir.as_path().parent().unwrap(),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            final_dir.as_path()
        } else {
            payload.as_path()
        };
        make_writable_tree(target.as_std_path());
        std::fs::rename(target, &original_root).unwrap();
        std::fs::rename(&replacement, target).unwrap();
        let replacement_inode = std::fs::metadata(target).unwrap().ino();
        let replacement_bytes = std::fs::read(target.join("instructions/outline.md")).unwrap();
        let original_bytes = std::fs::read(original_root.join("instructions/outline.md")).unwrap();

        let error = repo
            .remove("two-step", "1.0.0", Some(request_id.clone()))
            .unwrap_err();
        let Error::EffectPending {
            committed,
            request_id: actual_request,
            pending_request_id,
            cause,
            ..
        } = error
        else {
            panic!("删除不得把不同final/payload对象当作完成：{error:?}");
        };
        assert!(committed);
        assert_eq!(actual_request, request_id);
        assert!(pending_request_id.is_none());
        assert_eq!(cause, ErrorCode::StoreCorrupt);
        assert_eq!(std::fs::metadata(target).unwrap().ino(), replacement_inode);
        assert_eq!(
            std::fs::read(target.join("instructions/outline.md")).unwrap(),
            replacement_bytes
        );
        assert_eq!(
            std::fs::read(original_root.join("instructions/outline.md")).unwrap(),
            original_bytes
        );
        let marker_id = effects[0]["pending"]
            .as_str()
            .unwrap()
            .split('/')
            .nth(1)
            .unwrap();
        assert!(
            !home
                .pending_dir()
                .join_segment(&format!("{marker_id}.deleted"))
                .as_path()
                .exists()
        );
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        let reply_after: String = conn
            .query_row(
                "SELECT reply_json FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(reply_after, reply_before);
        drop(dir);
    }
}

// Task: T23
#[test]
fn kill_between_update_renames_leaves_prev_and_rollback_recovers() {
    let (d, home) = temp_home();
    let release = d.path().join("release");
    sheltie_runtime_test_release::make_release(&release, "9.9.9");
    let bin = std::path::PathBuf::from(home.bin_dir().as_str());
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::copy(sheltie_bin(), bin.join("sheltie")).unwrap();
    let old_bytes = std::fs::read(bin.join("sheltie")).unwrap();
    let out = Command::new(bin.join("sheltie"))
        .env("SHELTIE_FAILPOINT", "update_between_renames")
        .env("SHELTIE_RELEASE_BASE", release.to_str().unwrap())
        .args(["--home", home.root().as_str(), "self", "update"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(sheltie_runtime::failpoint::EXIT_CODE),
        "子进程输出：{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // 替换窗口的终态（存储合同 §9）：目标没了，`.prev` 就是替换前的旧二进制。
    assert_eq!(
        std::fs::read(bin.join("sheltie.prev")).unwrap(),
        old_bytes,
        ".prev 不是替换前的旧二进制"
    );
    assert!(!bin.join("sheltie").exists());
    sheltie_runtime::selfmgmt::rollback(&home).unwrap();
    assert_eq!(
        std::fs::read(bin.join("sheltie")).unwrap(),
        old_bytes,
        "rollback 恢复的不是旧二进制字节"
    );
    assert!(!bin.join("sheltie.prev").exists());
}

/// 造一个本地「发布目录」：tag 布局的 `dist-manifest.json` 与对应平台的包。
/// T20 定义精确格式、T15 改成 `latest/` + `v<version>/` 布局，本 helper 与之一致。
mod sheltie_runtime_test_release {
    use std::path::Path;

    pub fn make_release(dir: &Path, version: &str) {
        std::fs::create_dir_all(dir).unwrap();
        let platform = sheltie_runtime::selfmgmt::platform();
        let payload = format!("fake sheltie {version} for {platform}");
        let asset = format!("sheltie-{version}-{platform}");
        let digest = sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes());
        let manifest = serde_json::json!({
            "version": version,
            "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
        })
        .to_string();
        for tag in ["latest".to_string(), format!("v{version}")] {
            let tag_dir = dir.join(&tag);
            std::fs::create_dir_all(&tag_dir).unwrap();
            std::fs::write(tag_dir.join("dist-manifest.json"), &manifest).unwrap();
            if tag != "latest" {
                std::fs::write(tag_dir.join(&asset), &payload).unwrap();
            }
        }
    }
}
