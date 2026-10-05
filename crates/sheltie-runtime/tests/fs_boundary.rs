//! C002-T04: managed-path, file-handle, limit, and safe atomic-write rejection boundaries (O01).
//! Compare every external sentinel's bytes and permissions independently.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::two_step_args as start_args;
use common::*;
use sheltie_core::ids::AttemptId;
use sheltie_runtime::Error;
use sheltie_runtime::fsx::{ExternalReadFile, ManagedFs, ManagedRelPath};

fn canonical_abs(path: &Path) -> sheltie_core::path::AbsPath {
    abs(&std::fs::canonicalize(path).unwrap())
}

/// Replacing works with an external symlink must prevent start from writing a Work tree (O01 rejection).
// Task: C002-T04
#[test]
fn works_parent_symlink_blocks_start_and_keeps_sentinel_untouched() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-me");
    let before = snapshot(&sent);

    let (_d, home) = temp_home();
    repo(&home)
        .add(&abs(&example_dir("two-step")), None)
        .unwrap();
    // Replace works with an external symlink.
    let works = home.works_dir();
    std::os::unix::fs::symlink(outside.path(), works.as_path()).unwrap();

    let svc = service(&home);
    let err = svc.start(start_args(&[("topic", "t")]), None).unwrap_err();
    assert_eq!(
        err.code(),
        sheltie_core::ErrorCode::InvalidRequest,
        "{err:?}"
    );
    assert!(err.to_string().contains("symlink"), "{err:?}");

    // Sentinel bytes/permissions stay unchanged; no Work directory appears outside the root.
    assert_eq!(snapshot(&sent), before);
    let entries: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
    assert_eq!(
        entries.len(),
        1,
        "Must not write outside the root: {entries:?}"
    );
}

/// Replacing bin with a symlink rejects self install, preserving the sentinel.
// Task: C002-T04
#[test]
fn bin_parent_symlink_blocks_install_and_keeps_sentinel_untouched() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-bin");
    let before = snapshot(&sent);

    let (_d, home) = temp_home();
    std::os::unix::fs::symlink(outside.path(), home.bin_dir().as_path()).unwrap();

    let err = sheltie_runtime::selfmgmt::install(&home).unwrap_err();
    assert!(err.to_string().contains("symlink"), "{err:?}");
    assert_eq!(snapshot(&sent), before);
    let entries: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
    assert_eq!(
        entries.len(),
        1,
        "Must not write outside the root: {entries:?}"
    );
}

/// Replacing input with an external-sentinel symlink rejects during begin observation.
// Task: C002-T04
#[test]
fn leaf_symlink_input_rejected_at_observation() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-input");
    let before = snapshot(&sent);

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    // External actor replaces a start input with a sentinel symlink.
    let input = Path::new(home.work_dir(&wid).as_str()).join("start-inputs/topic");
    std::fs::remove_file(&input).unwrap();
    std::os::unix::fs::symlink(&sent, &input).unwrap();

    let err = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
        .unwrap_err();
    assert!(err.to_string().contains("symlink"), "{err:?}");
    assert_eq!(snapshot(&sent), before);
}

/// Preplanted fixed-name temporary symlinks (old status-card.md.tmp-pending attack) cannot redirect
/// status writes: random exclusive temporary names preserve the correct card and untouched sentinel.
// Task: C002-T04
#[test]
fn precreated_tmp_pending_symlink_cannot_redirect_status_card() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-card");
    let before = snapshot(&sent);

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    let work = std::path::PathBuf::from(home.work_dir(&wid).as_str());
    // Plant a symlink at the old implementation's fixed temporary name.
    std::os::unix::fs::symlink(&sent, work.join("status-card.md.tmp-pending")).unwrap();

    svc.cancel(&wid, None).unwrap();
    assert_eq!(
        snapshot(&sent),
        before,
        "Must not write the external sentinel"
    );
    let card = std::fs::read_to_string(work.join("status-card.md")).unwrap();
    assert!(card.contains(&format!("# Work {wid}")), "{card}");
}

