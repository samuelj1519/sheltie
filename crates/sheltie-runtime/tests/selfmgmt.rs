//! T20：二进制自管理（runtime 层）。发布源用本地目录，不联网。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_runtime::Error;
use sheltie_runtime::selfmgmt::{self, ReleaseSource};

fn make_release(dir: &Path, version: &str, tamper: bool) -> ReleaseSource {
    std::fs::create_dir_all(dir).unwrap();
    let platform = selfmgmt::platform();
    let payload = format!("fake sheltie {version} for {platform}");
    let asset = format!("sheltie-{version}-{platform}");
    std::fs::write(dir.join(&asset), &payload).unwrap();
    let digest = if tamper {
        "0".repeat(64)
    } else {
        sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes())
            .as_str()
            .to_string()
    };
    let manifest = serde_json::json!({
        "version": version,
        "assets": [{ "platform": platform, "name": asset, "sha256": digest }]
    });
    std::fs::write(dir.join("dist-manifest.json"), manifest.to_string()).unwrap();
    ReleaseSource {
        base: dir.to_str().unwrap().to_string(),
    }
}

// Task: T20
#[test]
fn install_copies_current_exe_and_is_idempotent() {
    let (_d, home) = temp_home();
    let first = selfmgmt::install(&home, false).unwrap();
    assert!(!first.already_installed);
    assert!(Path::new(first.installed_to.as_str()).exists());
    let second = selfmgmt::install(&home, false).unwrap();
    assert!(second.already_installed);
}

// Task: T20
#[test]
fn install_prints_path_hint_and_does_not_touch_rc_by_default() {
    let (d, home) = temp_home();
    let fake_rc = d.path().join(".zshrc");
    std::fs::write(&fake_rc, "# rc\n").unwrap();
    let out = selfmgmt::install(&home, false).unwrap();
    assert!(out.path_hint.contains(home.bin_dir().as_str()));
    assert_eq!(std::fs::read_to_string(&fake_rc).unwrap(), "# rc\n");
}

// Task: T20
#[test]
fn update_replaces_binary_and_keeps_prev() {
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let src = make_release(&d.path().join("rel"), "9.9.9", false);
    let out = selfmgmt::update(&home, &src, None).unwrap();
    assert_eq!(out.to, "9.9.9");
    let bin = std::path::PathBuf::from(home.bin_dir().as_str());
    assert!(bin.join("sheltie.prev").exists());
    assert!(
        std::fs::read_to_string(bin.join("sheltie"))
            .unwrap()
            .starts_with("fake sheltie 9.9.9")
    );
}

