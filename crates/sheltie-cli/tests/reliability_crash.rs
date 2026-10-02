//! 精确窗口的真实子进程终止；exit70与SIGKILL分别运行，不模拟Store提交。
#![cfg(feature = "failpoint")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

#[cfg(feature = "failpoint")]
use common::process::Process;
use common::store::store_rows;
use common::*;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
enum Termination {
    Exit70,
    Kill,
}
const MODES: [Termination; 2] = [Termination::Exit70, Termination::Kill];

fn stop(env: &Env, args: &[&str], point: &str, mode: Termination) {
    let output = match mode {
        Termination::Exit70 => {
            let mut command = env.cmd(args);
            command
                .env("SHELTIE_FAILPOINT", point)
                .timeout(Duration::from_secs(10));
            let output = command.output().unwrap();
            assert_eq!(output.status.code(), Some(70), "{point}: {output:?}");
            output
        }
        Termination::Kill => {
            let directory = tempfile::tempdir().unwrap();
            let mut process = Process::spawn(env, args, Some((point, point, directory.path())));
            process.reached(directory.path(), point);
            process.child.as_mut().unwrap().kill().unwrap();
            let output = process.finish();
            assert_eq!(output.status.signal(), Some(9), "{point}: {output:?}");
            output
        }
    };
    println!(
        "CRASH_RAW {}",
        json!({"point": point, "mode": format!("{mode:?}"), "argv": args, "exit": output.status.code(), "signal": output.status.signal(), "stdout": output.stdout, "stderr": output.stderr})
    );
}

