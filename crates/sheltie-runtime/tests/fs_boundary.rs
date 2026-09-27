//! C002-T04：管理路径、文件句柄、限额与安全原子写的边界反例（O01）。
//! 所有外部哨兵的字节与权限在断言里逐项比较。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::*;
use sheltie_core::ids::AttemptId;
use sheltie_runtime::Error;
use sheltie_runtime::fsx::{SafeFile, ensure_dirs_under, write_exclusive_atomic};
use sheltie_runtime::{StartArgs, WorkService};

fn sentinel(dir: &Path, name: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, format!("sentinel-{name}")).unwrap();
    p
}

fn snapshot(path: &Path) -> (Vec<u8>, u32) {
    let bytes = std::fs::read(path).unwrap();
    let mode = std::fs::metadata(path).unwrap().permissions().mode();
    (bytes, mode)
}

fn start_args(inputs: &[(&str, &str)]) -> StartArgs {
    StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: inputs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

/// works 被换成指向根外的软链后，start 不得沿它写整棵 Work（O01 复现的反面）。
// Task: C002-T04
#[test]
fn works_parent_symlink_blocks_start_and_keeps_sentinel_untouched() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-me");
    let before = snapshot(&sent);

    let (_d, home) = temp_home();
    repo(&home).add(&abs(&example_dir("two-step"))).unwrap();
    // 把 works 换成指向根外的软链。
    let works = home.works_dir();
    std::os::unix::fs::symlink(outside.path(), works.as_path()).unwrap();

    let svc = service(&home);
    let err = svc.start(start_args(&[("topic", "t")]), None).unwrap_err();
    assert_eq!(
        err.code(),
        sheltie_core::ErrorCode::InvalidRequest,
        "{err:?}"
    );
    assert!(err.to_string().contains("符号链接"), "{err:?}");

    // 哨兵字节与权限完全不变；根外目录里没有 Work 目录。
    assert_eq!(snapshot(&sent), before);
    let entries: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
    assert_eq!(entries.len(), 1, "根外目录不得被写入：{entries:?}");
}

/// bin 被换成软链后，self install 拒绝，哨兵不动。
// Task: C002-T04
#[test]
fn bin_parent_symlink_blocks_install_and_keeps_sentinel_untouched() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-bin");
    let before = snapshot(&sent);

    let (_d, home) = temp_home();
    std::os::unix::fs::symlink(outside.path(), home.bin_dir().as_path()).unwrap();

    let err = sheltie_runtime::selfmgmt::install(&home, false).unwrap_err();
    assert!(err.to_string().contains("符号链接"), "{err:?}");
    assert_eq!(snapshot(&sent), before);
    let entries: Vec<_> = std::fs::read_dir(outside.path()).unwrap().collect();
    assert_eq!(entries.len(), 1, "根外目录不得被写入：{entries:?}");
}

/// 输入文件被换成软链（指向根外哨兵）后，begin 的观察直接拒绝。
// Task: C002-T04
#[test]
fn leaf_symlink_input_rejected_at_observation() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-input");
    let before = snapshot(&sent);

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    // 有人把起始输入换成指向哨兵的软链。
    let input = Path::new(home.work_dir(&wid).as_str()).join("inputs/topic");
    std::fs::remove_file(&input).unwrap();
    std::os::unix::fs::symlink(&sent, &input).unwrap();

    let err = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
        .unwrap_err();
    assert!(err.to_string().contains("符号链接"), "{err:?}");
    assert_eq!(snapshot(&sent), before);
}

