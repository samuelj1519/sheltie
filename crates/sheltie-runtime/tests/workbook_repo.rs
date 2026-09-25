//! T14、T15：Workbook 仓库。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_runtime::workbook_repo::VerifyStatus;
use sheltie_runtime::{Error, WorkbookRepo};

fn is_readonly(p: &Path) -> bool {
    p.metadata().unwrap().permissions().readonly()
}

// ── T14 ───────────────────────────────────────────────────────

// Task: T14
#[test]
fn add_two_step_example_copies_and_marks_readonly() {
    let (_d, home) = temp_home();
    let added = repo(&home).add(&abs(&example_dir("two-step"))).unwrap();
    assert_eq!(
        (added.id.as_str(), added.version.as_str()),
        ("two-step", "1.0.0")
    );
    let dir =
        std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).to_path_buf();
    assert!(dir.join("workbook.toml").exists());
    assert!(dir.join("instructions/outline.md").exists());
    assert!(is_readonly(&dir.join("workbook.toml")));
    assert_eq!(added.digest, WorkbookRepo::digest_dir(&abs(&dir)).unwrap());
}

// Task: T14
#[test]
fn add_rejects_duplicate_id_version_with_workbook_exists() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    assert!(matches!(
        r.add(&abs(&example_dir("two-step"))),
        Err(Error::WorkbookExists { .. })
    ));
}

// Task: T14
#[test]
fn add_rejects_symlink_inside_workbook() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    std::os::unix::fs::symlink("/etc/hosts", src.join("instructions/evil.md")).unwrap();
    assert!(repo(&home).add(&abs(&src)).is_err());
    assert!(!std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T14
#[test]
fn add_rejects_file_over_32mib() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    let big = std::fs::File::create(src.join("instructions/big.bin")).unwrap();
    big.set_len(33_554_433).unwrap();
    assert!(repo(&home).add(&abs(&src)).is_err());
}

// Task: T14
#[test]
fn add_failure_leaves_no_staging_and_no_row() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    std::fs::write(src.join("workbook.toml"), "schema = \"workbook/v9\"\n").unwrap();
    let r = repo(&home);
    assert!(r.add(&abs(&src)).is_err());
    let staging = std::path::PathBuf::from(home.staging_dir().as_str());
    assert!(!staging.exists() || std::fs::read_dir(staging).unwrap().next().is_none());
    assert!(r.list().unwrap().is_empty());
}

// Task: T14
#[test]
fn load_recompiles_graph_from_installed_copy() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("article-review"))).unwrap();
    let loaded = r.load("article-review", None).unwrap();
    assert_eq!(loaded.flows.len(), 1);
    assert_eq!(loaded.flow("default").unwrap().1.node_count(), 3);
    assert!(matches!(r.load("nope", None), Err(Error::NotFound { .. })));
}

// Task: T14
#[test]
fn list_orders_by_id_then_version() {
    let (d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    let src = copy_example("two-step", d.path());
    let m = std::fs::read_to_string(src.join("workbook.toml"))
        .unwrap()
        .replace("1.0.0", "1.1.0");
    std::fs::write(src.join("workbook.toml"), m).unwrap();
    r.add(&abs(&src)).unwrap();
    r.add(&abs(&example_dir("article-review"))).unwrap();
    let rows: Vec<(String, String)> = r
        .list()
        .unwrap()
        .into_iter()
        .map(|w| (w.id, w.version))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("article-review".into(), "1.0.0".into()),
            ("two-step".into(), "1.0.0".into()),
            ("two-step".into(), "1.1.0".into())
        ]
    );
}

// ── T15 ───────────────────────────────────────────────────────

// Task: T15
#[test]
#[ignore = "T15"]
fn remove_deletes_row_and_directory() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    r.remove("two-step", "1.0.0").unwrap();
    assert!(r.list().unwrap().is_empty());
    assert!(!std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T15
#[test]
#[ignore = "T15"]
fn remove_requires_explicit_version() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    assert!(matches!(
        r.remove("two-step", ""),
        Err(Error::InvalidRequest { .. })
    ));
}

// Task: T15
#[test]
#[ignore = "T15"]
fn remove_rejects_when_active_work_references_version() {
    let (_d, home, svc) = home_with_example("two-step");
    let resp = start_two_step(&svc);
    let err = repo(&home).remove("two-step", "1.0.0").unwrap_err();
    match err {
        Error::WorkbookInUse { works, .. } => assert_eq!(works, vec![work_id_of(&resp)]),
        other => panic!("{other:?}"),
    }
}

// Task: T15
#[test]
#[ignore = "T15"]
fn remove_allows_when_only_terminal_works_reference_version() {
    let (_d, home, svc) = home_with_example("two-step");
    let resp = start_two_step(&svc);
    svc.cancel(&work_id_of(&resp), None).unwrap();
    repo(&home).remove("two-step", "1.0.0").unwrap();
}

// Task: T15
#[test]
#[ignore = "T15"]
fn remove_moves_dir_to_tmp_before_delete() {
    // 观察不到中间态就看结果：目录消失、tmp 下无残留。
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    r.remove("two-step", "1.0.0").unwrap();
    let tmp = std::path::PathBuf::from(home.tmp_dir().as_str());
    assert!(!tmp.exists() || std::fs::read_dir(tmp).unwrap().next().is_none());
}

// Task: T15
#[test]
#[ignore = "T15"]
fn verify_reports_ok_for_untouched_install() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    let rows = r.verify(Some(("two-step", "1.0.0"))).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, VerifyStatus::Ok);
}

// Task: T15
#[test]
#[ignore = "T15"]
fn verify_reports_tampered_after_byte_change() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    let f = std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str())
        .join("instructions/outline.md");
    std::fs::set_permissions(&f, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
    std::fs::write(&f, "改了").unwrap();
    assert_eq!(r.verify(None).unwrap()[0].status, VerifyStatus::Tampered);
}

// Task: T15
#[test]
#[ignore = "T15"]
fn verify_reports_missing_when_directory_gone() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    let dir = std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str());
    for entry in walk(&dir) {
        // 目录保留可遍历的 0755（同本文件 `make_writable` 的约定），否则 remove_dir_all 进不了子目录。
        let mode = if entry.is_dir() { 0o755 } else { 0o644 };
        std::fs::set_permissions(&entry, std::os::unix::fs::PermissionsExt::from_mode(mode))
            .unwrap();
    }
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(r.verify(None).unwrap()[0].status, VerifyStatus::Missing);
}

// Task: T15
#[test]
#[ignore = "T15"]
fn verify_all_when_filter_omitted() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step"))).unwrap();
    r.add(&abs(&example_dir("gated-release"))).unwrap();
    assert_eq!(r.verify(None).unwrap().len(), 2);
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walk(&p));
        }
        out.push(p);
    }
    out
}