#[derive(Debug)]
struct Record {
    reply: Value,
    effects: Value,
    published: bool,
    work: Option<String>,
}
fn connection(env: &Env) -> Connection {
    Connection::open_with_flags(
        env.dir.path().join("store.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap()
}
fn record(env: &Env, id: &str) -> Option<Record> {
    connection(env)
        .query_row(
            "SELECT reply_json,effects_json,published,work_id FROM requests WHERE request_id=?1",
            [id],
            |row| {
                let reply: String = row.get(0)?;
                let effects: String = row.get(1)?;
                Ok(Record {
                    reply: serde_json::from_str(&reply).unwrap(),
                    effects: serde_json::from_str(&effects).unwrap(),
                    published: row.get(2)?,
                    work: row.get(3)?,
                })
            },
        )
        .optional()
        .unwrap()
}
fn tree(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                visit(root, &entry.path(), files);
            } else {
                assert!(kind.is_file(), "原件不能含链接/特殊文件");
                files.insert(
                    entry.path().strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(path, path, &mut files);
    files
}
fn same_files(path: &Path, expected: &BTreeMap<PathBuf, Vec<u8>>) {
    for (relative, bytes) in expected {
        assert_eq!(
            &std::fs::read(path.join(relative)).unwrap(),
            bytes,
            "{}",
            relative.display()
        );
    }
}
fn container(env: &Env, id: &str) -> PathBuf {
    for entry in std::fs::read_dir(env.dir.path().join("pending")).unwrap() {
        let entry = entry.unwrap();
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "owner")
        {
            let owner: Value =
                serde_json::from_slice(&std::fs::read(entry.path()).unwrap()).unwrap();
            if owner["request_id"] == id {
                return env
                    .dir
                    .path()
                    .join("pending")
                    .join(owner["internal_id"].as_str().unwrap());
            }
        }
    }
    panic!("{id}的owner不存在");
}
fn original_reply(replay: &Value, stored: &Value, id: &str, work: Option<&str>) {
    assert_eq!(replay["request_id"], id);
    let mut expected = stored["data"].clone();
    expected
        .as_object_mut()
        .unwrap()
        .insert("replayed".into(), json!(true));
    assert_eq!(replay["data"], expected);
    if let Some(next) = stored.get("next") {
        let work = work.unwrap();
        let expected: Vec<Value> = next
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                let mut rendered = json!({"op":item["op"],"args":{"work":work}});
                if let Some(node) = item.get("node") {
                    rendered["args"]["node"] = node.clone();
                }
                if let Some(attempt) = item.get("attempt") {
                    rendered["args"]["attempt"] = json!(format!(
                        "{}#{}.{}",
                        attempt["node"].as_str().unwrap(),
                        attempt["occurrence"],
                        attempt["retry"]
                    ));
                }
                for key in ["edge", "executor", "tier"] {
                    if let Some(value) = item.get(key).filter(|value| !value.is_null()) {
                        rendered[key] = value.clone();
                    }
                }
                rendered
            })
            .collect();
        assert_eq!(replay["next"], json!(expected));
    }
    if let Some(revision) = stored.get("revision") {
        assert_eq!(&replay["revision"], revision);
    }
}
fn no_pending(env: &Env) {
    let count: i64 = connection(env)
        .query_row(
            "SELECT count(*) FROM requests WHERE published=0",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}
fn finished_files(env: &Env, stored: &Record) {
    for effect in stored.effects.as_array().unwrap() {
        if effect["kind"] == "write_file" {
            assert_eq!(
                std::fs::read(env.dir.path().join(effect["path"].as_str().unwrap())).unwrap(),
                effect["content"].as_str().unwrap().as_bytes()
            );
        }
    }
}

// Task: C002-T31
#[test]
fn start_and_add_recover_each_publication_window_after_exit70_and_actual_kill() {
    for operation in ["start", "add"] {
        for point in [
            "pending_owner_synced_before_payload",
            "before_commit",
            "after_commit_before_effects",
            "tree_rename_before_parent_sync",
            "request_published_before_cleanup",
        ] {
            for mode in MODES {
                let env = Env::new();
                let source = example_dir("two-step");
                if operation == "start" {
                    env.add_example("two-step");
                }
                let id = format!("t31-{operation}-{point}-{mode:?}");
                let args = if operation == "start" {
                    vec![
                        "--request-id",
                        &id,
                        "work",
                        "start",
                        "--workbook",
                        "two-step",
                        "--flow",
                        "default",
                        "--input",
                        "topic=starter",
                    ]
                } else {
                    vec![
                        "--request-id",
                        &id,
                        "workbook",
                        "add",
                        source.to_str().unwrap(),
                    ]
                };
                stop(&env, &args, point, mode);
                let pending = container(&env, &id);
                let prior = record(&env, &id);
                if point == "pending_owner_synced_before_payload" {
                    assert!(prior.is_none());
                    assert!(!pending.join("payload").exists());
                }
                let bytes = if let Some(prior) = &prior {
                    let final_path = env
                        .dir
                        .path()
                        .join(prior.effects[0]["final"].as_str().unwrap());
                    let location = if pending.join("payload").exists() {
                        pending.join("payload")
                    } else {
                        final_path
                    };
                    assert_eq!(prior.published, point == "request_published_before_cleanup");
                    Some(tree(&location))
                } else {
                    None
                };
                let replay = env.ok(&args);
                let current = record(&env, &id).unwrap();
                assert!(current.published);
                assert!(!pending.exists(), "无引用旧容器/完成元数据应清理");
                if let Some(prior) = &prior {
                    original_reply(&replay, &prior.reply, &id, prior.work.as_deref());
                    same_files(
                        &env.dir
                            .path()
                            .join(prior.effects[0]["final"].as_str().unwrap()),
                        bytes.as_ref().unwrap(),
                    );
                } else {
                    assert_eq!(replay["data"]["replayed"], false);
                }
                if operation == "start" {
                    let work = replay["data"]["work_id"].as_str().unwrap();
                    assert_eq!(
                        std::fs::read(
                            env.dir
                                .path()
                                .join("works")
                                .join(work)
                                .join("start-inputs/topic")
                        )
                        .unwrap(),
                        b"starter"
                    );
                    env.ok(&["attempt", "begin", work, "--node", "outline"]);
                } else {
                    same_files(
                        &env.dir.path().join("workbooks/two-step/1.0.0"),
                        &tree(&source),
                    );
                    env.start("two-step", &[("topic", "cross-entry")]);
                }
                no_pending(&env);
                println!(
                    "CRASH_ORACLE {}",
                    json!({"operation":operation,"point":point,"mode":format!("{mode:?}"),"committed_before_restart":prior.is_some(),"published_after_restart":current.published})
                );
            }
        }
    }
}

// Task: C002-T31
#[test]
fn begin_and_submit_recover_registered_bytes_and_seals_after_real_process_termination() {
    for operation in ["begin", "submit"] {
        let points = if operation == "begin" {
            [
                "after_commit_before_effects",
                "after_first_history_file_write",
            ]
        } else {
            ["after_commit_before_effects", "submit_before_seal"]
        };
        for point in points {
            for mode in MODES {
                let env = Env::new();
                env.add_example("two-step");
                let work = env.start("two-step", &[("topic", "attempt")]);
                let id = format!("t31-{operation}-{point}-{mode:?}");
                let output = if operation == "submit" {
                    let begun = env.ok(&["attempt", "begin", &work, "--node", "outline"]);
                    let output =
                        PathBuf::from(begun["data"]["outputs"]["outline"].as_str().unwrap());
                    std::fs::write(&output, b"independent bytes").unwrap();
                    Some(output)
                } else {
                    None
                };
                let args = if operation == "begin" {
                    vec![
                        "--request-id",
                        &id,
                        "attempt",
                        "begin",
                        &work,
                        "--node",
                        "outline",
                    ]
                } else {
                    vec![
                        "--request-id",
                        &id,
                        "attempt",
                        "submit",
                        &work,
                        "--attempt",
                        "outline#1.0",
                        "--summary",
                        "done",
                    ]
                };
                stop(&env, &args, point, mode);
                let stored = record(&env, &id).unwrap();
                assert_eq!(stored.work.as_deref(), Some(work.as_str()));
                assert!(!stored.published);
                env.ok(&["work", "cancel", &work]);
                finished_files(&env, &stored);
                let replay = env.ok(&args);
                original_reply(&replay, &stored.reply, &id, stored.work.as_deref());
                if let Some(output) = &output {
                    assert_eq!(std::fs::read(output).unwrap(), b"independent bytes");
                    assert!(std::fs::metadata(output).unwrap().permissions().readonly());
                }
                let card = env.ok(&["work", "status", &work]);
                assert_eq!(card["data"]["status"]["kind"], "cancelled");
                no_pending(&env);
            }
        }
    }
}

// Task: C002-T31
#[test]
fn remove_real_windows_preserve_unknown_results_and_do_not_touch_a_new_lifecycle() {
    for point in [
        "after_commit_before_effects",
        "tree_rename_before_parent_sync",
        "delete_payload_verified_before_remove",
        "delete_after_first_payload_child",
        "delete_after_tree_removed_before_marker",
        "delete_marker_synced_before_mark",
        "request_published_before_cleanup",
    ] {
        for mode in MODES {
            let env = Env::new();
            env.add_example("two-step");
            let id = format!("t31-remove-{point}-{mode:?}");
            let args = ["--request-id", &id, "workbook", "remove", "two-step@1.0.0"];
            stop(&env, &args, point, mode);
            let stored = record(&env, &id).unwrap();
            assert_eq!(
                stored.published,
                point == "request_published_before_cleanup"
            );
            let pending = container(&env, &id);
            let payload = pending.join("payload");
            if matches!(
                point,
                "delete_after_first_payload_child" | "delete_after_tree_removed_before_marker"
            ) {
                let remaining = payload.exists().then(|| tree(&payload));
                let (own, code) = env.fail(&args);
                assert_eq!(code, 1);
                assert_eq!(own["error"]["code"], "EFFECT_PENDING");
                assert_eq!(own["committed"], true);
                let (blocked, code) = env.fail(&[
                    "--request-id",
                    "t31-blocked-add",
                    "workbook",
                    "add",
                    example_dir("gated-release").to_str().unwrap(),
                ]);
                assert_eq!(code, 1);
                assert_eq!(blocked["committed"], false);
                assert_eq!(blocked["error"]["detail"]["pending_request_id"], id);
                assert!(record(&env, "t31-blocked-add").is_none());
                assert!(!record(&env, &id).unwrap().published);
                if let Some(remaining) = remaining {
                    assert_eq!(tree(&payload), remaining);
                }
                assert!(pending.exists());
            } else {
                let replay = env.ok(&args);
                original_reply(&replay, &stored.reply, &id, stored.work.as_deref());
                assert!(!env.dir.path().join("workbooks/two-step/1.0.0").exists());
                let source = copy_example("two-step", &env.dir.path().join("new-source"));
                std::fs::write(source.join("instructions/outline.md"), b"new lifecycle\n").unwrap();
                env.ok(&["workbook", "add", source.to_str().unwrap()]);
                let final_path = env.dir.path().join("workbooks/two-step/1.0.0");
                let before = tree(&final_path);
                let replay = env.ok(&args);
                original_reply(&replay, &stored.reply, &id, stored.work.as_deref());
                assert_eq!(tree(&final_path), before);
                no_pending(&env);
            }
        }
    }
}

fn release_source(path: &Path) {
    let tag = path.join("v9.9.9");
    std::fs::create_dir_all(&tag).unwrap();
    let asset = "sheltie-fixture";
    std::fs::write(tag.join(asset), b"candidate binary fixture").unwrap();
    let hash = Command::new("shasum")
        .args(["-a", "256"])
        .arg(tag.join(asset))
        .output()
        .unwrap();
    assert!(hash.status.success());
    let digest = String::from_utf8(hash.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();
    let manifest = json!({"version":"9.9.9","assets":[{"platform":sheltie_runtime::selfmgmt::platform(),"name":asset,"sha256":digest}]});
    std::fs::write(
        tag.join("dist-manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    std::fs::create_dir_all(path.join("latest")).unwrap();
    std::fs::write(
        path.join("latest/dist-manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
}

// Task: C002-T31
#[test]
fn installed_prev_rolls_back_after_update_termination_without_changing_store_bytes() {
    for mode in MODES {
        let env = Env::new();
        env.add_example("two-step");
        env.ok(&["self", "install"]);
        let binary = env.dir.path().join("bin/sheltie");
        let original = std::fs::read(&binary).unwrap();
        let database = std::fs::read(env.dir.path().join("store.db")).unwrap();
        let release = env.dir.path().join("release-source");
        release_source(&release);
        let point = "update_between_renames";
        let directory = tempfile::tempdir().unwrap();
        let mut command = Command::new(&binary);
        command
            .args(["--home", &env.home(), "--json", "self", "update"])
            .env("SHELTIE_RELEASE_BASE", &release)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match mode {
            Termination::Exit70 => {
                command.env("SHELTIE_FAILPOINT", point);
            }
            Termination::Kill => {
                command
                    .env("SHELTIE_TEST_RENDEZVOUS_NAME", point)
                    .env("SHELTIE_TEST_RENDEZVOUS_ID", point)
                    .env("SHELTIE_TEST_RENDEZVOUS_DIR", directory.path());
            }
        }
        let mut process = Process {
            child: Some(command.spawn().unwrap()),
            release: Some(directory.path().join("release")),
        };
        if matches!(mode, Termination::Kill) {
            process.reached(directory.path(), point);
            process.child.as_mut().unwrap().kill().unwrap();
        }
        let output = process.finish();
        if matches!(mode, Termination::Kill) {
            assert_eq!(output.status.signal(), Some(9));
        } else {
            assert_eq!(output.status.code(), Some(70), "{output:?}");
        }
        println!(
            "CRASH_RAW {}",
            json!({"point":point,"mode":format!("{mode:?}"),"signal":output.status.signal(),"exit":output.status.code(),"stdout":output.stdout,"stderr":output.stderr})
        );
        let prev = env.dir.path().join("bin/sheltie.prev");
        assert_eq!(std::fs::read(&prev).unwrap(), original);
        assert!(!binary.exists());
        let rollback = Command::new(&prev)
            .args(["--home", &env.home(), "--json", "self", "rollback"])
            .output()
            .unwrap();
        assert!(rollback.status.success(), "{rollback:?}");
        assert_eq!(std::fs::read(&binary).unwrap(), original);
        assert!(!prev.exists());
        assert_eq!(
            std::fs::read(env.dir.path().join("store.db")).unwrap(),
            database
        );
    }
}

// Task: C002-T31
#[test]
fn purge_partial_termination_preserves_root_lock_and_can_finish_without_reviving_work() {
    use std::os::unix::fs::MetadataExt;
    for mode in MODES {
        let env = Env::new();
        env.add_example("two-step");
        env.ok(&["self", "install"]);
        let work = env.start("two-step", &[("topic", "purge")]);
        let lock = std::fs::metadata(env.dir.path().join(".lock")).unwrap();
        let root = std::fs::metadata(env.dir.path()).unwrap();
        let database = std::fs::read(env.dir.path().join("store.db")).unwrap();
        stop(
            &env,
            &["self", "uninstall", "--purge", "--yes"],
            "purge_after_top_level_delete",
            mode,
        );
        assert_eq!(
            std::fs::read(env.dir.path().join("store.db")).unwrap(),
            database,
            "Store应最后删除"
        );
        let current = std::fs::metadata(env.dir.path().join(".lock")).unwrap();
        assert_eq!((current.dev(), current.ino()), (lock.dev(), lock.ino()));
        let current = std::fs::metadata(env.dir.path()).unwrap();
        assert_eq!((current.dev(), current.ino()), (root.dev(), root.ino()));
        env.ok(&["self", "uninstall", "--purge", "--yes"]);
        assert!(!env.dir.path().join("store.db").exists());
        let (_, code) = env.fail(&["work", "cancel", &work]);
        assert_eq!(code, 1);
        assert!(!env.dir.path().join("store.db").exists());
        env.ok(&["self", "install"]);
        let rows: i64 = connection(&env)
            .query_row("SELECT count(*) FROM works", [], |row| row.get(0))
            .unwrap();
        assert_eq!(rows, 0);
    }
}

// Task: C002-T31
#[test]
fn purge_waiters_reach_the_failed_try_lock_before_initialization_or_old_work_rejection() {
    use std::os::unix::fs::MetadataExt;
    for action in ["install", "add", "old-work"] {
        let env = Env::new();
        env.add_example("two-step");
        env.ok(&["self", "install"]);
        let work = env.start("two-step", &[("topic", "wait")]);
        let before = std::fs::metadata(env.dir.path().join(".lock")).unwrap();
        let database = std::fs::read(env.dir.path().join("store.db")).unwrap();
        let purge_directory = tempfile::tempdir().unwrap();
        let purge_point = "purge_after_top_level_delete";
        let mut purge = Process::spawn(
            &env,
            &["self", "uninstall", "--purge", "--yes"],
            Some((purge_point, purge_point, purge_directory.path())),
        );
        purge.reached(purge_directory.path(), purge_point);
        let waiting_directory = tempfile::tempdir().unwrap();
        let lock_path = env.dir.path().join(".lock").canonicalize().unwrap();
        let source = example_dir("two-step");
        let args = match action {
            "install" => vec!["self", "install"],
            "add" => vec!["workbook", "add", source.to_str().unwrap()],
            _ => vec!["work", "cancel", &work],
        };
        let mut waiter = Process::spawn(
            &env,
            &args,
            Some((
                "home_lock_waiting",
                lock_path.to_str().unwrap(),
                waiting_directory.path(),
            )),
        );
        waiter.reached(waiting_directory.path(), "home_lock_waiting");
        assert_eq!(
            std::fs::read(env.dir.path().join("store.db")).unwrap(),
            database
        );
        if let Ok(wal) = std::fs::metadata(env.dir.path().join("store.db-wal")) {
            assert_eq!(wal.len(), 0, "只读不能产生WAL记录");
            println!(
                "SQLITE_CONTROL_RAW {}",
                json!({"action":action,"wal_bytes":wal.len(),"main_bytes_unchanged":true})
            );
        }
        waiter.release();
        assert!(waiter.child.as_mut().unwrap().try_wait().unwrap().is_none());
        purge.release();
        let purged = purge.finish();
        assert!(purged.status.success(), "{purged:?}");
        let result = waiter.finish();
        if action == "old-work" {
            assert_eq!(result.status.code(), Some(1));
            assert!(!env.dir.path().join("store.db").exists());
        } else {
            assert!(result.status.success(), "{result:?}");
            let rows: i64 = connection(&env)
                .query_row("SELECT count(*) FROM works", [], |row| row.get(0))
                .unwrap();
            assert_eq!(rows, 0);
        }
        let after = std::fs::metadata(&lock_path).unwrap();
        assert_eq!((after.dev(), after.ino()), (before.dev(), before.ino()));
        println!(
            "PURGE_WAITER_RAW {}",
            json!({"action":action,"event":"home_lock_waiting","purge_exit":purged.status.code(),"waiter_exit":result.status.code(),"waiter_stdout":result.stdout,"waiter_stderr":result.stderr})
        );
    }
}

// Task: C002-T31
#[test]
fn moved_delete_recovery_resyncs_both_rename_parents_before_deleting_payload() {
    let env = Env::new();
    env.add_example("two-step");
    let id = "t31-delete-parent-sync";
    let args = ["--request-id", id, "workbook", "remove", "two-step@1.0.0"];
    stop(
        &env,
        &args,
        "tree_rename_before_parent_sync",
        Termination::Kill,
    );
    let pending = container(&env, id);
    let before = tree(&pending.join("payload"));
    let home = sheltie_runtime::Home::resolve(Some(
        (sheltie_core::path::AbsPath::new(env.home()).unwrap()).as_str(),
    ))
    .unwrap();
    for sync_point in ["publish_source_parent_sync", "publish_target_parent_sync"] {
        sheltie_runtime::failpoint::arm_sync_error(home.root().as_str(), sync_point).unwrap();
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                sheltie_runtime::failpoint::disarm_sync_error().unwrap();
            }
        }
        let guard = Guard;
        let error = sheltie_runtime::WorkbookRepo::new(home.clone())
            .remove("two-step", "1.0.0", Some(id.into()))
            .unwrap_err();
        let sheltie_runtime::Error::EffectPending {
            committed,
            cause,
            request_id,
            ..
        } = error
        else {
            panic!("{error:?}");
        };
        assert!(committed);
        assert_eq!(request_id, id);
        assert_eq!(cause, sheltie_core::ErrorCode::Io);
        assert!(!record(&env, id).unwrap().published);
        assert_eq!(tree(&pending.join("payload")), before);
        drop(guard);
    }
    env.ok(&args);
    assert!(record(&env, id).unwrap().published);
    assert!(!pending.exists());
}

// Task: C002-T31
#[test]
fn same_request_retry_stops_before_registration_if_its_uncommitted_original_cannot_be_cleaned() {
    for operation in ["start", "add"] {
        for point in ["pending_owner_synced_before_payload", "before_commit"] {
            let env = Env::new();
            if operation == "start" {
                env.add_example("two-step");
            }
            let source = example_dir("two-step");
            let id = format!("t31-clean-fail-{operation}-{point}");
            let args = if operation == "start" {
                vec![
                    "--request-id",
                    &id,
                    "work",
                    "start",
                    "--workbook",
                    "two-step",
                    "--flow",
                    "default",
                    "--input",
                    "topic=cleanup",
                ]
            } else {
                vec![
                    "--request-id",
                    &id,
                    "workbook",
                    "add",
                    source.to_str().unwrap(),
                ]
            };
            stop(&env, &args, point, Termination::Exit70);
            let old = container(&env, &id);
            let bytes = tree(&old);
            let audits: i64 = connection(&env)
                .query_row("SELECT count(*) FROM audit", [], |row| row.get(0))
                .unwrap();
            let output = env
                .cmd(&args)
                .env("SHELTIE_FAILPOINT", "pending_cleanup_before_remove")
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1), "{output:?}");
            let error: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(error["error"]["code"], "IO");
            assert!(record(&env, &id).is_none());
            assert_eq!(tree(&old), bytes);
            let after: i64 = connection(&env)
                .query_row("SELECT count(*) FROM audit", [], |row| row.get(0))
                .unwrap();
            assert_eq!(audits, after);
            env.ok(&args);
            assert!(record(&env, &id).unwrap().published);
            assert!(!old.exists());
        }
    }
}

// Task: C002-T31
#[test]
fn owned_test_directory_cleanup_removes_readonly_trees_without_chmod_of_external_links() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let outside = OwnedTempDir::new();
    let sentinel = outside.path().join("sentinel");
    std::fs::write(&sentinel, b"outside original").unwrap();
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o444)).unwrap();
    std::fs::set_permissions(outside.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let mode = std::fs::metadata(outside.path()).unwrap().mode();
    let path;
    {
        let env = Env::new();
        path = env.dir.path().to_path_buf();
        let frozen = env.dir.path().join("frozen");
        std::fs::create_dir(&frozen).unwrap();
        std::fs::write(frozen.join("original"), b"owned original").unwrap();
        std::fs::set_permissions(&frozen, std::fs::Permissions::from_mode(0o555)).unwrap();
        std::os::unix::fs::symlink(outside.path(), env.dir.path().join("external-directory"))
            .unwrap();
        std::fs::hard_link(&sentinel, env.dir.path().join("external-file")).unwrap();
    }
    assert!(!path.exists());
    assert_eq!(std::fs::metadata(outside.path()).unwrap().mode(), mode);
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"outside original");
    assert_eq!(std::fs::metadata(&sentinel).unwrap().mode() & 0o777, 0o444);
}

