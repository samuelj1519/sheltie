#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Env;
use std::fs::{File, FileTimes};
use std::path::Path;
use std::time::{Duration, SystemTime};

fn set_age(path: &Path, age: Duration) {
    File::open(path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(SystemTime::now() - age))
        .unwrap();
}

fn set_link_age(path: &Path) {
    use rustix::fs::{AtFlags, CWD, Timestamps, utimensat};
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - 86_410;
    let time = rustix::fs::Timespec {
        tv_sec: seconds,
        tv_nsec: 0,
    };
    utimensat(
        CWD,
        path,
        &Timestamps {
            last_access: time,
            last_modification: time,
        },
        AtFlags::SYMLINK_NOFOLLOW,
    )
    .unwrap();
}

fn stale_file(env: &Env) -> std::path::PathBuf {
    let path = env.dir.path().join("tmp/stale");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"stale bytes").unwrap();
    set_age(&path, Duration::from_secs(86_410));
    path
}

// Task: C002-T39
#[test]
fn successful_write_cleans_expired_tmp_and_preserves_recent_future_and_pending() {
    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "test")]);
    let stale = stale_file(&env);
    let root = env.dir.path().join("tmp");
    let old_tree = root.join("abandoned-store-init");
    std::fs::create_dir(&old_tree).unwrap();
    std::fs::write(old_tree.join("store.db"), b"private staged database").unwrap();
    set_age(&old_tree, Duration::from_secs(86_410));
    let recent = root.join("recent");
    std::fs::write(&recent, b"recent bytes").unwrap();
    let future = root.join("future");
    std::fs::write(&future, b"future bytes").unwrap();
    File::open(&future)
        .unwrap()
        .set_times(FileTimes::new().set_modified(SystemTime::now() + Duration::from_secs(60)))
        .unwrap();
    let pending = env.dir.path().join("pending/unowned/keep");
    std::fs::create_dir_all(pending.parent().unwrap()).unwrap();
    std::fs::write(&pending, b"pending bytes").unwrap();
    set_age(pending.parent().unwrap(), Duration::from_secs(86_410));

    let reply = env.ok(&["work", "cancel", &work]);
    assert_eq!(reply["data"]["work_status"]["kind"], "cancelled");
    assert!(!stale.exists());
    assert!(!old_tree.exists());
    assert_eq!(std::fs::read(recent).unwrap(), b"recent bytes");
    assert_eq!(std::fs::read(future).unwrap(), b"future bytes");
    assert_eq!(std::fs::read(pending).unwrap(), b"pending bytes");
}

// Task: C002-T39
#[test]
fn readonly_and_failed_calls_preserve_expired_tmp() {
    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "test")]);
    let stale = stale_file(&env);
    env.status(&work);
    assert_eq!(std::fs::read(&stale).unwrap(), b"stale bytes");
    let (error, _) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
    ]);
    assert_eq!(error["error"]["code"], "INPUT_MISSING");
    assert_eq!(std::fs::read(stale).unwrap(), b"stale bytes");
}

// Task: C002-T39
#[test]
fn tmp_cleanup_failure_warns_without_changing_successful_replay() {
    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "test")]);
    let first = env.ok(&["--request-id", "cancel-once", "work", "cancel", &work]);
    let stale = stale_file(&env);
    let linked = env.dir.path().join("tmp/hardlink");
    std::fs::hard_link(&stale, &linked).unwrap();
    let output = env
        .cmd(&["--request-id", "cancel-once", "work", "cancel", &work])
        .output()
        .unwrap();
    assert!(output.status.success());
    let mut replay: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(replay["data"]["replayed"], true);
    replay["data"]["replayed"] = serde_json::json!(false);
    assert_eq!(replay, first);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("tmp维护")
    );
    assert_eq!(std::fs::read(stale).unwrap(), b"stale bytes");
    assert_eq!(std::fs::read(linked).unwrap(), b"stale bytes");
}

// Task: C002-T39
#[test]
fn self_write_also_cleans_expired_tmp() {
    let env = Env::new();
    env.add_example("two-step");
    let stale = stale_file(&env);
    env.ok(&["self", "uninstall"]);
    assert!(!stale.exists());
}

// Task: C002-T39
#[test]
fn expired_leaf_and_nested_links_are_unlinked_without_touching_outside_targets() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "test")]);
    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("sentinel");
    std::fs::write(&sentinel, b"outside bytes").unwrap();
    let mode = std::fs::metadata(&sentinel).unwrap().permissions().mode();
    let tmp = env.dir.path().join("tmp");
    let leaf = tmp.join("old-link");
    symlink(&sentinel, &leaf).unwrap();
    set_link_age(&leaf);
    let tree = tmp.join("old-tree");
    std::fs::create_dir(&tree).unwrap();
    symlink(outside.path(), tree.join("outside-directory")).unwrap();
    set_age(&tree, Duration::from_secs(86_410));
    env.ok(&["work", "cancel", &work]);
    assert!(std::fs::symlink_metadata(leaf).is_err());
    assert!(!tree.exists());
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"outside bytes");
    assert_eq!(
        std::fs::metadata(sentinel).unwrap().permissions().mode(),
        mode
    );
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 1);
}

// Task: C002-T39
#[test]
fn special_tmp_entry_is_preserved_and_warned_after_successful_write() {
    use std::os::unix::fs::FileTypeExt;
    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "test")]);
    let path = env.dir.path().join("tmp/fifo");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    set_link_age(&path);
    let output = env.cmd(&["work", "cancel", &work]).output().unwrap();
    assert!(output.status.success());
    let reply: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(reply["data"]["work_status"]["kind"], "cancelled");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("特殊文件")
    );
    assert!(
        std::fs::symlink_metadata(path)
            .unwrap()
            .file_type()
            .is_fifo()
    );
}

// Task: C002-T39
#[cfg(feature = "failpoint")]
#[test]
fn replaced_expired_directory_is_preserved_and_warned_without_changing_the_reply() {
    use std::process::{Command, Stdio};
    use std::time::Instant;

    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "test")]);
    let args = [
        "--request-id",
        "cancel-before-cleanup",
        "work",
        "cancel",
        &work,
    ];
    let original = env.ok(&args);
    let old = env.dir.path().join("tmp/expired");
    std::fs::create_dir(&old).unwrap();
    std::fs::write(old.join("old-payload"), b"old private tmp").unwrap();
    set_age(&old, Duration::from_secs(86_410));
    let sync = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sheltie"))
        .args(["--home", &env.home(), "--json"])
        .args(args)
        .env("SHELTIE_TEST_RENDEZVOUS_NAME", "delete_before_root_unlink")
        .env("SHELTIE_TEST_RENDEZVOUS_ID", "tmp-cleanup")
        .env("SHELTIE_TEST_RENDEZVOUS_DIR", sync.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !sync.path().join("reached").exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!("过期目录未停在unlink前：{output:?}");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let held = env.dir.path().join("tmp/retained-old");
    std::fs::rename(&old, &held).unwrap();
    std::fs::create_dir(&old).unwrap();
    std::fs::write(old.join("sentinel"), b"replacement bytes").unwrap();
    std::fs::write(sync.path().join("release"), b"release").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let mut reply: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(reply["data"]["replayed"], true);
    reply["data"]["replayed"] = serde_json::json!(false);
    assert_eq!(reply, original);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("tmp维护")
    );
    assert_eq!(
        std::fs::read(old.join("sentinel")).unwrap(),
        b"replacement bytes"
    );
    assert!(held.is_dir());
    assert_eq!(std::fs::read_dir(&held).unwrap().count(), 0);
}
