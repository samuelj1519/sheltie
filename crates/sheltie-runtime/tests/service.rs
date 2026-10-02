//! T16：Work 服务的库级端到端。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_core::work::WorkStatus;
use sheltie_runtime::request::InputValue;
use sheltie_runtime::{Error, StartArgs};

static POST_COMMIT_SYNC_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn unpublished_start_fixture() -> (
    OwnedTempDir,
    sheltie_runtime::Home,
    sheltie_runtime::WorkService,
    sheltie_core::ids::WorkId,
    String,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let (dir, home, svc) = home_with_example("two-step");
    let request_id = format!("t26-start-{}", uuid::Uuid::now_v7());
    let started = svc
        .start(
            two_step_args(&[("topic", "给新人介绍 Sheltie")]),
            Some(request_id.clone()),
        )
        .unwrap();
    let work = work_id_of(&started);
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let internal_id = pending.split('/').nth(1).unwrap();
    let payload = home
        .rel(&format!("pending/{internal_id}/payload"))
        .unwrap()
        .as_path()
        .to_path_buf()
        .into_std_path_buf();
    let final_path = std::path::PathBuf::from(home.work_dir(&work).as_str());
    std::fs::rename(&final_path, &payload).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [&request_id],
    )
    .unwrap();
    drop(conn);
    (dir, home, svc, work, request_id, payload, final_path)
}

fn assert_blocked_unpublished_start(
    home: &sheltie_runtime::Home,
    service: &sheltie_runtime::WorkService,
    work: &sheltie_core::ids::WorkId,
    request_id: &str,
    payload: &Path,
    final_path: &Path,
    cause: sheltie_core::error::ErrorCode,
) {
    let blocker_id = "t26-blocked-request";
    let error = service
        .cancel(work, Some(blocker_id.to_string()))
        .unwrap_err();
    let Error::EffectPending {
        committed,
        request_id: current,
        pending_request_id,
        cause: actual_cause,
        ..
    } = error
    else {
        panic!("恢复失败必须保留pending归属：{error:?}");
    };
    assert!(!committed);
    assert_eq!(current, blocker_id);
    assert_eq!(pending_request_id.as_deref(), Some(request_id));
    assert_eq!(actual_cause, cause);
    assert!(payload.exists(), "失败后保留pending原件");
    assert!(!final_path.exists(), "身份不符时不得发布final");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = ?1",
            [blocker_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

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
    svc.submit(&wid, &attempt("outline#1.0"), &lit("两个要点"), None)
        .unwrap();

    let b2 = svc.begin(&wid, &node("summary"), None).unwrap();
    write_output(&output_dir_of(&b2), "summary.md", "摘要正文。");
    let done = svc
        .submit(&wid, &attempt("summary#1.0"), &lit("写完了"), None)
        .unwrap();

    let (_card, json) = svc.status(&wid).unwrap();
    assert_eq!(json.status, WorkStatus::Succeeded);
    assert!(done.next.is_empty());
    let outline = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/outputs/outline.md");
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
        inputs: inputs_lit(&[("topic", "x")]),
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
    let brief = {
        let brief_path = match &b.reply {
            sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => brief_path.clone(),
            other => panic!("{other:?}"),
        };
        std::fs::read_to_string(brief_path.as_str()).unwrap()
    };
    assert!(brief.contains("列一份提纲"));
    assert!(!brief.contains("被改过的说明"));
}

// Task: T16
#[test]
fn begin_writes_brief_md_with_absolute_input_paths() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let b = svc.begin(&wid, &node("outline"), None).unwrap();
    let brief = {
        let brief_path = match &b.reply {
            sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => brief_path.clone(),
            other => panic!("{other:?}"),
        };
        std::fs::read_to_string(brief_path.as_str()).unwrap()
    };
    assert!(brief.contains(&format!(
        "| topic | {}/start-inputs/topic |",
        home.work_dir(&wid)
    )));
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
                inputs: inputs_lit(&[("topic", "x")]),
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let b = svc.begin(&wid, &node("draft"), None).unwrap();
    write_output(&output_dir_of(&b), "article.md", "文章");
    svc.submit(&wid, &attempt("draft#1.0"), &lit("ok"), None)
        .unwrap();
    let r = svc.begin(&wid, &node("review"), None).unwrap();
    let brief_path = match &r.reply {
        sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => brief_path.clone(),
        other => panic!("{other:?}"),
    };
    let brief = std::fs::read_to_string(brief_path.as_str()).unwrap();
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
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let svc = svc.clone();
            let wid = wid.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                svc.submit(&wid, &attempt("outline#1.0"), &lit("并发"), None)
            })
        })
        .collect();
    barrier.wait();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let ok = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(ok, 1);
    assert!(results.iter().any(|r| matches!(
        r,
        Err(Error::Core(sheltie_core::Error::AttemptNotRunning { .. }))
            | Err(Error::RevisionConflict { .. })
    )));
}

// Task: T16
#[test]
fn resolve_work_unique_prefix_resolves() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let prefix: String = wid.as_str().chars().take(20).collect();
    assert_eq!(svc.resolve_work(&prefix).unwrap(), wid);
    assert_eq!(svc.resolve_work(wid.as_str()).unwrap(), wid);
}

// Task: T16
#[test]
fn resolve_work_rejects_unknown_prefix() {
    let (_d, _home, svc) = home_with_example("two-step");
    let _ = start_two_step(&svc);
    assert!(matches!(
        svc.resolve_work("1999-01-01"),
        Err(Error::NotFound { .. })
    ));
}

// Task: T16
#[test]
fn response_revision_increments_with_each_commit() {
    let (_d, _home, svc) = home_with_example("two-step");
    let started = start_two_step(&svc);
    assert_eq!(started.revision, 1);
    let wid = work_id_of(&started);
    let b = svc.begin(&wid, &node("outline"), None).unwrap();
    assert_eq!(b.revision, 2);
    write_output(&output_dir_of(&b), "outline.md", "x");
    let s = svc
        .submit(&wid, &attempt("outline#1.0"), &lit("ok"), None)
        .unwrap();
    assert_eq!(s.revision, 3);
}

// Task: T16
#[test]
fn begin_replay_returns_original_reply_and_rewrites_brief() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let first = svc
        .begin(&wid, &node("outline"), Some("r-begin".into()))
        .unwrap();
    let brief = std::path::PathBuf::from(
        match &first.reply {
            sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => brief_path.clone(),
            other => panic!("{other:?}"),
        }
        .as_str(),
    );
    std::fs::remove_file(&brief).unwrap();
    let again = svc
        .begin(&wid, &node("outline"), Some("r-begin".into()))
        .unwrap();
    assert!(again.replayed);
    assert_eq!(again.revision, first.revision, "重放不推进 revision");
    assert_eq!(again.reply, first.reply, "重放返回原响应");
    assert!(brief.exists(), "重放补写任务书");
    assert!(
        !brief.parent().unwrap().join("engine/stats.json").exists(),
        "没有 engine.stats 输入的 Attempt 不生成 stats.json"
    );
}

// Task: T16
#[test]
fn begin_same_request_id_different_node_is_request_conflict() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    svc.begin(&wid, &node("outline"), Some("r-x".into()))
        .unwrap();
    assert!(matches!(
        svc.begin(&wid, &node("summary"), Some("r-x".into())),
        Err(Error::RequestConflict { .. })
    ));
}

// Task: T16
#[test]
fn start_same_request_id_different_inputs_is_request_conflict() {
    let (_d, _home, svc) = home_with_example("two-step");
    let args = || StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: inputs_lit(&[("topic", "x")]),
    };
    svc.start(args(), Some("req-9".into())).unwrap();
    let mut changed = args();
    changed.inputs.insert(
        "topic".to_string(),
        InputValue::Literal {
            text: "y".to_string(),
        },
    );
    assert!(matches!(
        svc.start(changed, Some("req-9".into())),
        Err(Error::RequestConflict { .. })
    ));
}

// Task: T16
#[test]
fn audit_stores_command_with_instruction_text_redacted() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    svc.begin(&wid, &node("outline"), None).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let json: String = conn
        .query_row(
            "SELECT command_json FROM audit ORDER BY seq DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(json.contains("字节"), "说明书原文换成字节数：{json}");
    assert!(!json.contains("列一份提纲"), "说明书原文不进审计表：{json}");
}