// Task: C002-T31
#[test]
fn another_write_kind_directly_recovers_start_or_add_after_commit_process_termination() {
    for operation in ["start", "add"] {
        for mode in MODES {
            let env = Env::new();
            let source = example_dir("two-step");
            if operation == "start" {
                env.add_example("two-step");
            }
            let id = format!("t31-cross-{operation}-{mode:?}");
            let args = if operation == "start" {
                vec![
                    "--request-id",
                    &id,
                    "work",
                    "start",
                    "--workbook",
                    "two-step",
                    "--flow",
                    "default",
                    "--input",
                    "topic=original",
                ]
            } else {
                vec![
                    "--request-id",
                    &id,
                    "workbook",
                    "add",
                    source.to_str().unwrap(),
                ]
            };
            stop(&env, &args, "after_commit_before_effects", mode);
            let original = record(&env, &id).unwrap();
            assert!(!original.published);
            let bytes = tree(&container(&env, &id).join("payload"));
            let new_id = "t31-cross-new-write";
            if operation == "start" {
                env.ok(&[
                    "--request-id",
                    new_id,
                    "workbook",
                    "add",
                    example_dir("gated-release").to_str().unwrap(),
                ]);
            } else {
                env.ok(&[
                    "--request-id",
                    new_id,
                    "work",
                    "start",
                    "--workbook",
                    "two-step",
                    "--flow",
                    "default",
                    "--input",
                    "topic=new writer",
                ]);
            }
            assert!(record(&env, &id).unwrap().published);
            assert!(record(&env, new_id).unwrap().published);
            same_files(
                &env.dir
                    .path()
                    .join(original.effects[0]["final"].as_str().unwrap()),
                &bytes,
            );
            if let Some(work) = &original.work {
                let card = env.ok(&["work", "status", work]);
                assert_eq!(card["data"]["status"]["kind"], "active");
                assert_eq!(
                    std::fs::read(
                        env.dir
                            .path()
                            .join("works")
                            .join(work)
                            .join("start-inputs/topic")
                    )
                    .unwrap(),
                    b"original"
                );
            }
            let replay = env.ok(&args);
            original_reply(&replay, &original.reply, &id, original.work.as_deref());
            no_pending(&env);
            println!(
                "CROSS_KIND_RECOVERY_RAW {}",
                json!({"old_operation":operation,"mode":format!("{mode:?}"),"old_request":id,"new_request":new_id,"old_published":true,"new_published":true})
            );
        }
    }
}

