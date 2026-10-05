//! T21: review-loop scenarios.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;

fn start_review(env: &Env) -> String {
    env.add_example("article-review");
    env.start("article-review", &[("topic", "Why write tests")])
}

/// draft#n -> review#n with rejection; return the review-submission response.
fn draft_then_review_fail(env: &Env, wid: &str) -> serde_json::Value {
    let b = env.begin(wid, "draft");
    let s = env.submit_all(wid, &b, "Draft");
    let r = env.follow_begin(&s, "review");
    env.submit_all(wid, &r, "Rejected. Paragraph two lacks evidence.")
}

// Task: T21
#[test]
fn review_back_edge_creates_second_draft_occurrence() {
    let env = Env::new();
    let wid = start_review(&env);
    let after_review = draft_then_review_fail(&env, &wid);
    let b2 = env.follow_begin(&after_review, "draft");
    assert_eq!(b2["data"]["attempt"], "draft#2.0");
    assert_eq!(b2["data"]["occurrence"], 2);
    let s2 = env.submit_all(&wid, &b2, "Revised");
    let r2 = env.follow_begin(&s2, "review");
    assert_eq!(r2["data"]["attempt"], "review#2.0");
    let s3 = env.submit_all(&wid, &r2, "Approved");
    let p = env.follow_begin(&s3, "publish");
    assert_eq!(p["data"]["attempt"], "publish#1.0");
    let article = p["data"]["inputs"]["article"].as_str().unwrap();
    assert!(
        article.contains("/attempts/draft/occurrence-002/attempt-000/outputs/article.md"),
        "{article}"
    );
}

// Task: T21
#[test]
fn next_after_review_offers_both_main_and_back_with_kinds() {
    let env = Env::new();
    let wid = start_review(&env);
    let v = draft_then_review_fail(&env, &wid);
    let next = v["next"].as_array().unwrap();
    let publish = next
        .iter()
        .find(|n| n["args"]["node"] == "publish")
        .unwrap();
    let draft = next.iter().find(|n| n["args"]["node"] == "draft").unwrap();
    assert_eq!(publish["edge"], "main");
    assert_eq!(draft["edge"], "back");
    assert_eq!(publish["executor"], "human");
    assert_eq!(draft["tier"], "standard");
}

// Task: T21
#[test]
fn max_visits_exhaustion_blocks_with_no_legal_edge() {
    let env = Env::new();
    let src = env.dir.path().join("ar1");
    copy_dir(&example_dir("article-review"), &src);
    let flow = src.join("flows/default.toml");
    let text = std::fs::read_to_string(&flow).unwrap().replacen(
        "max_visits = 3\n\n[[nodes]]\nid = \"publish\"",
        "max_visits = 1\n\n[[nodes]]\nid = \"publish\"",
        1,
    );
    std::fs::write(&flow, text).unwrap();
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let wid = env.start("article-review", &[("topic", "x")]);
    let after_review = draft_then_review_fail(&env, &wid);
    let b2 = env.follow_begin(&after_review, "draft");
    let s2 = env.submit_all(&wid, &b2, "Revised");
    assert!(!next_begin_nodes(&s2).contains(&"review".to_string()));
    assert_eq!(s2["data"]["work_status"]["kind"], "blocked");
    assert_eq!(s2["data"]["work_status"]["reason"], "no_legal_edge");
}

// Task: T21
#[test]
fn human_executor_node_is_begun_and_submitted_like_agent() {
    let env = Env::new();
    let wid = start_review(&env);
    let b = env.begin(&wid, "draft");
    let s = env.submit_all(&wid, &b, "Draft");
    let r = env.follow_begin(&s, "review");
    let s2 = env.submit_all(&wid, &r, "Approved");
    let p = env.follow_begin(&s2, "publish");
    let brief = std::fs::read_to_string(p["data"]["brief_path"].as_str().unwrap()).unwrap();
    assert!(brief.contains("Executor: human"));
    assert!(brief.contains(&format!(
        "sheltie attempt submit {wid} --attempt publish#1.0"
    )));
    let done = env.submit_all(&wid, &p, "Finalized");
    assert_eq!(done["data"]["work_status"]["kind"], "succeeded");
}

// Task: T21
#[test]
fn review_brief_lists_checklist_resource_with_frozen_path() {
    let env = Env::new();
    let wid = start_review(&env);
    let b = env.begin(&wid, "draft");
    let s = env.submit_all(&wid, &b, "Draft");
    let r = env.follow_begin(&s, "review");
    let checklist = r["data"]["inputs"]["checklist"].as_str().unwrap();
    // Since T04, normalize the root on entry; macOS /var links to /private/var, so recorded paths
    // are canonical, as are expectations; cwd/tmp ancestors resolve symlinks.
    let frozen = env
        .work_dir(&wid)
        .join("workbook/resources/review-checklist.md");
    let frozen = std::fs::canonicalize(&frozen).unwrap();
    assert_eq!(Path::new(checklist), frozen);
    let repo_copy = env
        .workbook_dir("article-review", "1.0.1")
        .join("resources/review-checklist.md");
    assert_eq!(
        std::fs::read(&frozen).unwrap(),
        std::fs::read(&repo_copy).unwrap()
    );
}

