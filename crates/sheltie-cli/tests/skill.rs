//! T24: every command in skills/sheltie/SKILL.md actually exists.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use assert_cmd::Command;

// Task: T24
#[test]
fn every_command_in_skill_exists() {
    let skill = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/sheltie/SKILL.md");
    let text = std::fs::read_to_string(&skill).unwrap();
    let mut seen = 0;
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("sheltie ") else {
            continue;
        };
        let mut words = rest.split_whitespace();
        let group = words.next().unwrap();
        let verb = words.next().unwrap_or("--help");
        Command::cargo_bin("sheltie")
            .unwrap()
            .args([group, verb, "--help"])
            .assert()
            .success();
        seen += 1;
    }
    assert!(
        seen >= 8,
        "Skill must demonstrate at least eight commands; actual {seen}"
    );
}
