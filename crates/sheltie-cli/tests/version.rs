//! T01：二进制能跑。这是骨架里唯一不禁用的端到端测试。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;

#[test]
fn t01_version_prints_name_and_version() {
    Command::cargo_bin("sheltie")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout("sheltie 0.1.0\n");
}

#[test]
fn t01_unknown_subcommand_exits_2() {
    Command::cargo_bin("sheltie")
        .unwrap()
        .arg("frobnicate")
        .assert()
        .code(2);
}