// Task: T16
#[test]
fn begin_replay_regenerates_missing_stats_json() {
    let (_d, home) = temp_home();
    let src = Path::new(_d.path()).join("stats-wb");
    std::fs::create_dir_all(src.join("flows")).unwrap();
    std::fs::write(
        src.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"stats-wb\"\nversion = \"1.0.0\"\nname = \"统计重放\"\ndescription = \"engine.stats 重放。\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        src.join("flows/default.toml"),
        "schema = \"flow/v1\"\nid = \"default\"\nentry = \"a\"\n\n[[nodes]]\nid = \"a\"\ntitle = \"甲\"\nexecutor = \"agent\"\ninstruction = { text = \"做甲。\" }\noutputs = [{ name = \"x\", path = \"x.md\", max_bytes = 65536 }]\n\n[[nodes]]\nid = \"b\"\ntitle = \"乙\"\nexecutor = \"agent\"\ninstruction = { text = \"做乙。\" }\ninputs = [{ name = \"stats\", from = \"engine.stats\" }, { name = \"x\", from = \"a.x\" }]\noutputs = [{ name = \"y\", path = \"y.md\", max_bytes = 65536 }]\n\n[[edges]]\nfrom = \"a\"\nto = \"b\"\nkind = \"main\"\n",
    )
    .unwrap();
    let r = repo(&home);
    r.add(&abs(&src), None).unwrap();
    let svc = service(&home);
    let started = svc
        .start(
            StartArgs {
                workbook_id: "stats-wb".into(),
                version: None,
                flow: "default".into(),
                name: None,
                inputs: Default::default(),
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let begin_a = svc.begin(&wid, &node("a"), None).unwrap();
    write_output(&output_dir_of(&begin_a), "x.md", "x");
    svc.submit(&wid, &attempt("a#1.0"), &lit("ok"), None)
        .unwrap();
    let begin_b = svc.begin(&wid, &node("b"), Some("r-b".into())).unwrap();
    // engine/stats.json 在 Attempt 目录的 engine/ 之下（T07 布局）。
    let stats = match &begin_b.reply {
        sheltie_core::work::Reply::AttemptBegun { inputs, .. } => {
            inputs["stats"].as_ref().unwrap().clone()
        }
        other => panic!("{other:?}"),
    };
    let stats = std::path::PathBuf::from(stats.as_str());
    let original = std::fs::read(&stats).unwrap();
    std::fs::remove_file(&stats).unwrap();
    let replay = svc.begin(&wid, &node("b"), Some("r-b".into())).unwrap();
    assert!(replay.replayed);
    assert_eq!(
        std::fs::read(&stats).unwrap(),
        original,
        "重放按提交时的口径重算 stats.json"
    );
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

// Task: C002-T20
#[test]
fn tampered_work_dir_is_rejected_before_reading_outside_paths() {
    use std::os::unix::fs::PermissionsExt;

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("status-card.md");
    std::fs::write(&sentinel, b"external owner data").unwrap();
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o640)).unwrap();
    let before_bytes = std::fs::read(&sentinel).unwrap();
    let before_mode = std::fs::metadata(&sentinel).unwrap().permissions().mode();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let state_json: String = conn
        .query_row(
            "SELECT state_json FROM works WHERE work_id = ?1",
            [wid.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let mut state: serde_json::Value = serde_json::from_str(&state_json).unwrap();
    state["work_dir"] = serde_json::Value::String(outside.path().to_string_lossy().into_owned());
    conn.execute(
        "UPDATE works SET state_json = ?1 WHERE work_id = ?2",
        rusqlite::params![serde_json::to_string(&state).unwrap(), wid.as_str()],
    )
    .unwrap();
    drop(conn);

    let err = svc.status(&wid).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err:?}");
    assert_eq!(std::fs::read(&sentinel).unwrap(), before_bytes);
    assert_eq!(
        std::fs::metadata(&sentinel).unwrap().permissions().mode(),
        before_mode
    );
}

// Task: C002-T20
#[test]
fn invalid_later_effect_is_rejected_before_any_effect_runs() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let request_id = begun.request_id.clone();
    let brief = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/brief.md");
    std::fs::remove_file(&brief).unwrap();

    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("outside.md");
    std::fs::write(&sentinel, b"outside sentinel").unwrap();
    let before = std::fs::read(&sentinel).unwrap();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects_json: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| row.get(0),
        )
        .unwrap();
    let mut effects: serde_json::Value = serde_json::from_str(&effects_json).unwrap();
    let writes = effects
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "write_file")
        .collect::<Vec<_>>();
    let mut invalid_later_write = (*writes[0]).clone();
    invalid_later_write["path"] = serde_json::Value::String("../outside.md".to_string());
    effects.as_array_mut().unwrap().push(invalid_later_write);
    conn.execute(
        "UPDATE requests SET effects_json = ?1, published = 0 WHERE request_id = ?2",
        rusqlite::params![serde_json::to_string(&effects).unwrap(), request_id],
    )
    .unwrap();
    drop(conn);

    let err = svc.cancel(&wid, None).unwrap_err();
    assert_effect_pending(err, false, None, Some(&request_id));
    assert!(!brief.exists(), "首个合法效果也必须在批次校验后才执行");
    assert_eq!(std::fs::read(&sentinel).unwrap(), before);
}

