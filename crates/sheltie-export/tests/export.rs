#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::*;
use serde_json::Value;
use std::path::Path;

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn export_publishes_all_final_bytes_and_deterministic_provenance_without_changing_work() {
    let fixture = Fixture::complete();
    let before = ok(&fixture.home, &["work", "status", &fixture.work]);
    let out = fixture.export();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let reply: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(reply["format"], "work-export/v1");
    assert_eq!(reply["status"], "complete");
    assert_eq!(reply["work_id"], fixture.work);
    assert_eq!(reply["revision"], fixture.revision);
    assert_eq!(reply["staging_path"], Value::Null);
    let target = Path::new(reply["target_path"].as_str().unwrap());
    assert_eq!(target.parent().unwrap(), fixture.parent);
    assert_eq!(
        std::fs::read(target.join("artifacts/0001/final.bin")).unwrap(),
        fixture.binary
    );
    assert_eq!(
        std::fs::read(target.join("artifacts/0002/empty.bin")).unwrap(),
        Vec::<u8>::new()
    );
    assert_eq!(
        std::fs::read(target.join("artifacts/0003/task")).unwrap(),
        "原目标".as_bytes()
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(target.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["format"], "work-export-manifest/v1");
    assert_eq!(manifest["result"], fixture.result);
    let files = manifest["files"].as_array().unwrap();
    for (i, key) in ["binary", "empty", "task"].iter().enumerate() {
        assert_eq!(files[i]["key"], *key);
        assert_eq!(files[i]["sha256"], fixture.result["artifacts"][i]["sha256"]);
        assert_eq!(files[i]["bytes"], fixture.result["artifacts"][i]["bytes"]);
    }
    assert_eq!(
        ok(&fixture.home, &["work", "status", &fixture.work]),
        before
    );
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn export_rerun_creates_a_new_copy_and_preserves_edits_in_the_previous_copy() {
    let fixture = Fixture::complete();
    let first = fixture.export();
    assert!(first.status.success());
    let first: Value = serde_json::from_slice(&first.stdout).unwrap();
    let first = Path::new(first["target_path"].as_str().unwrap());
    std::fs::write(
        first.join("artifacts/0001/final.bin"),
        b"editable user copy",
    )
    .unwrap();
    let second = fixture.export();
    assert!(second.status.success());
    let second: Value = serde_json::from_slice(&second.stdout).unwrap();
    let second = Path::new(second["target_path"].as_str().unwrap());
    assert_ne!(first, second);
    assert_eq!(
        std::fs::read(first.join("artifacts/0001/final.bin")).unwrap(),
        b"editable user copy"
    );
    assert_eq!(
        std::fs::read(second.join("artifacts/0001/final.bin")).unwrap(),
        fixture.binary
    );
    assert_eq!(
        std::fs::read(first.join("manifest.json")).unwrap(),
        std::fs::read(second.join("manifest.json")).unwrap()
    );
    let source = fixture.result["artifacts"][0]["path"].as_str().unwrap();
    assert_eq!(std::fs::read(source).unwrap(), fixture.binary);
}

// Task: C006-T02
#[test]
#[ignore = "C006-T02"]
fn export_source_integrity_failure_preserves_owned_staging_without_publishing_a_final_directory() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::complete();
    let source = Path::new(fixture.result["artifacts"][0]["path"].as_str().unwrap());
    std::fs::set_permissions(source, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(source, b"changed sealed source").unwrap();
    let before = ok(&fixture.home, &["work", "status", &fixture.work]);
    let out = fixture.export();
    assert_eq!(out.status.code(), Some(1));
    let reply: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(reply["status"], "failed_before_publish");
    assert_eq!(reply["target_path"], Value::Null);
    let staging = Path::new(reply["staging_path"].as_str().unwrap());
    assert!(staging.is_dir());
    let entries: Vec<_> = std::fs::read_dir(&fixture.parent)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].to_str().unwrap().starts_with(".sheltie-export-"));
    assert_eq!(
        ok(&fixture.home, &["work", "status", &fixture.work]),
        before
    );
}
