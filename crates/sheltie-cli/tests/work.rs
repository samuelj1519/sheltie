//! T18：`work` 组。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use predicates::prelude::*;

// Task: T18
#[test]
fn work_start_creates_work_and_prints_next() {
    let env = Env::new();
    env.add_example("two-step");
    let v = env.ok(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    assert!(v["data"]["work_id"].as_str().unwrap().ends_with("-default"));
    assert_eq!(next_begin_nodes(&v), vec!["outline"]);
    assert!(next_ops(&v).contains(&"work cancel".to_string()));
    assert!(
        env.work_dir(v["data"]["work_id"].as_str().unwrap())
            .join("status-card.md")
            .exists()
    );
}

// Task: T18
#[test]
fn work_start_missing_input_exits_1_with_input_missing() {
    let env = Env::new();
    env.add_example("two-step");
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "INPUT_MISSING");
}

// Task: T18
#[test]
fn work_start_accepts_at_file_input() {
    let env = Env::new();
    env.add_example("two-step");
    let f = env.dir.path().join("topic.txt");
    std::fs::write(&f, "来自文件").unwrap();
    let wid = env.start("two-step", &[("topic", &format!("@{}", f.display()))]);
    let content = std::fs::read_to_string(env.work_dir(&wid).join("start-inputs/topic")).unwrap();
    assert_eq!(content, "来自文件");
}

// Task: T18
#[test]
fn work_list_shows_status_and_current() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let v = env.ok(&["work", "list"]);
    let rows = v["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["work_id"], wid);
    assert_eq!(rows[0]["status"]["kind"], "active");
    assert_eq!(rows[0]["current"], "outline#1");
}

// Task: T18
#[test]
fn work_status_prints_status_card() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.cmd_text(&["work", "status", &wid])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("# Work {wid}"))
                .and(predicate::str::contains("current: outline#1")),
        );
}

// Task: T18
#[test]
fn work_stats_prints_table_and_json() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.begin(&wid, "outline");
    env.cmd_text(&["work", "stats", &wid])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("# Stats {wid}"))
                .and(predicate::str::contains("| outline | 1/1 | 1 | 0 |")),
        );
    let v = env.ok(&["work", "stats", &wid]);
    assert_eq!(v["data"]["nodes"][0]["node"], "outline");
    assert_eq!(v["data"]["nodes"][0]["attempts"], 1);
    assert_eq!(v["data"]["nodes"][1]["attempts"], 0);
}

// Task: T18
#[test]
fn work_status_json_matches_schema() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let v = env.status(&wid);
    for key in [
        "work_id",
        "name",
        "workbook",
        "flow",
        "status",
        "current",
        "done",
        "pending",
        "visits",
        "last_attempt",
        "next",
    ] {
        assert!(v["data"].get(key).is_some(), "缺 {key}");
    }
    assert_eq!(v["data"]["current"], "outline#1");
}

// Task: T18
#[test]
fn work_cancel_then_any_write_is_work_terminal() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let v = env.ok(&["work", "cancel", &wid]);
    assert!(v["next"].as_array().unwrap().is_empty());
    let (e, _) = env.fail(&["attempt", "begin", &wid, "--node", "outline"]);
    assert_eq!(e["error"]["code"], "WORK_TERMINAL");
}

// Task: T18
#[test]
fn work_id_prefix_resolves_when_unique() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let prefix = &wid[..14];
    assert_eq!(env.status(prefix)["data"]["work_id"], wid);
}

// Task: T18
#[test]
fn work_id_prefix_ambiguous_lists_candidates() {
    let env = Env::new();
    env.add_example("two-step");
    let a = env.start("two-step", &[("topic", "x")]);
    let b = env.start("two-step", &[("topic", "y")]);
    let (e, _) = env.fail(&["work", "status", &a[..10]]);
    assert_eq!(e["error"]["code"], "INVALID_REQUEST");
    let msg = e["error"].to_string();
    assert!(msg.contains(&a) && msg.contains(&b));
}

// Task: T18
#[test]
fn work_start_default_name_is_flow_id() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    assert!(wid.ends_with("-default"));
}