// Task: C002-T20
#[test]
fn tampered_artifact_path_is_rejected_before_observing_external_file() {
    use std::os::unix::fs::PermissionsExt;

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    write_output(
        &output_dir_of(&begun),
        "outline.md",
        "same registered bytes",
    );
    svc.submit(&wid, &attempt("outline#1.0"), &lit("outline"), None)
        .unwrap();

    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("external-outline.md");
    std::fs::write(&sentinel, b"same registered bytes").unwrap();
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o600)).unwrap();
    let before = std::fs::read(&sentinel).unwrap();
    let mode_before = std::fs::metadata(&sentinel).unwrap().permissions().mode();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let state_json: String = conn
        .query_row(
            "SELECT state_json FROM works WHERE work_id = ?1",
            [wid.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let mut state: serde_json::Value = serde_json::from_str(&state_json).unwrap();
    state["attempts"][0]["outputs"]["outline"]["path"] =
        serde_json::Value::String(sentinel.to_string_lossy().into_owned());
    conn.execute(
        "UPDATE works SET state_json = ?1 WHERE work_id = ?2",
        rusqlite::params![serde_json::to_string(&state).unwrap(), wid.as_str()],
    )
    .unwrap();
    drop(conn);

    let err = svc.begin(&wid, &node("summary"), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err:?}");
    assert_eq!(std::fs::read(&sentinel).unwrap(), before);
    assert_eq!(
        std::fs::metadata(&sentinel).unwrap().permissions().mode(),
        mode_before
    );
}

// Task: C002-T20
#[test]
fn engine_stats_effect_must_match_its_bound_artifact_reference() {
    let (_d, home) = temp_home();
    let source = Path::new(_d.path()).join("stats-wb");
    std::fs::create_dir_all(source.join("flows")).unwrap();
    std::fs::write(
        source.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"stats-wb\"\nversion = \"1.0.0\"\nname = \"stats\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        source.join("flows/default.toml"),
        "schema = \"flow/v1\"\nid = \"default\"\nentry = \"a\"\n\n[[nodes]]\nid = \"a\"\ntitle = \"A\"\nexecutor = \"agent\"\ninstruction = { text = \"A\" }\noutputs = [{ name = \"x\", path = \"x.md\", max_bytes = 65536 }]\n\n[[nodes]]\nid = \"b\"\ntitle = \"B\"\nexecutor = \"agent\"\ninstruction = { text = \"B\" }\ninputs = [{ name = \"stats\", from = \"engine.stats\" }, { name = \"x\", from = \"a.x\" }]\noutputs = [{ name = \"y\", path = \"y.md\", max_bytes = 65536 }]\n\n[[edges]]\nfrom = \"a\"\nto = \"b\"\nkind = \"main\"\n",
    )
    .unwrap();
    repo(&home).add(&abs(&source), None).unwrap();
    let svc = service(&home);
    let started = svc
        .start(
            StartArgs {
                workbook_id: "stats-wb".into(),
                version: None,
                flow: "default".into(),
                name: None,
                inputs: Default::default(),
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let begun = svc.begin(&wid, &node("a"), None).unwrap();
    write_output(&output_dir_of(&begun), "x.md", "x");
    svc.submit(&wid, &attempt("a#1.0"), &lit("ok"), None)
        .unwrap();
    let begin_b = svc.begin(&wid, &node("b"), Some("stats-b".into())).unwrap();
    let stats_path = match &begin_b.reply {
        sheltie_core::work::Reply::AttemptBegun { inputs, .. } => {
            inputs["stats"].as_ref().unwrap().clone()
        }
        other => panic!("{other:?}"),
    };

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let state_json: String = conn
        .query_row(
            "SELECT state_json FROM works WHERE work_id = ?1",
            [wid.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let mut state: serde_json::Value = serde_json::from_str(&state_json).unwrap();
    state["attempts"][1]["inputs"]["stats"]["bytes"] = serde_json::json!(
        state["attempts"][1]["inputs"]["stats"]["bytes"]
            .as_u64()
            .unwrap()
            + 1
    );
    conn.execute(
        "UPDATE works SET state_json = ?1 WHERE work_id = ?2",
        rusqlite::params![serde_json::to_string(&state).unwrap(), wid.as_str()],
    )
    .unwrap();
    drop(conn);

    let err = svc
        .begin(&wid, &node("b"), Some("stats-b".into()))
        .unwrap_err();
    assert_effect_pending(err, true, Some("stats-b"), None);
    assert!(std::path::Path::new(stats_path.as_str()).exists());
}

// Task: C002-T25
#[test]
fn tampered_begin_snapshot_path_is_rejected_on_historical_replay() {
    use std::os::unix::fs::PermissionsExt;

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("task-brief.md");
    std::fs::write(&sentinel, b"external data").unwrap();
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o640)).unwrap();
    let before = std::fs::read(&sentinel).unwrap();
    let mode_before = std::fs::metadata(&sentinel).unwrap().permissions().mode();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let reply_json: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [&begun.request_id],
            |row| row.get(0),
        )
        .unwrap();
    let mut snapshot: serde_json::Value = serde_json::from_str(&reply_json).unwrap();
    snapshot["data"]["brief_path"] =
        serde_json::Value::String(sentinel.to_string_lossy().into_owned());
    conn.execute(
        "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
        rusqlite::params![serde_json::to_string(&snapshot).unwrap(), begun.request_id],
    )
    .unwrap();
    drop(conn);

    let err = svc
        .begin(&wid, &node("outline"), Some(begun.request_id.clone()))
        .unwrap_err();
    assert_effect_pending_without_original(err, true, &begun.request_id, None);
    snapshot["data"] = serde_json::json!({});
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
        rusqlite::params![serde_json::to_string(&snapshot).unwrap(), begun.request_id],
    )
    .unwrap();
    drop(conn);
    let missing_data = svc
        .begin(&wid, &node("outline"), Some(begun.request_id.clone()))
        .unwrap_err();
    assert_effect_pending_without_original(missing_data, true, &begun.request_id, None);
    let mut unknown_reply = serde_json::to_value(&begun).unwrap();
    unknown_reply["reply"]["unrecognized"] = serde_json::json!("outside path");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
        rusqlite::params![
            serde_json::to_string(&unknown_reply).unwrap(),
            begun.request_id
        ],
    )
    .unwrap();
    drop(conn);
    let unknown_reply_error = svc
        .begin(&wid, &node("outline"), Some(begun.request_id.clone()))
        .unwrap_err();
    assert_effect_pending_without_original(unknown_reply_error, true, &begun.request_id, None);

    let mut unknown_next = serde_json::to_value(&begun).unwrap();
    unknown_next["next"][0]["unrecognized"] = serde_json::json!("extra command data");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
        rusqlite::params![
            serde_json::to_string(&unknown_next).unwrap(),
            begun.request_id
        ],
    )
    .unwrap();
    drop(conn);
    let unknown_next_error = svc
        .begin(&wid, &node("outline"), Some(begun.request_id.clone()))
        .unwrap_err();
    assert_effect_pending_without_original(unknown_next_error, true, &begun.request_id, None);
    assert_eq!(std::fs::read(&sentinel).unwrap(), before);
    assert_eq!(
        std::fs::metadata(&sentinel).unwrap().permissions().mode(),
        mode_before
    );
}

// 原任务 C002-T25；T34收紧原响应资格断言。
// Task: C002-T34
#[test]
fn duplicate_audit_owner_is_rejected_before_recovery_io() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let request_id = begun.request_id.clone();
    let brief = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/brief.md");
    std::fs::remove_file(&brief).unwrap();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let audit: (String, i64, String, String) = conn
        .query_row(
            "SELECT work_id, revision, principal, command_json FROM audit WHERE request_id = ?1",
            [&request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    let at: String = conn
        .query_row(
            "SELECT at FROM audit WHERE request_id = ?1",
            [&request_id],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "INSERT INTO audit (work_id, revision, request_id, principal, command_json, at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![audit.0, audit.1, request_id, audit.2, audit.3, at],
    )
    .unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [&request_id],
    )
    .unwrap();
    drop(conn);

    let blocked_request = "t34-duplicate-audit-blocked";
    let err = svc.cancel(&wid, Some(blocked_request.into())).unwrap_err();
    assert_effect_pending_without_original(err, false, blocked_request, Some(&request_id));
    assert!(!brief.exists(), "重复audit归属时不得执行第一条效果");
}

// Task: C002-T20
#[test]
fn invalid_published_flag_is_not_treated_as_completed() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let request_id = begun.request_id.clone();
    let brief = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/brief.md");
    std::fs::remove_file(&brief).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 2 WHERE request_id = ?1",
        [&request_id],
    )
    .unwrap();
    drop(conn);

    let err = svc.cancel(&wid, None).unwrap_err();
    assert_effect_pending(err, false, None, Some(&request_id));
    assert!(!brief.exists(), "非0/1的published不能跳过效果校验");
}

// 原任务 C002-T20；T34收紧原响应资格断言。
// Task: C002-T34
#[test]
fn missing_audit_does_not_hide_unpublished_request_from_recovery() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let request_id = begun.request_id.clone();
    let brief = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/brief.md");
    std::fs::remove_file(&brief).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [&request_id],
    )
    .unwrap();
    conn.execute("DELETE FROM audit WHERE request_id = ?1", [&request_id])
        .unwrap();
    let other_requests_before: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id != ?1 AND work_id = ?2",
            rusqlite::params![request_id, wid.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    drop(conn);

    let blocked_request = "t34-missing-audit-blocked";
    let err = svc.cancel(&wid, Some(blocked_request.into())).unwrap_err();
    assert_effect_pending_without_original(err, false, blocked_request, Some(&request_id));
    assert!(!brief.exists());
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let published: i64 = conn
        .query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(published, 0, "缺少唯一audit时不能标记效果完成");
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id != ?1 AND work_id = ?2",
            rusqlite::params![request_id, wid.as_str()],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        other_requests_before,
        "恢复失败不能提交新的cancel请求"
    );
}

// Task: C002-T20
#[test]
fn empty_effect_batch_is_rejected_without_marking_published() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let request_id = begun.request_id.clone();
    let brief = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/brief.md");
    std::fs::remove_file(&brief).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET effects_json = '[]', published = 0 WHERE request_id = ?1",
        [&request_id],
    )
    .unwrap();
    drop(conn);

    let err = svc.cancel(&wid, None).unwrap_err();
    assert_effect_pending(err, false, None, Some(&request_id));
    assert!(!brief.exists());
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let published: i64 = conn
        .query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(published, 0, "缺少必需效果时不得记录已完成");
}

// Task: C002-T20
#[test]
fn pending_owner_json_roundtrips_opaque_request_id() {
    let (_d, home, svc) = home_with_example("two-step");
    let request_id = "opaque\"request\nline";
    svc.start(
        StartArgs {
            workbook_id: "two-step".to_string(),
            version: None,
            flow: "default".to_string(),
            name: None,
            inputs: inputs_lit(&[("topic", "owner-json")]),
        },
        Some(request_id.to_string()),
    )
    .unwrap();

    let sidecars = std::fs::read_dir(home.pending_dir().as_path())
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".owner"))
                .then_some(path)
        })
        .collect::<Vec<_>>();
    let owners = sidecars
        .iter()
        .map(|path| {
            serde_json::from_slice::<serde_json::Value>(&std::fs::read(path).unwrap()).unwrap()
        })
        .collect::<Vec<_>>();
    assert!(owners.iter().any(|owner| {
        owner["request_id"] == request_id
            && owner["format"] == "pending/v1"
            && owner["op"] == "start_work"
    }));
}

