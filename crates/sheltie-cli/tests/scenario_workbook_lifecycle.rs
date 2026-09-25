//! T22：Workbook 生命周期场景。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

// Task: T22
#[test]
fn remove_in_use_workbook_is_rejected_with_work_list() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let (e, _) = env.fail(&["workbook", "remove", "two-step@1.0.0"]);
    assert_eq!(e["error"]["code"], "WORKBOOK_IN_USE");
    assert_eq!(e["error"]["detail"]["works"], serde_json::json!([wid]));
}

// Task: T22
#[test]
fn remove_after_work_succeeds_then_status_still_renders() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let b1 = env.begin(&wid, "outline");
    let s1 = env.submit_all(&wid, &b1, "ok");
    let b2 = env.follow_begin(&s1, "summary");
    env.submit_all(&wid, &b2, "ok");
    env.ok(&["workbook", "remove", "two-step@1.0.0"]);
    let st = env.status(&wid);
    assert_eq!(st["data"]["status"]["kind"], "succeeded");
    assert_eq!(
        st["data"]["done"],
        serde_json::json!(["outline#1", "summary#1"])
    );
}

// Task: T22
#[test]
fn editing_repository_copy_does_not_change_running_work_brief() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let f = env
        .workbook_dir("two-step", "1.0.0")
        .join("instructions/outline.md");
    make_writable(&f);
    std::fs::write(&f, "被改过").unwrap();
    let b = env.begin(&wid, "outline");
    let brief = std::fs::read_to_string(b["data"]["brief_path"].as_str().unwrap()).unwrap();
    assert!(brief.contains("列一份提纲"));
}

// Task: T22
#[test]
fn verify_detects_the_edit() {
    let env = Env::new();
    env.add_example("two-step");
    let f = env
        .workbook_dir("two-step", "1.0.0")
        .join("instructions/outline.md");
    make_writable(&f);
    std::fs::write(&f, "被改过").unwrap();
    let (e, _) = env.fail(&["workbook", "verify", "two-step@1.0.0"]);
    assert_eq!(e["error"]["code"], "WORKBOOK_TAMPERED");
    assert_eq!(e["error"]["detail"]["results"][0]["status"], "tampered");
}

// Task: T22
#[test]
fn add_second_version_marks_it_latest_and_start_defaults_to_it() {
    let env = Env::new();
    env.add_example("two-step");
    let src = env.dir.path().join("v2");
    copy_dir(&example_dir("two-step"), &src);
    let m = std::fs::read_to_string(src.join("workbook.toml"))
        .unwrap()
        .replace("1.0.0", "2.0.0");
    std::fs::write(src.join("workbook.toml"), m).unwrap();
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let list = env.ok(&["workbook", "list"]);
    let rows = list["data"].as_array().unwrap();
    let latest: Vec<&str> = rows
        .iter()
        .filter(|r| r["latest"] == true)
        .map(|r| r["version"].as_str().unwrap())
        .collect();
    assert_eq!(latest, vec!["2.0.0"]);
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
    assert_eq!(v["data"]["workbook"]["version"], "2.0.0");
}
