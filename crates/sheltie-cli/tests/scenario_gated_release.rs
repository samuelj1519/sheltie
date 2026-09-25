//! T22：门槛场景。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

fn blocked_at_gate(env: &Env) -> (String, serde_json::Value) {
    env.add_example("gated-release");
    let wid = env.start("gated-release", &[("version", "1.2.0")]);
    let b = env.begin(&wid, "notes");
    let s = env.submit_all(&wid, &b, "写好了");
    (wid, s)
}

// Task: T22
#[test]
fn gate_node_success_blocks_work_and_next_has_only_approve_and_cancel() {
    let env = Env::new();
    let (_wid, s) = blocked_at_gate(&env);
    assert_eq!(s["data"]["work_status"]["kind"], "blocked");
    assert_eq!(s["data"]["work_status"]["reason"], "gate");
    assert_eq!(next_ops(&s), vec!["gate approve", "work cancel"]);
}

// Task: T22
#[test]
fn begin_next_node_before_approve_is_illegal_next() {
    let env = Env::new();
    let (wid, _) = blocked_at_gate(&env);
    let (e, code) = env.fail(&["attempt", "begin", &wid, "--node", "archive"]);
    assert_eq!(code, 1);
    assert_eq!(e["error"]["code"], "ILLEGAL_NEXT");
    assert!(
        e["error"]["detail"]["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("gate approve"))
    );
}

// Task: T22
#[test]
fn approve_records_os_user_and_unblocks() {
    let env = Env::new();
    let (wid, _) = blocked_at_gate(&env);
    let v = env.ok(&["gate", "approve", &wid, "--node", "notes"]);
    assert_eq!(next_begin_nodes(&v), vec!["archive"]);
    let card = std::fs::read_to_string(env.work_dir(&wid).join("status-card.md")).unwrap();
    assert!(card.contains("status: active"));
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".into());
    assert_eq!(v["data"]["by"], user);
}

// Task: T22
#[test]
fn approve_on_terminal_gate_node_succeeds_work() {
    let env = Env::new();
    let src = env.dir.path().join("gate-only");
    copy_dir(&example_dir("gated-release"), &src);
    let flow = src.join("flows/default.toml");
    let text = std::fs::read_to_string(&flow).unwrap();
    let cut = text.find("[[nodes]]\nid = \"archive\"").unwrap();
    std::fs::write(&flow, &text[..cut]).unwrap();
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let wid = env.start("gated-release", &[("version", "1.2.0")]);
    let b = env.begin(&wid, "notes");
    env.submit_all(&wid, &b, "写好了");
    let v = env.ok(&["gate", "approve", &wid, "--node", "notes"]);
    assert_eq!(env.status(&wid)["data"]["status"]["kind"], "succeeded");
    assert!(v["next"].as_array().unwrap().is_empty());
}