// Task: T18
#[test]
fn work_start_with_chinese_name_creates_matching_directory() {
    let env = Env::new();
    env.add_example("two-step");
    let v = env.ok(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--name",
        "文章 初稿",
        "--input",
        "topic=x",
    ]);
    let wid = v["data"]["work_id"].as_str().unwrap();
    assert!(wid.ends_with("-文章-初稿"), "{wid}");
    assert!(env.work_dir(wid).is_dir());
}

// Task: C002-T28
#[test]
fn work_status_reports_pending_publication_while_reading_the_frozen_copy() {
    let env = Env::new();
    env.add_example("two-step");
    let started = env.ok(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=pending",
        "--request-id",
        "t28-cli-pending-work",
    ]);
    let work = started["data"]["work_id"].as_str().unwrap();
    let connection = rusqlite::Connection::open(env.dir.path().join("store.db")).unwrap();
    let effects: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-cli-pending-work'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let payload = env.dir.path().join(pending);
    let internal_id = pending.split('/').nth(1).unwrap();
    std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
    let owner = serde_json::json!({
        "format": "pending/v1",
        "internal_id": internal_id,
        "request_id": "t28-cli-pending-work",
        "op": "start_work",
    });
    std::fs::write(
        env.dir.path().join(format!("pending/{internal_id}.owner")),
        format!("{}\n", serde_json::to_string(&owner).unwrap()),
    )
    .unwrap();
    let lock_path = env.dir.path().join(".lock");
    std::fs::remove_file(&lock_path).unwrap();
    connection
        .execute(
            "UPDATE requests SET published = 0 WHERE request_id = 't28-cli-pending-work'",
            [],
        )
        .unwrap();
    drop(connection);
    let store_before = std::fs::read(env.dir.path().join("store.db")).unwrap();
    std::fs::rename(env.work_dir(work), payload).unwrap();

    let status = env.ok(&["work", "status", work]);
    assert_eq!(status["data"]["pending_publish"], true);
    assert_eq!(status["data"]["status"]["kind"], "active");
    let text = env.cmd_text(&["work", "status", work]).output().unwrap();
    assert!(String::from_utf8_lossy(&text.stdout).contains("待完成"));
    assert_eq!(
        std::fs::read(env.dir.path().join("store.db")).unwrap(),
        store_before
    );
    assert!(!lock_path.exists(), "只读status不能创建HomeLock");
    assert!(
        !env.work_dir(work).exists(),
        "read-only status does not recover"
    );
}

// ── C002-T02：start 的无副作用预检（GF-30） ─────────────────────

/// 独立 oracle：递归列出管理根下的相对路径（跳过 SQLite 的 -wal/-shm，连接关闭时会回收）。
fn snapshot_tree(root: &std::path::Path) -> Vec<String> {
    fn walk(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map(|it| it.flatten().collect())
            .unwrap_or_default();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with("-wal") || name.ends_with("-shm") {
                continue;
            }
            let rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            out.push(rel.clone());
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                walk(&entry.path(), &rel, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, "", &mut out);
    out
}

// Task: C002-T02
#[test]
fn start_deterministic_rejections_leave_home_unchanged_and_do_not_burn_seq() {
    let env = Env::new();
    env.add_example("two-step");
    let before = snapshot_tree(std::path::Path::new(&env.home()));

    // 缺 topic：INPUT_MISSING，detail 点名缺的键。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "INPUT_MISSING");
    assert_eq!(
        v["error"]["detail"]["missing"],
        serde_json::json!(["topic"])
    );
    assert_eq!(snapshot_tree(std::path::Path::new(&env.home())), before);

    // 多 key：INPUT_MISSING，detail 点名多的键。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
        "--input",
        "bonus=y",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "INPUT_MISSING");
    assert_eq!(v["error"]["detail"]["extra"], serde_json::json!(["bonus"]));
    assert_eq!(snapshot_tree(std::path::Path::new(&env.home())), before);

    // 非法名字：INVALID_REQUEST。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--name",
        "a/b",
        "--input",
        "topic=x",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "INVALID_REQUEST");
    assert_eq!(snapshot_tree(std::path::Path::new(&env.home())), before);

    // 缺 workbook：NOT_FOUND。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "ghost",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "NOT_FOUND");
    assert_eq!(snapshot_tree(std::path::Path::new(&env.home())), before);

    // 缺 flow：NOT_FOUND。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "ghost",
        "--input",
        "topic=x",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "NOT_FOUND");
    assert_eq!(snapshot_tree(std::path::Path::new(&env.home())), before);

    // 拒绝后只补缺条件即成功；当日序号没有被烧掉，仍是 001。
    let wid = env.start("two-step", &[("topic", "x")]);
    assert!(wid.contains("-001-"), "{wid}");
}

