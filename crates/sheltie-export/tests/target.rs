#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
#[cfg(feature = "failpoint")]
use std::path::Path;
use std::path::PathBuf;

use sheltie_core::digest::Sha256Hex;
use sheltie_core::ids::{FlowId, WorkId, WorkbookId};
use sheltie_core::path::AbsPath;
use sheltie_core::work::{WorkStatus, WorkbookRef};
use sheltie_export::Error;
use sheltie_export::model::{Artifact, ArtifactSource, CopiedFile, SelectedResult, SlotKind};
use sheltie_export::target::{Destination, Staging};

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    parent: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let home = root.join("home");
        let parent = root.join("destination");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&parent).unwrap();
        Self {
            _temp: temp,
            root,
            home,
            parent,
        }
    }
    fn stage(&self) -> Staging {
        Destination::open(&self.home, &self.parent)
            .unwrap()
            .stage(&work())
            .unwrap()
    }
}

fn work() -> WorkId {
    WorkId::parse("2026-10-03-001-test").unwrap()
}

fn artifact(key: &str, leaf: &str) -> Artifact {
    Artifact {
        key: key.to_string().try_into().unwrap(),
        path: AbsPath::new(format!("/source/{leaf}")).unwrap(),
        sha256: Sha256Hex::new(
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".to_string(),
        )
        .unwrap(),
        bytes: 5,
        source: ArtifactSource {
            attempt: "deliver#1.0".to_string(),
            kind: SlotKind::Output,
            name: key.to_string().try_into().unwrap(),
        },
    }
}

fn result(artifacts: Vec<Artifact>) -> SelectedResult {
    SelectedResult {
        format: "work-result/v1".to_string(),
        work_id: work(),
        revision: 7,
        workbook: WorkbookRef {
            id: WorkbookId::new("test").unwrap(),
            version: "1.0.0".to_string(),
            digest: Sha256Hex::new("a".repeat(64)).unwrap(),
        },
        flow: FlowId::new("default").unwrap(),
        status: WorkStatus::Succeeded,
        effects_pending: false,
        r#final: true,
        artifacts,
    }
}

fn receive(stage: &mut Staging, index: usize, artifact: &Artifact) -> CopiedFile {
    let mut writer = stage.create_artifact(index, artifact).unwrap();
    writer.write_all(b"hello").unwrap();
    stage.finish_artifact(writer, artifact).unwrap()
}

fn final_candidate(stage: &Staging) -> PathBuf {
    let path = stage.staging_path().unwrap();
    let random = path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix(".sheltie-export-")
        .unwrap();
    path.parent().unwrap().join(format!("{}-{random}", work()))
}