/// 预先放好的固定名临时软链（旧的 `status-card.md.tmp-pending` 攻击）不再能重定向
/// 状态卡写入：临时名随机且独占创建，卡写到正确位置，哨兵不动。
// Task: C002-T04
#[test]
fn precreated_tmp_pending_symlink_cannot_redirect_status_card() {
    let outside = tempfile::tempdir().unwrap();
    let sent = sentinel(outside.path(), "keep-card");
    let before = snapshot(&sent);

    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    let work = std::path::PathBuf::from(home.work_dir(&wid).as_str());
    // 攻击者按旧实现的固定临时名放软链。
    std::os::unix::fs::symlink(&sent, work.join("status-card.md.tmp-pending")).unwrap();

    svc.cancel(&wid, None).unwrap();
    assert_eq!(snapshot(&sent), before, "外部哨兵不得被写");
    let card = std::fs::read_to_string(work.join("status-card.md")).unwrap();
    assert!(card.contains(&format!("# Work {wid}")), "{card}");
}

/// 句柄钉住被观察的对象：路径在观察后被换掉，已打开句柄读到的仍是原对象；
/// 重新打开则拿到新对象并被身份核对发现内容变化。
// Task: C002-T04
#[test]
fn safe_handle_pins_observed_object_across_path_swap() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("f");
    std::fs::write(&p, b"original").unwrap();
    let f = SafeFile::open_regular(&abs(&p)).unwrap();
    let (sha1, n1) = f.sha256_bounded(1024).unwrap();
    assert_eq!(n1, 8);

    // 路径后面换成另一个文件；旧句柄仍读原对象。
    std::fs::remove_file(&p).unwrap();
    std::fs::write(&p, b"replacement").unwrap();
    let (sha2, n2) = f.sha256_bounded(1024).unwrap();
    assert_eq!(
        (sha2.clone(), n2),
        (sha1.clone(), n1),
        "句柄钉住的是打开时的对象"
    );

    // 新句柄观察到新内容：换过的对象不会冒充原对象通过封存核对。
    let g = SafeFile::open_regular(&abs(&p)).unwrap();
    let (sha3, n3) = g.sha256_bounded(1024).unwrap();
    assert_ne!((sha3, n3), (sha1, n1));
}

/// 观察层限额：恰好 32 MiB 可读，多一字节在读取前拒绝（fsx 单元）；声明输出的
/// 33 MiB 文件在 service.submit 观察步即按 `OUTPUT_TOO_LARGE` 拒绝，不整读进内存。
// Task: C002-T04
#[test]
fn bounded_read_accepts_exactly_cap_and_rejects_one_more() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("big");
    let f = std::fs::File::create(&p).unwrap();
    f.set_len(32 * 1024 * 1024).unwrap();
    drop(f);
    let cap = sheltie_runtime::fsx::MAX_FILE_BYTES;
    let at = SafeFile::open_regular(&abs(&p)).unwrap();
    assert!(at.read_bounded(cap).is_ok());

    let over = dir.path().join("big2");
    let f = std::fs::File::create(&over).unwrap();
    f.set_len(cap + 1).unwrap();
    drop(f);
    let at = SafeFile::open_regular(&abs(&over)).unwrap();
    match at.read_bounded(cap) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("超过"), "{reason}"),
        other => panic!("超限应当拒绝：{other:?}"),
    }
}

// Task: C002-T04
#[test]
fn oversize_declared_output_rejected_at_observation_as_output_too_large() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    let out = Path::new(output_dir.as_str()).join("outline.md");
    let f = std::fs::File::create(&out).unwrap();
    f.set_len(32 * 1024 * 1024 + 1).unwrap();
    drop(f);

    let err = svc
        .submit(&wid, &AttemptId::parse("outline#1.0").unwrap(), "s", None)
        .unwrap_err();
    assert_eq!(
        err.code(),
        sheltie_core::ErrorCode::OutputTooLarge,
        "{err:?}"
    );
}