// Task: T20
#[test]
fn update_rejects_checksum_mismatch_and_leaves_binary_intact() {
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let before =
        std::fs::read(std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie")).unwrap();
    let src = make_release(&d.path().join("rel"), "9.9.9", true);
    assert!(matches!(
        selfmgmt::update(&home, &src, None),
        Err(Error::UpdateChecksumMismatch { .. })
    ));
    let after =
        std::fs::read(std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie")).unwrap();
    assert_eq!(before, after);
    let tmp = std::path::PathBuf::from(home.tmp_dir().as_str());
    assert!(
        !tmp.exists() || std::fs::read_dir(tmp).unwrap().next().is_none(),
        "下载文件已删"
    );
}

// Task: T20
#[test]
fn update_reports_unavailable_when_no_asset_for_platform() {
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("dist-manifest.json"),
        r#"{"version":"9.9.9","assets":[]}"#,
    )
    .unwrap();
    let src = ReleaseSource {
        base: dir.to_str().unwrap().to_string(),
    };
    assert!(matches!(
        selfmgmt::update(&home, &src, None),
        Err(Error::UpdateUnavailable { .. })
    ));
}

// Task: T20
#[test]
fn rollback_swaps_prev_back() {
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let original =
        std::fs::read(std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie")).unwrap();
    let src = make_release(&d.path().join("rel"), "9.9.9", false);
    selfmgmt::update(&home, &src, None).unwrap();
    selfmgmt::rollback(&home).unwrap();
    assert_eq!(
        std::fs::read(std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie")).unwrap(),
        original
    );
    assert!(
        matches!(selfmgmt::rollback(&home), Err(Error::NotFound { .. })),
        "只保留一级"
    );
}

// Task: T20
#[test]
fn rollback_recovers_when_current_missing() {
    let (_d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let bin = std::path::PathBuf::from(home.bin_dir().as_str());
    std::fs::rename(bin.join("sheltie"), bin.join("sheltie.prev")).unwrap();
    selfmgmt::rollback(&home).unwrap();
    assert!(bin.join("sheltie").exists());
}

// Task: T20
#[test]
fn uninstall_keeps_store_and_works() {
    let (_d, home, svc) = home_with_example("two-step");
    start_two_step(&svc);
    selfmgmt::install(&home, false).unwrap();
    selfmgmt::uninstall(&home, false, false).unwrap();
    assert!(!std::path::PathBuf::from(home.bin_dir().as_str()).exists());
    assert!(std::path::PathBuf::from(home.store_path().as_str()).exists());
    assert!(std::path::PathBuf::from(home.works_dir().as_str()).exists());
}

// Task: T20
#[test]
fn uninstall_purge_requires_yes() {
    let (_d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    assert!(matches!(
        selfmgmt::uninstall(&home, true, false),
        Err(Error::InvalidRequest { .. })
    ));
    selfmgmt::uninstall(&home, true, true).unwrap();
    assert!(!std::path::PathBuf::from(home.root().as_str()).exists());
}

// ── M3 里程碑审查补测：发布链与 install 语义（挂 T20） ─────────────────────

// Task: T20
#[test]
fn platform_matches_supported_target_triples() {
    // 发布包按 Rust target triple 命名（存储合同 §9）；平台串错了就永远找不到包。
    let expect = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-gnu"),
        _ => None,
    };
    if let Some(e) = expect {
        assert_eq!(selfmgmt::platform(), e);
    }
}

// Task: T20
#[test]
fn install_replaces_divergent_binary_instead_of_short_circuit() {
    let (_d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    std::fs::write(&bin, "被换掉的旧文件").unwrap();
    let out = selfmgmt::install(&home, false).unwrap();
    // 幂等短路只认「字节相同」；分叉了的二进制必须重装回当前可执行文件。
    assert!(!out.already_installed, "分叉后仍报 already_installed");
    let current = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    assert_eq!(std::fs::read(&bin).unwrap(), current);
}

// Task: T20
#[test]
fn update_version_flag_mismatch_is_unavailable() {
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let src = make_release(&d.path().join("rel"), "9.9.9", false);
    assert!(matches!(
        selfmgmt::update(&home, &src, Some("8.8.8")),
        Err(Error::UpdateUnavailable { .. })
    ));
    let out = selfmgmt::update(&home, &src, Some("9.9.9")).unwrap();
    assert_eq!(out.to, "9.9.9");
    // 下载来的资产是普通文件，装上后必须可执行。
    use std::os::unix::fs::PermissionsExt as _;
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    let mode = std::fs::metadata(&bin).unwrap().permissions().mode();
    assert!(mode & 0o111 != 0, "bin/sheltie 不可执行（mode {mode:o}）");
}

// Task: T20
#[test]
fn update_unpacks_tarball_asset_and_keeps_executable_bit() {
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let dir = d.path().join("rel");
    let platform = selfmgmt::platform();
    let payload = format!("tarred sheltie for {platform}");
    use std::os::unix::fs::PermissionsExt as _;
    // cargo-dist 0.32 的真实包内布局（T25 用 dist build 的产出核过）：
    // <产物名去掉扩展>/sheltie，二进制在内层目录根部。
    let asset = format!("sheltie-cli-9.9.9-{platform}.tar.gz");
    let inner = format!("sheltie-cli-9.9.9-{platform}");
    std::fs::create_dir_all(dir.join(&inner)).unwrap();
    std::fs::write(dir.join(&inner).join("sheltie"), &payload).unwrap();
    std::fs::set_permissions(
        dir.join(&inner).join("sheltie"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let tar = std::process::Command::new("tar")
        .args(["-czf", &asset, &inner])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        tar.status.success(),
        "tar 打包失败：{}",
        String::from_utf8_lossy(&tar.stderr)
    );
    let bytes = std::fs::read(dir.join(&asset)).unwrap();
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(&bytes);
    let manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    });
    std::fs::write(dir.join("dist-manifest.json"), manifest.to_string()).unwrap();
    selfmgmt::update(
        &home,
        &ReleaseSource {
            base: dir.to_str().unwrap().to_string(),
        },
        None,
    )
    .unwrap();
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    assert_eq!(std::fs::read(&bin).unwrap(), payload.as_bytes());
    let mode = std::fs::metadata(&bin).unwrap().permissions().mode();
    assert!(
        mode & 0o111 != 0,
        "解包后的 bin/sheltie 不可执行（mode {mode:o}）"
    );
}

// Task: T20
#[test]
fn update_adapts_cargo_dist_manifest_format() {
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let dir = d.path().join("rel");
    let platform = selfmgmt::platform();
    let payload = format!("cargo-dist style sheltie for {platform}");
    let asset = format!("sheltie-cli-7.7.7-{platform}");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(&asset), &payload).unwrap();
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes());
    // 完整 dist-manifest.json 的真实形态（0.32.0，T25 用 dist build 的产出核过）：
    // artifacts 是按产物名索引的对象；checksum 是校验文件名，真哈希在 checksums.sha256。
    // 非 executable-zip 的产物要被跳过，别的平台的也要被跳过。
    let manifest = serde_json::json!({
        "dist_version": "0.32.0",
        "announcement_tag": "v7.7.7",
        "announcement_is_prerelease": false,
        "releases": [{ "app_name": "sheltie-cli", "app_version": "7.7.7", "artifacts": ["sheltie-cli-installer.sh", &asset] }],
        "artifacts": {
            "sheltie-cli-installer.sh": { "name": "sheltie-cli-installer.sh", "kind": "installer", "target_triples": [] },
            asset.clone(): { "name": &asset, "kind": "executable-zip", "target_triples": [&platform],
              "checksum": format!("{asset}.sha256"),
              "checksums": { "sha256": digest.as_str() } },
            "sheltie-cli-7.7.7-other-platform": { "name": "sheltie-cli-7.7.7-other-platform", "kind": "executable-zip",
              "target_triples": ["other-platform"], "checksum": "sheltie-cli-7.7.7-other-platform.sha256",
              "checksums": { "sha256": "0".repeat(64) } },
        }
    });
    std::fs::write(dir.join("dist-manifest.json"), manifest.to_string()).unwrap();
    let src = ReleaseSource {
        base: dir.to_str().unwrap().to_string(),
    };
    let out = selfmgmt::update(&home, &src, None).unwrap();
    assert_eq!(out.to, "7.7.7");
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    assert_eq!(std::fs::read(&bin).unwrap(), payload.as_bytes());
}

/// 找带默认特性的 `sheltie` 二进制。全局 `~/.cargo/config.toml` 的 target-dir
/// 让 `../../target` 推断失效，直接问 cargo 要路径（同 crash.rs 的做法）。
fn sheltie_bin() -> std::path::PathBuf {
    let out = std::process::Command::new("cargo")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args(["build", "-p", "sheltie-cli", "--message-format=json"])
        .output()
        .unwrap();
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

// Task: T20
#[test]
fn install_modify_path_appends_export_line_to_shell_rc() {
    let (d, home) = temp_home();
    // rc 写进子进程的临时 $HOME，不碰真实家目录。
    let fake_home = d.path().join("fakehome");
    std::fs::create_dir_all(&fake_home).unwrap();
    let out = std::process::Command::new(sheltie_bin())
        .env("HOME", &fake_home)
        .env("SHELL", "/bin/zsh")
        .args([
            "--home",
            home.root().as_str(),
            "self",
            "install",
            "--modify-path",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "self install --modify-path 失败：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rc = std::fs::read_to_string(fake_home.join(".zshrc")).unwrap();
    // T04 起管理根为真实形式；期望与 bin_dir 同源即可（它随 root 一起规范化）。
    assert!(
        rc.contains(&format!(
            "export PATH=\"{}:$PATH\"",
            std::fs::canonicalize(home.bin_dir().as_path())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| home.bin_dir().as_str().to_string())
        )),
        ".zshrc 里没有 PATH 行：{rc}"
    );
}

// Task: T20
#[test]
fn update_unpacks_tgz_named_asset() {
    // 覆盖后缀判定的第三个区段：.tgz 与 .tar.gz、.tar.xz 是并列写法，
    // 只测 .tar.gz 时「|| 换 &&」的突变体测不出来。
    let (d, home) = temp_home();
    selfmgmt::install(&home, false).unwrap();
    let dir = d.path().join("rel");
    let platform = selfmgmt::platform();
    let payload = format!("tgz sheltie for {platform}");
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::create_dir_all(dir.join("sheltie/bin")).unwrap();
    std::fs::write(dir.join("sheltie/bin/sheltie"), &payload).unwrap();
    std::fs::set_permissions(
        dir.join("sheltie/bin/sheltie"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let asset = format!("sheltie-9.9.9-{platform}.tgz");
    let tar = std::process::Command::new("tar")
        .args(["-czf", &asset, "sheltie"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        tar.status.success(),
        "tar 打包失败：{}",
        String::from_utf8_lossy(&tar.stderr)
    );
    let bytes = std::fs::read(dir.join(&asset)).unwrap();
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(&bytes);
    let manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    });
    std::fs::write(dir.join("dist-manifest.json"), manifest.to_string()).unwrap();
    selfmgmt::update(
        &home,
        &ReleaseSource {
            base: dir.to_str().unwrap().to_string(),
        },
        None,
    )
    .unwrap();
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    assert_eq!(std::fs::read(&bin).unwrap(), payload.as_bytes());
}