// Task: C006-T01
#[test]
fn native_directory_publication_preserves_bytes_manifest_and_private_modes() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let selected = result(vec![artifact("", "same.txt"), artifact("中文", "same.txt")]);
    let files = selected
        .artifacts
        .iter()
        .enumerate()
        .map(|(index, artifact)| receive(&mut stage, index, artifact))
        .collect::<Vec<_>>();
    let target = stage.publish(&selected, &files).unwrap();
    assert!(stage.staging_path().is_none());
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for (index, file) in files.iter().enumerate() {
        assert_eq!(
            file.path.as_str(),
            format!("artifacts/{:04}/same.txt", index + 1)
        );
        let path = target.join(file.path.as_str());
        assert_eq!(fs::read(&path).unwrap(), b"hello");
        let metadata = fs::metadata(&path).unwrap();
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(
            fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(target.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["format"], "work-export-manifest/v1");
    assert_eq!(manifest["result"]["revision"], 7);
    assert_eq!(manifest["files"][0]["key"], "");
    assert_eq!(manifest["files"][1]["key"], "中文");
    assert_eq!(manifest["files"][0]["path"], "artifacts/0001/same.txt");
    assert_eq!(manifest["files"][0]["bytes"], 5);
    assert_eq!(
        manifest["files"][0]["sha256"],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    assert_eq!(
        fs::metadata(target.join("manifest.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

// Task: C006-T01
#[test]
fn atomic_noreplace_refuses_existing_directory_without_changing_competitor() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let selected = result(vec![artifact("a", "a.txt")]);
    let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
    let target = final_candidate(&stage);
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel"), b"untouched").unwrap();
    let identity = fs::metadata(&target).unwrap();
    assert!(matches!(
        stage.publish(&selected, &files),
        Err(Error::Io {
            operation: "publish_noreplace",
            ..
        })
    ));
    assert_eq!(fs::read(target.join("sentinel")).unwrap(), b"untouched");
    let after = fs::metadata(&target).unwrap();
    assert_eq!(after.ino(), identity.ino());
    assert_eq!(after.permissions().mode(), identity.permissions().mode());
    assert!(stage.staging_path().unwrap().is_dir());
}

// Task: C006-T01
#[test]
fn target_rejects_links_nonexistent_relative_and_management_overlap_without_writes() {
    let fixture = Fixture::new();
    let linked = fixture.root.join("linked");
    std::os::unix::fs::symlink(&fixture.parent, &linked).unwrap();
    for parent in [
        &linked,
        &fixture.home,
        &fixture.root,
        &fixture.root.join("missing"),
        &PathBuf::from("relative"),
    ] {
        assert!(Destination::open(&fixture.home, parent).is_err());
    }
    let inside = fixture.home.join("nested");
    fs::create_dir(&inside).unwrap();
    assert!(Destination::open(&fixture.home, &inside).is_err());
    assert_eq!(fs::read_dir(&fixture.parent).unwrap().count(), 0);
    assert!(Destination::open(&linked, &fixture.parent).is_err());
}

// Task: C006-T01
#[test]
fn artifact_requires_safe_leaf_contiguous_index_and_exclusive_single_file() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let original = artifact("a", "a.txt");
    assert!(stage.create_artifact(1, &original).is_err());
    for path in ["/", "/source/..", "/source/a\0.txt", "/source/a\\b"] {
        let mut unsafe_artifact = original.clone();
        unsafe_artifact.path = AbsPath::new(path.to_string()).unwrap();
        assert!(
            stage.create_artifact(0, &unsafe_artifact).is_err(),
            "{path:?}"
        );
    }
    let mut writer = stage.create_artifact(0, &original).unwrap();
    assert!(writer.write_all(b"sixxxx").is_err());
    writer.write_all(b"hello").unwrap();
    stage.finish_artifact(writer, &original).unwrap();
    assert!(stage.create_artifact(1, &original).is_err());
}

// Task: C006-T01
#[test]
fn target_refuses_injected_leaf_symlink_and_hardlink_without_touching_sentinel() {
    for hard in [false, true] {
        let fixture = Fixture::new();
        let mut stage = fixture.stage();
        let artifact = artifact("a", "a.txt");
        let mut writer = stage.create_artifact(0, &artifact).unwrap();
        writer.write_all(b"hello").unwrap();
        let path = stage.staging_path().unwrap().join("artifacts/0001/a.txt");
        let sentinel = fixture.root.join("sentinel");
        fs::write(&sentinel, b"untouched").unwrap();
        let before = fs::metadata(&sentinel).unwrap();
        if hard {
            fs::hard_link(&path, fixture.root.join("extra-link")).unwrap();
        } else {
            fs::rename(&path, fixture.root.join("held-original")).unwrap();
            std::os::unix::fs::symlink(&sentinel, &path).unwrap();
        }
        assert!(stage.finish_artifact(writer, &artifact).is_err());
        assert_eq!(fs::read(&sentinel).unwrap(), b"untouched");
        assert_eq!(fs::metadata(&sentinel).unwrap().ino(), before.ino());
        assert_eq!(
            fs::metadata(&sentinel).unwrap().permissions().mode(),
            before.permissions().mode()
        );
        assert!(stage.staging_path().is_some());
    }
}

// Task: C006-T01
#[test]
fn parent_or_staging_path_replacement_stops_writes_to_moved_object() {
    for replace_parent in [false, true] {
        let fixture = Fixture::new();
        let mut stage = fixture.stage();
        let artifact = artifact("a", "a.txt");
        let mut writer = stage.create_artifact(0, &artifact).unwrap();
        let original = stage.staging_path().unwrap();
        let moved = fixture.root.join("moved");
        if replace_parent {
            fs::rename(&fixture.parent, &moved).unwrap();
            fs::create_dir(&fixture.parent).unwrap();
        } else {
            fs::rename(&original, &moved).unwrap();
            fs::create_dir(&original).unwrap();
        }
        assert!(writer.write_all(b"hello").is_err());
        assert!(stage.staging_path().is_none());
        let held = if replace_parent {
            moved.join(original.file_name().unwrap())
        } else {
            moved.clone()
        };
        assert_eq!(fs::read(held.join("artifacts/0001/a.txt")).unwrap(), b"");
        assert_eq!(
            fs::read_dir(if replace_parent {
                fixture.parent
            } else {
                original
            })
            .unwrap()
            .count(),
            0
        );
    }
}

// Task: C006-T01
#[test]
fn final_readback_detects_same_size_rewrite_and_preserves_failed_staging() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let selected = result(vec![artifact("a", "a.txt")]);
    let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
    let staging = stage.staging_path().unwrap();
    fs::write(staging.join("artifacts/0001/a.txt"), b"other").unwrap();
    assert!(matches!(
        stage.publish(&selected, &files),
        Err(Error::Integrity { .. })
    ));
    assert_eq!(
        fs::read(staging.join("artifacts/0001/a.txt")).unwrap(),
        b"other"
    );
    assert!(stage.staging_path().is_some());
    assert_eq!(fs::read_dir(&fixture.parent).unwrap().count(), 1);
}

// Task: C006-T01
#[test]
fn publish_requires_all_finished_files_exact_source_mapping_and_no_extra_objects() {
    let fixture = Fixture::new();
    let selected = result(vec![artifact("a", "a.txt")]);
    let mut stage = fixture.stage();
    let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
    let mut incorrect = files.clone();
    incorrect[0].bytes = 4;
    assert!(stage.publish(&selected, &incorrect).is_err());
    fs::write(stage.staging_path().unwrap().join("extra"), b"unselected").unwrap();
    assert!(stage.publish(&selected, &files).is_err());
    let mut unfinished = fixture.stage();
    let _writer = unfinished
        .create_artifact(0, &selected.artifacts[0])
        .unwrap();
    assert!(unfinished.publish(&selected, &files).is_err());
}

// Task: C006-T01
#[test]
fn repeated_exports_make_independent_new_directories_without_reusing_old_staging() {
    let fixture = Fixture::new();
    let selected = result(vec![artifact("a", "a.txt")]);
    let mut abandoned = fixture.stage();
    let mut writer = abandoned
        .create_artifact(0, &selected.artifacts[0])
        .unwrap();
    writer.write_all(b"he").unwrap();
    let residual = abandoned.staging_path().unwrap();
    drop(writer);
    drop(abandoned);
    let mut stage = fixture.stage();
    let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
    let target = stage.publish(&selected, &files).unwrap();
    assert_eq!(
        fs::read(residual.join("artifacts/0001/a.txt")).unwrap(),
        b"he"
    );
    assert_eq!(
        fs::read(target.join("artifacts/0001/a.txt")).unwrap(),
        b"hello"
    );
    assert!(residual.exists());
}

// Task: C006-T01
#[test]
fn artifact_byte_boundary_accepts_32_mib_and_rejects_one_more_declared_or_written_byte() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let mut large = artifact("large", "large.bin");
    large.bytes = 33_554_432;
    large.sha256 = Sha256Hex::new(
        "05f052c8f6da8ee5228ec291820b559c4be183773b9e97a6b82e30dacff85dd3".to_string(),
    )
    .unwrap();
    let mut over = large.clone();
    over.bytes += 1;
    assert!(stage.create_artifact(0, &over).is_err());
    let mut writer = stage.create_artifact(0, &large).unwrap();
    for _ in 0..512 {
        writer.write_all(&[b'x'; 65536]).unwrap();
    }
    assert!(writer.write_all(b"x").is_err());
    let copied = stage.finish_artifact(writer, &large).unwrap();
    let target = stage.publish(&result(vec![large]), &[copied]).unwrap();
    assert_eq!(
        fs::metadata(target.join("artifacts/0001/large.bin"))
            .unwrap()
            .len(),
        33_554_432
    );
}

// Task: C006-T01
#[test]
fn empty_artifact_and_binary_bytes_are_read_back_without_text_conversion() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let mut empty = artifact("a", "empty.bin");
    empty.bytes = 0;
    empty.sha256 = Sha256Hex::new(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
    )
    .unwrap();
    let mut binary = artifact("b", "binary.bin");
    binary.bytes = 3;
    binary.sha256 = Sha256Hex::new(
        "712450d3c4a79eea9509e75dc1dacdeff58034df538536cfae2da882bd8a0c50".to_string(),
    )
    .unwrap();
    let empty_writer = stage.create_artifact(0, &empty).unwrap();
    let first = stage.finish_artifact(empty_writer, &empty).unwrap();
    let mut binary_writer = stage.create_artifact(1, &binary).unwrap();
    binary_writer.write_all(&[0, 255, 10]).unwrap();
    let second = stage.finish_artifact(binary_writer, &binary).unwrap();
    let target = stage
        .publish(&result(vec![empty, binary]), &[first, second])
        .unwrap();
    assert_eq!(
        fs::read(target.join("artifacts/0001/empty.bin")).unwrap(),
        b""
    );
    assert_eq!(
        fs::read(target.join("artifacts/0002/binary.bin")).unwrap(),
        [0, 255, 10]
    );
}

// Task: C006-T01
#[test]
fn target_declared_total_accepts_256_mib_and_rejects_the_next_file_before_creation() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let mut writers = Vec::new();
    for index in 0..8 {
        let mut declaration = artifact(&format!("file-{index}"), "data.bin");
        declaration.bytes = 33_554_432;
        writers.push(stage.create_artifact(index, &declaration).unwrap());
    }
    let mut over = artifact("file-8", "data.bin");
    over.bytes = 1;
    assert!(stage.create_artifact(8, &over).is_err());
    assert!(
        !stage
            .staging_path()
            .unwrap()
            .join("artifacts/0009")
            .exists()
    );
    assert_eq!(writers.len(), 8);
}

