//! T23: CLI request replay.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

// Task: T23
#[test]
fn same_request_id_same_payload_returns_replayed_true() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let a = env.ok(&[
        "--request-id",
        "r-1",
        "attempt",
        "begin",
        &wid,
        "--node",
        "outline",
    ]);
    let b = env.ok(&[
        "--request-id",
        "r-1",
        "attempt",
        "begin",
        &wid,
        "--node",
        "outline",
    ]);
    assert_eq!(a["data"]["attempt"], b["data"]["attempt"]);
    assert_eq!(b["data"]["replayed"], true);
    assert_eq!(a["revision"], b["revision"]);
}

// Task: T23
#[test]
fn same_request_id_different_payload_is_request_conflict() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.ok(&[
        "--request-id",
        "r-1",
        "attempt",
        "begin",
        &wid,
        "--node",
        "outline",
    ]);
    let (e, _) = env.fail(&[
        "--request-id",
        "r-1",
        "attempt",
        "fail",
        &wid,
        "--attempt",
        "outline#1.0",
        "--reason",
        "x",
    ]);
    assert_eq!(e["error"]["code"], "REQUEST_CONFLICT");
}

// Task: C002-T24
#[test]
fn start_request_replay_does_not_require_the_original_input_file() {
    let env = Env::new();
    env.add_example("two-step");
    let input = env.dir.path().join("topic.txt");
    std::fs::write(&input, "first observed value").unwrap();
    let input_arg = format!("topic=@{}", input.display());
    let args = [
        "--request-id",
        "r-start-file",
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        input_arg.as_str(),
    ];
    let first = env.ok(&args);
    let work = first["data"]["work_id"].as_str().unwrap();
    assert_eq!(
        std::fs::read_to_string(env.work_dir(work).join("start-inputs/topic")).unwrap(),
        "first observed value"
    );
    std::fs::write(&input, "changed after commit").unwrap();
    let changed = env.ok(&args);
    assert_eq!(changed["data"]["replayed"], true);
    assert_eq!(changed["data"]["work_id"], first["data"]["work_id"]);
    std::fs::remove_file(&input).unwrap();
    let replay = env.ok(&args);
    assert_eq!(replay["data"]["replayed"], true);
    assert_eq!(replay["data"]["work_id"], first["data"]["work_id"]);
    assert_eq!(replay["revision"], first["revision"]);
}

// Task: C002-T24
#[test]
fn historical_work_id_resolves_before_a_now_ambiguous_prefix() {
    let env = Env::new();
    env.add_example("two-step");
    let original = env.start("two-step", &[("topic", "first")]);
    let prefix = &original[..10];
    let first = env.ok(&[
        "--request-id",
        "r-historical-work",
        "attempt",
        "begin",
        prefix,
        "--node",
        "outline",
    ]);
    let second = env.start("two-step", &[("topic", "second")]);
    assert_ne!(original, second);
    let ordinary = env.fail(&["work", "status", prefix]).0;
    assert_eq!(ordinary["error"]["code"], "INVALID_REQUEST");

    let replay = env.ok(&[
        "--request-id",
        "r-historical-work",
        "attempt",
        "begin",
        prefix,
        "--node",
        "outline",
    ]);
    assert_eq!(replay["data"]["attempt"], first["data"]["attempt"]);
    assert_eq!(replay["data"]["replayed"], true);
    let wrong = env.fail(&[
        "--request-id",
        "r-historical-work",
        "attempt",
        "begin",
        &second,
        "--node",
        "outline",
    ]);
    assert_eq!(wrong.0["error"]["code"], "REQUEST_CONFLICT");
}

