#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::*;
use sheltie_core::ids::{AttemptId, NodeId, WorkId};
use sheltie_runtime::{Home, StartArgs, WorkService};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn fixture(bytes: &[u8]) -> (OwnedTempDir, Home, WorkService, WorkId, PathBuf) {
    let (directory, home) = temp_home();
    let source = directory.path().join("source");
    std::fs::create_dir_all(source.join("flows")).unwrap();
    std::fs::write(source.join("workbook.toml"),"schema='workbook/v1'\nid='raw-fixture'\nversion='1.0.0'\nname='Raw'\nflows=['flows/default.toml']\n").unwrap();
    std::fs::write(source.join("flows/default.toml"),"schema='flow/v1'\nid='default'\nentry='finish'\n[[nodes]]\nid='finish'\ntitle='Finish'\nexecutor='agent'\ninstruction={text='Write the output.'}\noutputs=[{name='payload',path='payload.bin',result=true,max_bytes=33554432}]\n").unwrap();
    repo(&home).add(&abs(&source), None).unwrap();
    let service = common::service(&home);
    let work = work_id_of(
        &service
            .start(
                StartArgs {
                    workbook_id: "raw-fixture".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: inputs_lit(&[]),
                },
                None,
            )
            .unwrap(),
    );
    let begun = service
        .begin(&work, &NodeId::new("finish").unwrap(), None)
        .unwrap();
    let output = PathBuf::from(output_dir_of(&begun).join_segment("payload.bin").as_str());
    std::fs::write(&output, bytes).unwrap();
    service
        .submit(
            &work,
            &AttemptId::parse("finish#1.0").unwrap(),
            &lit("done"),
            None,
        )
        .unwrap();
    (directory, home, service, work, output)
}

// Task: C006-T01
#[test]
fn raw_reader_preserves_binary_bytes_and_store_rows_without_taking_a_lock() {
    let bytes = b"\0\xff\xfe\nlast byte\0";
    let (_directory, home, service, work, _) = fixture(bytes);
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let before = store_rows(&connection);
    let lock = std::fs::File::open(home.lock_path().as_path()).unwrap();
    fs4::fs_std::FileExt::lock_exclusive(&lock).unwrap();
    let mut received = Vec::new();
    service
        .write_result_artifact(&work, "payload", 3, &mut received)
        .unwrap();
    assert_eq!(received, bytes);
    assert_eq!(store_rows(&connection), before);
    fs4::fs_std::FileExt::unlock(&lock).unwrap();
}

// Task: C006-T01
#[test]
fn raw_reader_rejects_revision_key_and_pending_effects_before_output() {
    let (_directory, home, service, work, _) = fixture(b"bytes");
    for (key, revision) in [("payload", 2), ("unknown", 3)] {
        let mut output = Vec::new();
        assert!(
            service
                .write_result_artifact(&work, key, revision, &mut output)
                .is_err()
        );
        assert!(output.is_empty());
    }
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    connection.execute("UPDATE requests SET published=0 WHERE work_id=?1 AND request_id IN (SELECT request_id FROM audit WHERE revision=3)",[work.as_str()]).unwrap();
    let before = store_rows(&connection);
    let mut output = Vec::new();
    assert!(
        service
            .write_result_artifact(&work, "payload", 3, &mut output)
            .is_err()
    );
    assert!(output.is_empty());
    assert_eq!(store_rows(&connection), before);
}

// Task: C006-T01
#[test]
fn raw_reader_rejects_modified_truncated_and_unsafe_originals() {
    for kind in ["changed", "truncated", "symlink", "hardlink"] {
        let (directory, _home, service, work, path) = fixture(b"original bytes");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let sentinel = directory.path().join("sentinel");
        std::fs::write(&sentinel, b"outside original").unwrap();
        match kind {
            "changed" => std::fs::write(&path, b"differentbytes").unwrap(),
            "truncated" => std::fs::write(&path, b"short").unwrap(),
            "symlink" => {
                std::fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink(&sentinel, &path).unwrap();
            }
            "hardlink" => std::fs::hard_link(&path, directory.path().join("alias")).unwrap(),
            _ => unreachable!(),
        }
        assert!(
            service
                .write_result_artifact(&work, "payload", 3, &mut Vec::new())
                .is_err(),
            "{kind}"
        );
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"outside original");
    }
}