// Task: C002-T31
#[test]
fn committed_history_and_attempt_directory_type_changes_are_integrity_errors_with_original_snapshot()
 {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    for kind in [
        "brief-symlink",
        "brief-dangling-symlink",
        "brief-hardlink",
        "brief-fifo",
        "outputs-file",
    ] {
        let env = Env::new();
        env.add_example("two-step");
        let work = env.start("two-step", &[("topic", "integrity")]);
        let id = format!("t31-history-{kind}");
        let args = [
            "--request-id",
            &id,
            "attempt",
            "begin",
            &work,
            "--node",
            "outline",
        ];
        let original = if kind == "outputs-file" {
            stop(
                &env,
                &args,
                "after_commit_before_effects",
                Termination::Exit70,
            );
            record(&env, &id).unwrap()
        } else {
            env.ok(&args);
            record(&env, &id).unwrap()
        };
        let snapshot: String = connection(&env)
            .query_row(
                "SELECT reply_json FROM requests WHERE request_id=?1",
                [&id],
                |row| row.get(0),
            )
            .unwrap();
        let outside = OwnedTempDir::new();
        let sentinel = outside.path().join("sentinel");
        std::fs::write(&sentinel, b"external original").unwrap();
        std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o640)).unwrap();
        let mode = std::fs::metadata(&sentinel).unwrap().mode();
        let target = if kind == "outputs-file" {
            let dirs = original
                .effects
                .as_array()
                .unwrap()
                .iter()
                .find(|effect| effect["kind"] == "prepare_attempt")
                .unwrap()["dirs"]
                .as_array()
                .unwrap();
            let path = env.dir.path().join(
                dirs.iter()
                    .find(|path| path.as_str().unwrap().ends_with("/outputs"))
                    .unwrap()
                    .as_str()
                    .unwrap(),
            );
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"preserve conflicting object").unwrap();
            path
        } else {
            let path = env.dir.path().join(
                original
                    .effects
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|effect| {
                        effect["kind"] == "write_file"
                            && effect["path"].as_str().unwrap().ends_with("/brief.md")
                    })
                    .unwrap()["path"]
                    .as_str()
                    .unwrap(),
            );
            std::fs::remove_file(&path).unwrap();
            match kind {
                "brief-symlink" => std::os::unix::fs::symlink(&sentinel, &path).unwrap(),
                "brief-dangling-symlink" => {
                    std::os::unix::fs::symlink(outside.path().join("missing"), &path).unwrap()
                }
                "brief-hardlink" => std::fs::hard_link(&sentinel, &path).unwrap(),
                _ => {
                    assert!(
                        Command::new("mkfifo")
                            .arg(&path)
                            .status()
                            .unwrap()
                            .success()
                    );
                }
            }
            path
        };
        let (error, code) = env.fail(&args);
        assert_eq!(code, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING");
        assert_eq!(error["committed"], true);
        assert_eq!(
            error["error"]["detail"]["cause"], "STORE_CORRUPT",
            "{kind}: {error}"
        );
        assert_eq!(error["original"]["request_id"], id);
        assert!(error["original"]["data"].is_object());
        assert!(error["original"]["next"].is_array());
        assert_eq!(record(&env, &id).unwrap().published, original.published);
        let after: String = connection(&env)
            .query_row(
                "SELECT reply_json FROM requests WHERE request_id=?1",
                [&id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(snapshot, after);
        assert!(std::fs::symlink_metadata(&target).is_ok());
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"external original");
        assert_eq!(std::fs::metadata(&sentinel).unwrap().mode(), mode);
        if kind == "outputs-file" {
            assert_eq!(
                std::fs::read(&target).unwrap(),
                b"preserve conflicting object"
            );
        }
    }
}

