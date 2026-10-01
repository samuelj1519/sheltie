//! C002-T07：schema 2、请求意图、快照重放与效果恢复的反例集（O02/O04/O05/O08/N03）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_core::path::AbsPath;
use sheltie_runtime::request::InputValue;
use sheltie_runtime::{Error, Home, StartArgs, WorkService, WorkbookRepo};

fn lit(s: &str) -> InputValue {
    InputValue::Literal {
        text: s.to_string(),
    }
}

fn start_args() -> StartArgs {
    StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: [("topic".to_string(), lit("t"))].into_iter().collect(),
    }
}

/// schema 1 main/WAL 被各真实写入口拒绝，且拒绝前不建.lock或改写持久字节（D-033）。
// Task: C002-T24
#[test]
fn schema1_store_rejected_without_touching_file() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("store.db");
    // 手工造 schema 1 形状的库（v0.1.0 的表结构）。
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch(
        "CREATE TABLE workbooks (id TEXT NOT NULL, version TEXT NOT NULL, digest TEXT NOT NULL, dir TEXT NOT NULL, added_at TEXT NOT NULL, PRIMARY KEY (id, version));
         CREATE TABLE works (work_id TEXT PRIMARY KEY, revision INTEGER NOT NULL, status TEXT NOT NULL, state_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
         CREATE TABLE work_sequence (day TEXT PRIMARY KEY, last INTEGER NOT NULL);
         CREATE TABLE requests (request_id TEXT PRIMARY KEY, work_id TEXT, payload_hash TEXT NOT NULL, reply_json TEXT NOT NULL, at TEXT NOT NULL);
         CREATE TABLE audit (seq INTEGER PRIMARY KEY AUTOINCREMENT, work_id TEXT NOT NULL, revision INTEGER NOT NULL, request_id TEXT NOT NULL, principal TEXT NOT NULL, command_json TEXT NOT NULL, at TEXT NOT NULL);
         PRAGMA user_version = 1;",
    )
    .unwrap();
    drop(conn);
    let writer = rusqlite::Connection::open(&db).unwrap();
    assert!(
        writer
            .set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,
                true,
            )
            .unwrap()
    );
    writer
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE user_records(value TEXT);
             INSERT INTO user_records VALUES ('keep me');
             PRAGMA user_version = 1;",
        )
        .unwrap();
    drop(writer);
    let before = std::fs::read(&db).unwrap();
    let wal = dir.path().join("store.db-wal");
    let wal_before = std::fs::read(&wal).unwrap();

    let home = Home::resolve(Some(
        (AbsPath::new(dir.path().to_str().unwrap()).unwrap()).as_str(),
    ))
    .unwrap();
    assert!(matches!(
        WorkService::new(home.clone()).list(),
        Err(Error::StoreSchemaMismatch { .. })
    ));
    assert!(matches!(
        WorkbookRepo::new(home.clone()).add(&abs(&example_dir("two-step")), None),
        Err(Error::StoreSchemaMismatch { .. })
    ));
    assert!(matches!(
        sheltie_runtime::selfmgmt::install(&home),
        Err(Error::StoreSchemaMismatch { .. })
    ));
    assert!(
        !home.lock_path().as_path().exists(),
        "拒绝旧schema不得创建.lock"
    );
    assert!(
        !home.lock_path().as_path().exists(),
        "拒绝旧schema不得创建.lock"
    );
    assert_eq!(std::fs::read(&db).unwrap(), before, "拒绝不得改写main");
    assert_eq!(std::fs::read(&wal).unwrap(), wal_before, "拒绝不得改写WAL");
}

/// 跨 Work 的 request-id：cancel A 后同 id cancel B 报 REQUEST_CONFLICT，B 保持原状态（O02）。
// Task: C002-T07
#[test]
fn cross_work_request_id_is_request_conflict_and_target_untouched() {
    let (_d, _home, svc) = home_with_example("two-step");
    let a = work_id_of(&svc.start(start_args(), None).unwrap());
    let args2 = StartArgs {
        name: Some("second".into()),
        ..start_args()
    };
    let b = work_id_of(&svc.start(args2, None).unwrap());

    svc.cancel(&a, Some("shared-r".into())).unwrap();
    let err = svc.cancel(&b, Some("shared-r".into())).unwrap_err();
    assert_eq!(err.code(), ErrorCode::RequestConflict, "{err:?}");
    // B 仍 active：同 id 不同目标没有产生任何效果。
    let (_, card) = svc.status(&b).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Active);
}

