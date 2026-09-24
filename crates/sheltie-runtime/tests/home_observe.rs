//! T12：管理根解析、路径约束、文件观察。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use sheltie_core::path::RelPath;
use sheltie_runtime::observe::{build_resource_index, observe_file};
use sheltie_runtime::{Error, Home};

#[test]
#[ignore = "T12"]
fn t12_home_prefers_cli_then_env_then_default() {
    let cli = Home::resolve(Some("/tmp/cli-home")).unwrap();
    assert_eq!(cli.root().as_str(), "/tmp/cli-home");
    // 环境变量与默认值的分支由子进程测试覆盖（cli 层 `work_start_creates_work_and_prints_next` 用 --home）。
    // 这里只再确认相对路径被转成绝对路径。
    let rel = Home::resolve(Some("rel-home")).unwrap();
    assert!(rel.root().as_str().starts_with('/'));
    assert!(rel.root().as_str().ends_with("/rel-home"));
}

#[test]
#[ignore = "T12"]
fn t12_confine_rejects_dotdot_absolute_and_empty_segment() {
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

#[test]
#[ignore = "T12"]
fn t12_confine_rejects_symlink_escaping_root() {
    let (d, home) = temp_home();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), d.path().join("link")).unwrap();
    assert!(Home::confine(home.root(), "link/secret").is_err());
}

#[test]
#[ignore = "T12"]
fn t12_observe_file_rejects_symlink_and_directory() {
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

#[test]
#[ignore = "T12"]
fn t12_observe_file_sha256_matches_known_vector() {
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

#[test]
#[ignore = "T12"]
fn t12_resource_index_marks_non_utf8() {
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