// 原任务 C002-T25；T34收紧原响应资格断言。
// Task: C002-T34
#[test]
fn unpublished_work_start_requires_its_owner_sidecar() {
    let (_d, home, svc) = home_with_example("two-step");
    let started = svc
        .start(
            StartArgs {
                workbook_id: "two-step".to_string(),
                version: None,
                flow: "default".to_string(),
                name: None,
                inputs: inputs_lit(&[("topic", "owner-required")]),
            },
            Some("start-owner-required".to_string()),
        )
        .unwrap();
    let wid = work_id_of(&started);
    let sidecar = std::fs::read_dir(home.pending_dir().as_path())
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".owner"))
                && serde_json::from_slice::<serde_json::Value>(&std::fs::read(path).unwrap())
                    .is_ok_and(|owner| owner["request_id"] == "start-owner-required")
        })
        .unwrap();
    std::fs::remove_file(sidecar).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = 'start-owner-required'",
        [],
    )
    .unwrap();
    drop(conn);

    let blocked_request = "t34-unverified-start-blocked";
    let err = svc.cancel(&wid, Some(blocked_request.into())).unwrap_err();
    assert_effect_pending_without_original(
        err,
        false,
        blocked_request,
        Some("start-owner-required"),
    );
    assert!(home.work_dir(&wid).as_path().join("workbook").is_dir());
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
                inputs: inputs_lit(&[("topic", "x")]),
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let b = svc.begin(&wid, &node("draft"), None).unwrap();
    write_output(&output_dir_of(&b), "article.md", "文章");
    svc.submit(&wid, &attempt("draft#1.0"), &lit("ok"), None)
        .unwrap();
    // `resource.<path>` 输入没有单独记录的摘要，由副本整体摘要覆盖。
    let copy = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("workbook");
    make_writable(&copy);
    std::fs::write(copy.join("resources/review-checklist.md"), "被改过的清单").unwrap();
    let err = svc.begin(&wid, &node("review"), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err}");
}

fn committed_unpublished_submit() -> (
    OwnedTempDir,
    sheltie_runtime::Home,
    sheltie_core::ids::WorkId,
    std::path::PathBuf,
    String,
) {
    let (dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let output = std::path::PathBuf::from(output_dir_of(&begun).as_str()).join("outline.md");
    std::fs::write(&output, b"committed bytes").unwrap();
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o644)).unwrap();
    let request_id = "t22-submit-recovery".to_string();
    svc.submit(
        &wid,
        &attempt("outline#1.0"),
        &lit("ok"),
        Some(request_id.clone()),
    )
    .unwrap();
    drop(svc);
    (dir, home, wid, output, request_id)
}

fn mark_submit_unpublished(home: &sheltie_runtime::Home, request_id: &str) {
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [request_id],
    )
    .unwrap();
}

fn recover_submit_stops_and_keeps_unpublished(
    home: &sheltie_runtime::Home,
    wid: &sheltie_core::ids::WorkId,
    request_id: &str,
) {
    let err = service(home)
        .begin(wid, &node("summary"), None)
        .unwrap_err();
    assert!(matches!(err, Error::EffectPending { .. }), "{err:?}");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let published: i64 = conn
        .query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(published, 0, "失败的seal不得标记已发布");
}

