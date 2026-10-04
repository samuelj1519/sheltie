//! T12：管理根解析、路径约束、文件观察。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use sheltie_runtime::fsx::{ExternalReadFile, MAX_FILE_BYTES};
use sheltie_runtime::workbook_digest::digest_dir_v2;
use sheltie_runtime::{Error, Home};

// Task: T12
#[test]
fn home_prefers_cli_then_env_then_default() {
    // T04 起根在入口规范化：对最深已存在祖先取真实形式（macOS 的 /tmp → /private/tmp），
    // 余下段保持词法。期望用同一规则独立构造。
    let cli = Home::resolve(Some("/tmp/cli-home")).unwrap();
    let expected = format!(
        "{}/cli-home",
        std::fs::canonicalize("/tmp")
            .unwrap()
            .to_string_lossy()
            .into_owned()
    );
    assert_eq!(cli.root().as_str(), expected);
    // 环境变量与默认值的分支由子进程测试覆盖（cli 层 `work_start_creates_work_and_prints_next` 用 --home）。
    // 这里只再确认相对路径被转成绝对路径。
    let rel = Home::resolve(Some("rel-home")).unwrap();
    assert!(rel.root().as_str().starts_with('/'));
    assert!(rel.root().as_str().ends_with("/rel-home"));
}

// Task: C002-T19
#[test]
fn home_rel_rejects_invalid_paths_instead_of_falling_back_to_root() {
    let (_d, home) = temp_home();
    assert!(home.rel("works/w1/status-card.md").is_ok());
    for bad in ["", "../outside", "a/../outside", "/outside", "a//b", "a\0b"] {
        assert!(home.rel(bad).is_err(), "must reject {bad:?}");
    }
}

// Task: C002-T19
#[test]
fn home_lock_rejects_symlink_and_hardlink_leaf_without_touching_target() {
    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("lock-target");
    std::fs::write(&sentinel, b"keep lock target").unwrap();
    let before = std::fs::read(&sentinel).unwrap();

    let (_d, home) = temp_home();
    std::os::unix::fs::symlink(&sentinel, home.lock_path().as_path()).unwrap();
    assert!(home.acquire_lock().is_err());
    assert_eq!(std::fs::read(&sentinel).unwrap(), before);
    std::fs::remove_file(home.lock_path().as_path()).unwrap();

    let (_d2, hardlink_home) = temp_home();
    std::fs::hard_link(&sentinel, hardlink_home.lock_path().as_path()).unwrap();
    assert!(hardlink_home.acquire_lock().is_err());
    assert_eq!(std::fs::read(&sentinel).unwrap(), before);
}

// Task: T12
#[test]
fn confine_rejects_dotdot_absolute_and_empty_segment() {
    let (_d, home) = temp_home();
    let base = home.root();
    assert!(Home::confine(base, "a/b.md").is_ok());
    for bad in ["../x", "a/../b", "/abs", "a//b", ""] {
        assert!(
            matches!(
                Home::confine(base, bad),
                Err(Error::Core(sheltie_core::Error::InvalidPath { .. }))
            ),
            "{bad:?}"
        );
    }
}

// Task: T12
#[test]
fn confine_rejects_symlink_escaping_root() {
    let (d, home) = temp_home();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), d.path().join("link")).unwrap();
    assert!(Home::confine(home.root(), "link/secret").is_err());
}

// Task: C009-T02
#[test]
fn external_file_rejects_symlink_directory_and_missing() {
    let (d, _home) = temp_home();
    let real = d.path().join("real.txt");
    std::fs::write(&real, "x").unwrap();
    std::os::unix::fs::symlink(&real, d.path().join("link.txt")).unwrap();
    assert!(ExternalReadFile::open_regular(&abs(&d.path().join("link.txt"))).is_err());
    assert!(ExternalReadFile::open_regular(&abs(d.path())).is_err());
    assert!(matches!(
        ExternalReadFile::open_regular(&abs(&d.path().join("nope"))),
        Err(Error::NotFound { .. })
    ));
}

// Task: T12
#[test]
fn confine_errors_on_unreadable_ancestor() {
    let (d, home) = temp_home();
    let locked = d.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let r = Home::confine(home.root(), "locked/x.md");
    // 恢复权限再断言，否则 TempDir 收尾删不掉（fe7f5fe 的教训）。
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(r.is_err(), "不可读祖先上的观察错误不得当成不存在放行");
}

// Task: C009-T02
#[test]
fn external_file_sha256_matches_known_vector() {
    let (d, _home) = temp_home();
    let p = d.path().join("hello.txt");
    std::fs::write(&p, "hello\n").unwrap();
    let file = ExternalReadFile::open_regular(&abs(&p)).unwrap();
    let (sha256, bytes) = file.sha256_bounded(MAX_FILE_BYTES).unwrap();
    assert_eq!(
        sha256.as_str(),
        "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
    );
    assert_eq!(bytes, 6);
}