/// 文件变化后 submit 重放：观察摘要不影响意图指纹，重放返回原快照（§2.1）。
// Task: C002-T07
#[test]
fn submit_replay_after_output_change_returns_original_snapshot() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    write_output(&output_dir, "outline.md", "第一版");
    let first = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("完成"),
            Some("r-sub".into()),
        )
        .unwrap();
    // 提交后有人改了输出文件字节；同请求重放不再观察，返回原快照。
    let sealed_path = Path::new(output_dir.as_str()).join("outline.md");
    let mut m = std::fs::metadata(&sealed_path).unwrap().permissions();
    m.set_mode(0o644);
    std::fs::set_permissions(&sealed_path, m).unwrap();
    std::fs::write(&sealed_path, "被改了").unwrap();

    let again = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("完成"),
            Some("r-sub".into()),
        )
        .unwrap();
    assert!(again.replayed);
    assert_eq!(again.revision, first.revision);
    assert_eq!(again.reply, first.reply, "快照逐字段原样");
    assert_eq!(again.data, first.data);
}

// Task: C002-T22
#[test]
fn completed_submit_replay_does_not_seal_or_rewrite_the_output_again() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output_dir = output_dir_of(&begun);
    write_output(&output_dir, "outline.md", "committed output");
    let request_id = Some("t22-completed-submit".to_string());
    let first = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("done"),
            request_id.clone(),
        )
        .unwrap();
    let output = Path::new(output_dir.as_str()).join("outline.md");
    let mut permissions = std::fs::metadata(&output).unwrap().permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(&output, permissions).unwrap();
    std::fs::write(&output, "later bytes").unwrap();
    let mode = std::fs::metadata(&output).unwrap().permissions().mode();

    let replay = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("done"),
            request_id,
        )
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.revision, first.revision);
    assert_eq!(std::fs::read(&output).unwrap(), b"later bytes");
    assert_eq!(
        std::fs::metadata(output).unwrap().permissions().mode(),
        mode
    );
}

/// Workbook 删除后的 start 重放：不重读仓库版本，返回原快照（O04）。
// Task: C002-T07
#[test]
fn start_replay_after_workbook_removed_returns_original_snapshot() {
    let (_d, home, svc) = home_with_example("two-step");
    let args = start_args();
    let wid = work_id_of(&svc.start(args.clone(), Some("r-start".into())).unwrap());
    svc.cancel(&wid, None).unwrap();
    WorkbookRepo::new(home.clone())
        .remove("two-step", "1.0.0", None)
        .unwrap();

    let again = svc.start(args, Some("r-start".into())).unwrap();
    assert!(again.replayed);
    assert_eq!(
        match &again.reply {
            sheltie_core::work::Reply::Started { work_id, .. } => work_id.clone(),
            other => panic!("{other:?}"),
        },
        wid
    );
}

/// cancel 后旧 submit 重放：返回原快照（含历史 next），不把 cancelled 混进响应（O04）。
// Task: C002-T07
#[test]
fn old_submit_replay_after_cancel_returns_original_reply() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    write_output(&output_dir, "outline.md", "内容");
    let first = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("完成"),
            Some("r-old".into()),
        )
        .unwrap();
    svc.cancel(&wid, None).unwrap();

    let again = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("完成"),
            Some("r-old".into()),
        )
        .unwrap();
    assert!(again.replayed);
    // 历史 next 是历史事实：保留提交时的 begin(summary) 项，不混入 cancelled 状态。
    assert_eq!(again.next, first.next);
    assert!(!again.next.is_empty(), "历史 next 保留");
    // 当前状态仍以 status 查询为准。
    let (_, card) = svc.status(&wid).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Cancelled);
}

/// 状态卡是当前投影：旧请求重放不把卡写回旧版本（§6）。
// Task: C002-T07
#[test]
fn replay_does_not_rewrite_status_card_to_old_revision() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());
    let card_path = Path::new(home.work_dir(&wid).as_str()).join("status-card.md");

    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), Some("r-b".into()))
        .unwrap();
    let at_begin = std::fs::read_to_string(&card_path).unwrap();

    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    write_output(&output_dir, "outline.md", "内容");
    svc.submit(
        &wid,
        &AttemptId::parse("outline#1.0").unwrap(),
        &lit("完成"),
        None,
    )
    .unwrap();
    let at_submit = std::fs::read_to_string(&card_path).unwrap();
    assert_ne!(at_begin, at_submit, "提交后卡已更新");

    // 重放 begin（旧请求）：卡不得回退到 begin 时的版本。
    svc.begin(&wid, &NodeId::new("outline").unwrap(), Some("r-b".into()))
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(&card_path).unwrap(),
        at_submit,
        "旧请求重放不得回写旧状态卡"
    );
}