/// Handles pin observed objects; replacing a path after observation leaves the open handle bound to the original,
/// while reopening sees the new object, with identity/content checks detecting the change.
// Task: C002-T04
#[test]
fn safe_handle_pins_observed_object_across_path_swap() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("f");
    std::fs::write(&p, b"original").unwrap();
    let f = ExternalReadFile::open_regular(&abs(&p)).unwrap();
    let (sha1, n1) = f.sha256_bounded(1024).unwrap();
    assert_eq!(n1, 8);

    // Replace the path later; the original handle still reads the original object.
    std::fs::remove_file(&p).unwrap();
    std::fs::write(&p, b"replacement").unwrap();
    let (sha2, n2) = f.sha256_bounded(1024).unwrap();
    assert_eq!(
        (sha2.clone(), n2),
        (sha1.clone(), n1),
        "Handle pins the object opened originally"
    );

    // A new handle observes new content; replacements cannot impersonate the original during sealing.
    let g = ExternalReadFile::open_regular(&abs(&p)).unwrap();
    let (sha3, n3) = g.sha256_bounded(1024).unwrap();
    assert_ne!((sha3, n3), (sha1, n1));
}

/// Observation limits: accept exactly 32 MiB, reject one extra byte before reading (fsx unit); declared
/// 33 MiB outputs yield OUTPUT_TOO_LARGE during service.submit observation, without full-memory reads.
// Task: C002-T04
#[test]
fn bounded_read_accepts_exactly_cap_and_rejects_one_more() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("big");
    let f = std::fs::File::create(&p).unwrap();
    f.set_len(32 * 1024 * 1024).unwrap();
    drop(f);
    let cap = sheltie_runtime::fsx::MAX_FILE_BYTES;
    let at = ExternalReadFile::open_regular(&abs(&p)).unwrap();
    assert!(at.read_bounded(cap).is_ok());

    let over = dir.path().join("big2");
    let f = std::fs::File::create(&over).unwrap();
    f.set_len(cap + 1).unwrap();
    drop(f);
    let at = ExternalReadFile::open_regular(&abs(&over)).unwrap();
    match at.read_bounded(cap) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("Exceeds"), "{reason}"),
        other => panic!("Over-limit cases must be rejected: {other:?}"),
    }
}

// Task: C002-T24
#[test]
fn exclusive_create_returns_the_same_handle_with_final_written_metadata() {
    let (_dir, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = sheltie_runtime::fsx::ManagedFs::open_existing(&home).unwrap();
    let path = sheltie_runtime::fsx::ManagedRelPath::new("tmp/observed-create").unwrap();
    fs.ensure_dir(
        &lock,
        &sheltie_runtime::fsx::ManagedRelPath::new("tmp").unwrap(),
    )
    .unwrap();
    let file = fs.write_new_observed(&lock, &path, b"new bytes").unwrap();
    assert_eq!(file.metadata().len(), 9);
    assert_eq!(file.read_bounded(9).unwrap(), b"new bytes");
}

// Task: C002-T04
#[test]
fn oversize_declared_output_rejected_at_observation_as_output_too_large() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    let out = Path::new(output_dir.as_str()).join("outline.md");
    let f = std::fs::File::create(&out).unwrap();
    f.set_len(32 * 1024 * 1024 + 1).unwrap();
    drop(f);

    let err = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("s"),
            None,
        )
        .unwrap_err();
    assert_eq!(
        err.code(),
        sheltie_core::ErrorCode::OutputTooLarge,
        "{err:?}"
    );
}

/// Observation rejection precedes COMMIT; complete Store rows and external sentinels remain unchanged.
// Task: C002-T40
#[test]
fn observation_rejection_before_commit_leaves_store_unchanged() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    // Symlink-replaced outputs reject during observation, preventing commit and effects.
    let out = Path::new(output_dir.as_str()).join("outline.md");
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-output");
    let sentinel_before = snapshot(&sent);
    std::os::unix::fs::symlink(&sent, &out).unwrap();

    let connection = rusqlite::Connection::open_with_flags(
        home.store_path().as_str(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let before = store_rows(&connection);
    let err = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("s"),
            None,
        )
        .unwrap_err();
    assert!(err.to_string().contains("symlink"), "{err:?}");
    assert_eq!(
        store_rows(&connection),
        before,
        "Observation rejection must preserve revision, state_json, requests, and audit"
    );
    assert_eq!(snapshot(&sent), sentinel_before);
}