// Task: C002-T31
#[cfg(target_os = "macos")]
#[test]
fn exact_system_tmp_parent_alias_reads_the_same_at_file_and_rejects_user_symlinks() {
    use std::io::Write;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mut source = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    source.write_all(b"same external input").unwrap();
    source.flush().unwrap();
    std::fs::set_permissions(source.path(), std::fs::Permissions::from_mode(0o640)).unwrap();
    let name = source.path().file_name().unwrap().to_str().unwrap();
    for selector in [format!("@/tmp/{name}"), format!("@/private/tmp/{name}")] {
        let env = Env::new();
        env.add_example("two-step");
        let work = env.start("two-step", &[("topic", &selector)]);
        assert_eq!(
            std::fs::read(
                env.dir
                    .path()
                    .join("works")
                    .join(work)
                    .join("start-inputs/topic")
            )
            .unwrap(),
            b"same external input"
        );
    }
    let env = Env::new();
    env.add_example("two-step");
    let link = env.dir.path().join("external-link");
    std::os::unix::fs::symlink(source.path(), &link).unwrap();
    let selector = format!("topic=@{}", link.display());
    let (_, code) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        &selector,
    ]);
    assert_eq!(code, 2);
    let rows: i64 = connection(&env)
        .query_row("SELECT count(*) FROM works", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 0);
    assert_eq!(
        std::fs::read(source.path()).unwrap(),
        b"same external input"
    );
    assert_eq!(
        std::fs::metadata(source.path()).unwrap().mode() & 0o777,
        0o640
    );
}

fn no_edge_gate(env: &Env) -> String {
    let source = env.dir.path().join("cycle-workbook");
    std::fs::create_dir_all(source.join("flows")).unwrap();
    std::fs::write(source.join("workbook.toml"),"schema = \"workbook/v1\"\nid = \"gate-cycle\"\nversion = \"1.0.0\"\nname = \"Gate cycle\"\nflows = [\"flows/default.toml\"]\n").unwrap();
    std::fs::write(
        source.join("flows/default.toml"),
        r#"schema = "flow/v1"
id = "default"
entry = "draft"
[[nodes]]
id = "draft"
title = "Draft"
executor = "agent"
max_visits = 1
instruction = { text = "draft" }
[[nodes]]
id = "review"
title = "Review"
executor = "agent"
gate = true
max_visits = 1
instruction = { text = "review" }
[[nodes]]
id = "done"
title = "Done"
executor = "agent"
instruction = { text = "done" }
[[edges]]
from = "draft"
to = "done"
kind = "branch"
[[edges]]
from = "draft"
to = "review"
kind = "main"
[[edges]]
from = "review"
to = "draft"
kind = "back"
"#,
    )
    .unwrap();
    env.ok(&["workbook", "add", source.to_str().unwrap()]);
    let work = env.start("gate-cycle", &[]);
    let draft = env.begin(&work, "draft");
    env.submit_all(&work, &draft, "done");
    let review = env.begin(&work, "review");
    env.submit_all(&work, &review, "done");
    work
}

