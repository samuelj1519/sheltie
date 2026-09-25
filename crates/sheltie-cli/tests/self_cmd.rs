//! T20：`self` 组（cli 层）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

// Task: T20
#[test]
#[ignore = "T20"]
fn self_version_works_without_home() {
    let env = Env::new();
    // 管理根目录存在但里面什么都没有；self version 不需要 store.db。
    let v = env.ok(&["self", "version"]);
    assert_eq!(v["data"]["version"], "0.1.0");
    assert_eq!(v["data"]["schema_version"], 1);
    assert!(!env.dir.path().join("store.db").exists());
}
