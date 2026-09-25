//! T24：`skills/sheltie/SKILL.md` 里出现的每条命令都真实存在。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use assert_cmd::Command;

// Task: T24
#[test]
#[ignore = "T24"]
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
    assert!(seen >= 8, "skill 里至少要演示八条命令，实际 {seen}");
}