// Task: C002-T31
#[test]
fn gate_approval_without_a_remaining_edge_adds_a_second_blocked_event() {
    let env = Env::new();
    let work = no_edge_gate(&env);
    let first = env.ok(&["work", "stats", &work]);
    assert_eq!(first["data"]["blocked_count"], 1);
    env.ok(&["gate", "approve", &work, "--node", "review"]);
    let second = env.ok(&["work", "stats", &work]);
    assert_eq!(second["data"]["blocked_count"], 2);
    let status = env.ok(&["work", "status", &work]);
    assert_eq!(
        status["data"]["status"],
        json!({"kind":"blocked","reason":"no_legal_edge"})
    );
}

// Task: C002-T31
#[test]
fn impossible_persisted_blocked_counts_are_structured_errors_without_writes_or_panic() {
    for count in [u32::MAX, 0, 3] {
        let env = Env::new();
        let work = no_edge_gate(&env);
        let conn = Connection::open(env.dir.path().join("store.db")).unwrap();
        let raw: String = conn
            .query_row(
                "SELECT state_json FROM works WHERE work_id=?1",
                [&work],
                |r| r.get(0),
            )
            .unwrap();
        let mut state: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(state["blocked_count"], 1);
        state["blocked_count"] = count.into();
        conn.execute(
            "UPDATE works SET state_json=?1 WHERE work_id=?2",
            rusqlite::params![state.to_string(), work],
        )
        .unwrap();
        drop(conn);
        let before = store_rows(&env);
        let files = tree(&env.dir.path().join("works"));
        for args in [
            vec!["work", "status", &work],
            vec!["work", "stats", &work],
            vec!["work", "list"],
        ] {
            let (error, exit) = env.fail(&args);
            eprintln!("counter={count} argv={args:?} exit={exit} response={error}");
            assert_eq!(exit, 1);
            assert_eq!(error["error"]["code"], "STORE_CORRUPT");
            assert_eq!(store_rows(&env), before);
            same_files(&env.dir.path().join("works"), &files);
        }
        let id = "t31-corrupt-counter-approve";
        let args = [
            "--request-id",
            id,
            "gate",
            "approve",
            &work,
            "--node",
            "review",
        ];
        let (error, exit) = env.fail(&args);
        eprintln!("counter={count} argv={args:?} exit={exit} response={error}");
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "STORE_CORRUPT");
        assert!(record(&env, id).is_none());
        assert_eq!(store_rows(&env), before);
        same_files(&env.dir.path().join("works"), &files);
    }
}

