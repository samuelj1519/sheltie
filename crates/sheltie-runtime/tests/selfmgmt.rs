//! T20: runtime binary self-management, using local release directories without network access.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_runtime::Error;
use sheltie_runtime::selfmgmt::{self, ReleaseSource};

fn pack_tar(stage: &Path, asset: &str, entry: &str, flag: &str) -> Vec<u8> {
    let packed = std::process::Command::new("tar")
        .args([flag, asset, entry])
        .current_dir(stage)
        .output()
        .unwrap();
    assert!(
        packed.status.success(),
        "tar packaging failed: {}",
        String::from_utf8_lossy(&packed.stderr)
    );
    std::fs::read(stage.join(asset)).unwrap()
}

/// Write a tag directory: <base>/<tag>/dist-manifest.json and its assets.
/// Local releases mirror remote tag layout (storage §9); latest/ is mutable,
/// while pinned manifest/assets live beneath v<version>/.
fn write_tag_dir(base: &Path, tag: &str, manifest: &str, assets: &[(&str, &[u8])]) {
    let dir = base.join(tag);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("dist-manifest.json"), manifest).unwrap();
    for (name, bytes) in assets {
        std::fs::write(dir.join(name), bytes).unwrap();
    }
}

fn make_release(dir: &Path, version: &str) -> ReleaseSource {
    std::fs::create_dir_all(dir).unwrap();
    let platform = selfmgmt::platform();
    let payload = format!("fake sheltie {version} for {platform}");
    let asset = format!("sheltie-{version}-{platform}");
    let digest = sheltie_core::digest::Sha256Hex::of_bytes(payload.as_bytes());
    let manifest = serde_json::json!({
        "version": version,
        "assets": [{ "platform": platform, "name": asset, "sha256": digest.as_str() }]
    })
    .to_string();
    // Use latest/ only for version discovery; fetch manifest/assets from pinned v<version>/.
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
    assert_eq!(first.installed_to, home.bin_dir().join_segment("sheltie"));
    assert_eq!(
        std::fs::read(first.installed_to.as_path()).unwrap(),
        std::fs::read(std::env::current_exe().unwrap()).unwrap()
    );
    assert!(home.store_path().as_path().exists());
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

// Task: C002-T23
#[cfg(feature = "failpoint")]
#[test]
fn update_stops_if_verified_candidate_bytes_change_before_any_replacement() {
    let _serial = PURGE_PARTIAL_LOCK.lock().unwrap();
    let (dir, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let prev = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie.prev");
    std::fs::write(&prev, b"older previous binary").unwrap();
    let prev_before = std::fs::read(&prev).unwrap();
    let source = make_release(&dir.path().join("release"), "9.9.9");
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "update_after_candidate_verify",
        home.root().as_str(),
        rendezvous.path(),
    )
    .unwrap();
    let update_home = home.clone();
    let update = std::thread::spawn(move || selfmgmt::update(&update_home, &source, None));
    let release = rendezvous.path().join("release");
    let mut worker = RendezvousWorker::single(update, rendezvous.path());
    worker.wait("update did not reach the synchronization point after candidate verification");
    let tmp = std::fs::read_dir(home.tmp_dir().as_path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(tmp.join("sheltie-candidate"), b"unverified replacement").unwrap();
    std::fs::write(&release, b"release").unwrap();
    assert!(matches!(
        worker.finish().unwrap(),
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
    let src = make_release(&d.path().join("rel"), "9.9.9");
    selfmgmt::update(&home, &src, None).unwrap();
    selfmgmt::rollback(&home).unwrap();
    assert_eq!(
        std::fs::read(std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie")).unwrap(),
        original
    );
    assert!(
        matches!(selfmgmt::rollback(&home), Err(Error::NotFound { .. })),
        "Retain only one previous version"
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
    assert!(error.to_string().contains("symlink"), "{error:?}");
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
    assert!(error.to_string().contains("symlink"), "{error:?}");
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
        "tar packaging failed: {}",
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
    assert!(
        error.to_string().contains("links or special objects"),
        "{error:?}"
    );
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
    let purge_home = home.clone();
    let purge = std::thread::spawn(move || selfmgmt::uninstall(&purge_home, true, true));
    let release = rendezvous.path().join("release");
    let mut worker = RendezvousWorker::single(purge, rendezvous.path());
    worker.wait(
        "purge did not reach the synchronization point after removing the first root directory",
    );
    let blocked = home.work_dir(&wid).join_segment("purge-special");
    let fifo = std::process::Command::new("mkfifo")
        .arg(blocked.as_str())
        .status()
        .unwrap();
    assert!(fifo.success());
    std::fs::write(&release, b"release").unwrap();
    let result = worker.finish().unwrap();

    match result {
        Err(Error::Io { path, source }) => {
            assert!(
                path.contains("works"),
                "Failure location must be explicit: {path}"
            );
            let detail = source.to_string();
            assert!(detail.contains("may be partially cleared"), "{detail}");
            assert!(
                detail.contains("workbooks"),
                "Completed top-level items must be explicit: {detail}"
            );
        }
        other => panic!("Partial purge caused by a special file was not reported: {other:?}"),
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

// ── M3 review additions: release chain and install semantics, owned by T20 ─────────────────────

// Task: T20
#[test]
fn platform_matches_supported_target_triples() {
    // Artifacts use Rust target triples (storage §9); an incorrect platform name cannot locate them.
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
    let purge_home = home.clone();
    let purge = std::thread::spawn(move || selfmgmt::uninstall(&purge_home, true, true));
    let release = rendezvous.path().join("release");
    let mut worker = RendezvousWorker::single(purge, rendezvous.path());
    worker.wait(
        "purge did not reach the synchronization point before final sqlite control-file scanning",
    );
    std::fs::write(
        home.root().join_segment("store.db-shm").as_path(),
        b"late shm",
    )
    .unwrap();
    std::fs::write(&release, b"release").unwrap();
    let kept = worker.finish().unwrap().unwrap();
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
    let purge_home = home.clone();
    let purge = std::thread::spawn(move || selfmgmt::uninstall(&purge_home, true, true));
    let release = rendezvous.path().join("release");
    let mut worker = RendezvousWorker::single(purge, rendezvous.path());
    worker.wait("purge did not reach the synchronization point before final scanning");
    std::fs::remove_file(home.lock_path().as_path()).unwrap();
    std::fs::write(&release, b"release").unwrap();
    let result = worker.finish().unwrap();
    assert!(matches!(result, Err(Error::Io { .. })));
    assert!(
        !home.lock_path().as_path().exists(),
        "Must not recreate a lock under an incorrect lock"
    );
    assert!(home.root().as_path().exists());
}

// Task: T20
#[test]
fn install_replaces_divergent_binary_instead_of_short_circuit() {
    let (_d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let bin = std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie");
    std::fs::write(&bin, "Replaced old file").unwrap();
    let out = selfmgmt::install(&home).unwrap();
    // Idempotency requires identical bytes; restore divergent binaries from the running executable.
    assert!(
        !out.already_installed,
        "Divergent binary still reported already_installed"
    );
    let current = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    assert_eq!(std::fs::read(&bin).unwrap(), current);
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
    // Actual cargo-dist 0.32 archive layout, checked against T25 dist build:
    // <artifact name without extension>/sheltie, binary at the nested directory root.
    let asset = format!("sheltie-cli-9.9.9-{platform}.tar.gz");
    let inner = format!("sheltie-cli-9.9.9-{platform}");
    // Package in staging, placing artifacts beneath the pinned tag (storage §9).
    let stage = d.path().join("stage");
    std::fs::create_dir_all(stage.join(&inner)).unwrap();
    std::fs::write(stage.join(&inner).join("sheltie"), &payload).unwrap();
    std::fs::set_permissions(
        stage.join(&inner).join("sheltie"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let bytes = pack_tar(&stage, &asset, &inner, "-czf");
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
        "Extracted bin/sheltie is not executable (mode {mode:o})"
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
    // Actual full dist-manifest.json shape (0.32.0), checked against T25 dist build:
    // artifacts is a name-keyed object; checksum is the checksum filename, and checksums.sha256 is the real digest.
    // Skip non-executable-zip artifacts and other platforms.
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
    // After latest/ discovers 7.7.7, fetch manifest/assets again from v7.7.7/, pinning one identity.
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
    // Cover the third suffix branch: .tgz alongside .tar.gz and .tar.xz;
    // .tar.gz alone cannot distinguish the ||-to-&& mutant.
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
    let bytes = pack_tar(&stage, &asset, "sheltie", "-czf");
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

// ── C002-T15: pinned tags, failure windows, concurrency, and release shapes ────────────────────────

/// Write one v<version>/ tag directory without changing latest/.
/// Create drift rejection cases by making latest/ differ from its pinned tag.
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

/// No incomplete tmp/ artifacts after any failure window (storage §9, step 3).
fn assert_tmp_clean(home: &sheltie_runtime::Home) {
    let tmp = std::path::PathBuf::from(home.tmp_dir().as_str());
    assert!(
        !tmp.exists() || std::fs::read_dir(&tmp).unwrap().next().is_none(),
        "Incomplete artifacts remain in tmp/"
    );
}

/// Failure windows must not create sheltie.prev; only replacement changes it (storage §9, step 4).
fn assert_no_prev(home: &sheltie_runtime::Home) {
    assert!(
        !std::path::PathBuf::from(home.bin_dir().as_str())
            .join("sheltie.prev")
            .exists(),
        "sheltie.prev must not appear in this failure window"
    );
}

// Task: C002-T15
#[test]
fn update_pinned_version_installs_only_that_tag() {
    // --version 8.8.8 pins v8.8.8 without reading latest, even if latest points to 9.9.9 (storage §9, step 1).
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9");
    let eight = "payload of pinned 8.8.8";
    write_version_release(&dir, "8.8.8", eight.as_bytes());
    let out = selfmgmt::update(&home, &release_source(&dir), Some("8.8.8")).unwrap();
    assert_eq!(out.to, "8.8.8");
    assert_eq!(
        std::fs::read_to_string(bin_path(&home)).unwrap(),
        eight,
        "Installed artifact is not from v8.8.8"
    );
}

// Task: C002-T15
#[test]
fn update_latest_drift_does_not_mix_manifest_and_assets() {
    // latest/ has a self-consistent poisoned manifest/artifact; v9.9.9/ is correct.
    // Discover the version once from latest, then fetch manifest/assets from the pinned tag;
    // inconsistent resolution either installs poison or fails the digest check.
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
    assert_eq!(
        got,
        good.as_bytes(),
        "Installed the drifting latest artifact"
    );
    assert_ne!(got, poisoned.as_bytes());
}

// Task: C002-T15
#[test]
fn update_rejects_forged_asset_name() {
    // Forged manifest asset names contain path segments allowing {tag}/{name} to escape the release directory.
    // Change one filename condition each time; reject before download, preserving the external sentinel and old binary.
    let (_d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let outside = tempfile::tempdir().unwrap();
    let secret = outside.path().join("secret");
    std::fs::write(&secret, b"outside secret bytes").unwrap();
    // Release directory is beside the sentinel; ../../secret from v9.9.9/ would read it.
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
            "Forged asset name {forged:?} was accepted"
        );
    }
    assert_eq!(
        std::fs::read(&secret).unwrap(),
        b"outside secret bytes",
        "External sentinel changed"
    );
    assert_eq!(
        std::fs::read(bin_path(&home)).unwrap(),
        before,
        "Old binary was replaced"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn update_rejects_path_like_version_argument() {
    // --version also forms a path segment; reject empty, ., .., separators, and NUL
    // with the same rules as checked_asset_name.
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9");
    let src = release_source(&dir);
    for bad in ["..", "../9.9.9", "a/b", "a\\b", "", ".", "v", "x\0y"] {
        assert!(
            matches!(
                selfmgmt::update(&home, &src, Some(bad)),
                Err(Error::UpdateUnavailable { .. })
            ),
            "Invalid version {bad:?} was accepted"
        );
    }
}

// Task: C002-T15
#[test]
fn update_missing_tag_preserves_binary_and_allows_pinned_retry() {
    // Only latest/ and v9.9.9/ exist; pinning v5.5.5 cannot locate its manifest,
    // so name the missing tag, preserve the binary, and clear tmp.
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9");
    match selfmgmt::update(&home, &release_source(&dir), Some("5.5.5")) {
        Err(Error::UpdateUnavailable { reason }) => {
            assert!(
                reason.contains("v5.5.5"),
                "Diagnostic did not name the missing tag: {reason}"
            );
        }
        other => panic!("Missing tag directory unexpectedly succeeded: {other:?}"),
    }
    assert_eq!(
        std::fs::read(bin_path(&home)).unwrap(),
        before,
        "Failure window replaced the binary"
    );
    assert_tmp_clean(&home);
    let out = selfmgmt::update(&home, &release_source(&dir), Some("9.9.9")).unwrap();
    assert_eq!(out.to, "9.9.9");
    use std::os::unix::fs::PermissionsExt as _;
    let mode = std::fs::metadata(bin_path(&home))
        .unwrap()
        .permissions()
        .mode();
    assert!(
        mode & 0o111 != 0,
        "bin/sheltie is not executable (mode {mode:o})"
    );
}

// Task: C002-T15
#[test]
fn update_rejects_manifest_version_not_matching_tag() {
    // v9.9.9/ manifest declares 9.9.8; reject identity mismatch rather than installing the wrong version.
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
            assert!(reason.contains("does not match the tag"), "{reason}");
        }
        other => panic!("Identity mismatch was accepted: {other:?}"),
    }
}

// Task: C002-T15
#[test]
fn update_failed_download_leaves_old_binary_and_cleans_tmp() {
    // Declared asset missing from the tag: download failure preserves the old binary and clears tmp.
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
        "Old binary was replaced"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn update_failed_digest_leaves_old_binary_and_cleans_tmp() {
    // Asset/manifest digest mismatch: failure preserves the old binary and clears tmp (storage §9, step 3).
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let before = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let asset = format!("sheltie-9.9.9-{platform}");
    // Change one condition: downloadable artifact, but the manifest hashes different bytes.
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
        "Old binary was replaced"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn update_failed_extract_leaves_old_binary_and_cleans_tmp() {
    // Malformed archive with matching digest: extraction failure preserves the old binary and clears tmp.
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
        "Old binary was replaced"
    );
    assert_no_prev(&home);
    assert_tmp_clean(&home);
}

// Task: C002-T15
#[test]
fn clean_home_install_then_update_two_step() {
    // New-root two-step lifecycle: install creates root/database/bin; update replaces binary and retains .prev.
    let (d, home) = temp_home();
    let first = selfmgmt::install(&home).unwrap();
    assert!(!first.already_installed);
    assert!(
        std::path::Path::new(home.store_path().as_str()).exists(),
        "install did not create storage"
    );
    let original = std::fs::read(bin_path(&home)).unwrap();
    let store_before = std::fs::read(home.store_path().as_str()).unwrap();
    let src = make_release(&d.path().join("rel"), "9.9.9");
    let out = selfmgmt::update(&home, &src, None).unwrap();
    assert_eq!(out.to, "9.9.9");
    assert_eq!(
        std::fs::read(home.store_path().as_str()).unwrap(),
        store_before,
        "update changed store.db (storage §9 forbids update from changing storage)"
    );
    assert_eq!(
        std::fs::read(std::path::PathBuf::from(home.bin_dir().as_str()).join("sheltie.prev"))
            .unwrap(),
        original,
        ".prev is not the pre-update binary"
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
    // Rollback restores the old binary after pinned update (local release fixture, storage §9).
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let original = std::fs::read(bin_path(&home)).unwrap();
    let dir = d.path().join("rel");
    make_release(&dir, "9.9.9");
    let eight = "payload of pinned 8.8.8";
    write_version_release(&dir, "8.8.8", eight.as_bytes());
    let store_before = std::fs::read(home.store_path().as_str()).unwrap();
    selfmgmt::update(&home, &release_source(&dir), Some("8.8.8")).unwrap();
    assert_eq!(std::fs::read_to_string(bin_path(&home)).unwrap(), eight);
    assert_eq!(
        std::fs::read(home.store_path().as_str()).unwrap(),
        store_before,
        "update changed store.db (storage §9 forbids update from changing storage)"
    );
    selfmgmt::rollback(&home).unwrap();
    assert_eq!(std::fs::read(bin_path(&home)).unwrap(), original);
    assert_eq!(
        std::fs::read(home.store_path().as_str()).unwrap(),
        store_before,
        "rollback changed store.db (only binary restoration, without Store downgrade)"
    );
    assert!(
        !std::path::PathBuf::from(home.bin_dir().as_str())
            .join("sheltie.prev")
            .exists(),
        ".prev remains after rollback"
    );
}

// Task: C002-T15
#[test]
fn update_adapts_cargo_dist_plan_real_manifest_shape() {
    // Shape copied from actual cargo-dist 0.32.0 dist plan --output-format=json
    // (evidence/t15/dist-plan.json; artifact bytes/digests are synthetic):
    // artifacts is keyed by artifact name; checksum names a file, real digests are in checksums.sha256;
    // built artifacts (local tarball and source.tar.gz) have checksums; unbuilt executable-zip
    // lacks the checksums key, rather than null; skip it without treating filenames as hashes.
    // top-level assets is an exe dictionary without kind, name sheltie; do not fetch it as release assets;
    // a same-named poison file in the tag detects incorrect selection.
    let (d, home) = temp_home();
    selfmgmt::install(&home).unwrap();
    let dir = d.path().join("rel");
    std::fs::create_dir_all(&dir).unwrap();
    let platform = selfmgmt::platform();
    let payload = format!("dist-plan shaped sheltie for {platform}");
    let asset = format!("sheltie-cli-7.7.7-{platform}.tar.xz");
    // The actual filename ends in .tar.xz; use a real archive with T25's verified layout.
    let inner = format!("sheltie-cli-7.7.7-{platform}");
    let stage = d.path().join("stage");
    std::fs::create_dir_all(stage.join(&inner)).unwrap();
    std::fs::write(stage.join(&inner).join("sheltie"), &payload).unwrap();
    let bytes = pack_tar(&stage, &asset, &inner, "-cJf");
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
            // Actual plan omits checksums for unbuilt executable-zip, rather than using null.
            "sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz": {
                "name": "sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz",
                "kind": "executable-zip",
                "target_triples": ["x86_64-unknown-linux-gnu"],
                "assets": ["sheltie-cli-7.7.7-x86_64-unknown-linux-gnu-exe-sheltie"],
                "checksum": "sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz.sha256",
                "path": "target/distrib/sheltie-cli-7.7.7-x86_64-unknown-linux-gnu.tar.xz",
            },
            // Built source.tar.gz also has a digest, but skip it because its kind is not executable-zip.
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
        // Top-level assets matches actual plan keys: no kind, name sheltie.
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
    // Same-named poison in the tag catches top-level-assets selection, either by error or incorrect installation;
    // either fails the assertion.
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
        "Installed binary is not from the platform tarball (top-level assets selected as release assets)"
    );
}

// Task: C002-T15
#[test]
fn home_lock_identity_detects_replaced_lock_file() {
    // §2.2 recheck: replacing .lock with a same-named new object breaks identity continuity.
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
    // §2.2 recheck: deleting/recreating the root changes dev/inode even if .lock reappears.
    let (_d, home) = temp_home();
    let guard = home.acquire_lock().unwrap();
    assert!(guard.identity_still_valid());
    // Simulate same-user external replacement; std cleans fixtures, while production deletes through runtime handle APIs.
    std::fs::remove_dir_all(home.root().as_path()).unwrap();
    std::fs::create_dir_all(home.root().as_str()).unwrap();
    std::fs::write(home.lock_path().as_str(), b"replaced").unwrap();
    assert!(!guard.identity_still_valid());
}

// Task: C002-T15
#[test]
fn self_install_and_work_writes_serialize_under_home_lock() {
    // self and Work writes share the root lock (§2.2); after concurrent interleaving,
    // storage stays readable, Work state complete, and binary present without partial corruption.
    let (_d, home, svc) = home_with_example("two-step");
    let home_b = home.clone();
    let installs = std::thread::spawn(move || {
        for _ in 0..8 {
            selfmgmt::install(&home_b).unwrap();
        }
    });
    // Create a fresh Work and begin per iteration, without reusing an Attempt.
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
        assert!(
            json.last_attempt.is_some(),
            "Work {} state corrupted by concurrent writes",
            wid
        );
    }
}

// Task: C002-T31
#[cfg(feature = "failpoint")]
#[test]
fn purge_late_sqlite_control_files_accept_only_empty_single_link_wal_or_safe_shm() {
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
        let worker_home = home.clone();
        let worker =
            std::thread::spawn(move || selfmgmt::uninstall(&worker_home, true, true).map(|_| ()));
        let mut guard = RendezvousWorker::single(worker, point.path());
        guard.wait("purge did not reach the synchronization point before final SQLite control-file scanning");
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
        let result = guard.finish().unwrap();
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
