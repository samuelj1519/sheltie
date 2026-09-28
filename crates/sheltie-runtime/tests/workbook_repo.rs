//! T14、T15：Workbook 仓库。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_core::testkit::Fixture;
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
    let added = repo(&home)
        .add(&abs(&example_dir("two-step")), None)
        .unwrap();
    assert_eq!(
        (
            added.data["id"].as_str().unwrap(),
            added.data["version"].as_str().unwrap()
        ),
        ("two-step", "1.0.0")
    );
    let dir =
        std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).to_path_buf();
    assert!(dir.join("workbook.toml").exists());
    assert!(dir.join("instructions/outline.md").exists());
    assert!(is_readonly(&dir.join("workbook.toml")));
    assert_eq!(
        added.data["digest"].as_str().unwrap(),
        WorkbookRepo::digest_dir(&abs(&dir)).unwrap().as_str()
    );
}

// Task: T14
#[test]
fn add_rejects_duplicate_id_version_with_workbook_exists() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    assert!(matches!(
        r.add(&abs(&example_dir("two-step")), None),
        Err(Error::WorkbookExists { .. })
    ));
}

// Task: T14
#[test]
fn add_rejects_symlink_inside_workbook() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    std::os::unix::fs::symlink("/etc/hosts", src.join("instructions/evil.md")).unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
    assert!(!std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T14
#[test]
fn add_rejects_file_over_32mib() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    let big = std::fs::File::create(src.join("instructions/big.bin")).unwrap();
    big.set_len(33_554_433).unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

// Task: T14
#[test]
fn add_rejects_non_regular_file() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    let status = std::process::Command::new("mkfifo")
        .arg(src.join("instructions/pipe"))
        .status()
        .unwrap();
    assert!(status.success());
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

// Task: T14
#[test]
fn add_failure_leaves_no_staging_and_no_row() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    std::fs::write(src.join("workbook.toml"), "schema = \"workbook/v9\"\n").unwrap();
    let r = repo(&home);
    assert!(r.add(&abs(&src), None).is_err());
    let staging = std::path::PathBuf::from(home.staging_dir().as_str());
    assert!(!staging.exists() || std::fs::read_dir(staging).unwrap().next().is_none());
    assert!(r.list().unwrap().is_empty());
}

// Task: T14
#[test]
fn add_rejects_hard_link_inside_workbook() {
    let (d, home) = temp_home();
    let src = copy_example("two-step", d.path());
    std::fs::hard_link(src.join("workbook.toml"), src.join("instructions/hard.md")).unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

/// 一个最小的合法 Workbook，用来凑文件大小边界。
fn write_minimal_workbook(dir: &Path) {
    std::fs::create_dir_all(dir.join("flows")).unwrap();
    std::fs::write(
        dir.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"big\"\nversion = \"1.0.0\"\nname = \"大小边界\"\ndescription = \"凑上限用。\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("flows/default.toml"),
        "schema = \"flow/v1\"\nid = \"default\"\nentry = \"a\"\n\n[[nodes]]\nid = \"a\"\ntitle = \"甲\"\nexecutor = \"agent\"\ninstruction = { text = \"做。\" }\noutputs = [{ name = \"x\", path = \"x.md\", max_bytes = 65536 }]\n",
    )
    .unwrap();
}

/// 目录现有小文件的总字节数（只算普通文件，与 `copy_confined` 的口径一致）。
fn small_files_bytes(dir: &Path) -> u64 {
    walk(dir)
        .iter()
        .filter(|p| p.is_file())
        .map(|p| p.metadata().unwrap().len())
        .sum()
}

// Task: T14
#[test]
fn add_accepts_files_at_exact_limits() {
    // 7 个恰好 32 MiB 的文件加一个凑数文件，总量恰好 256 MiB：两个上限都顶到且接受。
    let (d, home) = temp_home();
    let src = d.path().join("big");
    write_minimal_workbook(&src);
    let small = small_files_bytes(&src);
    let max_file = sheltie_runtime::workbook_repo::MAX_FILE_BYTES;
    let max_total = sheltie_runtime::workbook_repo::MAX_TOTAL_BYTES;
    for i in 0..7 {
        std::fs::File::create(src.join(format!("big{i}.bin")))
            .unwrap()
            .set_len(max_file)
            .unwrap();
    }
    let last = max_total - 7 * max_file - small;
    assert!(last > 0 && last <= max_file);
    std::fs::File::create(src.join("big7.bin"))
        .unwrap()
        .set_len(last)
        .unwrap();
    repo(&home).add(&abs(&src), None).unwrap();
}

// Task: T14
#[test]
fn add_rejects_when_total_over_256mib() {
    // 总量上限多 1 字节。
    let (d, home) = temp_home();
    let src = d.path().join("big");
    write_minimal_workbook(&src);
    let small = small_files_bytes(&src);
    let max_file = sheltie_runtime::workbook_repo::MAX_FILE_BYTES;
    let max_total = sheltie_runtime::workbook_repo::MAX_TOTAL_BYTES;
    for i in 0..7 {
        std::fs::File::create(src.join(format!("big{i}.bin")))
            .unwrap()
            .set_len(max_file)
            .unwrap();
    }
    std::fs::File::create(src.join("big7.bin"))
        .unwrap()
        .set_len(max_total - 7 * max_file - small + 1)
        .unwrap();
    assert!(repo(&home).add(&abs(&src), None).is_err());
}

// Task: T14
#[test]
fn load_with_explicit_version_picks_that_version() {
    let (d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let src = copy_example("two-step", d.path());
    let m = std::fs::read_to_string(src.join("workbook.toml"))
        .unwrap()
        .replace("1.0.0", "1.1.0");
    std::fs::write(src.join("workbook.toml"), m).unwrap();
    r.add(&abs(&src), None).unwrap();
    assert_eq!(
        r.load("two-step", Some("1.0.0"))
            .unwrap()
            .manifest
            .version(),
        "1.0.0"
    );
    assert_eq!(
        r.load("two-step", None).unwrap().manifest.version(),
        "1.1.0"
    );
}

// Task: T14
#[test]
fn load_recompiles_graph_from_installed_copy() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("article-review")), None).unwrap();
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
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let src = copy_example("two-step", d.path());
    let m = std::fs::read_to_string(src.join("workbook.toml"))
        .unwrap()
        .replace("1.0.0", "1.1.0");
    std::fs::write(src.join("workbook.toml"), m).unwrap();
    r.add(&abs(&src), None).unwrap();
    r.add(&abs(&example_dir("article-review")), None).unwrap();
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
fn remove_deletes_row_and_directory() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    r.remove("two-step", "1.0.0", None).unwrap();
    assert!(r.list().unwrap().is_empty());
    assert!(!std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T15
#[test]
fn remove_requires_explicit_version() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    assert!(matches!(
        r.remove("two-step", "", None),
        Err(Error::InvalidRequest { .. })
    ));
}

