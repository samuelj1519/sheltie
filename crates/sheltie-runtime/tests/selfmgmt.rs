//! T20：二进制自管理（runtime 层）。发布源用本地目录，不联网。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_runtime::Error;
use sheltie_runtime::selfmgmt::{self, ReleaseSource};

fn sentinel(dir: &Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, format!("sentinel-{name}")).unwrap();
    path
}

fn snapshot(path: &Path) -> (Vec<u8>, u32) {
    use std::os::unix::fs::PermissionsExt as _;
    (
        std::fs::read(path).unwrap(),
        std::fs::metadata(path).unwrap().permissions().mode(),
    )
}

/// 写一个 tag 目录：`<base>/<tag>/dist-manifest.json` 与该 tag 的资产。
/// 本地发布目录镜像远端 tag 布局（存储合同 §9）：`latest/` 是移动别名，
/// 固定 tag 的清单与资产都在 `v<version>/` 下。
fn write_tag_dir(base: &Path, tag: &str, manifest: &str, assets: &[(&str, &[u8])]) {
    let dir = base.join(tag);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("dist-manifest.json"), manifest).unwrap();
    for (name, bytes) in assets {
        std::fs::write(dir.join(name), bytes).unwrap();
    }
}

fn make_release(dir: &Path, version: &str, tamper: bool) -> ReleaseSource {
    std::fs::create_dir_all(dir).unwrap();
    let platform = selfmgmt::platform();
    let payload = format!("fake sheltie {version} for {platform}");
    let asset = format!("sheltie-{version}-{platform}");
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
    })
    .to_string();
    // `latest/` 只用来发现版本号；清单与资产都从固定后的 `v<version>/` 取。
    write_tag_dir(dir, "latest", &manifest, &[]);
    write_tag_dir(
        dir,
        &format!("v{version}"),
        &manifest,
        &[(&asset, payload.as_bytes())],
    );
    ReleaseSource {
        base: dir.to_str().unwrap().to_string(),
    }
}

// Task: T20
#[test]
fn install_copies_current_exe_and_is_idempotent() {
    let (_d, home) = temp_home();
    let first = selfmgmt::install(&home).unwrap();
    assert!(!first.already_installed);
    assert!(Path::new(first.installed_to.as_str()).exists());
    let second = selfmgmt::install(&home).unwrap();
    assert!(second.already_installed);
}

// Task: T20
#[test]
fn install_prints_path_hint_and_does_not_touch_rc_by_default() {
    let (d, home) = temp_home();
    let fake_rc = d.path().join(".zshrc");
    std::fs::write(&fake_rc, "# rc\n").unwrap();
    let out = selfmgmt::install(&home).unwrap();
    assert!(out.path_hint.contains(home.bin_dir().as_str()));
    assert_eq!(std::fs::read_to_string(&fake_rc).unwrap(), "# rc\n");
}

// Task: T20
#[test]
fn update_replaces_binary_and_keeps_prev() {
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
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
    selfmgmt::install(&home).unwrap();
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

// Task: C002-T23
#[cfg(feature = "failpoint")]
#[test]
fn update_stops_if_verified_candidate_bytes_change_before_any_replacement() {
    use std::time::{Duration, Instant};

    let _serial = PURGE_PARTIAL_LOCK.lock().unwrap();
    let (dir, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let prev = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie.prev");
    std::fs::write(&prev, b"older previous binary").unwrap();
    let prev_before = std::fs::read(&prev).unwrap();
    let source = make_release(&dir.path().join("release"), "9.9.9", false);
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "update_after_candidate_verify",
        home.root().as_str(),
        rendezvous.path(),
    )
    .unwrap();
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        }
    }
    let _guard = Guard;
    let update_home = home.clone();
    let update = std::thread::spawn(move || selfmgmt::update(&update_home, &source, None));
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        let _ = update.join();
        panic!("update 未到达候选核验后的同步点");
    }
    let tmp = std::fs::read_dir(home.tmp_dir().as_path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(tmp.join("sheltie-candidate"), b"unverified replacement").unwrap();
    std::fs::write(&release, b"release").unwrap();
    assert!(matches!(
        update.join().unwrap(),
        Err(Error::InvalidRequest { .. })
    ));
    assert_eq!(std::fs::read(bin_path(&home)).unwrap(), before);
    assert_eq!(std::fs::read(&prev).unwrap(), prev_before);
    assert_tmp_clean(&home);
}

// Task: T20
#[test]
fn update_reports_unavailable_when_no_asset_for_platform() {
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    let empty = r#"{"version":"9.9.9","assets":[]}"#;
    write_tag_dir(&dir, "latest", empty, &[]);
    write_tag_dir(&dir, "v9.9.9", empty, &[]);
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
    selfmgmt::install(&home).unwrap();
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
    selfmgmt::install(&home).unwrap();
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
    selfmgmt::install(&home).unwrap();
    selfmgmt::uninstall(&home, false, false).unwrap();
    assert!(!std::path::PathBuf::from(home.bin_dir().as_str()).exists());
    assert!(std::path::PathBuf::from(home.store_path().as_str()).exists());
    assert!(std::path::PathBuf::from(home.works_dir().as_str()).exists());
}