// Task: C006-T01
#[test]
fn private_permission_drift_is_rejected_without_chmod_repair_or_publication() {
    let fixture = Fixture::new();
    let mut stage = fixture.stage();
    let selected = result(vec![artifact("a", "a.txt")]);
    let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
    let path = stage.staging_path().unwrap().join("artifacts/0001/a.txt");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(stage.publish(&selected, &files).is_err());
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o644
    );
    assert!(!final_candidate(&stage).exists());
}

// Task: C006-T01
#[test]
fn foreign_staging_writer_is_rejected_and_two_independent_exports_can_finish_concurrently() {
    let fixture = Fixture::new();
    let selected = result(vec![artifact("a", "a.txt")]);
    let mut first = fixture.stage();
    let mut second = fixture.stage();
    let mut writer = first.create_artifact(0, &selected.artifacts[0]).unwrap();
    writer.write_all(b"hello").unwrap();
    assert!(
        second
            .finish_artifact(writer, &selected.artifacts[0])
            .is_err()
    );
    let mut workers = Vec::new();
    for _ in 0..2 {
        let home = fixture.home.clone();
        let parent = fixture.parent.clone();
        let selected = selected.clone();
        workers.push(std::thread::spawn(move || {
            let mut stage = Destination::open(&home, &parent)
                .unwrap()
                .stage(&work())
                .unwrap();
            let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
            stage.publish(&selected, &files).unwrap()
        }));
    }
    let first = workers.remove(0).join().unwrap();
    let second = workers.remove(0).join().unwrap();
    assert_ne!(first, second);
    assert_eq!(
        fs::read(first.join("artifacts/0001/a.txt")).unwrap(),
        b"hello"
    );
    assert_eq!(
        fs::read(second.join("artifacts/0001/a.txt")).unwrap(),
        b"hello"
    );
}