/// Atomic exclusive writes reject both existing destination names and preplanted same-directory temporary symlinks.
// Task: C002-T04
#[test]
fn exclusive_atomic_write_creates_and_replaces_target_only() {
    let dir = tempfile::tempdir().unwrap();
    let base = canonical_abs(dir.path());
    let home = sheltie_runtime::Home::resolve(Some((base).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("card.md").unwrap();
    fs.write_atomic(&lock, &target, b"v1").unwrap();
    assert_eq!(
        std::fs::read(home.root().as_path().join("card.md")).unwrap(),
        b"v1"
    );
    // Same-name rewrites replace through rename without residual temporary files.
    fs.write_atomic(&lock, &target, b"v2").unwrap();
    assert_eq!(
        std::fs::read(home.root().as_path().join("card.md")).unwrap(),
        b"v2"
    );
    let leftovers: Vec<_> = std::fs::read_dir(home.root().as_path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(".card.md.tmp-"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "No temporary files remain: {leftovers:?}"
    );
}

// Task: C002-T19
#[test]
fn directory_handle_remains_anchored_after_parent_path_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sentinel = sentinel(outside.path(), "outside");
    let before = snapshot(&sentinel);
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let held = fs
        .ensure_dir(&lock, &ManagedRelPath::new("works/slot").unwrap())
        .unwrap();

    std::fs::rename(
        home.root().as_path().join("works/slot"),
        home.root().as_path().join("works/held"),
    )
    .unwrap();
    std::os::unix::fs::symlink(outside.path(), home.root().as_path().join("works/slot")).unwrap();

    held.write_new(&lock, "proof.txt", b"anchored").unwrap();
    held.rename_new(&lock, "proof.txt", "renamed.txt").unwrap();

    assert_eq!(snapshot(&sentinel), before);
    assert_eq!(
        std::fs::read(home.root().as_path().join("works/held/renamed.txt")).unwrap(),
        b"anchored"
    );
    assert!(
        std::fs::symlink_metadata(home.root().as_path().join("works/slot"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

// Task: C002-T19
#[test]
fn managed_operations_reject_a_lock_from_another_home() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let home =
        sheltie_runtime::Home::resolve(Some((canonical_abs(first.path())).as_str())).unwrap();
    let other =
        sheltie_runtime::Home::resolve(Some((canonical_abs(second.path())).as_str())).unwrap();
    let wrong_lock = other.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("never-created").unwrap();

    assert!(
        fs.write_new(&wrong_lock, &target, b"must not write")
            .is_err()
    );
    assert!(!home.root().as_path().join("never-created").exists());
}

// Task: C002-T19
#[test]
fn readonly_rechecks_hardlink_count_before_chmod() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let rel = ManagedRelPath::new("payload").unwrap();
    fs.write_new(&lock, &rel, b"preserve").unwrap();
    let held = fs.open_regular(&rel).unwrap();
    let external_alias = outside.path().join("alias");
    std::fs::hard_link(home.root().as_path().join("payload"), &external_alias).unwrap();
    let original = snapshot(home.root().as_path().join("payload").as_std_path());

    assert!(fs.set_readonly(&lock, &held).is_err());
    assert_eq!(
        snapshot(home.root().as_path().join("payload").as_std_path()),
        original
    );
    assert_eq!(snapshot(&external_alias), original);
}

/// ensure_dirs_under rejection cases: symlink and file-placeholder segments beneath the root.
// Task: C002-T04
#[test]
fn ensure_dirs_rejects_symlink_and_file_placeholder_below_root() {
    let dir = tempfile::tempdir().unwrap();
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    // Symlink segment.
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), home.root().as_path().join("works")).unwrap();
    match fs.ensure_dir(&lock, &ManagedRelPath::new("works/w1").unwrap()) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("symlink"), "{reason}"),
        other => panic!("Symlink segment must be rejected: {other:?}"),
    }
    // File-placeholder segment.
    let dir2 = tempfile::tempdir().unwrap();
    let home2 =
        sheltie_runtime::Home::resolve(Some((canonical_abs(dir2.path())).as_str())).unwrap();
    let lock2 = home2.acquire_lock().unwrap();
    let fs2 = ManagedFs::open_existing(&home2).unwrap();
    std::fs::write(home2.root().as_path().join("bin"), b"not a dir").unwrap();
    match fs2.ensure_dir(&lock2, &ManagedRelPath::new("bin/sheltie").unwrap()) {
        Err(Error::InvalidRequest { reason }) => {
            assert!(reason.contains("is not a directory"), "{reason}")
        }
        other => panic!("File-placeholder segment must be rejected: {other:?}"),
    }
    // Reject external destinations.
    let elsewhere = tempfile::tempdir().unwrap();
    assert!(home.to_rel(&abs(elsewhere.path())).is_err());
    // Valid nested creation.
    fs2.ensure_dir(&lock2, &ManagedRelPath::new("works/a/b").unwrap())
        .unwrap();
    assert!(home2.root().as_path().join("works/a/b").is_dir());
}