// Task: C002-T33
#[test]
fn file_inputs_reject_same_volume_replacement_links_and_special_files_at_open() {
    for change in ["normal", "replace", "hardlink", "symlink", "fifo"] {
        let env = Env::new();
        env.add_example("two-step");
        let source = tempfile::tempdir().unwrap();
        let input = source.path().join("input.txt");
        let original = b"original input";
        std::fs::write(&input, original).unwrap();
        let selector = format!("topic=@{}", input.display());
        let id = format!("input-open-{change}");
        let sync = tempfile::tempdir().unwrap();
        let mut process = Process::spawn(
            &env,
            &[
                "--request-id",
                &id,
                "work",
                "start",
                "--workbook",
                "two-step",
                "--flow",
                "default",
                "--input",
                &selector,
            ],
            Some((
                "regular_open_after_stat",
                input.to_str().unwrap(),
                sync.path(),
            )),
        );
        process.reached(sync.path(), "regular_open_after_stat");
        let parked = source.path().join("original.txt");
        match change {
            "normal" => {}
            "hardlink" => std::fs::hard_link(&input, &parked).unwrap(),
            "replace" | "symlink" | "fifo" => {
                std::fs::rename(&input, &parked).unwrap();
                match change {
                    "replace" => std::fs::write(&input, b"replaced input").unwrap(),
                    "symlink" => std::os::unix::fs::symlink(&parked, &input).unwrap(),
                    "fifo" => assert!(
                        Command::new("mkfifo")
                            .arg(&input)
                            .status()
                            .unwrap()
                            .success()
                    ),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        process.release();
        let output = process.finish();
        let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
        if change == "normal" {
            assert!(output.status.success(), "{reply}");
        } else {
            assert!(!output.status.success(), "{change}: {reply}");
            assert_eq!(
                connection(&env)
                    .query_row("SELECT count(*) FROM works", [], |row| row.get::<_, i64>(0))
                    .unwrap(),
                0
            );
            assert_eq!(
                connection(&env)
                    .query_row(
                        "SELECT count(*) FROM requests WHERE request_id=?1",
                        [&id],
                        |row| row.get::<_, i64>(0)
                    )
                    .unwrap(),
                0
            );
        }
        assert_eq!(
            std::fs::read(if change == "normal" { &input } else { &parked }).unwrap(),
            original
        );
    }
}

// Task: C002-T33
#[test]
fn workbook_source_rejects_a_replaced_directory_at_the_open_boundary() {
    for replace in [false, true] {
        let env = Env::new();
        let source_root = tempfile::tempdir().unwrap();
        let source = copy_example("two-step", source_root.path());
        let scope = std::fs::canonicalize(&source).unwrap().join("flows");
        let sync = tempfile::tempdir().unwrap();
        let mut process = Process::spawn(
            &env,
            &[
                "--request-id",
                "source-directory",
                "workbook",
                "add",
                source.to_str().unwrap(),
            ],
            Some((
                "directory_open_after_stat",
                scope.to_str().unwrap(),
                sync.path(),
            )),
        );
        process.reached(sync.path(), "directory_open_after_stat");
        if replace {
            let parked = source_root.path().join("original-flows");
            std::fs::rename(&scope, &parked).unwrap();
            copy_dir(&parked, &scope);
        }
        process.release();
        let output = process.finish();
        let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.success(), !replace, "{reply}");
        if replace {
            assert!(!Path::new(&env.home()).join("store.db").exists());
        }
    }
}

// Task: C002-T33
#[test]
fn readonly_sqlite_open_rejects_root_or_store_rebinding_with_unchanged_database_bytes() {
    for change in ["normal", "store", "root"] {
        let env = Env::new();
        env.add_example("two-step");
        Connection::open(env.dir.path().join("store.db"))
            .unwrap()
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")
            .unwrap();
        let home = std::fs::canonicalize(Path::new(&env.home())).unwrap();
        let store = home.join("store.db");
        let original = std::fs::read(&store).unwrap();
        let sync = tempfile::tempdir().unwrap();
        let mut process = Process::spawn(
            &env,
            &["workbook", "list"],
            Some((
                "store_before_sqlite_open",
                store.to_str().unwrap(),
                sync.path(),
            )),
        );
        process.reached(sync.path(), "store_before_sqlite_open");
        let parking = tempfile::tempdir().unwrap();
        match change {
            "normal" => {}
            "store" => {
                let old = parking.path().join("original.db");
                std::fs::rename(&store, &old).unwrap();
                std::fs::copy(&old, &store).unwrap();
                assert_eq!(std::fs::read(old).unwrap(), original);
            }
            "root" => {
                let old = parking.path().join("original-root");
                std::fs::rename(&home, &old).unwrap();
                std::fs::create_dir(&home).unwrap();
                for entry in std::fs::read_dir(&old).unwrap() {
                    let entry = entry.unwrap();
                    std::fs::rename(entry.path(), home.join(entry.file_name())).unwrap();
                }
            }
            _ => unreachable!(),
        }
        process.release();
        let output = process.finish();
        let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            output.status.success(),
            change == "normal",
            "{change}: {reply}"
        );
        if change != "normal" {
            assert_eq!(reply["error"]["code"], "INVALID_REQUEST");
        }
        assert_eq!(std::fs::read(&store).unwrap(), original);
    }
}

// Task: C002-T33
#[test]
fn zero_work_revision_is_rejected_by_readonly_and_write_cli_entries() {
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
        "topic=revision",
    ]);
    let work = started["data"]["work_id"].as_str().unwrap();
    let conn = Connection::open(env.dir.path().join("store.db")).unwrap();
    conn.execute("UPDATE works SET revision=0 WHERE work_id=?1", [work])
        .unwrap();
    let before: String = conn
        .query_row(
            "SELECT state_json FROM works WHERE work_id=?1",
            [work],
            |row| row.get(0),
        )
        .unwrap();
    let requests: i64 = conn
        .query_row("SELECT count(*) FROM requests", [], |row| row.get(0))
        .unwrap();
    for args in [
        vec!["work", "status", work],
        vec!["work", "stats", work],
        vec!["work", "list"],
        vec!["--request-id", "zero-cancel", "work", "cancel", work],
        vec![
            "--request-id",
            "zero-begin",
            "attempt",
            "begin",
            work,
            "--node",
            "outline",
        ],
    ] {
        let (reply, _) = env.fail(&args);
        assert_eq!(reply["error"]["code"], "STORE_CORRUPT");
        assert_eq!(
            conn.query_row(
                "SELECT state_json FROM works WHERE work_id=?1",
                [work],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
            before
        );
        assert_eq!(
            conn.query_row(
                "SELECT revision FROM works WHERE work_id=?1",
                [work],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM requests", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            requests
        );
    }
}

// Task: C002-T33
#[test]
fn relative_input_resolution_reports_a_removed_current_directory_without_registering_a_request() {
    let env = Env::new();
    env.add_example("two-step");
    let cwd = tempfile::tempdir().unwrap();
    let sync = tempfile::tempdir().unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_sheltie"))
        .args([
            "--home",
            &env.home(),
            "--json",
            "--request-id",
            "removed-cwd",
            "work",
            "start",
            "--workbook",
            "two-step",
            "--flow",
            "default",
            "--input",
            "topic=@input.txt",
        ])
        .current_dir(cwd.path())
        .env("SHELTIE_TEST_RENDEZVOUS_NAME", "lexical_before_cwd")
        .env("SHELTIE_TEST_RENDEZVOUS_ID", "input.txt")
        .env("SHELTIE_TEST_RENDEZVOUS_DIR", sync.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut process = Process {
        child: Some(child),
        release: Some(sync.path().join("release")),
    };
    process.reached(sync.path(), "lexical_before_cwd");
    std::fs::remove_dir(cwd.path()).unwrap();
    process.release();
    let output = process.finish();
    let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!output.status.success());
    assert_eq!(reply["error"]["code"], "INVALID_REQUEST");
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("current_dir")
    );
    let conn = connection(&env);
    for table in ["works", "work_sequence", "requests"] {
        assert_eq!(
            conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            if table == "requests" { 1 } else { 0 }
        );
    }
}

// Task: C002-T34
#[test]
fn corrupt_remove_snapshot_cannot_be_projected_as_a_successful_original() {
    for (field, replacement) in [("id", "other-step"), ("version", "2.0.0")] {
        let env = Env::new();
        env.add_example("two-step");
        let rid = "t34-remove-original";
        let args = ["--request-id", rid, "workbook", "remove", "two-step@1.0.0"];
        let committed = env.ok(&args);
        assert_eq!(
            committed["data"],
            json!({"id":"two-step", "version":"1.0.0", "replayed":false})
        );
        let mut expected_replay = committed.clone();
        expected_replay["data"]["replayed"] = json!(true);
        assert_eq!(env.ok(&args), expected_replay);
        let mut snapshot = record(&env, rid).unwrap().reply;
        snapshot["data"][field] = json!(replacement);
        Connection::open(env.dir.path().join("store.db"))
            .unwrap()
            .execute(
                "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
                rusqlite::params![snapshot.to_string(), rid],
            )
            .unwrap();
        let before = store_rows(&env);
        let (error, exit) = env.fail(&args);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING");
        assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
        assert_eq!(error["committed"], true);
        assert_eq!(error["request_id"], rid);
        assert!(error.get("original").is_none(), "{field}: {error}");
        assert_eq!(store_rows(&env), before);
    }
}

// Task: C002-T34
#[test]
fn corrupt_pending_remove_snapshot_cannot_be_projected_as_a_blockers_original() {
    for (field, replacement) in [("id", "other-step"), ("version", "2.0.0")] {
        let env = Env::new();
        env.add_example("two-step");
        let old = "t34-old-remove";
        stop(
            &env,
            &["--request-id", old, "workbook", "remove", "two-step@1.0.0"],
            "after_commit_before_effects",
            Termination::Exit70,
        );
        let stored = record(&env, old).unwrap();
        assert!(!stored.published);
        assert_eq!(
            stored.reply["data"],
            json!({"id":"two-step", "version":"1.0.0"})
        );
        let mut snapshot = stored.reply;
        snapshot["data"][field] = json!(replacement);
        Connection::open(env.dir.path().join("store.db"))
            .unwrap()
            .execute(
                "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
                rusqlite::params![snapshot.to_string(), old],
            )
            .unwrap();
        let before_rows = store_rows(&env);
        let before_files = ["workbooks", "pending"].map(|name| tree(&env.dir.path().join(name)));
        let next = "t34-blocked-add";
        let (error, exit) = env.fail(&[
            "--request-id",
            next,
            "workbook",
            "add",
            example_dir("article-review").to_str().unwrap(),
        ]);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING");
        assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
        assert_eq!(error["committed"], false);
        assert_eq!(error["request_id"], next);
        assert_eq!(error["error"]["detail"]["pending_request_id"], old);
        assert!(error.get("original").is_none());
        assert!(
            error["error"]["detail"].get("pending_original").is_none(),
            "{field}: {error}"
        );
        assert_eq!(store_rows(&env), before_rows);
        assert_eq!(
            ["workbooks", "pending"].map(|name| tree(&env.dir.path().join(name))),
            before_files
        );
    }
}

