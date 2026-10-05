#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::*;
use serde_json::Value;
use std::process::Command;

// Task: C006-T02
#[test]
fn exporter_rejects_an_overlapping_or_symlinked_parent_without_modifying_the_target() {
    let fixture = Fixture::complete();
    let sentinel = fixture.parent.join("sentinel");
    std::fs::write(&sentinel, b"preserve existing target").unwrap();
    let alias = fixture.dir.path().canonicalize().unwrap().join("alias");
    std::os::unix::fs::symlink(&fixture.parent, &alias).unwrap();
    for parent in [&fixture.home, fixture.home.parent().unwrap(), &alias] {
        let out = Command::new(&binaries().1)
            .args([
                "--sheltie",
                binaries().0.to_str().unwrap(),
                "--home",
                fixture.home.to_str().unwrap(),
                "--work",
                &fixture.work,
                "--to",
                parent.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        let response: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(response["status"], "rejected");
        assert_eq!(response["target_path"], Value::Null);
    }
    assert_eq!(
        std::fs::read(&sentinel).unwrap(),
        b"preserve existing target"
    );
    assert_eq!(std::fs::read_dir(&fixture.parent).unwrap().count(), 1);
}

// Task: C006-T02
#[test]
fn exporter_rejects_a_nonfinal_work_before_creating_staging() {
    let fixture = Fixture::complete();
    let active = ok(
        &fixture.home,
        &[
            "work",
            "start",
            "--workbook",
            "byte-fixture",
            "--flow",
            "default",
            "--name",
            "active",
            "--input",
            "task=unfinished",
        ],
    );
    let work = active["data"]["work_id"].as_str().unwrap();
    let before = ok(&fixture.home, &["work", "status", work]);
    let out = Command::new(&binaries().1)
        .args([
            "--sheltie",
            binaries().0.to_str().unwrap(),
            "--home",
            fixture.home.to_str().unwrap(),
            "--work",
            work,
            "--to",
            fixture.parent.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(response["status"], "rejected");
    assert_eq!(response["staging_path"], Value::Null);
    assert_eq!(std::fs::read_dir(&fixture.parent).unwrap().count(), 0);
    assert_eq!(ok(&fixture.home, &["work", "status", work]), before);
}

// Task: C006-T01
#[test]
fn exporter_version_is_available_without_an_engine_or_management_root() {
    let out = Command::new(&binaries().1)
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("sheltie-export {}\n", env!("CARGO_PKG_VERSION"))
    );
}
