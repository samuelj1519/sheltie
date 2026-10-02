//! T21：审查回环场景。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;

fn start_review(env: &Env) -> String {
    env.add_example("article-review");
    env.start("article-review", &[("topic", "为什么要写测试")])
}

/// draft#n → review#n 不通过，返回 review 提交后的响应。
fn draft_then_review_fail(env: &Env, wid: &str) -> serde_json::Value {
    let b = env.begin(wid, "draft");
    let s = env.submit_all(wid, &b, "初稿");
    let r = env.follow_begin(&s, "review");
    env.submit_all(wid, &r, "不通过。第二段论据不足。")
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
    let s2 = env.submit_all(&wid, &b2, "改了");
    let r2 = env.follow_begin(&s2, "review");
    assert_eq!(r2["data"]["attempt"], "review#2.0");
    let s3 = env.submit_all(&wid, &r2, "通过");
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
    let s2 = env.submit_all(&wid, &b2, "改了");
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
    let s = env.submit_all(&wid, &b, "初稿");
    let r = env.follow_begin(&s, "review");
    let s2 = env.submit_all(&wid, &r, "通过");
    let p = env.follow_begin(&s2, "publish");
    let brief = std::fs::read_to_string(p["data"]["brief_path"].as_str().unwrap()).unwrap();
    assert!(brief.contains("执行者: human"));
    assert!(brief.contains(&format!(
        "sheltie attempt submit {wid} --attempt publish#1.0"
    )));
    let done = env.submit_all(&wid, &p, "定稿");
    assert_eq!(done["data"]["work_status"]["kind"], "succeeded");
}

// Task: T21
#[test]
fn review_brief_lists_checklist_resource_with_frozen_path() {
    let env = Env::new();
    let wid = start_review(&env);
    let b = env.begin(&wid, "draft");
    let s = env.submit_all(&wid, &b, "初稿");
    let r = env.follow_begin(&s, "review");
    let checklist = r["data"]["inputs"]["checklist"].as_str().unwrap();
    // T04 起管理根在入口规范化：macOS 的 /var 是 /private/var 的软链，记录的路径
    // 是真实形式，期望值同样 canonicalize（cwd/tmp 目录由此经过软链解析）。
    let frozen = env
        .work_dir(&wid)
        .join("workbook/resources/review-checklist.md");
    let frozen = std::fs::canonicalize(&frozen).unwrap();
    assert_eq!(Path::new(checklist), frozen);
    let repo_copy = env
        .workbook_dir("article-review", "1.0.0")
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
    // 未绑定的可选输入在 inputs 里占一行、值是 null（协议 §3 attempt begin 的返回说明）。
    // 用 get 断言：Index 对缺失键同样给 Null，固定不了 key 的存在性。
    assert_eq!(
        b["data"]["inputs"].as_object().unwrap().get("review"),
        Some(&serde_json::Value::Null)
    );
    let brief = std::fs::read_to_string(b["data"]["brief_path"].as_str().unwrap()).unwrap();
    // 合同 §4 模板：可选且未绑定写「尚无（上游 <node> 还没有产出）」。
    assert!(brief.contains("| review | 尚无（上游 review 还没有产出） | |"));
    assert!(brief.contains("来自: 入口"));
    // 说明书讲清首次没有意见（合同 §3.2 回环标准写法）。
    assert!(brief.contains("首次开工它标「尚无」"));
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
    // 期望路径手写，不从引擎的布局 helper 生成。
    assert!(
        review.ends_with("/attempts/review/occurrence-001/attempt-000/outputs/review.md"),
        "{review}"
    );
    let brief = std::fs::read_to_string(b2["data"]["brief_path"].as_str().unwrap()).unwrap();
    // 来源 Occurrence 在「来自」行，产物路径在输入表。
    assert!(brief.contains("来自: review#1（back 边）"));
    assert!(brief.contains(&format!("| review | {review} | ")));
    assert!(brief.contains("逐条回应审查意见再改"));
    // 只传文件路径、不内联历史正文：意见正文只在绑定的文档里。
    let body = std::fs::read_to_string(&review).unwrap();
    assert_eq!(body, "output for review#1.0\n");
    assert!(!brief.contains("output for review#1.0"));
    assert!(!brief.contains("不通过。第二段论据不足。"));
}

// Task: C002-T11
#[test]
fn later_back_to_draft_binds_latest_review_occurrence() {
    let env = Env::new();
    let wid = start_review(&env);
    let after_review = draft_then_review_fail(&env, &wid);
    let b2 = env.follow_begin(&after_review, "draft");
    let s2 = env.submit_all(&wid, &b2, "改了");
    let r2 = env.follow_begin(&s2, "review");
    let s3 = env.submit_all(&wid, &r2, "不通过。结论段还要再看。");
    let b3 = env.follow_begin(&s3, "draft");
    assert_eq!(b3["data"]["attempt"], "draft#3.0");
    // 只改轮次这一个条件：绑定的是最近一次成功 review 的产物，不是第一轮的。
    let review = b3["data"]["inputs"]["review"].as_str().unwrap().to_string();
    assert!(
        review.ends_with("/attempts/review/occurrence-002/attempt-000/outputs/review.md"),
        "{review}"
    );
    let brief = std::fs::read_to_string(b3["data"]["brief_path"].as_str().unwrap()).unwrap();
    assert!(brief.contains("来自: review#2（back 边）"));
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
    // 只去掉 required = false 这一个条件，其余与样例相同。
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
