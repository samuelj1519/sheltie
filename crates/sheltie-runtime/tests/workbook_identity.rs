//! C002-T05: Workbook identity, copy verification, read-only roots, and initialization (O03/O06/N04/N10/§5.3).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::NodeId;
use sheltie_runtime::{Error, Home, WorkService, WorkbookRepo};

/// After installed-directory changes, verify reports tampered and start rejects them (O03).
// Task: C002-T05
#[test]
fn load_rejects_tampered_registered_digest() {
    let (_d, home, svc) = home_with_example("two-step");
    let f =
        Path::new(home.workbook_dir("two-step", "1.0.1").as_str()).join("instructions/outline.md");
    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::write(&f, "Changed externally").unwrap();

    let rows = svc_status_repo(&home).verify(None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (&rows[0].id[..], &rows[0].version[..]),
        ("two-step", "1.0.1")
    );
    assert_eq!(rows[0].status, sheltie_runtime::VerifyStatus::Tampered);
    let err = svc
        .start(two_step_args(&[("topic", "t")]), None)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookTampered, "{err:?}");
    // Reject before any materialization.
    assert!(
        !home.works_dir().as_path().exists()
            || std::fs::read_dir(home.works_dir().as_path())
                .unwrap()
                .count()
                == 0
    );
}

fn svc_status_repo(home: &Home) -> WorkbookRepo {
    WorkbookRepo::new(home.clone())
}

/// Verify ownership before cleanup; changed installed directories reject remove and retain rows.
// Task: C002-T05
#[test]
fn remove_refuses_tampered_directory() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let f =
        Path::new(home.workbook_dir("two-step", "1.0.1").as_str()).join("instructions/outline.md");
    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::write(&f, "Revised").unwrap();

    let err = r.remove("two-step", "1.0.1", None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookTampered, "{err:?}");
    // The row remains.
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let rows: i64 = connection
        .query_row("SELECT COUNT(*) FROM workbooks", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 1);
}

/// Reject and name Finder metadata precisely under §5.3, without rows or final directories.
// Task: C002-T05
#[test]
fn ds_store_rejected_by_name_at_add() {
    let src = tempfile::tempdir().unwrap();
    let dst = copy_example("two-step", src.path());
    std::fs::write(dst.join(".DS_Store"), b"finder junk").unwrap();

    let (_d, home) = temp_home();
    let r = repo(&home);
    let err = r.add(&abs(&dst), None).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(".DS_Store"), "{msg}");
    assert!(
        !home.store_path().as_path().exists(),
        "Invalid add must not create Store"
    );
    assert!(
        !home.lock_path().as_path().exists(),
        "Invalid add must not create a lock"
    );
    assert!(!home.workbook_dir("two-step", "1.0.1").as_path().exists());
}

/// Read-only missing storage yields NOT_FOUND without directory creation (GF-30).
// Task: C002-T05
#[test]
fn readonly_open_never_creates_home() {
    let dir = tempfile::tempdir().unwrap();
    let home = Home::resolve(Some((abs(dir.path())).as_str())).unwrap();
    assert!(matches!(
        WorkService::new(home.clone()).list(),
        Err(Error::NotFound { .. })
    ));
    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert!(
        entries.is_empty(),
        "Read-only access must not create directories: {entries:?}"
    );
}

fn account_name_oracle() -> String {
    let out = std::process::Command::new("id")
        .arg("-un")
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// In-process reads also ignore USER; principal does not read the environment, so compare with the oracle here.
// Task: C002-T05
#[test]
fn principal_matches_id_un_oracle() {
    assert_eq!(
        sheltie_runtime::observe::principal().0,
        account_name_oracle()
    );
}

/// Accepted lifecycle: install, freeze, terminal query; terminal Work status remains complete after Workbook removal.
// Task: C002-T05
#[test]
fn work_readable_after_workbook_removed() {
    for submitted_history in [false, true] {
        let (_d, home, svc) = home_with_example("two-step");
        let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
        if submitted_history {
            let begun = svc
                .begin(&wid, &NodeId::new("outline").unwrap(), None)
                .unwrap();
            write_output(&output_dir_of(&begun), "outline.md", "Outline");
            // Cancel into terminal state after submission, then remove the Workbook.
            svc.submit(
                &wid,
                &sheltie_core::ids::AttemptId::parse("outline#1.0").unwrap(),
                &sheltie_runtime::request::InputValue::Literal {
                    text: "Completed".to_string(),
                },
                None,
            )
            .unwrap();
        }
        svc.cancel(&wid, None).unwrap();

        repo(&home).remove("two-step", "1.0.1", None).unwrap();
        let (card, json) = svc.status(&wid).unwrap();
        assert!(card.contains(&format!("# Work {wid}")));
        assert!(card.contains("status: cancelled"));
        assert_eq!(json.status, sheltie_core::work::WorkStatus::Cancelled);
    }
}

/// Read-only permissions reduce accidental writes; same-user chmod can alter frozen copies, then digest verification rejects them.
/// Do not claim permissions prevent tampering; load's digest check is the integrity boundary.
// Task: C002-T05
#[test]
fn readonly_bits_reduce_accidents_but_digest_is_the_guard() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let frozen = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("workbook");
    // Read-only permissions are set, including root 0555 (N10).
    let mode = std::fs::metadata(&frozen).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o555);
    // Same user relaxes permissions and changes bytes.
    let mut m = std::fs::metadata(&frozen).unwrap().permissions();
    m.set_mode(0o755);
    std::fs::set_permissions(&frozen, m).unwrap();
    for entry in std::fs::read_dir(&frozen).unwrap().flatten() {
        if entry.path().is_file() {
            let mut fm = entry.metadata().unwrap().permissions();
            fm.set_mode(0o644);
            std::fs::set_permissions(entry.path(), fm).unwrap();
        }
    }
    std::fs::write(frozen.join("workbook.toml"), "Revised").unwrap();
    // Independent recomputation proves the digest changed and differs from its stored record.
    let now = WorkbookRepo::digest_dir(&abs(&frozen)).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let state_json: String = conn
        .query_row(
            "SELECT state_json FROM works WHERE work_id = ?1",
            [wid.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let recorded = serde_json::from_str::<sheltie_core::work::WorkState>(&state_json)
        .unwrap()
        .workbook
        .digest;
    assert_ne!(now, recorded, "Tampering must change the digest");
    let err = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err:?}");
}
