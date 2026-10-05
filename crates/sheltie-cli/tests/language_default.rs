//! English product text with unchanged Unicode user content.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use assert_cmd::Command;
use common::Env;

// Task: C012-T01
#[test]
fn every_command_help_uses_english_without_creating_storage() {
    let env = Env::new();
    let mut commands: Vec<Vec<&str>> = vec![vec![]];
    for (group, verbs) in [
        (
            "self",
            &["install", "update", "rollback", "uninstall", "version"][..],
        ),
        ("workbook", &["add", "list", "show", "remove", "verify"][..]),
        (
            "work",
            &["start", "list", "status", "result", "stats", "cancel"][..],
        ),
        ("attempt", &["begin", "submit", "fail", "replace"][..]),
        ("gate", &["approve"][..]),
    ] {
        commands.push(vec![group]);
        for verb in verbs {
            commands.push(vec![group, verb]);
        }
    }
    for mut args in commands {
        args.push("--help");
        let output = Command::cargo_bin("sheltie")
            .unwrap()
            .args(["--home", &env.home()])
            .args(&args)
            .output()
            .unwrap();
        assert!(output.status.success(), "help failed: {args:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.is_ascii(), "non-English help for {args:?}: {text}");
        assert!(
            text.contains("Usage:"),
            "missing usage for {args:?}: {text}"
        );
        if args == ["--help"] {
            assert!(text.contains("A local workflow engine for coordinator agents"));
        }
    }
    assert!(!env.dir.path().join("store.db").exists());
    assert!(!env.dir.path().join(".lock").exists());
}

// Task: C012-T01
#[test]
fn parameter_errors_use_english_before_storage_access() {
    let env = Env::new();
    let (error, exit) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "missing",
        "--flow",
        "default",
        "--input",
        "broken",
    ]);
    assert_eq!(exit, 2);
    assert_eq!(error["error"]["code"], "INVALID_REQUEST");
    assert_eq!(
        error["error"]["message"],
        "--input value \"broken\" must be k=v"
    );
    assert!(!env.dir.path().join("store.db").exists());
    assert!(!env.dir.path().join(".lock").exists());
}

// Task: C012-T01
#[test]
fn english_status_preserves_chinese_user_inputs_and_summary() {
    let env = Env::new();
    env.add_example("two-step");
    let topic = "用户原文：中文内容与 emoji 🐕\n";
    let work = env.start("two-step", &[("topic", topic)]);
    assert_eq!(
        std::fs::read(env.work_dir(&work).join("start-inputs/topic")).unwrap(),
        topic.as_bytes()
    );
    let begun = env.begin(&work, "outline");
    let brief = std::fs::read_to_string(begun["data"]["brief_path"].as_str().unwrap()).unwrap();
    assert!(brief.starts_with("# Brief: Outline\n\n"));
    assert!(brief.contains("## Inputs\n"));
    assert!(brief.contains("## Output requirements\n"));
    let summary = "用户摘要：保留原文，不自动翻译。";
    env.submit_all(&work, &begun, summary);
    let json = env.status(&work);
    assert_eq!(json["data"]["last_attempt"]["summary"], summary);
    let output = env.cmd_text(&["work", "status", &work]).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("## Current task\n"));
    assert!(text.contains("## Latest Attempt\n"));
    assert!(text.contains("## Legal next actions\n"));
    assert!(text.contains(&format!("summary: {summary}\n")));
}