// Task: T20
#[test]
fn uninstall_purge_requires_yes() {
    let (_d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let lock_path = std::path::PathBuf::from(home.lock_path().as_str());
    let lock_before = std::fs::metadata(&lock_path).unwrap();
    use std::os::unix::fs::MetadataExt;
    assert!(matches!(
        selfmgmt::uninstall(&home, true, false),
        Err(Error::InvalidRequest { .. })
    ));
    selfmgmt::uninstall(&home, true, true).unwrap();
    assert!(std::path::PathBuf::from(home.root().as_str()).is_dir());
    let lock_after = std::fs::metadata(&lock_path).unwrap();
    assert_eq!(
        (lock_before.dev(), lock_before.ino()),
        (lock_after.dev(), lock_after.ino())
    );
    let entries: Vec<_> = std::fs::read_dir(home.root().as_path()).unwrap().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].as_ref().unwrap().file_name(), ".lock");
}

// Task: C002-T23
#[test]
fn purge_removes_frozen_work_and_binary_but_preserves_same_root_lock() {
    use std::os::unix::fs::MetadataExt as _;

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
        .unwrap();
    write_output(&output_dir_of(&begun), "outline.md", "draft");
    selfmgmt::install(&home).unwrap();
    drop(svc);
    let database_before = std::fs::read(home.store_path().as_path()).unwrap();
    let lock_before = std::fs::metadata(home.lock_path().as_path()).unwrap();

    let kept = selfmgmt::uninstall(&home, true, true).unwrap();

    assert_eq!(kept, vec![home.root().clone(), home.lock_path()]);
    assert_eq!(
        std::fs::read_dir(home.root().as_path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>(),
        vec![".lock"]
    );
    let lock_after = std::fs::metadata(home.lock_path().as_path()).unwrap();
    assert_eq!(
        (lock_after.dev(), lock_after.ino()),
        (lock_before.dev(), lock_before.ino())
    );
    assert!(!home.store_path().as_path().exists());
    assert!(!home.bin_dir().as_path().exists());
    assert!(!home.workbooks_dir().as_path().exists());
    assert!(!home.works_dir().as_path().exists());
    assert!(!home.pending_dir().as_path().exists());
    assert!(!home.tmp_dir().as_path().exists());
    assert!(!database_before.is_empty());
}

// Task: C002-T23
#[test]
fn install_rejects_tmp_parent_symlink_and_preserves_external_sentinel() {
    let outside = tempfile::tempdir().unwrap();
    let external = sentinel(outside.path(), "install-tmp-sentinel");
    let external_before = snapshot(&external);
    let (_dir, home) = temp_home();
    std::os::unix::fs::symlink(outside.path(), home.tmp_dir().as_path()).unwrap();

    let error = selfmgmt::install(&home).unwrap_err();
    assert!(error.to_string().contains("符号链接"), "{error:?}");
    assert_eq!(snapshot(&external), external_before);
    assert!(!home.bin_dir().as_path().join("sheltie").exists());
}

// Task: C002-T23
#[test]
fn rollback_rejects_bin_parent_symlink_and_preserves_external_binaries() {
    let outside = tempfile::tempdir().unwrap();
    let target = sentinel(outside.path(), "sheltie");
    let previous = sentinel(outside.path(), "sheltie.prev");
    let target_before = snapshot(&target);
    let previous_before = snapshot(&previous);
    let (_dir, home) = temp_home();
    std::os::unix::fs::symlink(outside.path(), home.bin_dir().as_path()).unwrap();

    let error = selfmgmt::rollback(&home).unwrap_err();
    assert!(error.to_string().contains("符号链接"), "{error:?}");
    assert_eq!(snapshot(&target), target_before);
    assert_eq!(snapshot(&previous), previous_before);
}

// Task: C002-T23
#[test]
fn update_rejects_symlink_in_archive_without_chmodding_its_target() {
    use std::os::unix::fs::{PermissionsExt as _, symlink};

    let (fixture, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let original_binary = std::fs::read(bin_path(&home)).unwrap();
    let external = sentinel(fixture.path(), "archive-link-target");
    std::fs::set_permissions(&external, std::fs::Permissions::from_mode(0o640)).unwrap();
    let external_before = snapshot(&external);

    let version = "8.6.1";
    let platform = selfmgmt::platform();
    let archive_name = format!("sheltie-cli-{version}-{platform}.tar.xz");
    let package_dir = format!("sheltie-cli-{version}-{platform}");
    let staging = fixture.path().join("symlink-archive");
    let package = staging.join(&package_dir);
    std::fs::create_dir_all(&package).unwrap();
    symlink(&external, package.join("sheltie")).unwrap();
    let archive = fixture.path().join(&archive_name);
    let packed = std::process::Command::new("tar")
        .args(["-cJf", archive.to_str().unwrap(), &package_dir])
        .current_dir(&staging)
        .output()
        .unwrap();
    assert!(
        packed.status.success(),
        "tar打包失败：{}",
        String::from_utf8_lossy(&packed.stderr)
    );
    let archive_bytes = std::fs::read(&archive).unwrap();
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(&archive_bytes);
    let manifest = serde_json::json!({
        "version": version,
        "assets": [{ "platform": platform, "name": archive_name.clone(), "sha256": digest.as_str() }]
    })
    .to_string();
    let release_root = fixture.path().join("release");
    write_tag_dir(&release_root, "latest", &manifest, &[]);
    write_tag_dir(
        &release_root,
        &format!("v{version}"),
        &manifest,
        &[(archive_name.as_str(), &archive_bytes)],
    );

    let error = selfmgmt::update(&home, &release_source(&release_root), Some(version)).unwrap_err();
    assert!(error.to_string().contains("链接或特殊对象"), "{error:?}");
    assert_eq!(std::fs::read(bin_path(&home)).unwrap(), original_binary);
    assert_eq!(snapshot(&external), external_before);
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

#[cfg(feature = "failpoint")]
static PURGE_PARTIAL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// Task: C002-T23
#[cfg(feature = "failpoint")]
#[test]
fn purge_reports_partial_roots_and_keeps_store_until_data_trees_are_removed() {
    use std::time::{Duration, Instant};

    let _serial = PURGE_PARTIAL_LOCK.lock().unwrap();
    let (dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    svc.begin(
        &wid,
        &sheltie_core::ids::NodeId::new("outline").unwrap(),
        None,
    )
    .unwrap();
    selfmgmt::install(&home).unwrap();
    let database = std::fs::read(home.store_path().as_path()).unwrap();
    use std::os::unix::fs::MetadataExt as _;
    let lock_before = std::fs::metadata(home.lock_path().as_path()).unwrap();
    drop(svc);

    let rendezvous = tempfile::tempdir().unwrap();
    let root_name = home.root().as_str().to_string();
    sheltie_runtime::failpoint::arm_rendezvous(
        "purge_after_top_level_delete",
        &root_name,
        rendezvous.path(),
    )
    .unwrap();
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        }
    }
    let _guard = Guard;
    let purge_home = home.clone();
    let purge = std::thread::spawn(move || selfmgmt::uninstall(&purge_home, true, true));
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        let _ = purge.join();
        panic!("purge 未到达首个根目录删除后的同步点");
    }
    let blocked = home.work_dir(&wid).join_segment("purge-special");
    let fifo = std::process::Command::new("mkfifo")
        .arg(blocked.as_str())
        .status()
        .unwrap();
    assert!(fifo.success());
    struct ReleaseGuard(std::path::PathBuf);
    impl Drop for ReleaseGuard {
        fn drop(&mut self) {
            let _ = std::fs::write(&self.0, b"release");
        }
    }
    let _release = ReleaseGuard(release.clone());
    std::fs::write(&release, b"release").unwrap();
    let result = purge.join().unwrap();

    match result {
        Err(Error::Io { path, source }) => {
            assert!(path.contains("works"), "失败位置要明确：{path}");
            let detail = source.to_string();
            assert!(detail.contains("部分清理"), "{detail}");
            assert!(detail.contains("workbooks"), "已完成顶层项要明确：{detail}");
        }
        other => panic!("特殊文件导致的部分purge未报告：{other:?}"),
    }
    assert!(!home.workbooks_dir().as_path().exists());
    assert!(home.works_dir().as_path().exists());
    assert!(home.bin_dir().as_path().exists());
    assert_eq!(
        std::fs::read(home.store_path().as_path()).unwrap(),
        database
    );
    let lock_after = std::fs::metadata(home.lock_path().as_path()).unwrap();
    assert_eq!(
        (lock_after.dev(), lock_after.ino()),
        (lock_before.dev(), lock_before.ino())
    );
    assert!(
        home.root()
            .as_path()
            .join("works")
            .join(wid.as_str())
            .join("purge-special")
            .exists()
    );
    drop(dir);
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

// Task: C002-T23
#[cfg(feature = "failpoint")]
#[test]
fn purge_final_rescan_removes_late_readonly_sqlite_shm_and_retains_lock() {
    use std::time::{Duration, Instant};

    let _serial = PURGE_PARTIAL_LOCK.lock().unwrap();
    let (_dir, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let lock_before = std::fs::metadata(home.lock_path().as_path()).unwrap();
    use std::os::unix::fs::MetadataExt as _;
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "purge_before_final_rescan",
        home.root().as_str(),
        rendezvous.path(),
    )
    .unwrap();
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        }
    }
    let _guard = Guard;
    let purge_home = home.clone();
    let purge = std::thread::spawn(move || selfmgmt::uninstall(&purge_home, true, true));
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        let _ = purge.join();
        panic!("purge 未到达最终复核同步点");
    }
    std::fs::write(
        home.root().join_segment("store.db-shm").as_path(),
        b"late shm",
    )
    .unwrap();
    std::fs::write(&release, b"release").unwrap();
    let kept = purge.join().unwrap().unwrap();
    assert_eq!(kept, vec![home.root().clone(), home.lock_path().clone()]);
    assert!(!home.root().join_segment("store.db-shm").as_path().exists());
    let lock_after = std::fs::metadata(home.lock_path().as_path()).unwrap();
    assert_eq!(lock_before.dev(), lock_after.dev());
    assert_eq!(lock_before.ino(), lock_after.ino());
}