// Task: C002-T02
#[test]
fn failed_start_on_new_home_creates_nothing() {
    let env = Env::new();
    // 新管理根上没有任何已装 Workbook；失败的 start 不得建 store.db 或任何目录。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "NOT_FOUND");
    let entries: Vec<_> = std::fs::read_dir(env.dir.path()).unwrap().collect();
    assert!(entries.is_empty(), "管理根必须为空：{entries:?}");

    // 补上「安装 Workbook」这个条件后同参数成功。
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    assert!(wid.contains("-001-"), "{wid}");
}

// Task: C002-T02
#[test]
fn start_parameter_syntax_errors_precede_storage_and_bad_files_are_invalid_requests() {
    let env = Env::new();

    // `--input` 的值不是 k=v：参数错误，退出码 2。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "noequal",
    ]);
    assert_eq!(code, 2);
    assert_eq!(v["error"]["code"], "INVALID_REQUEST");

    // 缺 `--workbook` 必填参数：clap 退出码 2。
    let out = env
        .cmd(&["work", "start", "--flow", "default", "--input", "topic=x"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

// Task: C002-T24
#[test]
fn invalid_new_input_file_is_runtime_invalid_request_with_path_and_reason() {
    let env = Env::new();
    env.add_example("two-step");
    let outside = tempfile::tempdir().unwrap();
    let external_file = outside.path().join("source.txt");
    std::fs::write(&external_file, b"external input sentinel").unwrap();
    let linked_parent = env.dir.path().join("linked-parent");
    std::os::unix::fs::symlink(outside.path(), &linked_parent).unwrap();
    let linked_input = format!("topic=@{}", linked_parent.join("source.txt").display());
    let (response, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        &linked_input,
    ]);
    assert_eq!(code, 2);
    assert_eq!(response["error"]["code"], "INVALID_REQUEST");
    assert!(
        response["error"]["detail"]["reason"]
            .as_str()
            .unwrap()
            .contains("符号链接")
    );
    assert_eq!(
        std::fs::read(external_file).unwrap(),
        b"external input sentinel"
    );

    let missing = env.dir.path().join("missing-input.txt");
    let input = format!("topic=@{}", missing.display());
    let (response, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        &input,
    ]);
    assert_eq!(code, 2);
    assert_eq!(response["error"]["code"], "INVALID_REQUEST");
    assert!(
        response["error"]["detail"]["path"]
            .as_str()
            .unwrap()
            .contains("missing-input.txt")
    );
    assert!(response["error"]["detail"]["reason"].is_string());
    assert!(
        env.ok(&["work", "list"])
            .get("data")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

// Task: C002-T10
#[test]
fn request_id_rejected_for_readonly_and_self_commands() {
    let env = Env::new();
    env.add_example("two-step");
    for args in [
        vec!["--request-id", "r-1", "workbook", "list"],
        vec!["--request-id", "r-1", "work", "status", "any"],
        vec!["--request-id", "r-1", "self", "version"],
    ] {
        let (v, code) = env.fail(&args);
        assert_eq!(code, 2, "{args:?}：只读与 self 不支持 request-id");
        assert_eq!(v["error"]["code"], "INVALID_REQUEST", "{args:?}");
    }
}

// Task: C002-T10
#[test]
fn named_but_missing_workbook_is_not_silently_replaced() {
    let env = Env::new();
    env.add_example("two-step");
    // 协调者按用户指定的 ghost 开工：NOT_FOUND，不换已装的 two-step。
    let (v, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "ghost",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "NOT_FOUND");
    assert!(
        v["error"]["message"].as_str().unwrap().contains("ghost"),
        "{v}"
    );
    // 没有任何 Work 被创建。
    assert!(
        env.ok(&["work", "list"])["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

// Task: C002-T10
#[test]
fn resume_after_replay_reads_current_status_not_historical_next() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let rid = "11111111-2222-3333-4444-555555555555";
    let begun = env.begin(&wid, "outline");
    for (_, path) in begun["data"]["outputs"].as_object().unwrap() {
        let p = std::path::Path::new(path.as_str().unwrap());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "输出内容\n").unwrap();
    }
    let first = env.ok(&[
        "attempt",
        "submit",
        &wid,
        "--attempt",
        "outline#1.0",
        "--summary",
        "完成",
        "--request-id",
        rid,
    ]);
    env.ok(&["work", "cancel", &wid]);
    // 同 id 重放：返回原快照（replayed=true），历史 next 保留；当前状态仍是 cancelled，
    // 续接以 status 为准，不用历史 next。
    let replay = env.ok(&[
        "attempt",
        "submit",
        &wid,
        "--attempt",
        "outline#1.0",
        "--summary",
        "完成",
        "--request-id",
        rid,
    ]);
    assert_eq!(replay["data"]["replayed"], serde_json::json!(true));
    assert_eq!(replay["data"]["attempt"], first["data"]["attempt"]);
    let status = env.status(&wid);
    assert_eq!(
        status["data"]["status"],
        serde_json::json!({"kind": "cancelled"})
    );
}

// Task: C002-T29
#[cfg(feature = "failpoint")]
#[test]
fn stats_and_next_keep_one_snapshot_when_a_writer_begins_after_reader_load() {
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};
    struct PausedReader {
        child: Option<Child>,
        release: std::path::PathBuf,
    }
    impl Drop for PausedReader {
        fn drop(&mut self) {
            if let Some(child) = self.child.as_mut() {
                let _ = std::fs::write(&self.release, b"release");
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "snapshot")]);
    let sync = tempfile::tempdir().unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_sheltie"))
        .args(["--home", &env.home(), "--json", "work", "stats", &work])
        .env("SHELTIE_TEST_RENDEZVOUS_NAME", "stats_after_load")
        .env("SHELTIE_TEST_RENDEZVOUS_ID", &work)
        .env("SHELTIE_TEST_RENDEZVOUS_DIR", sync.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = PausedReader {
        child: Some(child),
        release: sync.path().join("release"),
    };
    let deadline = Instant::now() + Duration::from_secs(15);
    while !sync.path().join("reached").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !sync.path().join("reached").exists() {
        panic!("真实CLI读者未到达stats装入后的同步点");
    }
    let writer = env
        .cmd(&["attempt", "begin", &work, "--node", "outline"])
        .timeout(Duration::from_secs(5))
        .output();
    std::fs::write(sync.path().join("release"), b"release").unwrap();
    let result = reader.child.take().unwrap().wait_with_output().unwrap();
    let writer = writer.unwrap();
    assert!(
        writer.status.success(),
        "{}",
        String::from_utf8_lossy(&writer.stderr)
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let old: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(old["data"]["nodes"][0]["attempts"], 0);
    assert_eq!(old["data"]["nodes"][0]["visits"], 1);
    assert_eq!(
        old["next"],
        serde_json::json!([
        {"op":"attempt begin", "args":{"work":work,"node":"outline"}, "executor":"agent", "tier":"standard"},
            {"op":"work cancel", "args":{"work":work}}
        ])
    );
    let current = env.ok(&["work", "stats", &work]);
    assert_eq!(current["data"]["nodes"][0]["attempts"], 1);
    assert_eq!(
        current["next"],
        serde_json::json!([
            {"op":"attempt submit", "args":{"work":work,"attempt":"outline#1.0"}},
            {"op":"attempt fail", "args":{"work":work,"attempt":"outline#1.0"}},
            {"op":"work cancel", "args":{"work":work}}
        ])
    );
}
