#![cfg(feature = "failpoint")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use serde_json::Value;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

struct ExportChild(Option<Child>);

impl Drop for ExportChild {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl ExportChild {
    fn wait_at(&mut self, marker: &Path, point: &str) {
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            if marker.exists() {
                assert_eq!(std::fs::read(marker).unwrap(), point.as_bytes());
                return;
            }
            if self.0.as_mut().unwrap().try_wait().unwrap().is_some() {
                let output = self.0.take().unwrap().wait_with_output().unwrap();
                panic!(
                    "未到达真实边界 {point}：status={} stdout={} stderr={}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            assert!(Instant::now() < deadline, "等待真实边界 {point} 超时");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn kill_and_wait(mut self) -> Output {
        let mut child = self.0.take().unwrap();
        child.kill().unwrap();
        child.wait_with_output().unwrap()
    }
}

fn command(fixture: &Fixture) -> Command {
    let mut command = Command::new(&binaries().1);
    command.args([
        "--sheltie",
        binaries().0.to_str().unwrap(),
        "--home",
        fixture.home.to_str().unwrap(),
        "--work",
        &fixture.work,
        "--to",
        fixture.parent.to_str().unwrap(),
        "--json",
    ]);
    command
}

fn only_scene(parent: &Path) -> PathBuf {
    let mut entries = std::fs::read_dir(parent)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    entries.pop().unwrap()
}

fn bytes_in_tree(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn collect(root: &Path, path: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            assert!(!metadata.file_type().is_symlink());
            if metadata.is_dir() {
                collect(root, &path, files);
            } else {
                assert!(metadata.is_file());
                files.push((
                    path.strip_prefix(root).unwrap().into(),
                    std::fs::read(path).unwrap(),
                ));
            }
        }
    }
    let mut files = Vec::new();
    collect(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn assert_whole_copy(fixture: &Fixture, target: &Path) {
    assert_eq!(target.parent(), Some(fixture.parent.as_path()));
    assert_eq!(
        std::fs::metadata(target).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for (index, leaf, expected) in [
        ("0001", "final.bin", fixture.binary.as_slice()),
        ("0002", "empty.bin", &[]),
        ("0003", "task", "原目标".as_bytes()),
    ] {
        let directory = target.join("artifacts").join(index);
        assert_eq!(
            std::fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        let path = directory.join(leaf);
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert!(metadata.is_file());
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(std::fs::read(path).unwrap(), expected);
    }
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(target.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["format"], "work-export-manifest/v1");
    assert_eq!(manifest["result"], fixture.result);
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), 3);
    for (index, (key, path)) in [
        ("binary", "artifacts/0001/final.bin"),
        ("empty", "artifacts/0002/empty.bin"),
        ("task", "artifacts/0003/task"),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(files[index]["key"], *key);
        assert_eq!(files[index]["path"], *path);
        assert_eq!(
            files[index]["sha256"],
            fixture.result["artifacts"][index]["sha256"]
        );
        assert_eq!(
            files[index]["bytes"],
            fixture.result["artifacts"][index]["bytes"]
        );
    }
}

fn killed_at(point: &str, published: bool) {
    let fixture = Fixture::complete();
    let before = ok(&fixture.home, &["work", "status", &fixture.work]);
    let rendezvous = fixture
        .dir
        .path()
        .canonicalize()
        .unwrap()
        .join("rendezvous");
    std::fs::create_dir(&rendezvous).unwrap();
    let mut child = ExportChild(Some(
        command(&fixture)
            .env("SHELTIE_EXPORT_TEST_POINT", point)
            .env("SHELTIE_EXPORT_TEST_DIRECTORY", &rendezvous)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    ));
    child.wait_at(&rendezvous.join("reached"), point);
    let scene = only_scene(&fixture.parent);
    let name = scene.file_name().unwrap().to_str().unwrap();
    assert_eq!(name.starts_with(".sheltie-export-"), !published);
    if published {
        assert_whole_copy(&fixture, &scene);
    }
    let retained_bytes = bytes_in_tree(&scene);
    let output = child.kill_and_wait();
    assert_eq!(output.status.signal(), Some(9));
    assert!(output.stdout.is_empty(), "被杀前不得输出完成响应");
    assert_eq!(
        ok(&fixture.home, &["work", "status", &fixture.work]),
        before
    );
    assert_eq!(
        ok(&fixture.home, &["work", "result", &fixture.work])["data"],
        fixture.result
    );
    assert_eq!(bytes_in_tree(&scene), retained_bytes);

    let rerun = fixture.export();
    assert!(
        rerun.status.success(),
        "{}",
        String::from_utf8_lossy(&rerun.stderr)
    );
    let report: Value = serde_json::from_slice(&rerun.stdout).unwrap();
    let target = Path::new(report["target_path"].as_str().unwrap());
    assert_ne!(target, scene);
    assert_whole_copy(&fixture, target);
    assert_eq!(std::fs::read_dir(&fixture.parent).unwrap().count(), 2);
    assert_eq!(bytes_in_tree(&scene), retained_bytes);
    assert_eq!(
        ok(&fixture.home, &["work", "status", &fixture.work]),
        before
    );
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_after_staging_creation_preserves_owned_scene_without_a_published_copy() {
    killed_at("staging_created", false);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_during_actual_receive_preserves_partial_scene_without_a_published_copy() {
    killed_at("during_receive", false);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_after_file_sync_preserves_owned_scene_without_a_published_copy() {
    killed_at("after_file_sync", false);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_after_independent_readback_preserves_owned_scene_without_a_published_copy() {
    killed_at("after_readback", false);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_after_manifest_write_preserves_owned_scene_without_a_published_copy() {
    killed_at("after_manifest_write", false);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_after_tree_sync_before_rename_preserves_staging_without_a_published_copy() {
    killed_at("before_rename", false);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_after_rename_before_parent_sync_leaves_a_whole_visible_copy_without_a_response() {
    killed_at("after_rename_before_parent_sync", true);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn kill_after_parent_sync_before_response_leaves_a_whole_copy_and_rerun_creates_another() {
    killed_at("after_parent_sync", true);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn sync_failure_before_publication_keeps_staging_and_rerun_preserves_the_original_scene() {
    let fixture = Fixture::complete();
    let before = ok(&fixture.home, &["work", "status", &fixture.work]);
    let output = command(&fixture)
        .env("SHELTIE_EXPORT_TEST_SYNC_ERROR", "before_rename")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "failed_before_publish");
    assert_eq!(report["error"]["code"], "IO");
    assert_eq!(report["target_path"], Value::Null);
    let scene = PathBuf::from(report["staging_path"].as_str().unwrap());
    assert_eq!(only_scene(&fixture.parent), scene);
    assert!(
        scene
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with(".sheltie-export-")
    );
    let retained_bytes = bytes_in_tree(&scene);
    let rerun = fixture.export();
    assert!(rerun.status.success());
    let rerun: Value = serde_json::from_slice(&rerun.stdout).unwrap();
    assert_whole_copy(&fixture, Path::new(rerun["target_path"].as_str().unwrap()));
    assert_eq!(bytes_in_tree(&scene), retained_bytes);
    assert_eq!(std::fs::read_dir(&fixture.parent).unwrap().count(), 2);
    assert_eq!(
        ok(&fixture.home, &["work", "status", &fixture.work]),
        before
    );
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn sync_failure_after_rename_reports_unconfirmed_and_keeps_the_whole_visible_copy() {
    for point in ["after_rename_before_parent_sync", "after_parent_sync"] {
        let fixture = Fixture::complete();
        let before = ok(&fixture.home, &["work", "status", &fixture.work]);
        let output = command(&fixture)
            .env("SHELTIE_EXPORT_TEST_SYNC_ERROR", point)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(3));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["status"], "publication_unconfirmed");
        assert_eq!(report["error"]["code"], "PUBLICATION_UNCONFIRMED");
        assert_eq!(report["staging_path"], Value::Null);
        let scene = PathBuf::from(report["target_path"].as_str().unwrap());
        assert_eq!(only_scene(&fixture.parent), scene);
        assert_whole_copy(&fixture, &scene);
        let retained_bytes = bytes_in_tree(&scene);
        let rerun = fixture.export();
        assert!(rerun.status.success());
        let rerun: Value = serde_json::from_slice(&rerun.stdout).unwrap();
        let target = Path::new(rerun["target_path"].as_str().unwrap());
        assert_ne!(target, scene);
        assert_whole_copy(&fixture, target);
        assert_eq!(bytes_in_tree(&scene), retained_bytes);
        assert_eq!(std::fs::read_dir(&fixture.parent).unwrap().count(), 2);
        assert_eq!(
            ok(&fixture.home, &["work", "status", &fixture.work]),
            before
        );
    }
}