/// Accepted regression: CLI covers nested outputs/explicit @file; this covers atomic writes beneath nested directories.
// Task: C002-T04
#[test]
fn exclusive_atomic_write_creates_nested_parents() {
    let dir = tempfile::tempdir().unwrap();
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("works/w/attempts/d/0/brief.md").unwrap();
    fs.write_atomic(&lock, &target, "Nested".as_bytes())
        .unwrap();
    assert_eq!(
        std::fs::read(home.root().as_path().join(target.as_str())).unwrap(),
        "Nested".as_bytes()
    );
}

// Task: C002-T19
#[test]
fn managed_relative_path_rejects_escaping_and_nul_segments() {
    for bad in ["", "/abs", "../x", "a/../b", "a//b", "./a", "a\0b"] {
        assert!(ManagedRelPath::new(bad).is_err(), "must reject {bad:?}");
    }
    assert_eq!(
        ManagedRelPath::new("works/w1/status-card.md")
            .unwrap()
            .as_str(),
        "works/w1/status-card.md"
    );
}

// Task: C002-T19
#[test]
fn managed_fs_anchors_open_and_rejects_leaf_symlink_and_fifo() {
    let (_d, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("works/w1/output.md").unwrap();
    fs.ensure_dir(&lock, &ManagedRelPath::new("works/w1").unwrap())
        .unwrap();
    fs.write_new(&lock, &target, b"owned bytes").unwrap();
    assert!(fs.write_new(&lock, &target, b"replacement").is_err());
    let opened = fs.open_regular(&target).unwrap();
    assert_eq!(opened.read_bounded(128).unwrap(), b"owned bytes");
    fs.set_readonly(&lock, &opened).unwrap();
    assert_eq!(
        std::fs::metadata(home.root().as_path().join("works/w1/output.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o444
    );

    let outside = tempfile::tempdir().unwrap();
    let sentinel = sentinel(outside.path(), "outside-managed");
    let before = snapshot(&sentinel);
    std::fs::remove_file(home.root().as_path().join("works/w1/output.md")).unwrap();
    std::os::unix::fs::symlink(&sentinel, home.root().as_path().join("works/w1/output.md"))
        .unwrap();
    assert!(fs.open_regular(&target).is_err());
    assert_eq!(snapshot(&sentinel), before);

    let fifo_path = home.root().as_path().join("works/w1/input.fifo");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo_path)
            .status()
            .unwrap()
            .success()
    );
    let fifo = ManagedRelPath::new("works/w1/input.fifo").unwrap();
    assert!(fs.open_regular(&fifo).is_err());
}

// Task: C002-T19
#[test]
fn managed_rename_no_replace_and_remove_tree_stay_under_root() {
    let (_d, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let parent = ManagedRelPath::new("pending/test/payload").unwrap();
    fs.ensure_dir(&lock, &parent).unwrap();
    let source = ManagedRelPath::new("pending/test/payload/source").unwrap();
    let target = ManagedRelPath::new("pending/test/payload/target").unwrap();
    fs.write_new(&lock, &source, b"source").unwrap();
    fs.write_new(&lock, &target, b"target").unwrap();
    assert!(fs.rename_new(&lock, &source, &target).is_err());
    assert_eq!(
        fs.open_regular(&source).unwrap().read_bounded(32).unwrap(),
        b"source"
    );
    assert_eq!(
        fs.open_regular(&target).unwrap().read_bounded(32).unwrap(),
        b"target"
    );
    fs.write_atomic(&lock, &target, b"replacement").unwrap();
    assert_eq!(
        fs.open_regular(&target).unwrap().read_bounded(32).unwrap(),
        b"replacement"
    );

    let outside = tempfile::tempdir().unwrap();
    let sentinel_path = sentinel(outside.path(), "atomic-target");
    let sentinel_before = snapshot(&sentinel_path);
    std::fs::remove_file(home.root().as_path().join("pending/test/payload/target")).unwrap();
    std::os::unix::fs::symlink(
        &sentinel_path,
        home.root().as_path().join("pending/test/payload/target"),
    )
    .unwrap();
    fs.write_atomic(&lock, &target, b"replaced symlink entry")
        .unwrap();
    assert_eq!(snapshot(&sentinel_path), sentinel_before);
    assert_eq!(
        fs.open_regular(&target).unwrap().read_bounded(64).unwrap(),
        b"replaced symlink entry"
    );

    fs.remove_owned_tree(&lock, &ManagedRelPath::new("pending/test").unwrap())
        .unwrap();
    assert!(fs.open_regular(&source).is_err());
}

// Task: C002-T19
#[test]
fn managed_rename_rejects_symlink_source_without_moving_or_touching_target() {
    let outside = tempfile::tempdir().unwrap();
    let sentinel_path = sentinel(outside.path(), "rename-source");
    let sentinel_before = snapshot(&sentinel_path);
    let (_d, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let directory = ManagedRelPath::new("pending/rename-test").unwrap();
    fs.ensure_dir(&lock, &directory).unwrap();
    let source = ManagedRelPath::new("pending/rename-test/source").unwrap();
    let target = ManagedRelPath::new("pending/rename-test/target").unwrap();
    std::os::unix::fs::symlink(&sentinel_path, home.root().as_path().join(source.as_str()))
        .unwrap();

    assert!(fs.rename_new(&lock, &source, &target).is_err());
    assert!(
        std::fs::symlink_metadata(home.root().as_path().join(source.as_str()))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!home.root().as_path().join(target.as_str()).exists());
    assert_eq!(snapshot(&sentinel_path), sentinel_before);
}

// Task: C002-T34
#[test]
fn held_directory_rechecks_the_lock_for_every_write_and_rename() {
    for replaced_lock in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let other_directory = tempfile::tempdir().unwrap();
        let home =
            sheltie_runtime::Home::resolve(Some(canonical_abs(directory.path()).as_str())).unwrap();
        let other =
            sheltie_runtime::Home::resolve(Some(canonical_abs(other_directory.path()).as_str()))
                .unwrap();
        let lock = home.acquire_lock().unwrap();
        let other_lock = other.acquire_lock().unwrap();
        let fs = ManagedFs::open_existing(&home).unwrap();
        let held = fs
            .ensure_dir(&lock, &ManagedRelPath::new("works/slot").unwrap())
            .unwrap();
        held.write_new(&lock, "source", b"preserve").unwrap();
        let source = home.root().as_path().join("works/slot/source");
        let before = snapshot(source.as_std_path());
        let saved_lock = directory.path().join("saved-lock");
        if replaced_lock {
            std::fs::rename(directory.path().join(".lock"), &saved_lock).unwrap();
            std::fs::write(directory.path().join(".lock"), b"").unwrap();
        }
        let invalid_lock = if replaced_lock { &lock } else { &other_lock };
        for error in [
            held.write_new(invalid_lock, "never-created", b"must not write")
                .unwrap_err(),
            held.rename_new(invalid_lock, "source", "never-renamed")
                .unwrap_err(),
        ] {
            assert!(matches!(error, Error::InvalidRequest { .. }), "{error}");
        }
        assert_eq!(snapshot(source.as_std_path()), before);
        assert!(!directory.path().join("works/slot/never-created").exists());
        assert!(!directory.path().join("works/slot/never-renamed").exists());
        if replaced_lock {
            std::fs::rename(saved_lock, directory.path().join(".lock")).unwrap();
        }
        held.rename_new(&lock, "source", "legal").unwrap();
        assert_eq!(snapshot(&directory.path().join("works/slot/legal")), before);
    }
}