// Task: C002-T25
#[test]
fn post_commit_card_failure_returns_committed_response() {
    let (_dir, home, svc) = home_with_example("two-step");
    let started = start_two_step(&svc);
    let wid = work_id_of(&started);
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    std::fs::remove_file(&card).unwrap();
    std::fs::create_dir(&card).unwrap();

    let request_id = "t25-card-failure";
    let err = svc
        .begin(&wid, &node("outline"), Some(request_id.to_string()))
        .unwrap_err();
    let Error::EffectPending {
        committed,
        request_id: actual,
        pending_request_id,
        cause,
        original,
        pending_original,
        ..
    } = err
    else {
        panic!("COMMIT后的状态卡错误必须保留请求归属：{err:?}");
    };
    assert!(committed);
    assert_eq!(actual, request_id);
    assert_eq!(pending_request_id, None);
    assert_eq!(cause, ErrorCode::Io);
    assert!(pending_original.is_none());

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let (stored, revision, published): (String, u64, i64) = conn
        .query_row(
            "SELECT r.reply_json, w.revision, r.published FROM requests r JOIN works w USING(work_id) WHERE r.request_id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(published, 0);
    assert_eq!(revision, 2);
    let original: serde_json::Value = serde_json::from_str(original.as_deref().unwrap()).unwrap();
    let stored: serde_json::Value = serde_json::from_str(&stored).unwrap();
    assert_eq!(original["ok"], true);
    assert_eq!(original["request_id"], request_id);
    assert_eq!(original["revision"], 2);
    let mut expected_data = stored["data"].clone();
    expected_data["replayed"] = serde_json::Value::Bool(false);
    assert_eq!(original["data"], expected_data);
    assert_eq!(original["data"]["replayed"], false);
    assert_eq!(
        original["next"],
        serde_json::json!([
            {
                "op": "attempt submit",
                "args": { "work": wid.as_str(), "attempt": "outline#1.0" }
            },
            {
                "op": "attempt fail",
                "args": { "work": wid.as_str(), "attempt": "outline#1.0" }
            },
            { "op": "work cancel", "args": { "work": wid.as_str() } }
        ])
    );
}

// Task: C002-T25
#[test]
fn historical_write_with_missing_parent_keeps_committed_snapshot() {
    let (_dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let request_id = "t25-missing-history-parent";
    let begun = svc
        .begin(&wid, &node("outline"), Some(request_id.to_string()))
        .unwrap();
    let brief = std::path::PathBuf::from(home.work_dir(&wid).as_str())
        .join("attempts/outline/occurrence-001/attempt-000/brief.md");
    let attempt_dir = brief.parent().unwrap();
    make_writable(attempt_dir);
    std::fs::remove_dir_all(attempt_dir).unwrap();

    let error = svc
        .begin(&wid, &node("outline"), Some(request_id.to_string()))
        .unwrap_err();
    let original = assert_effect_pending(error, true, Some(request_id), None);
    assert_eq!(original["revision"], begun.revision);
    assert!(!attempt_dir.exists(), "历史父目录缺失时不能自行补造");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        1,
        "同请求历史核验失败不能产生新记录"
    );
}

// Task: C002-T25
#[test]
fn malformed_started_identity_is_not_projected_as_success() {
    for (field, value) in [
        ("workbook", serde_json::json!({})),
        ("name", serde_json::json!("other-valid-name")),
    ] {
        let (_dir, home, svc) = home_with_example("two-step");
        let request_id = format!("t25-start-{field}");
        let args = two_step_args(&[("topic", "给新人介绍 Sheltie")]);
        svc.start(args.clone(), Some(request_id.clone())).unwrap();
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let reply: String = conn
            .query_row(
                "SELECT reply_json FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get(0),
            )
            .unwrap();
        let mut snapshot: serde_json::Value = serde_json::from_str(&reply).unwrap();
        snapshot["data"][field] = value;
        conn.execute(
            "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
            rusqlite::params![snapshot.to_string(), request_id],
        )
        .unwrap();
        let error = svc.start(args, Some(request_id.clone())).unwrap_err();
        assert_effect_pending_without_original(error, true, &request_id, None);
    }
}

// Task: C002-T25
#[test]
fn malformed_cancelled_status_is_not_projected_as_success() {
    for value in [
        serde_json::json!({"kind":"active"}),
        serde_json::json!({"kind":"cancelled","unknown":"extra"}),
    ] {
        let (_dir, home, svc) = home_with_example("two-step");
        let wid = work_id_of(&start_two_step(&svc));
        let request_id = "t25-cancelled-status";
        svc.cancel(&wid, Some(request_id.into())).unwrap();
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let reply: String = conn
            .query_row(
                "SELECT reply_json FROM requests WHERE request_id = ?1",
                [request_id],
                |row| row.get(0),
            )
            .unwrap();
        let mut snapshot: serde_json::Value = serde_json::from_str(&reply).unwrap();
        snapshot["data"]["work_status"] = value;
        conn.execute(
            "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
            rusqlite::params![snapshot.to_string(), request_id],
        )
        .unwrap();
        let error = svc.cancel(&wid, Some(request_id.into())).unwrap_err();
        assert_effect_pending_without_original(error, true, request_id, None);
    }
}

// Task: C002-T25
#[test]
fn malformed_gate_principal_is_not_projected_as_success() {
    let (_dir, home, svc) = home_with_example("gated-release");
    let started = svc
        .start(
            StartArgs {
                workbook_id: "gated-release".into(),
                version: None,
                flow: "default".into(),
                name: None,
                inputs: inputs_lit(&[("version", "1.0.0")]),
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let begun = svc.begin(&wid, &node("notes"), None).unwrap();
    write_output(&output_dir_of(&begun), "notes.md", "发布说明");
    svc.submit(&wid, &attempt("notes#1.0"), &lit("完成"), None)
        .unwrap();
    let request_id = "t25-gate-principal";
    svc.approve(&wid, &node("notes"), Some(request_id.into()))
        .unwrap();

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let reply: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let mut snapshot: serde_json::Value = serde_json::from_str(&reply).unwrap();
    snapshot["data"]["by"] = serde_json::Value::Bool(false);
    conn.execute(
        "UPDATE requests SET reply_json = ?1 WHERE request_id = ?2",
        rusqlite::params![serde_json::to_string(&snapshot).unwrap(), request_id],
    )
    .unwrap();
    drop(conn);

    let error = svc
        .approve(&wid, &node("notes"), Some(request_id.into()))
        .unwrap_err();
    assert_effect_pending_without_original(error, true, request_id, None);
}

// Task: C002-T25
#[test]
fn mark_published_zero_rows_returns_committed_recovery_error() {
    let (_dir, home, svc) = home_with_example("two-step");
    let started = start_two_step(&svc);
    let wid = work_id_of(&started);
    let request_id = "t25-mark-zero-rows";
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute_batch(&format!(
        "CREATE TRIGGER ignore_mark_t25 BEFORE UPDATE OF published ON requests
         WHEN OLD.request_id = '{request_id}'
         BEGIN SELECT RAISE(IGNORE); END;"
    ))
    .unwrap();
    drop(conn);

    let err = svc
        .begin(&wid, &node("outline"), Some(request_id.to_string()))
        .unwrap_err();
    let original = assert_effect_pending(err, true, Some(request_id), None);
    assert_eq!(original["revision"], 2);
    assert!(
        std::path::Path::new(&original["data"]["brief_path"].as_str().unwrap()).exists(),
        "mark失败发生在效果与状态卡完成之后"
    );

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let published: i64 = conn
        .query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(published, 0, "零行UPDATE不能报告发布完成");
}

// Task: C002-T25
#[test]
fn old_effect_blocks_new_request_with_distinct_identities() {
    let (_dir, home, svc) = home_with_example("two-step");
    let started = start_two_step(&svc);
    let wid = work_id_of(&started);
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    std::fs::remove_file(&card).unwrap();
    std::fs::create_dir(&card).unwrap();

    let old_request = "t25-old-A";
    let old_error = svc
        .begin(&wid, &node("outline"), Some(old_request.to_string()))
        .unwrap_err();
    assert!(matches!(
        old_error,
        Error::EffectPending {
            committed: true,
            ..
        }
    ));
    let new_request = "t25-new-B";
    let blocked = svc
        .begin(&wid, &node("review"), Some(new_request.to_string()))
        .unwrap_err();
    let Error::EffectPending {
        committed,
        request_id,
        pending_request_id,
        original,
        pending_original,
        ..
    } = blocked
    else {
        panic!("旧请求失败必须阻断新请求：{blocked:?}");
    };
    assert!(!committed);
    assert_eq!(request_id, new_request);
    assert_eq!(pending_request_id.as_deref(), Some(old_request));
    assert!(original.is_none());
    let pending_original: serde_json::Value =
        serde_json::from_str(pending_original.as_deref().unwrap()).unwrap();
    assert_eq!(pending_original["request_id"], old_request);
    assert_eq!(pending_original["revision"], 2);

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = ?1",
            [new_request],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0,
        "B 在 A 恢复前不得提交"
    );

    let replay = svc
        .begin(&wid, &node("outline"), Some(old_request.to_string()))
        .unwrap_err();
    assert!(matches!(
        replay,
        Error::EffectPending {
            committed: true,
            ref request_id,
            original: Some(_),
            ..
        } if request_id == old_request
    ));
}

// Task: C002-T25
#[test]
fn malformed_old_effect_row_is_attributed_to_the_blocking_request() {
    let (_dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let old_request = "t25-blob-effects-A";
    svc.begin(&wid, &node("outline"), Some(old_request.into()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET effects_json = ?1, published = 0 WHERE request_id = ?2",
        rusqlite::params![rusqlite::types::Value::Blob(vec![0xff]), old_request],
    )
    .unwrap();
    drop(conn);

    let new_request = "t25-blob-effects-B";
    let error = svc.cancel(&wid, Some(new_request.to_string())).unwrap_err();
    assert_effect_pending(error, false, Some(new_request), Some(old_request));
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = ?1",
            [new_request],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    let (effects_type, published): (String, i64) = conn
        .query_row(
            "SELECT typeof(effects_json), published FROM requests WHERE request_id = ?1",
            [old_request],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(effects_type, "blob");
    assert_eq!(published, 0);
}

// Task: C002-T25
#[test]
fn malformed_current_owner_column_keeps_committed_request_identity() {
    let (_dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let request_id = "t25-blob-current-owner";
    svc.begin(&wid, &node("outline"), Some(request_id.into()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET work_id = ?1, published = 0 WHERE request_id = ?2",
        rusqlite::params![rusqlite::types::Value::Blob(vec![0xff]), request_id],
    )
    .unwrap();
    drop(conn);

    let error = svc
        .begin(&wid, &node("outline"), Some(request_id.into()))
        .unwrap_err();
    let Error::EffectPending {
        committed,
        request_id: actual,
        pending_request_id,
        cause,
        original,
        ..
    } = error
    else {
        panic!("已提交请求的owner类型损坏必须保留提交归属：{error:?}");
    };
    assert!(committed);
    assert_eq!(actual, request_id);
    assert!(pending_request_id.is_none());
    assert_eq!(cause, ErrorCode::StoreCorrupt);
    assert!(original.is_none(), "损坏的owner让next无法可信绑定Work");
}

// Task: C002-T25
#[test]
fn malformed_current_reply_keeps_commit_identity_and_request_conflict_priority() {
    let (_dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let request_id = "t25-blob-current-reply";
    svc.begin(&wid, &node("outline"), Some(request_id.into()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET reply_json = ?1, published = 0 WHERE request_id = ?2",
        rusqlite::params![rusqlite::types::Value::Blob(vec![0xff]), request_id],
    )
    .unwrap();
    drop(conn);

    let replay_error = svc
        .begin(&wid, &node("outline"), Some(request_id.into()))
        .unwrap_err();
    assert_effect_pending_without_original(replay_error, true, request_id, None);

    let conflict = svc.cancel(&wid, Some(request_id.to_string())).unwrap_err();
    assert_eq!(conflict.code(), ErrorCode::RequestConflict, "{conflict:?}");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let (reply_type, row_count): (String, i64) = conn
        .query_row(
            "SELECT typeof(reply_json), COUNT(*) FROM requests WHERE request_id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(reply_type, "blob");
    assert_eq!(row_count, 1);
}

// Task: C002-T25
#[test]
fn malformed_old_reply_is_attributed_to_the_blocking_request() {
    let (_dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let old_request = "t25-blob-reply-A";
    svc.begin(&wid, &node("outline"), Some(old_request.into()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute(
        "UPDATE requests SET reply_json = ?1, published = 0 WHERE request_id = ?2",
        rusqlite::params![rusqlite::types::Value::Blob(vec![0xff]), old_request],
    )
    .unwrap();
    drop(conn);

    let new_request = "t25-blob-reply-B";
    let error = svc.cancel(&wid, Some(new_request.to_string())).unwrap_err();
    assert_effect_pending_without_original(error, false, new_request, Some(old_request));
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = ?1",
            [new_request],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT typeof(reply_json) FROM requests WHERE request_id = ?1",
            [old_request],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        "blob"
    );
}

#[cfg(feature = "failpoint")]
fn submit_at_seal_sync_point(
    mutate: impl FnOnce(&Path),
) -> (
    OwnedTempDir,
    sheltie_runtime::Home,
    sheltie_core::ids::WorkId,
    std::path::PathBuf,
    String,
    u64,
    std::result::Result<sheltie_runtime::Response, Error>,
) {
    let (dir, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let begun = svc.begin(&wid, &node("outline"), None).unwrap();
    let output = std::path::PathBuf::from(output_dir_of(&begun).as_str()).join("outline.md");
    std::fs::write(&output, b"committed bytes").unwrap();
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o644)).unwrap();
    let base_revision = begun.revision;
    drop(svc);

    let request_id = format!("t22-seal-{}", wid.as_str());
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "submit_after_commit_before_seal",
        &request_id,
        rendezvous.path(),
    )
    .unwrap();
    let submit_home = home.clone();
    let submit_work = wid.clone();
    let submit_request = request_id.clone();
    let child = std::thread::spawn(move || {
        service(&submit_home).submit(
            &submit_work,
            &attempt("outline#1.0"),
            &lit("ok"),
            Some(submit_request),
        )
    });
    let release_path = rendezvous.path().join("release");
    let mut worker = RendezvousWorker::single(child, rendezvous.path());
    worker.wait("submit 未到达 COMMIT 后、seal 前的同步点");
    mutate(&output);
    std::fs::write(&release_path, b"release").unwrap();
    let result = worker.finish().unwrap();
    (dir, home, wid, output, request_id, base_revision, result)
}

fn assert_post_commit_seal_failure(
    home: &sheltie_runtime::Home,
    wid: &sheltie_core::ids::WorkId,
    request_id: &str,
    output_path: &Path,
    base_revision: u64,
    result: std::result::Result<sheltie_runtime::Response, Error>,
) {
    match result.unwrap_err() {
        Error::EffectPending {
            committed: true,
            request_id: actual,
            ..
        } => assert_eq!(actual, request_id),
        other => panic!("COMMIT后的seal失败须保留已提交归属：{other:?}"),
    }
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let (revision, published): (u64, i64) = conn
        .query_row(
            "SELECT w.revision, r.published FROM works w JOIN requests r USING(work_id) WHERE r.request_id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(revision, base_revision + 1, "Store必须保留已提交revision");
    assert_eq!(published, 0, "封存失败不得标记效果完成");
    let (owned_work, state_json, effects_json): (String, String, String) = conn
        .query_row(
            "SELECT r.work_id, w.state_json, r.effects_json FROM requests r JOIN works w USING(work_id) WHERE r.request_id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(owned_work, wid.as_str());
    let state: sheltie_core::work::WorkState = serde_json::from_str(&state_json).unwrap();
    let attempt = state
        .attempt(&AttemptId::parse("outline#1.0").unwrap())
        .unwrap();
    let committed = attempt.outputs.get("outline").unwrap();
    assert_eq!(committed.path.as_path(), output_path);
    assert_eq!(
        committed.sha256.as_str(),
        "bd9d61af8c2082b7b8073f3fe347543aefba3dbddb0d6de46638f85ae7040717"
    );
    assert_eq!(committed.bytes, 15);
    let effects: Vec<sheltie_runtime::effects::EffectOp> =
        serde_json::from_str(&effects_json).unwrap();
    let [
        sheltie_runtime::effects::EffectOp::SealOutputs { refs },
        sheltie_runtime::effects::EffectOp::RefreshStatusCard { .. },
    ] = effects.as_slice()
    else {
        panic!("已提交效果须保留原SealOutputs引用：{effects:?}");
    };
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].path, home.to_rel(&abs(output_path)).unwrap());
    assert_eq!(refs[0].sha256, committed.sha256.as_str());
    assert_eq!(refs[0].bytes, committed.bytes);
}

// Task: C002-T22
#[cfg(feature = "failpoint")]
#[test]
fn real_submit_path_swap_after_commit_seals_original_and_preserves_external_target() {
    use std::os::unix::fs::{PermissionsExt as _, symlink};
    let _serial = POST_COMMIT_SYNC_LOCK.lock().unwrap();

    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("sentinel");
    std::fs::write(&sentinel, b"external sentinel").unwrap();
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o640)).unwrap();
    let sentinel_mode = std::fs::metadata(&sentinel).unwrap().permissions();
    let (dir, home, wid, output, request_id, revision, result) =
        submit_at_seal_sync_point(|path| {
            let original = outside.path().join("observed-original");
            std::fs::rename(path, &original).unwrap();
            symlink(&sentinel, path).unwrap();
        });
    let original = outside.path().join("observed-original");
    assert_post_commit_seal_failure(&home, &wid, &request_id, &output, revision, result);
    assert_eq!(
        std::fs::metadata(original).unwrap().permissions().mode() & 0o777,
        0o444
    );
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"external sentinel");
    assert_eq!(
        std::fs::metadata(&sentinel).unwrap().permissions(),
        sentinel_mode
    );
    assert!(
        std::fs::symlink_metadata(output)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    drop(dir);
}

// Task: C002-T22
#[cfg(feature = "failpoint")]
#[test]
fn real_submit_same_inode_byte_change_after_commit_stops_before_chmod() {
    use std::os::unix::fs::PermissionsExt as _;
    let _serial = POST_COMMIT_SYNC_LOCK.lock().unwrap();

    let mut output_mode = None;
    let (dir, home, wid, output, request_id, revision, result) =
        submit_at_seal_sync_point(|path| {
            output_mode = Some(std::fs::metadata(path).unwrap().permissions().mode());
            std::fs::write(path, b"tampered bytes!").unwrap();
        });
    assert_post_commit_seal_failure(&home, &wid, &request_id, &output, revision, result);
    assert_eq!(std::fs::read(&output).unwrap(), b"tampered bytes!");
    assert_eq!(
        std::fs::metadata(&output).unwrap().permissions().mode(),
        output_mode.unwrap()
    );
    drop(dir);
}

// Task: C002-T22
#[cfg(feature = "failpoint")]
#[test]
fn real_submit_hardlink_added_after_commit_stops_before_chmod() {
    use std::os::unix::fs::PermissionsExt as _;
    let _serial = POST_COMMIT_SYNC_LOCK.lock().unwrap();

    let outside = tempfile::tempdir().unwrap();
    let alias = outside.path().join("alias");
    let alias_mode = std::cell::RefCell::new(None);
    let (dir, home, wid, output, request_id, revision, result) =
        submit_at_seal_sync_point(|path| {
            std::fs::hard_link(path, &alias).unwrap();
            *alias_mode.borrow_mut() = Some(std::fs::metadata(&alias).unwrap().permissions());
        });
    assert_post_commit_seal_failure(&home, &wid, &request_id, &output, revision, result);
    assert_ne!(
        std::fs::metadata(&output).unwrap().permissions().mode() & 0o222,
        0
    );
    assert_eq!(
        std::fs::metadata(&alias).unwrap().permissions(),
        alias_mode.into_inner().unwrap()
    );
    drop(dir);
}

// Task: C002-T22
#[test]
fn recovery_refuses_same_inode_output_changed_after_submit() {
    use std::os::unix::fs::PermissionsExt as _;

    let (_dir, home, wid, output, request_id) = committed_unpublished_submit();
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::write(&output, b"tampered bytes!").unwrap();
    mark_submit_unpublished(&home, &request_id);

    recover_submit_stops_and_keeps_unpublished(&home, &wid, &request_id);
    assert_eq!(std::fs::read(output).unwrap(), b"tampered bytes!");
}

// Task: C002-T22
#[test]
fn recovery_refuses_hardlink_added_to_submitted_output() {
    let (dir, home, wid, output, request_id) = committed_unpublished_submit();
    let alias = dir.path().join("output-alias");
    std::fs::hard_link(&output, &alias).unwrap();
    let alias_mode = std::fs::metadata(&alias).unwrap().permissions();
    mark_submit_unpublished(&home, &request_id);

    recover_submit_stops_and_keeps_unpublished(&home, &wid, &request_id);
    assert_eq!(std::fs::metadata(alias).unwrap().permissions(), alias_mode);
}

// Task: C002-T22
#[test]
fn recovery_refuses_output_replaced_by_external_symlink() {
    use std::os::unix::fs::{PermissionsExt as _, symlink};

    let (_dir, home, wid, output, request_id) = committed_unpublished_submit();
    let outside = tempfile::tempdir().unwrap();
    let sentinel = outside.path().join("sentinel");
    std::fs::write(&sentinel, b"external sentinel").unwrap();
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o640)).unwrap();
    let before = std::fs::metadata(&sentinel).unwrap().permissions();
    std::fs::remove_file(&output).unwrap();
    symlink(&sentinel, &output).unwrap();
    mark_submit_unpublished(&home, &request_id);

    recover_submit_stops_and_keeps_unpublished(&home, &wid, &request_id);
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"external sentinel");
    assert_eq!(std::fs::metadata(&sentinel).unwrap().permissions(), before);
}

// Task: C002-T26
#[test]
fn publication_checks_each_start_input_reference_and_owner_field() {
    let corruptions = [
        "start_input_bytes",
        "start_input_ref_length",
        "owner_format",
        "owner_internal_id",
        "owner_request_id",
        "owner_op",
        "effect_owner",
        "effect_digest_root",
    ];
    for corruption in corruptions {
        let (_dir, home, svc, work, request_id, payload, final_path) = unpublished_start_fixture();
        let owner_file = payload.parent().unwrap().parent().unwrap().join(format!(
            "{}.owner",
            payload
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        ));
        match corruption {
            "start_input_bytes" => {
                let input = payload.join("start-inputs/topic");
                std::fs::write(input, "搁新人介绍 Sheltie".as_bytes()).unwrap();
            }
            "start_input_ref_length" => {
                let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
                let work_id: String = conn
                    .query_row(
                        "SELECT work_id FROM requests WHERE request_id = ?1",
                        [&request_id],
                        |row| row.get(0),
                    )
                    .unwrap();
                let state_raw: String = conn
                    .query_row(
                        "SELECT state_json FROM works WHERE work_id = ?1",
                        [&work_id],
                        |row| row.get(0),
                    )
                    .unwrap();
                let audit_raw: String = conn
                    .query_row(
                        "SELECT command_json FROM audit WHERE request_id = ?1",
                        [&request_id],
                        |row| row.get(0),
                    )
                    .unwrap();
                let mut state: serde_json::Value = serde_json::from_str(&state_raw).unwrap();
                let mut command: serde_json::Value = serde_json::from_str(&audit_raw).unwrap();
                state["inputs"]["topic"]["bytes"] =
                    serde_json::json!(state["inputs"]["topic"]["bytes"].as_u64().unwrap() + 1);
                command["inputs"]["topic"]["bytes"] = state["inputs"]["topic"]["bytes"].clone();
                conn.execute(
                    "UPDATE works SET state_json = ?1 WHERE work_id = ?2",
                    rusqlite::params![serde_json::to_string(&state).unwrap(), work_id],
                )
                .unwrap();
                conn.execute(
                    "UPDATE audit SET command_json = ?1 WHERE request_id = ?2",
                    rusqlite::params![serde_json::to_string(&command).unwrap(), request_id],
                )
                .unwrap();
            }
            "owner_format" | "owner_internal_id" | "owner_request_id" | "owner_op" => {
                let mut owner: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&owner_file).unwrap()).unwrap();
                let field = match corruption {
                    "owner_format" => "format",
                    "owner_internal_id" => "internal_id",
                    "owner_request_id" => "request_id",
                    _ => "op",
                };
                owner[field] = match field {
                    "format" => serde_json::json!("pending/v2"),
                    "internal_id" => serde_json::json!("00000000000000000000000000000000"),
                    "request_id" => serde_json::json!("another-opaque-id"),
                    _ => serde_json::json!("add_workbook"),
                };
                let mut bytes = serde_json::to_vec(&owner).unwrap();
                bytes.push(b'\n');
                std::fs::write(owner_file, bytes).unwrap();
            }
            "effect_owner" | "effect_digest_root" => {
                let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
                let raw: String = conn
                    .query_row(
                        "SELECT effects_json FROM requests WHERE request_id = ?1",
                        [&request_id],
                        |row| row.get(0),
                    )
                    .unwrap();
                let mut effects: serde_json::Value = serde_json::from_str(&raw).unwrap();
                if corruption == "effect_owner" {
                    effects[0]["owner"] = serde_json::json!("work:w20260929-999-tampered");
                } else {
                    effects[0]["digest_root"] = serde_json::json!("");
                }
                conn.execute(
                    "UPDATE requests SET effects_json = ?1 WHERE request_id = ?2",
                    rusqlite::params![serde_json::to_string(&effects).unwrap(), request_id],
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        assert_blocked_unpublished_start(
            &home,
            &svc,
            &work,
            &request_id,
            &payload,
            &final_path,
            sheltie_core::error::ErrorCode::StoreCorrupt,
        );
    }
}

// Task: C002-T26
#[test]
fn final_only_publication_recovery_checks_start_input_bytes() {
    let (_dir, home, svc, work, request_id, payload, final_path) = unpublished_start_fixture();
    std::fs::rename(&payload, &final_path).unwrap();
    let input = final_path.join("start-inputs/topic");
    assert_eq!(
        std::fs::read(&input).unwrap().len(),
        "搁新人介绍 Sheltie".len()
    );
    std::fs::write(&input, "搁新人介绍 Sheltie".as_bytes()).unwrap();
    let blocker_id = "t26-blocked-final-only";
    let error = svc.cancel(&work, Some(blocker_id.to_string())).unwrap_err();
    let Error::EffectPending {
        committed,
        pending_request_id,
        cause,
        ..
    } = error
    else {
        panic!("final-only恢复输入不符必须停止：{error:?}");
    };
    assert!(!committed);
    assert_eq!(pending_request_id.as_deref(), Some(request_id.as_str()));
    assert_eq!(cause, sheltie_core::error::ErrorCode::StoreCorrupt);
    assert!(!payload.exists());
    assert!(final_path.exists());
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = ?1",
            [blocker_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}

// Task: C002-T26
#[test]
fn publication_keeps_start_input_permission_failure_as_io() {
    use std::os::unix::fs::PermissionsExt as _;

    let (_dir, home, svc, work, request_id, payload, final_path) = unpublished_start_fixture();
    let input = payload.join("start-inputs/topic");
    let mode = std::fs::metadata(&input).unwrap().permissions().mode();
    std::fs::set_permissions(&input, std::fs::Permissions::from_mode(0o0)).unwrap();
    assert_blocked_unpublished_start(
        &home,
        &svc,
        &work,
        &request_id,
        &payload,
        &final_path,
        sheltie_core::error::ErrorCode::Io,
    );
    assert_eq!(
        std::fs::metadata(&input).unwrap().permissions().mode() & 0o777,
        0
    );
    std::fs::set_permissions(&input, std::fs::Permissions::from_mode(mode)).unwrap();
}

// Task: C002-T26
#[test]
fn real_workbook_add_and_start_final_publications_serve_status_and_show() {
    let (_dir, home) = temp_home();
    let workbook_source = example_dir("two-step");
    let added = repo(&home)
        .add(&abs(&workbook_source), Some("t26-workbook-add".to_string()))
        .unwrap();
    assert_eq!(added.request_id, "t26-workbook-add");
    let shown = repo(&home).load("two-step", None).unwrap();
    assert_eq!(shown.manifest.id().as_str(), "two-step");
    assert_eq!(shown.manifest.version(), "1.0.0");

    let svc = service(&home);
    let started = svc
        .start(
            two_step_args(&[("topic", "给新人介绍 Sheltie")]),
            Some("t26-work-start".to_string()),
        )
        .unwrap();
    let work = work_id_of(&started);
    let (card, _) = svc.status(&work).unwrap();
    assert_eq!(
        std::fs::read_to_string(
            home.work_dir(&work)
                .join_segment("status-card.md")
                .as_path()
        )
        .unwrap(),
        card
    );
    assert_eq!(
        std::fs::read(
            home.work_dir(&work)
                .join_segment("start-inputs/topic")
                .as_path()
        )
        .unwrap(),
        "给新人介绍 Sheltie".as_bytes()
    );

    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    for request_id in ["t26-workbook-add", "t26-work-start"] {
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
    }
}

// Task: C002-T26
#[cfg(feature = "failpoint")]
#[test]
fn publication_sync_failures_leave_effect_pending_and_retry_same_object() {
    use std::os::unix::fs::MetadataExt as _;

    let _serial = POST_COMMIT_SYNC_LOCK.lock().unwrap();
    for sync_point in [
        "publish_file_sync",
        "publish_tree_nested_dir_sync",
        "publish_tree_root_sync",
        "publish_source_parent_sync",
        "publish_target_parent_sync",
        "publish_final_root_sync",
        "publish_readonly_file_sync",
        "publish_readonly_nested_dir_sync",
        "publish_readonly_root_sync",
    ] {
        let (_dir, home, svc, work, request_id, payload, final_path) = unpublished_start_fixture();
        sheltie_runtime::failpoint::arm_sync_error(home.root().as_str(), sync_point).unwrap();
        let error = svc
            .cancel(&work, Some("t26-blocked-request".to_string()))
            .unwrap_err();
        sheltie_runtime::failpoint::disarm_sync_error().unwrap();
        let Error::EffectPending {
            committed,
            pending_request_id,
            cause,
            cause_detail,
            ..
        } = error
        else {
            panic!("sync失败必须准确返回EFFECT_PENDING：{error:?}");
        };
        assert!(!committed);
        assert_eq!(pending_request_id.as_deref(), Some(request_id.as_str()));
        assert_eq!(cause, sheltie_core::error::ErrorCode::Io);
        assert!(cause_detail.contains("injected sync failure"));
        let before_rename = matches!(
            sync_point,
            "publish_file_sync" | "publish_tree_nested_dir_sync" | "publish_tree_root_sync"
        );
        if before_rename {
            assert!(payload.exists(), "rename前文件sync失败时保留pending原件");
            assert!(!final_path.exists(), "文件sync未成功不得rename");
        } else {
            assert!(!payload.exists(), "rename后失败时pending路径不再存在");
            assert!(final_path.exists(), "rename后的唯一final原件必须保留");
        }
        let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM requests WHERE request_id = 't26-blocked-request'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );

        sheltie_runtime::failpoint::arm_sync_error(home.root().as_str(), sync_point).unwrap();
        let retry_error = svc
            .cancel(&work, Some("t26-blocked-request".to_string()))
            .unwrap_err();
        sheltie_runtime::failpoint::disarm_sync_error().unwrap();
        let Error::EffectPending {
            committed,
            pending_request_id,
            cause,
            cause_detail,
            ..
        } = retry_error
        else {
            panic!("恢复重试必须再次经过{sync_point}：{retry_error:?}");
        };
        assert!(!committed);
        assert_eq!(pending_request_id.as_deref(), Some(request_id.as_str()));
        assert_eq!(cause, sheltie_core::error::ErrorCode::Io);
        assert!(cause_detail.contains("injected sync failure"));
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0,
            "同一同步点第二次失败不得mark published"
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM requests WHERE request_id = 't26-blocked-request'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );

        let inode = if final_path.exists() {
            Some(std::fs::metadata(&final_path).unwrap().ino())
        } else {
            None
        };
        svc.cancel(&work, Some("t26-blocked-request".to_string()))
            .unwrap();
        if let Some(inode) = inode {
            assert_eq!(std::fs::metadata(&final_path).unwrap().ino(), inode);
        }
        assert_eq!(
            std::fs::read(final_path.join("start-inputs/topic")).unwrap(),
            "给新人介绍 Sheltie".as_bytes()
        );
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
    }
}

// Task: C002-T26
#[cfg(feature = "failpoint")]
#[test]
fn start_replay_sync_failure_preserves_own_commit_snapshot() {
    use std::os::unix::fs::MetadataExt as _;

    let _serial = POST_COMMIT_SYNC_LOCK.lock().unwrap();
    let (_dir, home, svc, work, request_id, payload, final_path) = unpublished_start_fixture();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let original_reply: String = conn
        .query_row(
            "SELECT reply_json FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| row.get(0),
        )
        .unwrap();

    for retry_index in 0..2 {
        sheltie_runtime::failpoint::arm_sync_error(
            home.root().as_str(),
            "publish_source_parent_sync",
        )
        .unwrap();
        let error = svc
            .start(
                two_step_args(&[("topic", "给新人介绍 Sheltie")]),
                Some(request_id.clone()),
            )
            .unwrap_err();
        sheltie_runtime::failpoint::disarm_sync_error().unwrap();
        let Error::EffectPending {
            committed,
            request_id: actual_request,
            pending_request_id,
            cause,
            original,
            ..
        } = error
        else {
            panic!("同一start重放的sync失败必须保留本请求归属：{error:?}");
        };
        assert!(committed);
        assert_eq!(actual_request, request_id);
        assert!(pending_request_id.is_none());
        assert_eq!(cause, sheltie_core::error::ErrorCode::Io);
        let original: serde_json::Value =
            serde_json::from_str(original.as_deref().unwrap()).unwrap();
        assert_eq!(original["ok"], true);
        assert_eq!(original["request_id"], request_id);
        assert_eq!(original["revision"], 1);
        assert!(original["next"].is_array());
        assert_eq!(original["data"]["work_id"], work.as_str());
        assert!(!payload.exists());
        assert!(final_path.exists());
        assert_eq!(
            conn.query_row(
                "SELECT published FROM requests WHERE request_id = ?1",
                [&request_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0,
            "第 {retry_index} 次同请求恢复的父sync仍失败，不得mark"
        );
    }

    let inode = std::fs::metadata(&final_path).unwrap().ino();
    let replayed = svc
        .start(
            two_step_args(&[("topic", "给新人介绍 Sheltie")]),
            Some(request_id.clone()),
        )
        .unwrap();
    assert!(replayed.replayed);
    assert_eq!(std::fs::metadata(&final_path).unwrap().ino(), inode);
    assert_eq!(
        std::fs::read(final_path.join("start-inputs/topic")).unwrap(),
        "给新人介绍 Sheltie".as_bytes()
    );
    let (published, reply_after, row_count): (i64, String, i64) = conn
        .query_row(
            "SELECT published, reply_json, COUNT(*) FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(published, 1);
    assert_eq!(
        reply_after, original_reply,
        "重放不可改写提交时snapshot字节"
    );
    assert_eq!(row_count, 1);
}

// Task: C002-T26
#[cfg(feature = "failpoint")]
#[test]
fn publication_refuses_payload_directory_replaced_after_sync() {
    let _serial = POST_COMMIT_SYNC_LOCK.lock().unwrap();
    let (dir, home, svc, work, request_id, payload, final_path) = unpublished_start_fixture();
    let moved = dir.path().join("synced-original-payload");
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "publish_after_tree_sync",
        &request_id,
        rendezvous.path(),
    )
    .unwrap();
    let recovery_service = svc.clone();
    let recovery_work = work.clone();
    let recovery = std::thread::spawn(move || {
        recovery_service.cancel(&recovery_work, Some("t26-replacement-blocker".to_string()))
    });
    let release = rendezvous.path().join("release");
    let mut worker = RendezvousWorker::single(recovery, rendezvous.path());
    worker.wait("发布没有到达目录同步后的身份交错点");
    std::fs::rename(&payload, &moved).unwrap();
    std::fs::create_dir(&payload).unwrap();
    std::fs::create_dir(payload.join("start-inputs")).unwrap();
    std::fs::write(
        payload.join("start-inputs/topic"),
        b"unverified replacement",
    )
    .unwrap();
    std::fs::write(&release, b"release").unwrap();

    let error = worker.finish().unwrap().unwrap_err();
    let Error::EffectPending {
        committed,
        pending_request_id,
        cause,
        ..
    } = error
    else {
        panic!("替换原件必须停止并保留归属：{error:?}");
    };
    assert!(!committed);
    assert_eq!(pending_request_id.as_deref(), Some(request_id.as_str()));
    assert_eq!(cause, sheltie_core::error::ErrorCode::StoreCorrupt);
    assert!(moved.join("start-inputs/topic").exists());
    assert_eq!(
        std::fs::read(moved.join("start-inputs/topic")).unwrap(),
        "给新人介绍 Sheltie".as_bytes()
    );
    assert_eq!(
        std::fs::read(payload.join("start-inputs/topic")).unwrap(),
        b"unverified replacement"
    );
    assert!(!final_path.exists(), "未核验的替代目录不得发布");
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [&request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}
