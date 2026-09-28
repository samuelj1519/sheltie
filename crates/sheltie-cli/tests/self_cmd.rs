//! T20：`self` 组（cli 层）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::{Path, PathBuf};

use common::*;
use serde_json::Value;

// Task: T20
#[test]
fn self_version_works_without_home() {
    let env = Env::new();
    // 管理根目录存在但里面什么都没有；self version 不需要 store.db。
    let v = env.ok(&["self", "version"]);
    assert_eq!(v["data"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(v["data"]["schema_version"], 2);
    assert!(!env.dir.path().join("store.db").exists());
}

// ── C002-T15：协议 JSON 纯度、schema 2 提示与固定 tag ────────────────────

/// 本地发布夹具：版本 = 当前版本，`self update` 走「已是最新」短路，不下载资产。
fn write_current_version_fixture(env: &Env) -> PathBuf {
    let rel = env.dir.path().join("rel");
    let manifest = format!(
        r#"{{"version":"{}","assets":[]}}"#,
        env!("CARGO_PKG_VERSION")
    );
    for tag in [
        "latest".to_string(),
        format!("v{}", env!("CARGO_PKG_VERSION")),
    ] {
        let dir = rel.join(&tag);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("dist-manifest.json"), &manifest).unwrap();
    }
    rel
}

/// 本地发布夹具：`latest/` 是 9.9.9，另写 `v8.8.8/` 的自洽发布。
fn write_two_version_fixture(env: &Env) -> PathBuf {
    let rel = env.dir.path().join("rel2");
    let platform = sheltie_runtime::selfmgmt::platform();
    for version in ["8.8.8", "9.9.9"] {
        let payload = format!("cli fixture payload {version} for {platform}");
        let asset = format!("sheltie-{version}-{platform}");
        let digest = sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes());
        let manifest = serde_json::json!({
            "version": version,
            "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
        })
        .to_string();
        let tags: Vec<String> = if version == "9.9.9" {
            vec!["latest".to_string(), format!("v{version}")]
        } else {
            vec![format!("v{version}")]
        };
        for tag in tags {
            let dir = rel.join(&tag);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("dist-manifest.json"), &manifest).unwrap();
            std::fs::write(dir.join(&asset), &payload).unwrap();
        }
    }
    rel
}

/// 带 `SHELTIE_RELEASE_BASE` 的 JSON 命令，返回解析后的响应封装。
fn ok_with_release_base(env: &Env, rel: &Path, args: &[&str]) -> Value {
    let out = env
        .cmd(args)
        .env("SHELTIE_RELEASE_BASE", rel)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "命令失败：{args:?}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true);
    v
}

// Task: C002-T15
#[test]
fn self_install_json_stdout_is_single_document() {
    // `--json` 的 stdout 只能是协议 JSON：from_slice 拒绝任何前后缀文本。
    // schema 2 提示只在文本模式出现（见下面的 text 用例）；JSON 里是响应封装
    // 的 data 字段（installed_to、path_hint），提示不混在 JSON 外面。
    let env = Env::new();
    let out = env.cmd(&["self", "install"]).output().unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["data"]["installed_to"].is_string());
    assert!(v["data"]["path_hint"].is_string());
}

// Task: C002-T15
#[test]
fn self_update_json_stdout_is_single_document() {
    // 更新（这里是「已是最新」短路）走同一封装，stdout 也是纯 JSON。
    let env = Env::new();
    let rel = write_current_version_fixture(&env);
    let v = ok_with_release_base(&env, &rel, &["self", "update"]);
    assert_eq!(v["data"]["up_to_date"], true);
}

// Task: C002-T23
#[test]
fn self_update_resolves_relative_release_base_from_command_directory() {
    let env = Env::new();
    write_current_version_fixture(&env);
    let out = env
        .cmd(&["self", "update"])
        .current_dir(env.dir.path())
        .env("SHELTIE_RELEASE_BASE", "rel")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let response: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["data"]["up_to_date"], true);
}

// Task: C002-T15
#[test]
fn self_install_text_prompt_explains_schema2_and_rollback_limit() {
    // 安装提示要写清 Store schema 2 与旧数据保留，不能误导成「只换二进制即可降级」。
    let env = Env::new();
    let out = env.cmd_text(&["self", "install"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Store schema 是 2"), "{text}");
    assert!(text.contains("旧数据保留在旧管理根"), "{text}");
    assert!(
        text.contains("rollback 只换回旧二进制，不降级 Store"),
        "{text}"
    );
}

// Task: C002-T15
#[test]
fn self_update_text_prompt_explains_schema2_and_rollback_limit() {
    // 更新提示同一口径。
    let env = Env::new();
    let rel = write_current_version_fixture(&env);
    let out = env
        .cmd_text(&["self", "update"])
        .env("SHELTIE_RELEASE_BASE", &rel)
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("已是最新"), "{text}");
    assert!(text.contains("Store schema 是 2"), "{text}");
    assert!(
        text.contains("rollback 只换回旧二进制，不降级 Store"),
        "{text}"
    );
}

// Task: C002-T15
#[test]
fn self_update_version_flag_pins_tag() {
    // 真实入口的 `--version 8.8.8` 固定到 v8.8.8；latest 指着 9.9.9 也不看。
    let env = Env::new();
    env.ok(&["self", "install"]);
    let rel = write_two_version_fixture(&env);
    let v = ok_with_release_base(&env, &rel, &["self", "update", "--version", "8.8.8"]);
    assert_eq!(v["data"]["to"], "8.8.8");
    let platform = sheltie_runtime::selfmgmt::platform();
    let expect = format!("cli fixture payload 8.8.8 for {platform}");
    let bin = env.dir.path().join("bin").join("sheltie");
    assert_eq!(std::fs::read_to_string(&bin).unwrap(), expect);
}

// Task: C002-T23
#[test]
fn purge_reports_the_retained_root_and_lock() {
    let env = Env::new();
    let result = env.ok(&["self", "uninstall", "--purge", "--yes"]);
    let home = std::fs::canonicalize(env.home()).unwrap();
    assert_eq!(
        result["data"]["kept"],
        serde_json::json!([home.to_str().unwrap(), home.join(".lock").to_str().unwrap()])
    );
    let text = String::from_utf8_lossy(
        &env.cmd_text(&["self", "uninstall", "--purge", "--yes"])
            .output()
            .unwrap()
            .stdout,
    )
    .to_string();
    assert!(text.contains("保留"), "{text}");
    assert!(home.join(".lock").is_file());
    assert_eq!(
        std::fs::read_dir(&home)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>(),
        vec![".lock"]
    );
}
