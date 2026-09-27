//! T23：崩溃窗口。用 `--features failpoint` 的子进程跑 `sheltie` 二进制，在指定点退出。
//! `.config/nextest.toml` 把本文件设为串行。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;
use std::process::Command;

use common::*;
use sheltie_core::ids::NodeId;

/// 找到 workspace 里的 `sheltie` 二进制，带 `failpoint` 特性构建。
/// `CARGO_BIN_EXE_*` 只在同 crate 可用；`../../target` 的推断又被全局
/// `~/.cargo/config.toml` 的 `target-dir` 打破（构建落在别处），
/// 所以直接问 cargo 要可执行文件路径。本文件被 nextest 设为串行，cargo 不会并发。
fn sheltie_bin() -> std::path::PathBuf {
    let out = Command::new("cargo")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args([
            "build",
            "-p",
            "sheltie-cli",
            "--features",
            "sheltie-runtime/failpoint",
            "--message-format=json",
        ])
        .output()
        .expect("起不了 cargo");
    assert!(
        out.status.success(),
        "cargo build 失败：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v["reason"] == "compiler-artifact"
            && v["target"]["name"] == "sheltie"
            && v["executable"].is_string()
        {
            let p = std::path::PathBuf::from(v["executable"].as_str().unwrap_or_default());
            assert!(p.exists(), "cargo 报的路径不存在：{}", p.display());
            return p;
        }
    }
    panic!("cargo 没报出 sheltie 的可执行文件路径");
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

// Task: T23
#[test]
fn status_card_missing_is_regenerated_on_next_write() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    std::fs::remove_file(&card).unwrap();
    svc.begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    assert!(card.exists());
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
    assert!(bin.join("sheltie.prev").exists());
    assert!(!bin.join("sheltie").exists());
    sheltie_runtime::selfmgmt::rollback(&home).unwrap();
    assert!(bin.join("sheltie").exists());
    assert!(!bin.join("sheltie.prev").exists());
}

/// 造一个本地「发布目录」：`dist-manifest.json` 与对应平台的包。T20 定义精确格式并让本 helper 与之一致。
mod sheltie_runtime_test_release {
    use std::path::Path;

    pub fn make_release(dir: &Path, version: &str) {
        std::fs::create_dir_all(dir).unwrap();
        let platform = sheltie_runtime::selfmgmt::platform();
        let payload = format!("fake sheltie {version} for {platform}");
        let asset = format!("sheltie-{version}-{platform}");
        std::fs::write(dir.join(&asset), &payload).unwrap();
        let digest = sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes());
        let manifest = serde_json::json!({
            "version": version,
            "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
        });
        std::fs::write(dir.join("dist-manifest.json"), manifest.to_string()).unwrap();
    }
}
