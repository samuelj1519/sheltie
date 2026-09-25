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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
#[ignore = "T20"]
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