// Task: C002-T14
#[test]
fn remove_stops_on_corrupt_row_even_when_redundant_status_looks_terminal() {
    let (_dir, home) = temp_home();
    let repo = repo(&home);
    repo.add(&abs(&example_dir("two-step")), None).unwrap();
    let state = Fixture::two_step()
        .started_with(&[("topic", "t")])
        .state()
        .clone();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "INSERT INTO works (work_id, revision, status, state_json, created_at, updated_at)
         VALUES (?1, 1, 'succeeded', ?2, ?3, ?4)",
        rusqlite::params![
            state.work_id.as_str(),
            serde_json::to_string(&state).unwrap(),
            state.created_at.as_str(),
            state.updated_at.as_str()
        ],
    )
    .unwrap();
    let error = repo.remove("two-step", "1.0.0", None).unwrap_err();
    assert!(matches!(&error, Error::StoreCorrupt { .. }), "{error}");
    assert_eq!(repo.list().unwrap().len(), 1);
    assert!(std::path::Path::new(home.workbook_dir("two-step", "1.0.0").as_str()).exists());
}

// Task: T15
#[test]
fn remove_rejects_when_active_work_references_version() {
    let (_d, home, svc) = home_with_example("two-step");
    let resp = start_two_step(&svc);
    let err = repo(&home).remove("two-step", "1.0.0", None).unwrap_err();
    match err {
        Error::WorkbookInUse { works, .. } => assert_eq!(works, vec![work_id_of(&resp)]),
        other => panic!("{other:?}"),
    }
}

// Task: T15
#[test]
fn remove_allows_when_only_terminal_works_reference_version() {
    let (_d, home, svc) = home_with_example("two-step");
    let resp = start_two_step(&svc);
    svc.cancel(&work_id_of(&resp), None).unwrap();
    repo(&home).remove("two-step", "1.0.0", None).unwrap();
}

// Task: T15
#[test]
fn remove_allows_when_other_workbook_shares_version() {
    // 活跃 Work 引用 two-step@1.0.0；同号的 article-review@1.0.0 不受牵连。
    let (_d, home, svc) = home_with_example("two-step");
    let r = repo(&home);
    r.add(&abs(&example_dir("article-review")), None).unwrap();
    let _ = start_two_step(&svc);
    r.remove("article-review", "1.0.0", None).unwrap();
    assert!(matches!(
        r.load("article-review", None),
        Err(Error::NotFound { .. })
    ));
}

// Task: T15
#[test]
fn remove_moves_dir_to_tmp_before_delete() {
    // 观察不到中间态就看结果：目录消失、tmp 下无残留。
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    r.remove("two-step", "1.0.0", None).unwrap();
    let tmp = std::path::PathBuf::from(home.tmp_dir().as_str());
    assert!(!tmp.exists() || std::fs::read_dir(tmp).unwrap().next().is_none());
}

// Task: T15
#[test]
fn verify_reports_ok_for_untouched_install() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let rows = r.verify(Some(("two-step", "1.0.0"))).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, VerifyStatus::Ok);
}

// Task: T15
#[test]
fn verify_reports_tampered_after_byte_change() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let f = std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str())
        .join("instructions/outline.md");
    std::fs::set_permissions(&f, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
    std::fs::write(&f, "改了").unwrap();
    assert_eq!(r.verify(None).unwrap()[0].status, VerifyStatus::Tampered);
}

// Task: T15
#[test]
fn verify_reports_missing_when_directory_gone() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let dir = std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str());
    // T04 起整棵含根置只读（0555）；删除前把根本身也放开。
    std::fs::set_permissions(&dir, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
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
fn verify_all_when_filter_omitted() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    r.add(&abs(&example_dir("gated-release")), None).unwrap();
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