// Task: C002-T33
#[test]
fn home_resolution_preserves_permission_errors_from_an_existing_ancestor() {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().unwrap();
    let denied = directory.path().join("denied");
    std::fs::create_dir(&denied).unwrap();
    std::fs::set_permissions(&denied, std::fs::Permissions::from_mode(0o000)).unwrap();
    let result = Home::resolve(Some(denied.join("child").to_str().unwrap()));
    std::fs::set_permissions(&denied, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        matches!(result, Err(Error::Io { source, .. }) if source.kind() == std::io::ErrorKind::PermissionDenied)
    );
}

// Task: C002-T33
#[cfg(feature = "failpoint")]
#[test]
fn external_reads_do_not_follow_a_temporary_alias_to_the_same_observed_object() {
    use sheltie_runtime::failpoint;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            let _ = failpoint::disarm_rendezvous();
        }
    }
    let _reset = Reset;
    for directory in [false, true] {
        let area = tempfile::tempdir().unwrap();
        let source = area.path().join("source");
        std::fs::create_dir(&source).unwrap();
        let target = source.join(if directory { "nested" } else { "file.txt" });
        if directory {
            std::fs::create_dir(&target).unwrap();
            std::fs::write(target.join("file.txt"), b"original bytes").unwrap();
        } else {
            std::fs::write(&target, b"original bytes").unwrap();
        }
        let target = if directory {
            std::fs::canonicalize(&target).unwrap()
        } else {
            target
        };
        let scope = target.to_str().unwrap();
        let before = tempfile::tempdir().unwrap();
        let after = tempfile::tempdir().unwrap();
        let prefix = if directory { "directory" } else { "regular" };
        failpoint::arm_rendezvous(&format!("{prefix}_open_after_stat"), scope, before.path())
            .unwrap();
        let worker_source = abs(&source);
        let worker_target = abs(&target);
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let result = if directory {
                digest_dir_v2(&worker_source).map(|_| ())
            } else {
                ExternalReadFile::open_regular(&worker_target)
                    .and_then(|file| file.sha256_bounded(MAX_FILE_BYTES))
                    .map(|_| ())
            };
            sender.send(result).unwrap();
        });
        let mut worker = RendezvousWorker::new(worker, before.path(), after.path());
        let deadline = Instant::now() + Duration::from_secs(10);
        while !before.path().join("reached").exists() {
            assert!(
                Instant::now() < deadline,
                "first open boundary was not reached"
            );
            assert!(
                receiver.try_recv().is_err(),
                "observer ended before its boundary"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let parked = area.path().join("original");
        std::fs::rename(&target, &parked).unwrap();
        std::os::unix::fs::symlink(&parked, &target).unwrap();
        failpoint::arm_rendezvous(&format!("{prefix}_open_after_open"), scope, after.path())
            .unwrap();
        std::fs::write(before.path().join("release"), b"release").unwrap();
        let result = loop {
            if let Ok(result) = receiver.try_recv() {
                break result;
            }
            if after.path().join("reached").exists() {
                std::fs::remove_file(&target).unwrap();
                std::fs::rename(&parked, &target).unwrap();
                std::fs::write(after.path().join("release"), b"release").unwrap();
                break receiver.recv_timeout(Duration::from_secs(10)).unwrap();
            }
            assert!(
                Instant::now() < deadline,
                "observer did not reject or finish opening"
            );
            std::thread::sleep(Duration::from_millis(1));
        };
        worker.finish().unwrap();
        if parked.exists() {
            std::fs::remove_file(&target).unwrap();
            std::fs::rename(&parked, &target).unwrap();
        }
        assert!(
            result.is_err(),
            "an alias must not be followed even when it names the original inode"
        );
        assert_eq!(
            std::fs::read(if directory {
                target.join("file.txt")
            } else {
                target
            })
            .unwrap(),
            b"original bytes"
        );
        failpoint::disarm_rendezvous().unwrap();
    }
}

// Task: C002-T33
#[test]
fn confine_does_not_treat_an_unreadable_existing_base_as_missing() {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().unwrap();
    let locked = directory.path().join("locked");
    let base = locked.join("base");
    std::fs::create_dir_all(&base).unwrap();
    let root = abs(&base);
    assert_eq!(
        Home::confine(&root, "next.txt").unwrap(),
        root.join_segment("next.txt")
    );
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let result = Home::confine(&root, "next.txt");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        matches!(result, Err(Error::Io { source, .. }) if source.kind() == std::io::ErrorKind::PermissionDenied)
    );
    assert!(!base.join("next.txt").exists());
}