// Task: C002-T11
#[test]
fn first_draft_marks_review_input_absent_without_body() {
    let env = Env::new();
    let wid = start_review(&env);
    let b = env.begin(&wid, "draft");
    assert_eq!(b["data"]["attempt"], "draft#1.0");
    // Unbound optional inputs retain a null slot in inputs (protocol §3 attempt begin).
    // Use get assertions; Index returns Null for missing keys too, failing to prove key presence.
    assert_eq!(
        b["data"]["inputs"].as_object().unwrap().get("review"),
        Some(&serde_json::Value::Null)
    );
    let brief = std::fs::read_to_string(b["data"]["brief_path"].as_str().unwrap()).unwrap();
    // Protocol §4: unbound optional inputs name the upstream node without claiming output availability.
    assert!(
        brief.contains("| review | Not available (upstream review has not produced output) | |")
    );
    assert!(brief.contains("From: entry"));
    // Instructions explain absent first-visit feedback (contract §3.2 standard loop).
    assert!(brief.contains("On first arrival it is unavailable"));
    assert!(!brief.contains("attempts/review/"));
}

// Task: C002-T11
#[test]
fn back_to_draft_binds_review_verdict_path_with_source_occurrence() {
    let env = Env::new();
    let wid = start_review(&env);
    let after_review = draft_then_review_fail(&env, &wid);
    let b2 = env.follow_begin(&after_review, "draft");
    assert_eq!(b2["data"]["attempt"], "draft#2.0");
    let review = b2["data"]["inputs"]["review"].as_str().unwrap().to_string();
    // Handwrite expected paths without engine layout helpers.
    assert!(
        review.ends_with("/attempts/review/occurrence-001/attempt-000/outputs/review.md"),
        "{review}"
    );
    let brief = std::fs::read_to_string(b2["data"]["brief_path"].as_str().unwrap()).unwrap();
    // The From line carries the incoming Occurrence; input tables carry artifact paths.
    assert!(brief.contains("From: review#1 (back edge)"));
    assert!(brief.contains(&format!("| review | {review} | ")));
    assert!(brief.contains("address each finding rather than rewording alone"));
    // Pass file paths without inline historical bodies; feedback content remains only in the bound document.
    let body = std::fs::read_to_string(&review).unwrap();
    assert_eq!(body, "output for review#1.0\n");
    assert!(!brief.contains("output for review#1.0"));
    assert!(!brief.contains("Rejected. Paragraph two lacks evidence."));
}

// Task: C002-T11
#[test]
fn later_back_to_draft_binds_latest_review_occurrence() {
    let env = Env::new();
    let wid = start_review(&env);
    let after_review = draft_then_review_fail(&env, &wid);
    let b2 = env.follow_begin(&after_review, "draft");
    let s2 = env.submit_all(&wid, &b2, "Revised");
    let r2 = env.follow_begin(&s2, "review");
    let s3 = env.submit_all(&wid, &r2, "Rejected. Review the conclusion again.");
    let b3 = env.follow_begin(&s3, "draft");
    assert_eq!(b3["data"]["attempt"], "draft#3.0");
    // Change only the round; bind the latest successful review output rather than the first round.
    let review = b3["data"]["inputs"]["review"].as_str().unwrap().to_string();
    assert!(
        review.ends_with("/attempts/review/occurrence-002/attempt-000/outputs/review.md"),
        "{review}"
    );
    let brief = std::fs::read_to_string(b3["data"]["brief_path"].as_str().unwrap()).unwrap();
    assert!(brief.contains("From: review#2 (back edge)"));
    let body = std::fs::read_to_string(&review).unwrap();
    assert_eq!(body, "output for review#2.0\n");
}

// Task: C002-T11
#[test]
fn required_review_verdict_blocks_first_draft_begin() {
    let env = Env::new();
    let src = env.dir.path().join("ar-required");
    copy_dir(&example_dir("article-review"), &src);
    let flow = src.join("flows/default.toml");
    // Change only required = false; preserve the rest of the example.
    let text = std::fs::read_to_string(&flow).unwrap().replacen(
        "{ name = \"review\", from = \"review.verdict\", required = false }",
        "{ name = \"review\", from = \"review.verdict\" }",
        1,
    );
    std::fs::write(&flow, text).unwrap();
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let wid = env.start("article-review", &[("topic", "x")]);
    let (e, code) = env.fail(&["attempt", "begin", &wid, "--node", "draft"]);
    assert_eq!(code, 1);
    assert_eq!(e["error"]["code"], "INPUT_UNAVAILABLE");
    assert_eq!(e["error"]["detail"]["input"], "review");
    assert_eq!(e["error"]["detail"]["node"], "review");
}