// Task: C006-T01
#[test]
fn raw_reader_rejects_corrupt_persisted_reference_before_reading_external_bytes() {
    let (directory, home, service, work, _) = fixture(b"original bytes");
    let sentinel = directory.path().join("outside.bin");
    std::fs::write(&sentinel, b"outside bytes").unwrap();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let raw: String = connection
        .query_row(
            "SELECT state_json FROM works WHERE work_id=?1",
            [work.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let mut state: serde_json::Value = serde_json::from_str(&raw).unwrap();
    state["attempts"][0]["outputs"]["payload"]["path"] =
        serde_json::json!(sentinel.to_str().unwrap());
    connection
        .execute(
            "UPDATE works SET state_json=?1 WHERE work_id=?2",
            rusqlite::params![state.to_string(), work.as_str()],
        )
        .unwrap();
    let before = store_rows(&connection);
    let mut output = Vec::new();
    assert!(
        service
            .write_result_artifact(&work, "payload", 3, &mut output)
            .is_err()
    );
    assert!(output.is_empty());
    assert_eq!(store_rows(&connection), before);
    assert_eq!(std::fs::read(sentinel).unwrap(), b"outside bytes");
}

struct MutatingWriter {
    original: PathBuf,
    action: &'static str,
    touched: bool,
    bytes: Vec<u8>,
}
impl Write for MutatingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        if !self.touched {
            self.touched = true;
            match self.action {
                "grow" => {
                    std::fs::OpenOptions::new()
                        .append(true)
                        .open(&self.original)?
                        .write_all(b"growth")?;
                }
                "truncate" => {
                    std::fs::OpenOptions::new()
                        .write(true)
                        .open(&self.original)?
                        .set_len(0)?;
                }
                "parent" => {
                    let parent = self.original.parent().unwrap();
                    let moved = parent.with_file_name("retained-outputs");
                    std::fs::rename(parent, &moved)?;
                    std::fs::create_dir(parent)?;
                    std::fs::rename(moved.join("payload.bin"), &self.original)?;
                }
                "leaf" => {
                    std::fs::rename(&self.original, self.original.with_file_name("retained.bin"))?;
                    std::fs::write(&self.original, b"replacement")?;
                }
                _ => unreachable!(),
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Task: C006-T01
#[test]
fn raw_reader_rejects_growth_truncation_and_parent_or_leaf_swaps_during_streaming() {
    for action in ["grow", "truncate", "parent", "leaf"] {
        let (_directory, home, service, work, path) = fixture(&vec![b'x'; 131072]);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let before = store_rows(&connection);
        let mut writer = MutatingWriter {
            original: path,
            action,
            touched: false,
            bytes: Vec::new(),
        };
        assert!(
            service
                .write_result_artifact(&work, "payload", 3, &mut writer)
                .is_err(),
            "{action}"
        );
        assert!(!writer.bytes.is_empty());
        assert_eq!(store_rows(&connection), before);
    }
}

// Task: C006-T01
#[test]
fn raw_reader_accepts_exact_32_mib_and_rejects_writer_failure() {
    let (_directory, _home, service, work, _) = fixture(&vec![0_u8; 33554432]);
    service
        .write_result_artifact(&work, "payload", 3, &mut std::io::sink())
        .unwrap();
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("writer refused"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(
        service
            .write_result_artifact(&work, "payload", 3, &mut Broken)
            .is_err()
    );
}

// Task: C006-T01
#[cfg(feature = "failpoint")]
#[test]
fn raw_reader_rechecks_identity_after_the_final_byte_was_proved() {
    for action in ["leaf", "same_inode"] {
        let (_directory, home, service, work, path) = fixture(b"verified bytes");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let rendezvous = tempfile::tempdir().unwrap();
        sheltie_runtime::failpoint::arm_rendezvous(
            "result_artifact_after_read",
            path.to_str().unwrap(),
            rendezvous.path(),
        )
        .unwrap();
        let reader_service = service.clone();
        let reader_work = work.clone();
        let reader = std::thread::spawn(move || {
            let mut output = Vec::new();
            (
                reader_service.write_result_artifact(&reader_work, "payload", 3, &mut output),
                output,
            )
        });
        let mut reader = RendezvousWorker::single(reader, rendezvous.path());
        reader.wait("raw读取没有到达最终身份核验前同步点");
        if action == "leaf" {
            std::fs::rename(&path, path.with_file_name("retained.bin")).unwrap();
            std::fs::write(&path, b"replacement").unwrap();
        } else {
            std::fs::write(&path, b"modified bytes").unwrap();
        }
        let (result, output) = reader.finish().unwrap();
        assert!(result.is_err());
        assert_eq!(output, b"verified bytes");
        if action == "leaf" {
            assert_eq!(
                std::fs::read(
                    home.work_dir(&work)
                        .join_segment("attempts")
                        .join_segment("finish")
                        .join_segment("occurrence-001")
                        .join_segment("attempt-000")
                        .join_segment("outputs")
                        .join_segment("retained.bin")
                        .as_path()
                )
                .unwrap(),
                b"verified bytes"
            );
        }
    }
}