// Task: C002-T23
#[cfg(feature = "failpoint")]
#[test]
fn purge_reports_partial_progress_if_locked_inode_is_unlinked() {
    use std::time::{Duration, Instant};

    let _serial = PURGE_PARTIAL_LOCK.lock().unwrap();
    let (_dir, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "purge_before_final_rescan",
        home.root().as_str(),
        rendezvous.path(),
    )
    .unwrap();
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        }
    }
    let _guard = Guard;
    let purge_home = home.clone();
    let purge = std::thread::spawn(move || selfmgmt::uninstall(&purge_home, true, true));
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        let _ = purge.join();
        panic!("purge 未到达最终复核同步点");
    }
    std::fs::remove_file(home.lock_path().as_path()).unwrap();
    std::fs::write(&release, b"release").unwrap();
    let result = purge.join().unwrap();
    assert!(matches!(result, Err(Error::Io { .. })));
    assert!(!home.lock_path().as_path().exists(), "不得在错误锁下重建锁");
    assert!(home.root().as_path().exists());
}

// Task: T20
#[test]
fn install_replaces_divergent_binary_instead_of_short_circuit() {
    let (_d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    std::fs::write(&bin, "被换掉的旧文件").unwrap();
    let out = selfmgmt::install(&home).unwrap();
    // 幂等短路只认「字节相同」；分叉了的二进制必须重装回当前可执行文件。
    assert!(!out.already_installed, "分叉后仍报 already_installed");
    let current = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    assert_eq!(std::fs::read(&bin).unwrap(), current);
}

// Task: T20
#[test]
fn update_version_flag_mismatch_is_unavailable() {
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
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
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    let platform = selfmgmt::platform();
    let payload = format!("tarred sheltie for {platform}");
    use std::os::unix::fs::PermissionsExt as _;
    // cargo-dist 0.32 的真实包内布局（T25 用 dist build 的产出核过）：
    // <产物名去掉扩展>/sheltie，二进制在内层目录根部。
    let asset = format!("sheltie-cli-9.9.9-{platform}.tar.gz");
    let inner = format!("sheltie-cli-9.9.9-{platform}");
    // 打包在暂存目录里做，产物放进固定 tag 的目录（存储合同 §9 的 tag 布局）。
    let stage = d.path().join("stage");
    std::fs::create_dir_all(stage.join(&inner)).unwrap();
    std::fs::write(stage.join(&inner).join("sheltie"), &payload).unwrap();
    std::fs::set_permissions(
        stage.join(&inner).join("sheltie"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let tar = std::process::Command::new("tar")
        .args(["-czf", &asset, &inner])
        .current_dir(&stage)
        .output()
        .unwrap();
    assert!(
        tar.status.success(),
        "tar 打包失败：{}",
        String::from_utf8_lossy(&tar.stderr)
    );
    let bytes = std::fs::read(stage.join(&asset)).unwrap();
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(&bytes);
    let manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    write_tag_dir(&dir, "latest", &manifest, &[]);
    write_tag_dir(&dir, "v9.9.9", &manifest, &[(&asset, &bytes)]);
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
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    let platform = selfmgmt::platform();
    let payload = format!("cargo-dist style sheltie for {platform}");
    let asset = format!("sheltie-cli-7.7.7-{platform}");
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
    })
    .to_string();
    // `latest/` 学出 7.7.7 后，清单与资产都从 `v7.7.7/` 重取（固定 tag 不混两次解析）。
    write_tag_dir(&dir, "latest", &manifest, &[]);
    write_tag_dir(&dir, "v7.7.7", &manifest, &[(&asset, payload.as_bytes())]);
    let src = ReleaseSource {
        base: dir.to_str().unwrap().to_string(),
    };
    let out = selfmgmt::update(&home, &src, None).unwrap();
    assert_eq!(out.to, "7.7.7");
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    assert_eq!(std::fs::read(&bin).unwrap(), payload.as_bytes());
}

// Task: T20
#[test]
fn update_unpacks_tgz_named_asset() {
    // 覆盖后缀判定的第三个区段：.tgz 与 .tar.gz、.tar.xz 是并列写法，
    // 只测 .tar.gz 时「|| 换 &&」的突变体测不出来。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    let platform = selfmgmt::platform();
    let payload = format!("tgz sheltie for {platform}");
    use std::os::unix::fs::PermissionsExt as _;
    let stage = d.path().join("stage");
    std::fs::create_dir_all(stage.join("sheltie/bin")).unwrap();
    std::fs::write(stage.join("sheltie/bin/sheltie"), &payload).unwrap();
    std::fs::set_permissions(
        stage.join("sheltie/bin/sheltie"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let asset = format!("sheltie-9.9.9-{platform}.tgz");
    let tar = std::process::Command::new("tar")
        .args(["-czf", &asset, "sheltie"])
        .current_dir(&stage)
        .output()
        .unwrap();
    assert!(
        tar.status.success(),
        "tar 打包失败：{}",
        String::from_utf8_lossy(&tar.stderr)
    );
    let bytes = std::fs::read(stage.join(&asset)).unwrap();
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(&bytes);
    let manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    write_tag_dir(&dir, "latest", &manifest, &[]);
    write_tag_dir(&dir, "v9.9.9", &manifest, &[(&asset, &bytes)]);
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

// ── C002-T15：固定 tag、失败窗口、并发与发布形状 ────────────────────────

/// 往发布目录写一个版本的 `v<version>/` tag 目录（不动 `latest/`）。
/// 漂移反例正是靠 `latest/` 与固定 tag 不一致做出来的。
fn write_version_release(dir: &Path, version: &str, payload: &[u8]) {
    let platform = selfmgmt::platform();
    let asset = format!("sheltie-{version}-{platform}");
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(payload);
    let manifest = serde_json::json!({
        "version": version,
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    write_tag_dir(dir, &format!("v{version}"), &manifest, &[(&asset, payload)]);
}

fn release_source(dir: &Path) -> ReleaseSource {
    ReleaseSource {
        base: dir.to_str().unwrap().to_string(),
    }
}

fn bin_path(home: &sheltie_runtime::Home) -> std::path::PathBuf {
    std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie")
}

/// tmp/ 在每个失败窗口后都不能留半成品（存储合同 §9 步 3）。
fn assert_tmp_clean(home: &sheltie_runtime::Home) {
    let tmp = std::path::PathBuf::from(home.tmp_dir().as_str());
    assert!(
        !tmp.exists() || std::fs::read_dir(&tmp).unwrap().next().is_none(),
        "tmp/ 留了半成品"
    );
}

/// 失败窗口不许建 `sheltie.prev`（只有替换步才动它，存储合同 §9 步 4）。
fn assert_no_prev(home: &sheltie_runtime::Home) {
    assert!(
        !std::path::PathBuf::from(home.bin_dir().as_str())
            .join("sheltie.prev")
            .exists(),
        "失败窗口不该出现 sheltie.prev"
    );
}

// Task: C002-T15
#[test]
fn update_pinned_version_installs_only_that_tag() {
    // `--version 8.8.8` 一次固定到 v8.8.8；latest 指着 9.9.9 也不看（存储合同 §9 步 1）。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9", false);
    let eight = "payload of pinned 8.8.8";
    write_version_release(&dir, "8.8.8", eight.as_bytes());
    let out = selfmgmt::update(&home, &release_source(&dir), Some("8.8.8")).unwrap();
    assert_eq!(out.to, "8.8.8");
    assert_eq!(
        std::fs::read_to_string(bin_path(&home)).unwrap(),
        eight,
        "装上的不是 v8.8.8 的包"
    );
}

// Task: C002-T15
#[test]
fn update_latest_drift_does_not_mix_manifest_and_assets() {
    // `latest/` 是自洽的毒发布（清单与包都是毒字节），`v9.9.9/` 是正确的。
    // 版本号只从 latest 学一次，清单与资产都从固定后的 tag 取：
    // 混用两次解析，要么装上毒包，要么被自己那份摘要卡住。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let asset = format!("sheltie-9.9.9-{platform}");
    let good = "correct payload from pinned tag";
    let poisoned = "poisoned payload from drifted latest";
    let bad_digest = sheltie_core::digest::Sha256Hex::of_bytes(poisoned.as_bytes());
    let bad_manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": bad_digest.as_str() }]
    })
    .to_string();
    write_tag_dir(
        &dir,
        "latest",
        &bad_manifest,
        &[(&asset, poisoned.as_bytes())],
    );
    write_version_release(&dir, "9.9.9", good.as_bytes());
    let out = selfmgmt::update(&home, &release_source(&dir), None).unwrap();
    assert_eq!(out.to, "9.9.9");
    let got = std::fs::read(bin_path(&home)).unwrap();
    assert_eq!(got, good.as_bytes(), "装上了漂移 latest 的包");
    assert_ne!(got, poisoned.as_bytes());
}

// Task: C002-T15
#[test]
fn update_rejects_forged_asset_name() {
    // 伪造清单的资产名带路径段：`{tag}/{name}` 能走出发布目录。
    // 逐个单条件改名字，必须在下载前拒绝；根外哨兵与旧二进制都不许动。
    let (_d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let outside = tempfile::tempdir().unwrap();
    let secret = outside.path().join("secret");
    std::fs::write(&secret, b"outside secret bytes").unwrap();
    // 发布目录挨着哨兵；`../../secret` 从 v9.9.9/ 出去正好读到它。
    let dir = outside.path().join("rel");
    std::fs::create_dir_all(dir.join("v9.9.9")).unwrap();
    let platform = selfmgmt::platform();
    for forged in ["../../secret", "sub/secret", "..\\secret", "x\0y"] {
        let digest = sheltie_core::digest::Sha256Hex::of_bytes(b"never used");
        let manifest = serde_json::json!({
            "version": "9.9.9",
            "assets": [{ "platform": platform, "name": forged, "sha256": digest.as_str() }]
        })
        .to_string();
        write_tag_dir(&dir, "v9.9.9", &manifest, &[]);
        assert!(
            matches!(
                selfmgmt::update(&home, &release_source(&dir), Some("9.9.9")),
                Err(Error::UpdateUnavailable { .. })
            ),
            "伪造资产名 {forged:?} 竟然被接受"
        );
    }
    assert_eq!(
        std::fs::read(&secret).unwrap(),
        b"outside secret bytes",
        "根外哨兵被动了"
    );
    assert_eq!(
        std::fs::read(bin_path(&home)).unwrap(),
        before,
        "旧二进制被换掉"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn update_rejects_path_like_version_argument() {
    // `--version` 同样是路径段来源：空、`.`、`..`、分隔符与 NUL 一律拒绝
    // （与 `checked_asset_name` 同一套拒法）。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9", false);
    let src = release_source(&dir);
    for bad in ["..", "../9.9.9", "a/b", "a\\b", "", ".", "v", "x\0y"] {
        assert!(
            matches!(
                selfmgmt::update(&home, &src, Some(bad)),
                Err(Error::UpdateUnavailable { .. })
            ),
            "版本 {bad:?} 竟然被接受"
        );
    }
}

// Task: C002-T15
#[test]
fn update_missing_tag_directory_reports_missing_tag() {
    // 只有 latest/ 与 v9.9.9/：固定到 v5.5.5 后清单取不到，
    // 诊断要点名缺的是哪个 tag，二进制不动、tmp 不留。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9", false);
    match selfmgmt::update(&home, &release_source(&dir), Some("5.5.5")) {
        Err(Error::UpdateUnavailable { reason }) => {
            assert!(reason.contains("v5.5.5"), "诊断没点名缺的 tag：{reason}");
        }
        other => panic!("缺 tag 目录竟然成功：{other:?}"),
    }
    assert_eq!(
        std::fs::read(bin_path(&home)).unwrap(),
        before,
        "失败窗口换了二进制"
    );
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn update_rejects_manifest_version_not_matching_tag() {
    // v9.9.9/ 的清单写着 9.9.8：固定 tag 与清单身份不符，拒绝而不是装错版本。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let payload = "mismatched manifest payload";
    let asset = format!("sheltie-9.9.9-{platform}");
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes());
    let manifest = serde_json::json!({
        "version": "9.9.8",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    write_tag_dir(&dir, "v9.9.9", &manifest, &[(&asset, payload.as_bytes())]);
    match selfmgmt::update(&home, &release_source(&dir), Some("9.9.9")) {
        Err(Error::UpdateUnavailable { reason }) => {
            assert!(reason.contains("与 tag 不符"), "{reason}");
        }
        other => panic!("身份不符竟然被接受：{other:?}"),
    }
}

// Task: C002-T15
#[test]
fn update_failed_download_leaves_old_binary_and_cleans_tmp() {
    // 清单声明的资产在 tag 目录里不存在：下载失败窗口——旧二进制不变，tmp 清空。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let asset = format!("sheltie-9.9.9-{platform}");
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(b"never downloaded");
    let manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    write_tag_dir(&dir, "latest", &manifest, &[]);
    write_tag_dir(&dir, "v9.9.9", &manifest, &[]);
    assert!(matches!(
        selfmgmt::update(&home, &release_source(&dir), None),
        Err(Error::UpdateUnavailable { .. })
    ));
    assert_eq!(
        std::fs::read(bin_path(&home)).unwrap(),
        before,
        "旧二进制被换掉"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn update_failed_digest_leaves_old_binary_and_cleans_tmp() {
    // 资产字节与清单摘要不符：摘要失败窗口——旧二进制不变，tmp 清空（存储合同 §9 步 3）。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let asset = format!("sheltie-9.9.9-{platform}");
    // 只改一个条件：资产本体下载得到，清单记的摘要是别的字节的哈希。
    let payload = b"asset bytes that do not match the manifest digest";
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(b"a different payload entirely");
    let manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    write_tag_dir(&dir, "latest", &manifest, &[]);
    write_tag_dir(&dir, "v9.9.9", &manifest, &[(&asset, payload)]);
    assert!(matches!(
        selfmgmt::update(&home, &release_source(&dir), None),
        Err(Error::UpdateChecksumMismatch { .. })
    ));
    assert_eq!(
        std::fs::read(bin_path(&home)).unwrap(),
        before,
        "旧二进制被换掉"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn update_failed_extract_leaves_old_binary_and_cleans_tmp() {
    // 资产是坏压缩包（摘要自洽）：解包失败窗口——旧二进制不变，tmp 清空。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let asset = format!("sheltie-9.9.9-{platform}.tar.gz");
    let garbage = b"this is not a tar archive";
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(garbage);
    let manifest = serde_json::json!({
        "version": "9.9.9",
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    write_tag_dir(&dir, "latest", &manifest, &[]);
    write_tag_dir(&dir, "v9.9.9", &manifest, &[(&asset, garbage)]);
    assert!(matches!(
        selfmgmt::update(&home, &release_source(&dir), None),
        Err(Error::UpdateUnavailable { .. })
    ));
    assert_eq!(
        std::fs::read(bin_path(&home)).unwrap(),
        before,
        "旧二进制被换掉"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn clean_home_install_then_update_two_step() {
    // 全新管理根的两步生命周期：install 建根建库建 bin，update 换二进制留 .prev。
    let (d, home) = temp_home();
    let first = selfmgmt::install(&home).unwrap();
    assert!(!first.already_installed);
    assert!(
        std::path::Path::new(home.store_path().as_str()).exists(),
        "install 没建库"
    );
    let original = std::fs::read(bin_path(&home)).unwrap();
    let store_before = std::fs::read(home.store_path().as_str()).unwrap();
    let src = make_release(&d.path().join("rel"), "9.9.9", false);
    let out = selfmgmt::update(&home, &src, None).unwrap();
    assert_eq!(out.to, "9.9.9");
    assert_eq!(
        std::fs::read(home.store_path().as_str()).unwrap(),
        store_before,
        "update 动了 store.db（存储合同 §9：update 不碰库）"
    );
    assert_eq!(
        std::fs::read(std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie.prev"))
            .unwrap(),
        original,
        ".prev 不是更新前的二进制"
    );
    assert!(
        std::fs::read_to_string(bin_path(&home))
            .unwrap()
            .starts_with("fake sheltie 9.9.9")
    );
}

// Task: C002-T15
#[test]
fn update_pinned_version_then_rollback_restores_previous() {
    // 指定版本更新后 rollback 换回旧二进制（本地 release fixture，存储合同 §9）。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let original = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9", false);
    let eight = "payload of pinned 8.8.8";
    write_version_release(&dir, "8.8.8", eight.as_bytes());
    let store_before = std::fs::read(home.store_path().as_str()).unwrap();
    selfmgmt::update(&home, &release_source(&dir), Some("8.8.8")).unwrap();
    assert_eq!(std::fs::read_to_string(bin_path(&home)).unwrap(), eight);
    assert_eq!(
        std::fs::read(home.store_path().as_str()).unwrap(),
        store_before,
        "update 动了 store.db（存储合同 §9：update 不碰库）"
    );
    selfmgmt::rollback(&home).unwrap();
    assert_eq!(std::fs::read(bin_path(&home)).unwrap(), original);
    assert_eq!(
        std::fs::read(home.store_path().as_str()).unwrap(),
        store_before,
        "rollback 动了 store.db（只换回旧二进制，不降级 Store）"
    );
    assert!(
        !std::path::PathBuf::from(home.bin_dir().as_str())
            .join("sheltie.prev")
            .exists(),
        "rollback 后仍留着 .prev"
    );
}

// Task: C002-T15
#[test]
fn update_adapts_cargo_dist_plan_real_manifest_shape() {
    // 形状照抄 cargo-dist 0.32.0 `dist plan --output-format=json` 的真实输出
    // （evidence/t15/dist-plan.json；包内字节与哈希是合成的）：
    // artifacts 按产物名索引；`checksum` 是校验文件名，真哈希在 `checksums.sha256`；
    // 已建产物（本机 tarball 与 source.tar.gz）带 `checksums`，未建的 executable-zip
    // **缺 `checksums` 键**（不是 null），适配器必须跳过而不是拿文件名当哈希；
    // 顶层 `assets` 是 exe 条目字典（无 `kind`、name 是 `sheltie`），不能当发布资产取
    // ——tag 目录里另放一份同名毒字节，谁取谁翻车。
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let payload = format!("dist-plan shaped sheltie for {platform}");
    let asset = format!("sheltie-cli-7.7.7-{platform}.tar.xz");
    // 真实产物名以 .tar.xz 结尾，资产就得是真压缩包（包内布局同 T25 实测）。
    let inner = format!("sheltie-cli-7.7.7-{platform}");
    let stage = d.path().join("stage");
    std::fs::create_dir_all(stage.join(&inner)).unwrap();
    std::fs::write(stage.join(&inner).join("sheltie"), &payload).unwrap();
    let tar = std::process::Command::new("tar")
        .args(["-cJf", &asset, &inner])
        .current_dir(&stage)
        .output()
        .unwrap();
    assert!(
        tar.status.success(),
        "tar 打包失败：{}",
        String::from_utf8_lossy(&tar.stderr)
    );
    let bytes = std::fs::read(stage.join(&asset)).unwrap();
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(&bytes);
    let manifest = serde_json::json!({
        "dist_version": "0.32.0",
        "announcement_tag": "v7.7.7",
        "announcement_tag_is_implicit": true,
        "announcement_is_prerelease": false,
        "releases": [{
            "app_name": "sheltie-cli",
            "app_version": "7.7.7",
            "artifacts": [asset.clone(), "source.tar.gz"],
        }],
        "artifacts": {
            "sha256.sum": {
                "name": "sha256.sum",
                "kind": "unified-checksum",
                "path": "target/distrib/sha256.sum",
            },
            asset.clone(): {
                "name": asset.clone(),
                "kind": "executable-zip",
                "target_triples": [platform.clone()],
                "assets": [format!("sheltie-cli-7.7.7-{platform}-exe-sheltie")],
                "checksum": format!("{asset}.sha256"),
                "checksums": { "sha256": digest.as_str() },
                "path": format!("target/distrib/{asset}"),
            },
            // 真 plan 里未建的 executable-zip 直接没有 `checksums` 键（不是 null）。
            "sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz": {
                "name": "sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz",
                "kind": "executable-zip",
                "target_triples": ["x86_64-unknown-linux-gnu"],
                "assets": ["sheltie-cli-7.7.7-x86_64-unknown-linux-gnu-exe-sheltie"],
                "checksum": "sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz.sha256",
                "path": "target/distrib/sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz",
            },
            // source.tar.gz 同样是已建产物、同样带哈希：kind 不是 executable-zip 就得跳过。
            "source.tar.gz": {
                "name": "source.tar.gz",
                "kind": "source-tarball",
                "checksum": "source.tar.gz.sha256",
                "checksums": {
                    "sha256": "d48e2f60de052e2948e7c3bdd0fd0db20ff32176e7d2b49a256f2047808a1fee"
                },
                "path": "target/distrib/source.tar.gz",
            },
            "sheltie-cli-installer.sh": {
                "name": "sheltie-cli-installer.sh",
                "kind": "installer",
                "target_triples": [platform.clone()],
                "description": "Installer script",
                "install_hint": "curl --proto '=https' --tlsv1.2 -LsSf …/sheltie-cli-installer.sh | sh",
                "path": "target/distrib/sheltie-cli-installer.sh",
            },
        },
        // 顶层 `assets` 照真 plan 的键集：无 `kind`，name 是 `sheltie`。
        "assets": {
            "sheltie-cli-7.7.7-aarch64-apple-darwin-exe-sheltie": {
                "id": format!("sheltie-cli-7.7.7-{platform}-exe-sheltie"),
                "name": "sheltie",
                "system": "build:host:",
                "target_triples": [platform.clone()],
                "linkage": { "system": [] },
            }
        },
    })
    .to_string();
    // 同名毒字节放进 tag 目录：谁把顶层 assets 当发布资产取，要么报错要么装上它，
    // 断言都会翻。
    let poison = b"poison bytes from the top-level assets entry";
    write_tag_dir(&dir, "latest", &manifest, &[]);
    write_tag_dir(
        &dir,
        "v7.7.7",
        &manifest,
        &[(&asset, &bytes), ("sheltie", poison)],
    );
    let out = selfmgmt::update(&home, &release_source(&dir), None).unwrap();
    assert_eq!(out.to, "7.7.7");
    assert_eq!(
        std::fs::read_to_string(bin_path(&home)).unwrap(),
        payload,
        "装上的不是平台 tarball 里的二进制（顶层 assets 被当成发布资产了？）"
    );
}

// Task: C002-T15
#[test]
fn home_lock_identity_detects_replaced_lock_file() {
    // §2.2 复核：`.lock` 路径换成另一个对象（同名新文件）后身份不再连续。
    let (_d, home) = temp_home();
    let guard = home.acquire_lock().unwrap();
    assert!(guard.identity_still_valid());
    std::fs::remove_file(home.lock_path().as_str()).unwrap();
    std::fs::write(home.lock_path().as_str(), b"replaced").unwrap();
    assert!(!guard.identity_still_valid());
}

// Task: C002-T15
#[test]
fn home_lock_identity_detects_replaced_root() {
    // §2.2 复核：管理根被 purge 掉又重建，即使 `.lock` 又出现，dev/inode 也换了。
    let (_d, home) = temp_home();
    let guard = home.acquire_lock().unwrap();
    assert!(guard.identity_still_valid());
    // 该用例模拟同用户锁外替换，测试夹具清理由std处理；产品路径删除走runtime句柄API。
    std::fs::remove_dir_all(home.root().as_path()).unwrap();
    std::fs::create_dir_all(home.root().as_str()).unwrap();
    std::fs::write(home.lock_path().as_str(), b"replaced").unwrap();
    assert!(!guard.identity_still_valid());
}

// Task: C002-T15
#[test]
fn self_install_and_work_writes_serialize_under_home_lock() {
    // self 写动词与 Work 写动词共用管理根写锁（§2.2）：并发交错跑完后，
    // 库可读、Work 状态完整、二进制在位，没有写坏的半状态。
    let (_d, home, svc) = home_with_example("two-step");
    let home_b = home.clone();
    let installs = std::thread::spawn(move || {
        for _ in 0..8 {
            selfmgmt::install(&home_b).unwrap();
        }
    });
    // 每轮新建一个 Work 再 begin（都是合法写），不复用同一 Attempt。
    let works = std::thread::spawn(move || {
        let mut ids = Vec::new();
        for _ in 0..8 {
            let wid = work_id_of(&start_two_step(&svc));
            svc.begin(
                &wid,
                &sheltie_core::ids::NodeId::new("outline").unwrap(),
                None,
            )
            .unwrap();
            ids.push(wid);
        }
        ids
    });
    installs.join().unwrap();
    let started = works.join().unwrap();
    assert!(bin_path(&home).exists());
    assert!(std::path::PathBuf::from(home.store_path().as_str()).exists());
    for wid in &started {
        let (_, json) = service(&home).status(wid).unwrap();
        assert!(json.last_attempt.is_some(), "Work {} 状态被并发写坏", wid);
    }
}

// Task: C002-T31
#[cfg(feature = "failpoint")]
#[test]
fn purge_late_sqlite_control_files_accept_only_empty_single_link_wal_or_safe_shm() {
    use std::time::{Duration, Instant};
    for kind in [
        "empty-wal",
        "nonempty-wal",
        "linked-wal",
        "symlink-wal",
        "regular-shm",
        "linked-shm",
        "symlink-shm",
        "directory-shm",
    ] {
        let (directory, home) = temp_home();
        repo(&home)
            .add(&abs(&example_dir("two-step")), None)
            .unwrap();
        let point = tempfile::tempdir().unwrap();
        sheltie_runtime::failpoint::arm_rendezvous(
            "purge_before_final_rescan",
            home.root().as_str(),
            point.path(),
        )
        .unwrap();
        struct Guard {
            release: std::path::PathBuf,
            thread: Option<std::thread::JoinHandle<sheltie_runtime::Result<()>>>,
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                let _ = std::fs::write(&self.release, b"release");
                if let Some(thread) = self.thread.take() {
                    let _ = thread.join();
                }
                let _ = sheltie_runtime::failpoint::disarm_rendezvous();
            }
        }
        let worker_home = home.clone();
        let worker =
            std::thread::spawn(move || selfmgmt::uninstall(&worker_home, true, true).map(|_| ()));
        let mut guard = Guard {
            release: point.path().join("release"),
            thread: Some(worker),
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        while !point.path().join("reached").exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(!home.store_path().as_path().exists());
        let outside = tempfile::tempdir().unwrap();
        let sentinel = outside.path().join("sentinel");
        std::fs::write(&sentinel, b"").unwrap();
        let wal = directory.path().join(if kind.ends_with("shm") {
            "store.db-shm"
        } else {
            "store.db-wal"
        });
        match kind {
            "empty-wal" => std::fs::write(&wal, b"").unwrap(),
            "regular-shm" => std::fs::write(&wal, b"SQLite shared index").unwrap(),
            "directory-shm" => std::fs::create_dir(&wal).unwrap(),
            "nonempty-wal" => std::fs::write(&wal, b"preserve late WAL records").unwrap(),
            "linked-wal" | "linked-shm" => std::fs::hard_link(&sentinel, &wal).unwrap(),
            _ => std::os::unix::fs::symlink(&sentinel, &wal).unwrap(),
        }
        std::fs::write(&guard.release, b"release").unwrap();
        let result = guard.thread.take().unwrap().join().unwrap();
        if matches!(kind, "empty-wal" | "regular-shm") {
            result.unwrap();
            assert!(!wal.exists());
        } else {
            assert!(result.is_err());
            assert!(std::fs::symlink_metadata(&wal).is_ok());
            if kind == "nonempty-wal" {
                assert_eq!(std::fs::read(&wal).unwrap(), b"preserve late WAL records");
            }
        }
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"");
    }
}
