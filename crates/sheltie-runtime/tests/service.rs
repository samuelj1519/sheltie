//! T16：Work 服务的库级端到端。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_core::work::WorkStatus;
use sheltie_runtime::{Error, StartArgs};

fn node(s: &str) -> NodeId {
    NodeId::new(s).unwrap()
}

fn attempt(s: &str) -> AttemptId {
    AttemptId::parse(s).unwrap()
}

/// 冻结副本整棵只读；篡改或删除前先放开权限，模拟有人绕过引擎动了文件。
fn make_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if path.is_dir() { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            make_writable(&entry.unwrap().path());
        }
    }
}

// Task: T16
#[test]
fn two_step_runs_to_succeeded() {
    let (_d, home, svc) = home_with_example("two-step");
    let started = start_two_step(&svc);
    let wid = work_id_of(&started);

    let b1 = svc.begin(&wid, &node("outline"), None).unwrap();
    write_output(&output_dir_of(&b1), "outline.md", "# 提纲\n- 一\n- 二\n");
    svc.submit(&wid, &attempt("outline#1.0"), "两个要点", None)
        .unwrap();

    let b2 = svc.begin(&wid, &node("summary"), None).unwrap();
    write_output(&output_dir_of(&b2), "summary.md", "摘要正文。");
    let done = svc
        .submit(&wid, &attempt("summary#1.0"), "写完了", None)
        .unwrap();

    let (_card, json) = svc.status(&wid).unwrap();
    assert_eq!(json.status, WorkStatus::Succeeded);
    assert!(done.next.is_empty());
    let outline = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/1/0/outline.md");
    assert!(
        outline.metadata().unwrap().permissions().readonly(),
        "产物只读"
    );
}

// Task: T16
#[test]
fn start_allocates_work_id_with_today_and_seq_001() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let today = sheltie_runtime::observe::now().day().to_string();
    assert!(wid.as_str().starts_with(&format!("{today}-001-")), "{wid}");
    assert!(wid.as_str().ends_with("-default"), "默认名字是 flow id");
}

// Task: T16
#[test]
fn start_replay_returns_same_work_id_without_new_seq() {
    let (_d, _home, svc) = home_with_example("two-step");
    let args = StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: Some("重放".into()),
        inputs: [("topic".to_string(), "x".to_string())]
            .into_iter()
            .collect(),
    };
    let a = svc.start(args.clone(), Some("req-1".into())).unwrap();
    let b = svc.start(args, Some("req-1".into())).unwrap();
    assert!(b.replayed);
    assert_eq!(work_id_of(&a), work_id_of(&b));
    assert_eq!(svc.list().unwrap().len(), 1);
}

// Task: T16
#[test]
fn start_copies_workbook_into_work_dir_readonly() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let frozen =
        std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("workbook/workbook.toml");
    assert!(frozen.exists());
    assert!(frozen.metadata().unwrap().permissions().readonly());
}

// Task: T16
#[test]
fn begin_loads_graph_from_frozen_copy_not_repository() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let repo_instr = std::path::PathBuf::from(home.workbook_dir("two-step", "1.0.0").as_str())
        .join("instructions/outline.md");
    std::fs::set_permissions(
        &repo_instr,
        std::os::unix::fs::PermissionsExt::from_mode(0o644),
    )
    .unwrap();
    std::fs::write(&repo_instr, "被改过的说明").unwrap();
    let b = svc.begin(&wid, &node("outline"), None).unwrap();
    let brief =
        std::fs::read_to_string(Path::new(output_dir_of(&b).as_str()).join("brief.md")).unwrap();
    assert!(brief.contains("列一份提纲"));
    assert!(!brief.contains("被改过的说明"));
}

// Task: T16
#[test]
fn status_works_after_workbook_removed() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    svc.cancel(&wid, None).unwrap();
    repo(&home).remove("two-step", "1.0.0").unwrap();
    let (card, json) = svc.status(&wid).unwrap();
    assert_eq!(json.status, WorkStatus::Cancelled);
    assert!(card.contains("status: cancelled"));
}

// Task: T16
#[test]
fn begin_writes_brief_md_with_absolute_input_paths() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let b = svc.begin(&wid, &node("outline"), None).unwrap();
    let brief =
        std::fs::read_to_string(Path::new(output_dir_of(&b).as_str()).join("brief.md")).unwrap();
    assert!(brief.contains(&format!("| topic | {}/inputs/topic |", home.work_dir(&wid))));
}

