//! T17：`workbook` 组。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use predicates::prelude::*;

// Task: T17
#[test]
fn workbook_add_prints_id_version_digest() {
    let env = Env::new();
    env.cmd_text(&["workbook", "add", example_dir("two-step").to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("two-step")
                .and(predicate::str::contains("1.0.0"))
                .and(predicate::str::is_match("[0-9a-f]{64}").unwrap()),
        );
}

// Task: T17
#[test]
fn workbook_add_json_has_ok_true_and_data() {
    let env = Env::new();
    let v = env.add_example("two-step");
    assert_eq!(v["data"]["id"], "two-step");
    assert_eq!(v["data"]["flows"], serde_json::json!(["default"]));
    assert!(v["next"].as_array().unwrap().is_empty());
}

// Task: T17
#[test]
fn workbook_add_invalid_dir_exits_1_with_workbook_invalid() {
    let env = Env::new();
    let bad = env.dir.path().join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    std::fs::write(bad.join("workbook.toml"), "schema = \"workbook/v9\"\n").unwrap();
    let (v, code) = env.fail(&["workbook", "add", bad.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "WORKBOOK_INVALID");
}

// Task: T17
#[test]
fn workbook_list_after_add_shows_one_row_marked_latest() {
    let env = Env::new();
    env.add_example("two-step");
    let v = env.ok(&["workbook", "list"]);
    let rows = v["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["latest"], true);
    env.cmd_text(&["workbook", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("two-step"));
}

// Task: T17
#[test]
fn workbook_show_lists_nodes_edges_and_requires() {
    let env = Env::new();
    env.add_example("article-review");
    let v = env.ok(&["workbook", "show", "article-review"]);
    assert_eq!(v["data"]["flows"][0]["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(v["data"]["flows"][0]["edges"].as_array().unwrap().len(), 3);
    assert!(v["data"]["requires"].as_array().unwrap().is_empty());
}

// Task: T17
#[test]
fn workbook_remove_without_version_exits_2() {
    let env = Env::new();
    env.add_example("two-step");
    let (_v, code) = env.fail(&["workbook", "remove", "two-step"]);
    assert_eq!(code, 2);
}

// Task: T17
#[test]
fn workbook_remove_then_list_is_empty() {
    let env = Env::new();
    env.add_example("two-step");
    env.ok(&["workbook", "remove", "two-step@1.0.0"]);
    assert!(
        env.ok(&["workbook", "list"])["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

// Task: T17
#[test]
fn workbook_verify_exits_1_after_tamper() {
    let env = Env::new();
    env.add_example("two-step");
    env.ok(&["workbook", "verify"]);
    let f = env
        .workbook_dir("two-step", "1.0.0")
        .join("instructions/outline.md");
    make_writable(&f);
    std::fs::write(&f, "改了").unwrap();
    let (v, code) = env.fail(&["workbook", "verify"]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "WORKBOOK_TAMPERED");
}

// Task: C002-T02
#[test]
fn show_lists_ordered_start_inputs_in_json_and_text() {
    let env = Env::new();
    env.add_example("article-review");
    // article-review 只有 draft 声明 start.topic；协调者不用失败 start 探测。
    let v = env.ok(&["workbook", "show", "article-review"]);
    assert_eq!(
        v["data"]["flows"][0]["start_inputs"],
        serde_json::json!(["topic"])
    );
    let out = env
        .cmd_text(&["workbook", "show", "article-review"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("起始输入: topic"), "{text}");

    // 没有起始输入的 Flow 写「无」。用临时目录造一个无键 Workbook。
    let src = env.dir.path().join("nokeys");
    std::fs::create_dir_all(src.join("flows")).unwrap();
    std::fs::write(
        src.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"nokeys\"\nversion = \"1.0.0\"\nname = \"无键\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        src.join("flows/default.toml"),
        "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = { text = \"做。\" }\n",
    )
    .unwrap();
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let v = env.ok(&["workbook", "show", "nokeys"]);
    assert_eq!(v["data"]["flows"][0]["start_inputs"], serde_json::json!([]));
}