// Task: C002-T24
#[test]
fn submit_request_replay_does_not_reread_a_deleted_summary_file() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "first")]);
    let begun = env.begin(&wid, "outline");
    let attempt = begun["data"]["attempt"].as_str().unwrap().to_string();
    for path in begun["data"]["outputs"].as_object().unwrap().values() {
        let path = std::path::Path::new(path.as_str().unwrap());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("output for {attempt}\n")).unwrap();
    }
    let summary = env.dir.path().join("summary.txt");
    std::fs::write(&summary, "recorded summary").unwrap();
    let summary_arg = format!("@{}", summary.display());
    let args = [
        "--request-id",
        "r-submit-file",
        "attempt",
        "submit",
        &wid,
        "--attempt",
        &attempt,
        "--summary",
        summary_arg.as_str(),
    ];
    let first = env.ok(&args);
    assert_eq!(
        env.status(&wid)["data"]["last_attempt"]["summary"],
        "recorded summary"
    );
    std::fs::remove_file(summary).unwrap();
    let replay = env.ok(&args);
    assert_eq!(replay["data"]["replayed"], true);
    assert_eq!(replay["revision"], first["revision"]);
    assert_eq!(replay["data"]["outputs"], first["data"]["outputs"]);
}

// Task: C002-T24
#[test]
fn fail_request_replay_does_not_reread_a_deleted_reason_file() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "first")]);
    let begun = env.begin(&wid, "outline");
    let attempt = begun["data"]["attempt"].as_str().unwrap().to_string();
    let reason = env.dir.path().join("reason.txt");
    std::fs::write(&reason, "recorded reason").unwrap();
    let reason_arg = format!("@{}", reason.display());
    let args = [
        "--request-id",
        "r-fail-file",
        "attempt",
        "fail",
        &wid,
        "--attempt",
        &attempt,
        "--reason",
        reason_arg.as_str(),
    ];
    let first = env.ok(&args);
    std::fs::remove_file(reason).unwrap();
    let replay = env.ok(&args);
    assert_eq!(replay["data"]["replayed"], true);
    assert_eq!(replay["revision"], first["revision"]);
    assert_eq!(replay["data"]["attempt"], first["data"]["attempt"]);
}

// Task: C002-T24
#[test]
fn start_replay_conflicts_before_normalizing_a_changed_name() {
    let env = Env::new();
    env.add_example("two-step");
    let original = env.ok(&[
        "--request-id",
        "r-name-intent",
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    let changed = env.fail(&[
        "--request-id",
        "r-name-intent",
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--name",
        "/",
        "--input",
        "topic=x",
    ]);
    assert_eq!(changed.0["error"]["code"], "REQUEST_CONFLICT");
    assert_eq!(
        env.ok(&["work", "list"])["data"].as_array().unwrap().len(),
        1
    );
    assert!(original["data"]["work_id"].is_string());
}

// Task: C002-T24
#[test]
fn omitted_name_and_explicit_flow_name_are_distinct_start_intents() {
    let env = Env::new();
    env.add_example("two-step");
    env.ok(&[
        "--request-id",
        "r-name-omitted",
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    let changed = env.fail(&[
        "--request-id",
        "r-name-omitted",
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--name",
        "default",
        "--input",
        "topic=x",
    ]);
    assert_eq!(changed.0["error"]["code"], "REQUEST_CONFLICT");
}

// Task: C002-T24
#[test]
fn work_prefix_matching_is_case_sensitive_and_replays_with_the_same_selector() {
    let env = Env::new();
    env.add_example("two-step");
    let started = env.ok(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--name",
        "article",
        "--input",
        "topic=x",
    ]);
    let work = started["data"]["work_id"].as_str().unwrap().to_string();
    let prefix = &work[..work.len() - 3];
    let upper = prefix.to_uppercase();
    let wrong_case = env.fail(&["work", "status", &upper]);
    assert_eq!(wrong_case.0["error"]["code"], "NOT_FOUND");

    let args = [
        "--request-id",
        "r-case-prefix",
        "attempt",
        "begin",
        prefix,
        "--node",
        "outline",
    ];
    let first = env.ok(&args);
    let replay = env.ok(&args);
    assert_eq!(replay["data"]["attempt"], first["data"]["attempt"]);
    assert_eq!(replay["data"]["replayed"], true);
}
