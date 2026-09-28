//! T12：管理根解析、路径约束、文件观察。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use sheltie_core::path::RelPath;
use sheltie_runtime::observe::{build_resource_index, observe_file};
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

// Task: T12
#[test]
fn observe_file_rejects_symlink_and_directory() {
    let (d, _home) = temp_home();
    let real = d.path().join("real.txt");
    std::fs::write(&real, "x").unwrap();
    std::os::unix::fs::symlink(&real, d.path().join("link.txt")).unwrap();
    assert!(observe_file(&abs(&d.path().join("link.txt"))).is_err());
    assert!(observe_file(&abs(d.path())).is_err());
    assert!(matches!(
        observe_file(&abs(&d.path().join("nope"))),
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

// Task: T12
#[test]
fn observe_file_sha256_matches_known_vector() {
    let (d, _home) = temp_home();
    let p = d.path().join("hello.txt");
    std::fs::write(&p, "hello\n").unwrap();
    let o = observe_file(&abs(&p)).unwrap();
    assert_eq!(
        o.sha256.as_str(),
        "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
    );
    assert_eq!(o.bytes, 6);
}

// Task: T12
#[test]
fn resource_index_marks_non_utf8() {
    let (d, _home) = temp_home();
    std::fs::create_dir_all(d.path().join("sub")).unwrap();
    std::fs::write(d.path().join("a.md"), "文字").unwrap();
    std::fs::write(d.path().join("sub/b.bin"), [0xff, 0xfe, 0x00]).unwrap();
    let idx = build_resource_index(&abs(d.path())).unwrap();
    assert!(idx.get(&RelPath::new("a.md").unwrap()).unwrap().is_utf8);
    assert!(
        !idx.get(&RelPath::new("sub/b.bin").unwrap())
            .unwrap()
            .is_utf8
    );
    assert_eq!(idx.files.len(), 2);
}