// Task: C002-T34
#[test]
fn add_snapshot_target_is_bound_before_an_original_response_is_released() {
    for field in ["id", "version", "digest"] {
        let env = Env::new();
        let rid = "t34-add-target";
        let source = example_dir("two-step");
        let args = [
            "--request-id",
            rid,
            "workbook",
            "add",
            source.to_str().unwrap(),
        ];
        let committed = env.ok(&args);
        let mut replay = committed.clone();
        replay["data"]["replayed"] = json!(true);
        assert_eq!(env.ok(&args), replay);
        let mut snapshot = record(&env, rid).unwrap().reply;
        snapshot["data"][field] = match field {
            "id" => json!("other-step"),
            "version" => json!("2.0.0"),
            "digest" => json!(if committed["data"]["digest"] == "0".repeat(64) {
                "1".repeat(64)
            } else {
                "0".repeat(64)
            }),
            _ => unreachable!(),
        };
        Connection::open(env.dir.path().join("store.db"))
            .unwrap()
            .execute(
                "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
                rusqlite::params![snapshot.to_string(), rid],
            )
            .unwrap();
        let before_rows = store_rows(&env);
        let before_files = tree(&env.workbook_dir("two-step", "1.0.0"));
        let (error, exit) = env.fail(&args);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING");
        assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
        assert_eq!(error["committed"], true);
        assert_eq!(error["request_id"], rid);
        assert!(error.get("original").is_none(), "{field}: {error}");
        assert_eq!(store_rows(&env), before_rows);
        assert_eq!(tree(&env.workbook_dir("two-step", "1.0.0")), before_files);
    }
}

// Task: C002-T34
#[test]
fn fail_snapshot_status_matches_the_original_retry_even_after_later_progress() {
    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "x")]);
    let first = env.begin(&work, "outline");
    let first_args = [
        "--request-id",
        "t34-fail-first",
        "attempt",
        "fail",
        &work,
        "--attempt",
        first["data"]["attempt"].as_str().unwrap(),
        "--reason",
        "controlled failure",
    ];
    let first_response = env.ok(&first_args);
    assert_eq!(
        first_response["data"]["work_status"],
        json!({"kind":"active"})
    );
    let last = env.begin(&work, "outline");
    let last_args = [
        "--request-id",
        "t34-fail-last",
        "attempt",
        "fail",
        &work,
        "--attempt",
        last["data"]["attempt"].as_str().unwrap(),
        "--reason",
        "controlled failure",
    ];
    let last_response = env.ok(&last_args);
    assert_eq!(
        last_response["data"]["work_status"],
        json!({"kind":"blocked","reason":"retries_exhausted"})
    );
    env.ok(&["work", "cancel", &work]);
    for (rid, args, committed) in [
        ("t34-fail-first", first_args, first_response),
        ("t34-fail-last", last_args, last_response),
    ] {
        let mut replay = committed.clone();
        replay["data"]["replayed"] = json!(true);
        assert_eq!(env.ok(&args), replay);
        let original = record(&env, rid).unwrap().reply;
        for status in [
            json!({"kind":"succeeded"}),
            json!({"kind":"cancelled"}),
            json!({"kind":"active"}),
            json!({"kind":"blocked","reason":"retries_exhausted"}),
            json!({"kind":"blocked","reason":"gate"}),
            json!({"kind":"blocked","reason":"no_legal_edge"}),
        ] {
            if status == committed["data"]["work_status"] {
                continue;
            }
            let mut changed = original.clone();
            changed["data"]["work_status"] = status.clone();
            Connection::open(env.dir.path().join("store.db"))
                .unwrap()
                .execute(
                    "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
                    rusqlite::params![changed.to_string(), rid],
                )
                .unwrap();
            let before = store_rows(&env);
            let (error, exit) = env.fail(&args);
            assert_eq!(exit, 1);
            assert_eq!(error["error"]["code"], "EFFECT_PENDING");
            assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
            assert_eq!(error["committed"], true);
            assert_eq!(error["request_id"], rid);
            assert!(error.get("original").is_none(), "{rid} {status}: {error}");
            assert_eq!(store_rows(&env), before);
            Connection::open(env.dir.path().join("store.db"))
                .unwrap()
                .execute(
                    "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
                    rusqlite::params![original.to_string(), rid],
                )
                .unwrap();
        }
        assert_eq!(env.ok(&args), replay);
    }
}

// Task: C002-T34
#[test]
fn begin_snapshot_requires_match_the_frozen_node_even_when_reply_and_data_agree() {
    let env = Env::new();
    env.add_example("two-step");
    let work = env.start("two-step", &[("topic", "x")]);
    let rid = "t34-begin-requires";
    let args = [
        "--request-id",
        rid,
        "attempt",
        "begin",
        &work,
        "--node",
        "outline",
    ];
    let committed = env.ok(&args);
    assert_eq!(committed["data"]["requires"], json!([]));
    let mut replay = committed;
    replay["data"]["replayed"] = json!(true);
    assert_eq!(env.ok(&args), replay);
    let mut snapshot = record(&env, rid).unwrap().reply;
    let requires =
        json!([{"kind":"mcp","name":"extra-resource","version":null,"digest":null,"source":null}]);
    snapshot["reply"]["requires"] = requires.clone();
    snapshot["data"]["requires"] = requires;
    Connection::open(env.dir.path().join("store.db"))
        .unwrap()
        .execute(
            "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
            rusqlite::params![snapshot.to_string(), rid],
        )
        .unwrap();
    let before_rows = store_rows(&env);
    let before_files = tree(&env.dir.path().join("works"));
    let (error, exit) = env.fail(&args);
    assert_eq!(exit, 1);
    assert_eq!(error["error"]["code"], "EFFECT_PENDING");
    assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
    assert_eq!(error["committed"], true);
    assert_eq!(error["request_id"], rid);
    assert!(error.get("original").is_none(), "{error}");
    assert_eq!(store_rows(&env), before_rows);
    assert_eq!(tree(&env.dir.path().join("works")), before_files);
}