#[cfg(feature = "failpoint")]
// Task: C006-T01
#[test]
fn sync_failure_before_and_after_atomic_move_reports_distinct_real_visibility() {
    use sheltie_export::target::{TargetPoint, arm_sync_error};
    for point in [
        TargetPoint::BeforeTreeSync,
        TargetPoint::AfterRenameBeforeParentSync,
    ] {
        let fixture = Fixture::new();
        let mut stage = fixture.stage();
        let selected = result(vec![artifact("a", "a.txt")]);
        let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
        let candidate = final_candidate(&stage);
        arm_sync_error(point);
        let error = stage.publish(&selected, &files).unwrap_err();
        if point == TargetPoint::BeforeTreeSync {
            assert!(matches!(error, Error::Io { .. }));
            assert!(!candidate.exists());
            assert!(stage.staging_path().is_some());
        } else {
            assert!(
                matches!(error, Error::PublicationUnconfirmed { target_path: Some(ref path), .. } if path == &candidate)
            );
            assert_eq!(
                fs::read(candidate.join("artifacts/0001/a.txt")).unwrap(),
                b"hello"
            );
            assert!(stage.staging_path().is_none());
        }
    }
}

#[cfg(feature = "failpoint")]
fn wait_ready(directory: &Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !directory.join("ready").exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(feature = "failpoint")]
// Task: C006-T01
#[test]
fn racing_target_directory_wins_without_any_overwrite() {
    use sheltie_export::target::{TargetPoint, arm_rendezvous};
    let fixture = Fixture::new();
    let selected = result(vec![artifact("a", "a.txt")]);
    let mut stage = fixture.stage();
    let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
    let candidate = final_candidate(&stage);
    let channel = fixture.root.join("rendezvous");
    fs::create_dir(&channel).unwrap();
    let worker_channel = channel.clone();
    let worker = std::thread::spawn(move || {
        arm_rendezvous(TargetPoint::BeforeRename, &worker_channel);
        stage.publish(&selected, &files)
    });
    wait_ready(&channel);
    fs::create_dir(&candidate).unwrap();
    fs::write(candidate.join("sentinel"), b"raced").unwrap();
    let before = fs::metadata(&candidate).unwrap();
    fs::write(channel.join("release"), b"release").unwrap();
    assert!(worker.join().unwrap().is_err());
    assert_eq!(fs::read(candidate.join("sentinel")).unwrap(), b"raced");
    assert_eq!(fs::metadata(&candidate).unwrap().ino(), before.ino());
}

