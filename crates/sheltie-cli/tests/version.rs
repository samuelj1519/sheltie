//! T01: executable smoke test, the skeleton's sole initially enabled end-to-end test.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;

// Task: T01
#[test]
fn version_prints_name_and_version() {
    Command::cargo_bin("sheltie")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(format!("sheltie {}\n", env!("CARGO_PKG_VERSION")));
}

// Task: T17
#[test]
fn unknown_subcommand_exits_2() {
    Command::cargo_bin("sheltie")
        .unwrap()
        .arg("frobnicate")
        .assert()
        .code(2);
}