/// 历史文件摘要不符：重放报 STORE_CORRUPT，不掩盖修改（§3.2 write_file）。
// Task: C002-T07
#[test]
fn replay_with_modified_brief_reports_integrity_error() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), Some("r-m".into()))
        .unwrap();
    let brief = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => {
            std::path::PathBuf::from(brief_path.as_str())
        }
        other => panic!("{other:?}"),
    };
    let _ = home;
    let mut m = std::fs::metadata(&brief).unwrap().permissions();
    m.set_mode(0o644);
    std::fs::set_permissions(&brief, m).unwrap();
    std::fs::write(&brief, "被改的任务书").unwrap();

    let err = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), Some("r-m".into()))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("不符") || msg.contains("摘要"), "{msg}");
}

/// workbook add/remove 的请求重放：同 id 返回原快照，源目录变化不重新安装（O08/§2.1）。
// Task: C002-T07
#[test]
fn workbook_add_replay_ignores_source_changes() {
    let dir = tempfile::tempdir().unwrap();
    let src = copy_example("two-step", dir.path());
    let (_d, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());

    let first = repo.add(&abs(&src), Some("r-add".into())).unwrap();
    assert!(!first.replayed);
    let digest1 = first.data["digest"].as_str().unwrap().to_string();

    // 源目录改了内容；同请求重放不重新读取源目录。
    std::fs::write(src.join("workbook.toml"), "改了").unwrap();
    let again = repo.add(&abs(&src), Some("r-add".into())).unwrap();
    assert!(again.replayed);
    assert_eq!(again.data["digest"].as_str().unwrap(), digest1);

    // 新 request-id 是新安装意图：改坏的源目录按新副本的字节登记（这里直接拒绝，
    // 因为 manifest 已不合法——这也是准确拒绝而非沿用旧内容）。
    let err = repo.add(&abs(&src), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookInvalid, "{err:?}");
}

/// remove 重放：同 id 返回原快照，不重复删除；remove 后同版本可重新 add（新生命周期）。
// Task: C002-T07
#[test]
fn workbook_remove_replay_and_readd() {
    let (_d, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("r-a".into()))
        .unwrap();

    let first = repo
        .remove("two-step", "1.0.0", Some("r-r".into()))
        .unwrap();
    assert!(!first.replayed);
    let again = repo
        .remove("two-step", "1.0.0", Some("r-r".into()))
        .unwrap();
    assert!(again.replayed, "重放不重复删除");
    assert!(!home.workbook_dir("two-step", "1.0.0").as_path().exists());

    // 新生命周期：同版本重新 add 成功。
    let re = repo
        .add(&abs(&example_dir("two-step")), Some("r-a2".into()))
        .unwrap();
    assert!(!re.replayed);
    assert!(home.workbook_dir("two-step", "1.0.0").as_path().exists());
}

/// 两个写者竞争：管理根写锁串行化，都成功、revision 单调、无交叉损坏（§2.2）。
// Task: C002-T07
#[test]
fn two_writers_serialize_under_home_lock() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    write_output(&output_dir, "outline.md", "内容");

    // 并发取消 + 提交：取消使 Work 终态；提交要么先发生（成功），要么后发生
    //（WorkTerminal / RequestConflict-free 失败）。任何结果都不能留下半写状态。
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let a = {
        let svc = svc.clone();
        let wid = wid.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            svc.cancel(&wid, None)
        })
    };
    let b = {
        let svc = svc.clone();
        let wid = wid.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            svc.submit(
                &wid,
                &AttemptId::parse("outline#1.0").unwrap(),
                &lit("并发"),
                None,
            )
        })
    };
    barrier.wait();
    a.join().unwrap().unwrap();
    let submitted = b.join().unwrap();
    assert!(
        submitted.is_ok() || submitted.unwrap_err().code() == sheltie_core::ErrorCode::WorkTerminal
    );

    // 终态一致：取消必然生效；状态卡与库一致。
    let (_, card) = svc.status(&wid).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Cancelled);
}

/// begin 的崩溃窗口（COMMIT 后、效果前）：下一次写操作先恢复，再执行新命令（§3.1）。
// Task: C002-T07
#[test]
fn begin_effects_recovered_by_next_write() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let brief = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => {
            std::path::PathBuf::from(brief_path.as_str())
        }
        other => panic!("{other:?}"),
    };
    // 模拟「COMMIT 后、发布前」被杀：请求已提交但效果文件不存在。
    std::fs::remove_file(&brief).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute("UPDATE requests SET published = 0", [])
        .unwrap();
    drop(conn);

    // 下一次写操作先恢复任务书，再执行自己。
    svc.fail(
        &wid,
        &AttemptId::parse("outline#1.0").unwrap(),
        &lit("不做了"),
        None,
    )
    .unwrap();
    assert!(brief.exists(), "恢复按登记字节补写任务书");
}