#[cfg(feature = "failpoint")]
// Task: C006-T01
#[test]
fn moved_final_identity_is_unconfirmed_without_reporting_a_competitor_as_owned() {
    use sheltie_export::target::{TargetPoint, arm_rendezvous};
    let fixture = Fixture::new();
    let selected = result(vec![artifact("a", "a.txt")]);
    let mut stage = fixture.stage();
    let files = vec![receive(&mut stage, 0, &selected.artifacts[0])];
    let candidate = final_candidate(&stage);
    let channel = fixture.root.join("rendezvous");
    fs::create_dir(&channel).unwrap();
    let worker_channel = channel.clone();
    let worker = std::thread::spawn(move || {
        arm_rendezvous(TargetPoint::AfterRenameBeforeParentSync, &worker_channel);
        stage.publish(&selected, &files)
    });
    wait_ready(&channel);
    let moved = fixture.root.join("moved-final");
    fs::rename(&candidate, &moved).unwrap();
    fs::create_dir(&candidate).unwrap();
    fs::write(candidate.join("sentinel"), b"competitor").unwrap();
    fs::write(channel.join("release"), b"release").unwrap();
    assert!(matches!(
        worker.join().unwrap(),
        Err(Error::PublicationUnconfirmed {
            target_path: None,
            ..
        })
    ));
    assert_eq!(
        fs::read(moved.join("artifacts/0001/a.txt")).unwrap(),
        b"hello"
    );
    assert_eq!(fs::read(candidate.join("sentinel")).unwrap(), b"competitor");
}
