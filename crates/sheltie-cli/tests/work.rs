//! T18：`work` 组。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use predicates::prelude::*;

// Task: T18
#[test]
fn work_start_creates_work_and_prints_next() {
    let env = Env::new();
    env.add_example("two-step");
    let v = env.ok(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    assert!(v["data"]["work_id"].as_str().unwrap().ends_with("-default"));
    assert_eq!(next_begin_nodes(&v), vec!["outline"]);
    assert!(next_ops(&v).contains(&"work cancel".to_string()));
    assert!(
        env.work_dir(v["data"]["work_id"].as_str().unwrap())
            .join("status-card.md")
            .exists()
    );
}

// Task: T18
#[test]
fn work_start_missing_input_exits_1_with_input_missing() {
    let env = Env::new();
    env.add_example("two-step");
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "INPUT_MISSING");
}

// Task: T18
#[test]
fn work_start_accepts_at_file_input() {
    let env = Env::new();
    env.add_example("two-step");
    let f = env.dir.path().join("topic.txt");
    std::fs::write(&f, "来自文件").unwrap();
    let wid = env.start("two-step", &[("topic", &format!("@{}", f.display()))]);
    let content = std::fs::read_to_string(env.work_dir(&wid).join("inputs/topic")).unwrap();
    assert_eq!(content, "来自文件");
}

// Task: T18
#[test]
fn work_list_shows_status_and_current() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let v = env.ok(&["work", "list"]);
    let rows = v["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["work_id"], wid);
    assert_eq!(rows[0]["status"]["kind"], "active");
    assert_eq!(rows[0]["current"], "outline#1");
}

// Task: T18
#[test]
fn work_status_prints_status_card() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.cmd_text(&["work", "status", &wid])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("# Work {wid}"))
                .and(predicate::str::contains("current: outline#1")),
        );
}

// Task: T18
#[test]
fn work_stats_prints_table_and_json() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.begin(&wid, "outline");
    env.cmd_text(&["work", "stats", &wid])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("# Stats {wid}"))
                .and(predicate::str::contains("| outline | 1/1 | 1 | 0 |")),
        );
    let v = env.ok(&["work", "stats", &wid]);
    assert_eq!(v["data"]["nodes"][0]["node"], "outline");
    assert_eq!(v["data"]["nodes"][0]["attempts"], 1);
    assert_eq!(v["data"]["nodes"][1]["attempts"], 0);
}

// Task: T18
#[test]
fn work_status_json_matches_schema() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let v = env.status(&wid);
    for key in [
        "work_id",
        "name",
        "workbook",
        "flow",
        "status",
        "current",
        "done",
        "pending",
        "visits",
        "last_attempt",
        "next",
    ] {
        assert!(v["data"].get(key).is_some(), "缺 {key}");
    }
    assert_eq!(v["data"]["current"], "outline#1");
}

// Task: T18
#[test]
fn work_cancel_then_any_write_is_work_terminal() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let v = env.ok(&["work", "cancel", &wid]);
    assert!(v["next"].as_array().unwrap().is_empty());
    let (e, _) = env.fail(&["attempt", "begin", &wid, "--node", "outline"]);
    assert_eq!(e["error"]["code"], "WORK_TERMINAL");
}

// Task: T18
#[test]
fn work_id_prefix_resolves_when_unique() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let prefix = &wid[..14];
    assert_eq!(env.status(prefix)["data"]["work_id"], wid);
}

// Task: T18
#[test]
fn work_id_prefix_ambiguous_lists_candidates() {
    let env = Env::new();
    env.add_example("two-step");
    let a = env.start("two-step", &[("topic", "x")]);
    let b = env.start("two-step", &[("topic", "y")]);
    let (e, _) = env.fail(&["work", "status", &a[..10]]);
    assert_eq!(e["error"]["code"], "INVALID_REQUEST");
    let msg = e["error"].to_string();
    assert!(msg.contains(&a) && msg.contains(&b));
}

// Task: T18
#[test]
fn work_start_default_name_is_flow_id() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    assert!(wid.ends_with("-default"));
}

// Task: T18
#[test]
fn work_start_with_chinese_name_creates_matching_directory() {
    let env = Env::new();
    env.add_example("two-step");
    let v = env.ok(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--name",
        "文章 初稿",
        "--input",
        "topic=x",
    ]);
    let wid = v["data"]["work_id"].as_str().unwrap();
    assert!(wid.ends_with("-文章-初稿"), "{wid}");
    assert!(env.work_dir(wid).is_dir());
}
