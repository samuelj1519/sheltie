//! T22：产物完整性场景。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;

#[test]
#[ignore = "T22"]
fn t22_modifying_upstream_output_makes_downstream_begin_fail_with_artifact_modified() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let b = env.begin(&wid, "outline");
    env.submit_all(&wid, &b, "ok");
    let outline = env.work_dir(&wid).join("attempts/outline/1/0/outline.md");
    make_writable(&outline);
    std::fs::write(&outline, "偷偷改了").unwrap();
    let (e, _) = env.fail(&["attempt", "begin", &wid, "--node", "summary"]);
    assert_eq!(e["error"]["code"], "ARTIFACT_MODIFIED");
    assert_eq!(e["error"]["detail"]["input"], "outline");
}

#[test]
#[ignore = "T22"]
fn t22_submit_without_required_output_is_output_missing_and_attempt_stays_running() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.begin(&wid, "outline");
    let (e, _) = env.fail(&[
        "attempt",
        "submit",
        &wid,
        "--attempt",
        "outline#1.0",
        "--summary",
        "空手",
    ]);
    assert_eq!(e["error"]["code"], "OUTPUT_MISSING");
    assert_eq!(e["error"]["detail"]["output"], "outline");
    assert_eq!(
        env.status(&wid)["data"]["last_attempt"]["status"],
        "running"
    );
}

#[test]
#[ignore = "T22"]
fn t22_submit_oversize_output_is_output_too_large() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let b = env.begin(&wid, "outline");
    let out = Path::new(b["data"]["output_dir"].as_str().unwrap()).join("outline.md");
    std::fs::write(&out, vec![b'x'; 65_537]).unwrap();
    let (e, _) = env.fail(&[
        "attempt",
        "submit",
        &wid,
        "--attempt",
        "outline#1.0",
        "--summary",
        "太大",
    ]);
    assert_eq!(e["error"]["code"], "OUTPUT_TOO_LARGE");
}

#[test]
#[ignore = "T22"]
fn t22_submit_with_symlink_output_is_rejected() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let b = env.begin(&wid, "outline");
    let out = Path::new(b["data"]["output_dir"].as_str().unwrap()).join("outline.md");
    std::os::unix::fs::symlink("/etc/hosts", &out).unwrap();
    let (e, code) = env.fail(&[
        "attempt",
        "submit",
        &wid,
        "--attempt",
        "outline#1.0",
        "--summary",
        "软链",
    ]);
    assert_eq!(code, 1);
    assert_ne!(e["ok"], true);
}