// Task: T16
#[test]
fn begin_binds_resource_input_to_frozen_copy_path() {
    let (_d, home, svc) = home_with_example("article-review");
    let started = svc
        .start(
            StartArgs {
                workbook_id: "article-review".into(),
                version: None,
                flow: "default".into(),
                name: None,
                inputs: [("topic".to_string(), "x".to_string())]
                    .into_iter()
                    .collect(),
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let b = svc.begin(&wid, &node("draft"), None).unwrap();
    write_output(&output_dir_of(&b), "article.md", "文章");
    svc.submit(&wid, &attempt("draft#1.0"), "ok", None).unwrap();
    let r = svc.begin(&wid, &node("review"), None).unwrap();
    let brief =
        std::fs::read_to_string(Path::new(output_dir_of(&r).as_str()).join("brief.md")).unwrap();
    assert!(brief.contains(&format!(
        "{}/workbook/resources/review-checklist.md",
        home.work_dir(&wid)
    )));
}

// Task: T16
#[test]
fn status_card_regenerated_after_each_commit() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    assert!(
        std::fs::read_to_string(&card)
            .unwrap()
            .contains("current: outline#1")
    );
    svc.begin(&wid, &node("outline"), None).unwrap();
    assert!(
        std::fs::read_to_string(&card)
            .unwrap()
            .contains("outline#1.0 running")
    );
}

// Task: T16
#[test]
fn concurrent_writers_one_gets_revision_conflict() {
    // 两个线程同时对同一 Attempt 提交：一个成功，另一个要么先报 REVISION_CONFLICT、重试后变成 ATTEMPT_NOT_RUNNING，要么直接报 ATTEMPT_NOT_RUNNING。
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let b = svc.begin(&wid, &node("outline"), None).unwrap();
    write_output(&output_dir_of(&b), "outline.md", "x");
    let results: Vec<_> = (0..2)
        .map(|_| {
            let svc = svc.clone();
            let wid = wid.clone();
            std::thread::spawn(move || svc.submit(&wid, &attempt("outline#1.0"), "并发", None))
        })
        .map(|h| h.join().unwrap())
        .collect();
    let ok = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(ok, 1);
    assert!(results.iter().any(|r| matches!(
        r,
        Err(Error::Core(sheltie_core::Error::AttemptNotRunning { .. }))
            | Err(Error::RevisionConflict { .. })
    )));
}

// ── M1 复核 O2：冻结副本缺失或被改，对本 Work 的操作报 STORE_CORRUPT（存储合同 §5.1）──

// Task: T16
#[test]
fn begin_on_tampered_frozen_copy_is_store_corrupt() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let copy = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("workbook");
    make_writable(&copy);
    std::fs::write(copy.join("instructions/outline.md"), "被改过的说明").unwrap();
    let err = svc.begin(&wid, &node("outline"), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err}");
}

// Task: T16
#[test]
fn missing_frozen_copy_is_store_corrupt_for_begin_and_status() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let copy = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("workbook");
    make_writable(&copy);
    std::fs::remove_dir_all(&copy).unwrap();
    // 仓库里的那份还在，也不回退去读它。
    let err = svc.begin(&wid, &node("outline"), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err}");
    let err = svc.status(&wid).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err}");
}

// Task: T16
#[test]
fn tampered_resource_input_is_store_corrupt_not_artifact_modified() {
    let (_d, home, svc) = home_with_example("article-review");
    let started = svc
        .start(
            StartArgs {
                workbook_id: "article-review".into(),
                version: None,
                flow: "default".into(),
                name: None,
                inputs: [("topic".to_string(), "x".to_string())]
                    .into_iter()
                    .collect(),
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let b = svc.begin(&wid, &node("draft"), None).unwrap();
    write_output(&output_dir_of(&b), "article.md", "文章");
    svc.submit(&wid, &attempt("draft#1.0"), "ok", None).unwrap();
    // `resource.<path>` 输入没有单独记录的摘要，由副本整体摘要覆盖。
    let copy = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("workbook");
    make_writable(&copy);
    std::fs::write(copy.join("resources/review-checklist.md"), "被改过的清单").unwrap();
    let err = svc.begin(&wid, &node("review"), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err}");
}
