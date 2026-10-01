//! 直接API恢复回归；子进程故障用例迁到sheltie-cli/tests/crash.rs，原Task归属不变。
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::*;
#[cfg(feature = "failpoint")]
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::NodeId;
#[cfg(feature = "failpoint")]
use sheltie_runtime::{Error, WorkbookRepo};

// Task: T23
#[test]
fn status_card_missing_is_regenerated_on_next_write() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&start_two_step(&svc));
    let card = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("status-card.md");
    std::fs::remove_file(&card).unwrap();
    svc.begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    assert!(card.exists());
}

// Task: C002-T27
#[cfg(feature = "failpoint")]
#[test]
fn marker_replaced_after_validation_cannot_prove_deletion() {
    use std::time::{Duration, Instant};

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
    let reached = rendezvous.path().join("reached");
    let release = rendezvous.path().join("release");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reached.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    if !reached.exists() {
        let _ = std::fs::write(&release, b"release");
        let _ = recovery.join();
        sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        panic!("marker校验后没有到达同步交错点");
    }
    std::fs::rename(marker.as_path(), &moved_marker).unwrap();
    std::fs::write(marker.as_path(), b"{\"format\":\"broken\"}\n").unwrap();
    std::fs::write(&release, b"release").unwrap();
    let error = recovery.join().unwrap().unwrap_err();
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