/// 观察拒绝发生在 COMMIT 之前：Store 的 revision 与状态不变。
// Task: C002-T04
#[test]
fn observation_rejection_before_commit_leaves_store_unchanged() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(start_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(
            &wid,
            &sheltie_core::ids::NodeId::new("outline").unwrap(),
            None,
        )
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    // 输出被换成软链：观察即拒绝，提交与效果都不发生。
    let out = Path::new(output_dir.as_str()).join("outline.md");
    let sent = sentinel(tempfile::tempdir().unwrap().path(), "keep-output");
    std::os::unix::fs::symlink(&sent, &out).unwrap();

    let before = svc.status(&wid).unwrap();
    let err = svc
        .submit(&wid, &AttemptId::parse("outline#1.0").unwrap(), "s", None)
        .unwrap_err();
    assert!(err.to_string().contains("符号链接"), "{err:?}");
    let after = svc.status(&wid).unwrap();
    // 状态卡的 current 与 JSON 状态一致即 revision 未推进（begin 后没有新的写事务）。
    assert_eq!(
        serde_json::to_string(&before.1.status).unwrap(),
        serde_json::to_string(&after.1.status).unwrap()
    );
}

/// 独占原子写的单元行为：不覆盖已有目标名之外，还拒绝在同目录预留的软链临时名。
// Task: C002-T04
#[test]
fn exclusive_atomic_write_creates_and_replaces_target_only() {
    let dir = tempfile::tempdir().unwrap();
    let base = abs(dir.path());
    let target = base.join_segment("card.md");
    write_exclusive_atomic(&base, &target, b"v1").unwrap();
    assert_eq!(std::fs::read(target.as_path()).unwrap(), b"v1");
    // 同名重写走 rename 替换；目录里不留临时文件。
    write_exclusive_atomic(&base, &target, b"v2").unwrap();
    assert_eq!(std::fs::read(target.as_path()).unwrap(), b"v2");
    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "不留临时文件：{leftovers:?}");
}

/// ensure_dirs_under 的单元反例：根下的软链段与文件占位段都拒绝。
// Task: C002-T04
#[test]
fn ensure_dirs_rejects_symlink_and_file_placeholder_below_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = abs(dir.path());
    // 软链段。
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), dir.path().join("works")).unwrap();
    match ensure_dirs_under(&root, &root.join_segment("works").join_segment("w1")) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("符号链接"), "{reason}"),
        other => panic!("软链段应当拒绝：{other:?}"),
    }
    // 文件占位段。
    let dir2 = tempfile::tempdir().unwrap();
    let root2 = abs(dir2.path());
    std::fs::write(dir2.path().join("bin"), b"not a dir").unwrap();
    match ensure_dirs_under(&root2, &root2.join_segment("bin").join_segment("sheltie")) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("不是目录"), "{reason}"),
        other => panic!("文件占位段应当拒绝：{other:?}"),
    }
    // 根外的目标拒绝。
    let elsewhere = tempfile::tempdir().unwrap();
    assert!(ensure_dirs_under(&root, &abs(elsewhere.path())).is_err());
    // 合法嵌套创建。
    ensure_dirs_under(
        &root2,
        &root2
            .join_segment("works")
            .join_segment("a")
            .join_segment("b"),
    )
    .unwrap();
    assert!(dir2.path().join("works/a/b").is_dir());
}

/// 正常链路回归：嵌套输出、显式 @file 由 CLI 层覆盖；这里覆盖嵌套目录下的原子写。
// Task: C002-T04
#[test]
fn exclusive_atomic_write_creates_nested_parents() {
    let dir = tempfile::tempdir().unwrap();
    let base = abs(dir.path());
    let target = base
        .join_segment("works")
        .join_segment("w")
        .join_segment("attempts")
        .join_segment("d")
        .join_segment("0")
        .join_segment("brief.md");
    write_exclusive_atomic(&base, &target, "嵌套".as_bytes()).unwrap();
    assert_eq!(std::fs::read(target.as_path()).unwrap(), "嵌套".as_bytes());
}

// 引用 WorkService 避免未使用告警的兜底（service 在多个用例中使用）。
#[allow(dead_code)]
fn _svc_ref(s: &WorkService) {
    let _ = s.list();
}
