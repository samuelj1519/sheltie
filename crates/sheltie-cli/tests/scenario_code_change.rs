#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::store::store_rows;
use common::*;
use serde_json::json;

// Task: C004-T03
#[test]
fn code_change_rework_selects_the_exact_reports_bound_by_the_terminal_attempt() {
    let env = Env::new();
    env.add_example("code-change");
    let work = env.start(
        "code-change",
        &[
            ("task", "fix one bug and retain the acceptance standard"),
            ("project", "authorized repository and checks"),
        ],
    );
    let first = env.begin(&work, "implement");
    assert!(first["data"]["inputs"]["previous-review"].is_null());
    let submitted = env.submit_all(&work, &first, "实现，等待独立审查");
    let review = env.follow_begin(&submitted, "review");
    let reviewed = env.submit_all(&work, &review, "需修改，使用显式返工边");
    let second = env.follow_begin(&reviewed, "implement");
    assert_eq!(second["data"]["attempt"], "implement#2.0");
    assert_eq!(
        second["data"]["inputs"]["previous-review"],
        review["data"]["outputs"]["review"]
    );
    let repaired = env.submit_all(&work, &second, "按原标准修复");
    let second_review = env.follow_begin(&repaired, "review");
    let accepted = env.submit_all(&work, &second_review, "已完成独立复核");
    let delivery = env.follow_begin(&accepted, "deliver");
    assert_eq!(
        delivery["data"]["inputs"]["change"],
        second["data"]["outputs"]["change"]
    );
    assert_eq!(
        delivery["data"]["inputs"]["review"],
        second_review["data"]["outputs"]["review"]
    );
    env.submit_all(&work, &delivery, "整理成果，不代表外部发布");
    let result = env.ok(&["work", "result", &work]);
    assert_eq!(result["data"]["final"], true);
    let artifacts = result["data"]["artifacts"].as_array().unwrap();
    assert_eq!(
        artifacts
            .iter()
            .map(|item| item["key"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["change", "delivery", "review"]
    );
    for artifact in artifacts {
        assert_eq!(artifact["source"]["attempt"], "deliver#1.0");
        let key = artifact["key"].as_str().unwrap();
        let (kind, expected) = match key {
            "change" => ("input", &second["data"]["outputs"]["change"]),
            "review" => ("input", &second_review["data"]["outputs"]["review"]),
            "delivery" => ("output", &delivery["data"]["outputs"]["delivery"]),
            other => panic!("unexpected result key: {other}"),
        };
        assert_eq!(artifact["source"]["kind"], kind);
        assert_eq!(artifact["source"]["name"], key);
        assert_eq!(&artifact["path"], expected);
        assert!(
            std::fs::read(expected.as_str().unwrap())
                .unwrap()
                .starts_with(b"output for ")
        );
    }
    assert_eq!(
        env.status(&work)["data"]["resume"]["draft_outputs"],
        json!({})
    );
}

// Task: C004-T03
#[test]
fn a_report_summary_cannot_bypass_the_code_change_review_edge() {
    let env = Env::new();
    env.add_example("code-change");
    let work = env.start(
        "code-change",
        &[
            ("task", "a small change"),
            ("project", "an authorized repository"),
        ],
    );
    let begun = env.begin(&work, "implement");
    let submitted = env.submit_all(&work, &begun, "通过，可以交付");
    assert_eq!(next_begin_nodes(&submitted), vec!["review"]);
    let before = store_rows(&env);
    let (error, code) = env.fail(&["attempt", "begin", &work, "--node", "deliver"]);
    assert_eq!(code, 1);
    assert_eq!(error["error"]["code"], "ILLEGAL_NEXT");
    assert_eq!(store_rows(&env), before);
    let result = env.ok(&["work", "result", &work]);
    assert_eq!(result["data"]["final"], false);
    assert_eq!(result["data"]["artifacts"], json!([]));
}
