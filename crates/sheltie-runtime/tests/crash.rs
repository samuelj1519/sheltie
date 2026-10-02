//! 直接API恢复回归；子进程故障用例迁到sheltie-cli/tests/crash.rs，原Task归属不变。
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::*;
#[cfg(feature = "failpoint")]
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::NodeId;
#[cfg(feature = "failpoint")]
use sheltie_runtime::{Error, WorkbookRepo};

// Task: C005-T02
#[test]
fn status_card_missing_is_regenerated_on_next_write() {
    let (_d, home, svc) = home_with_example("two-step");
    let started = svc
        .start(
            sheltie_runtime::StartArgs {
                name: Some("卡片恢复".into()),
                ..two_step_args(&[("topic", "给新人介绍 Sheltie")])
            },
            None,
        )
        .unwrap();
    let wid = work_id_of(&started);
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    std::fs::remove_file(&card).unwrap();
    svc.begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let bytes = std::fs::read(&card).unwrap();
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let (revision, state): (u64, String) = connection
        .query_row(
            "SELECT revision, state_json FROM works WHERE work_id = ?1",
            [wid.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(revision, 2);
    let state: serde_json::Value = serde_json::from_str(&state).unwrap();
    assert_eq!(state["status"]["kind"], "active");
    let work_dir = format!("{}/works/{wid}", home.root().as_str());
    let expected = format!(
        "# Work {wid}（卡片恢复）\n\n\
workbook: two-step@1.0.0   flow: default   status: active\n\
current: outline#1\n\
done: 无\n\
pending: summary\n\
visits: outline 1/1, summary 0/1\n\n\
## 当前任务\n\n\
attempt: outline#1.0\n\
brief_path: {work_dir}/attempts/outline/occurrence-001/attempt-000/brief.md\n\
inputs:\n\
\x20\x20topic → {work_dir}/start-inputs/topic (sha256 1ebd7fe40a02b8958d721b6ca82726ad34265e78dfb626b97c12bdfd8751f2f7, 23 B)\n\
draft_outputs:\n\
\x20\x20outline → {work_dir}/attempts/outline/occurrence-001/attempt-000/outputs/outline.md\n\n\
## 最近一次尝试\n\n\
outline#1.0 running\n\n\
## 合法下一步\n\n\
- sheltie attempt submit {wid} --attempt outline#1.0 --summary \"<一句话结论>\"\n\
- sheltie attempt fail {wid} --attempt outline#1.0 --reason \"<原因>\"\n\
- sheltie attempt replace {wid} --attempt outline#1.0 --reason \"<原因>\"\n\
- sheltie work cancel {wid}\n"
    );
    assert_eq!(
        bytes,
        expected.as_bytes(),
        "卡来自begin后的最新事实且字节符合状态卡合同"
    );
}

// Task: C002-T27
#[cfg(feature = "failpoint")]
#[test]
fn marker_replaced_after_validation_cannot_prove_deletion() {
    let (_dir, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(
        &abs(&example_dir("two-step")),
        Some("t27-marker-race-add".into()),
    )
    .unwrap();
    let request_id = "t27-marker-replaced-after-validation";
    repo.remove("two-step", "1.0.0", Some(request_id.into()))
        .unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let effects_raw: String = conn
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects_raw).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let internal_id = pending.split('/').nth(1).unwrap();
    let marker = home
        .pending_dir()
        .join_segment(&format!("{internal_id}.deleted"));
    let valid_marker = std::fs::read(marker.as_path()).unwrap();
    conn.execute(
        "UPDATE requests SET published = 0 WHERE request_id = ?1",
        [request_id],
    )
    .unwrap();

    let moved_marker_dir = tempfile::tempdir().unwrap();
    let moved_marker = moved_marker_dir.path().join("validated-marker");
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "delete_marker_after_validation_before_sync",
        request_id,
        rendezvous.path(),
    )
    .unwrap();
    let recovery_home = home.clone();
    let recovery_id = request_id.to_string();
    let recovery = std::thread::spawn(move || {
        WorkbookRepo::new(recovery_home).remove("two-step", "1.0.0", Some(recovery_id))
    });
    let release = rendezvous.path().join("release");
    let mut worker = RendezvousWorker::single(recovery, rendezvous.path());
    worker.wait("marker校验后没有到达同步交错点");
    std::fs::rename(marker.as_path(), &moved_marker).unwrap();
    std::fs::write(marker.as_path(), b"{\"format\":\"broken\"}\n").unwrap();
    std::fs::write(&release, b"release").unwrap();
    let error = worker.finish().unwrap().unwrap_err();
    sheltie_runtime::failpoint::disarm_rendezvous().unwrap();

    let Error::EffectPending {
        committed,
        request_id: actual_request,
        cause,
        ..
    } = error
    else {
        panic!("被替换的marker不能证明本次删除：{error:?}");
    };
    assert!(committed);
    assert_eq!(actual_request, request_id);
    assert_eq!(cause, ErrorCode::StoreCorrupt);
    assert_eq!(std::fs::read(&moved_marker).unwrap(), valid_marker);
    assert_eq!(
        std::fs::read(marker.as_path()).unwrap(),
        b"{\"format\":\"broken\"}\n"
    );
    assert_eq!(
        conn.query_row(
            "SELECT published FROM requests WHERE request_id = ?1",
            [request_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
}
