//! C002-T04：管理路径、文件句柄、限额与安全原子写的边界反例（O01）。
//! 所有外部哨兵的字节与权限在断言里逐项比较。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::*;
use sheltie_core::ids::AttemptId;
use sheltie_runtime::Error;
use sheltie_runtime::fsx::{ExternalReadFile, ManagedFs, ManagedRelPath};
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

fn canonical_abs(path: &Path) -> sheltie_core::path::AbsPath {
    abs(&std::fs::canonicalize(path).unwrap())
}

fn lit(s: &str) -> sheltie_runtime::request::InputValue {
    sheltie_runtime::request::InputValue::Literal {
        text: s.to_string(),
    }
}

fn start_args(inputs: &[(&str, &str)]) -> StartArgs {
    StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: inputs
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    sheltie_runtime::request::InputValue::Literal {
                        text: v.to_string(),
                    },
                )
            })
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
    repo(&home)
        .add(&abs(&example_dir("two-step")), None)
        .unwrap();
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

    let err = sheltie_runtime::selfmgmt::install(&home).unwrap_err();
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
    let input = Path::new(home.work_dir(&wid).as_str()).join("start-inputs/topic");
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
    let f = ExternalReadFile::open_regular(&abs(&p)).unwrap();
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
    let g = ExternalReadFile::open_regular(&abs(&p)).unwrap();
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
    let at = ExternalReadFile::open_regular(&abs(&p)).unwrap();
    assert!(at.read_bounded(cap).is_ok());

    let over = dir.path().join("big2");
    let f = std::fs::File::create(&over).unwrap();
    f.set_len(cap + 1).unwrap();
    drop(f);
    let at = ExternalReadFile::open_regular(&abs(&over)).unwrap();
    match at.read_bounded(cap) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("超过"), "{reason}"),
        other => panic!("超限应当拒绝：{other:?}"),
    }
}

// Task: C002-T24
#[test]
fn exclusive_create_returns_the_same_handle_with_final_written_metadata() {
    let (_dir, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = sheltie_runtime::fsx::ManagedFs::open_existing(&home).unwrap();
    let path = sheltie_runtime::fsx::ManagedRelPath::new("tmp/observed-create").unwrap();
    fs.ensure_dir(
        &lock,
        &sheltie_runtime::fsx::ManagedRelPath::new("tmp").unwrap(),
    )
    .unwrap();
    let file = fs.write_new_observed(&lock, &path, b"new bytes").unwrap();
    assert_eq!(file.metadata().len(), 9);
    assert_eq!(file.read_bounded(9).unwrap(), b"new bytes");
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
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("s"),
            None,
        )
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
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("s"),
            None,
        )
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
    let base = canonical_abs(dir.path());
    let home = sheltie_runtime::Home::resolve(Some((base).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("card.md").unwrap();
    fs.write_atomic(&lock, &target, b"v1").unwrap();
    assert_eq!(
        std::fs::read(home.root().as_path().join("card.md")).unwrap(),
        b"v1"
    );
    // 同名重写走 rename 替换；目录里不留临时文件。
    fs.write_atomic(&lock, &target, b"v2").unwrap();
    assert_eq!(
        std::fs::read(home.root().as_path().join("card.md")).unwrap(),
        b"v2"
    );
    let leftovers: Vec<_> = std::fs::read_dir(home.root().as_path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(".card.md.tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "不留临时文件：{leftovers:?}");
}

// Task: C002-T19
#[test]
fn directory_handle_remains_anchored_after_parent_path_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sentinel = sentinel(outside.path(), "outside");
    let before = snapshot(&sentinel);
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let held = fs
        .ensure_dir(&lock, &ManagedRelPath::new("works/slot").unwrap())
        .unwrap();

    std::fs::rename(
        home.root().as_path().join("works/slot"),
        home.root().as_path().join("works/held"),
    )
    .unwrap();
    std::os::unix::fs::symlink(outside.path(), home.root().as_path().join("works/slot")).unwrap();

    held.write_new(&lock, "proof.txt", b"anchored").unwrap();
    held.rename_new(&lock, "proof.txt", "renamed.txt").unwrap();

    assert_eq!(snapshot(&sentinel), before);
    assert_eq!(
        std::fs::read(home.root().as_path().join("works/held/renamed.txt")).unwrap(),
        b"anchored"
    );
    assert!(
        std::fs::symlink_metadata(home.root().as_path().join("works/slot"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

// Task: C002-T19
#[test]
fn managed_operations_reject_a_lock_from_another_home() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let home =
        sheltie_runtime::Home::resolve(Some((canonical_abs(first.path())).as_str())).unwrap();
    let other =
        sheltie_runtime::Home::resolve(Some((canonical_abs(second.path())).as_str())).unwrap();
    let wrong_lock = other.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("never-created").unwrap();

    assert!(
        fs.write_new(&wrong_lock, &target, b"must not write")
            .is_err()
    );
    assert!(!home.root().as_path().join("never-created").exists());
}

// Task: C002-T19
#[test]
fn readonly_rechecks_hardlink_count_before_chmod() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let rel = ManagedRelPath::new("payload").unwrap();
    fs.write_new(&lock, &rel, b"preserve").unwrap();
    let held = fs.open_regular(&rel).unwrap();
    let external_alias = outside.path().join("alias");
    std::fs::hard_link(home.root().as_path().join("payload"), &external_alias).unwrap();
    let original = snapshot(home.root().as_path().join("payload").as_std_path());

    assert!(fs.set_readonly(&lock, &held).is_err());
    assert_eq!(
        snapshot(home.root().as_path().join("payload").as_std_path()),
        original
    );
    assert_eq!(snapshot(&external_alias), original);
}

/// ensure_dirs_under 的单元反例：根下的软链段与文件占位段都拒绝。
// Task: C002-T04
#[test]
fn ensure_dirs_rejects_symlink_and_file_placeholder_below_root() {
    let dir = tempfile::tempdir().unwrap();
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    // 软链段。
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), home.root().as_path().join("works")).unwrap();
    match fs.ensure_dir(&lock, &ManagedRelPath::new("works/w1").unwrap()) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("符号链接"), "{reason}"),
        other => panic!("软链段应当拒绝：{other:?}"),
    }
    // 文件占位段。
    let dir2 = tempfile::tempdir().unwrap();
    let home2 =
        sheltie_runtime::Home::resolve(Some((canonical_abs(dir2.path())).as_str())).unwrap();
    let lock2 = home2.acquire_lock().unwrap();
    let fs2 = ManagedFs::open_existing(&home2).unwrap();
    std::fs::write(home2.root().as_path().join("bin"), b"not a dir").unwrap();
    match fs2.ensure_dir(&lock2, &ManagedRelPath::new("bin/sheltie").unwrap()) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("不是目录"), "{reason}"),
        other => panic!("文件占位段应当拒绝：{other:?}"),
    }
    // 根外的目标拒绝。
    let elsewhere = tempfile::tempdir().unwrap();
    assert!(home.to_rel(&abs(elsewhere.path())).is_err());
    // 合法嵌套创建。
    fs2.ensure_dir(&lock2, &ManagedRelPath::new("works/a/b").unwrap())
        .unwrap();
    assert!(home2.root().as_path().join("works/a/b").is_dir());
}

/// 正常链路回归：嵌套输出、显式 @file 由 CLI 层覆盖；这里覆盖嵌套目录下的原子写。
// Task: C002-T04
#[test]
fn exclusive_atomic_write_creates_nested_parents() {
    let dir = tempfile::tempdir().unwrap();
    let home = sheltie_runtime::Home::resolve(Some((canonical_abs(dir.path())).as_str())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("works/w/attempts/d/0/brief.md").unwrap();
    fs.write_atomic(&lock, &target, "嵌套".as_bytes()).unwrap();
    assert_eq!(
        std::fs::read(home.root().as_path().join(target.as_str())).unwrap(),
        "嵌套".as_bytes()
    );
}

// Task: C002-T19
#[test]
fn managed_relative_path_rejects_escaping_and_nul_segments() {
    for bad in ["", "/abs", "../x", "a/../b", "a//b", "./a", "a\0b"] {
        assert!(ManagedRelPath::new(bad).is_err(), "must reject {bad:?}");
    }
    assert_eq!(
        ManagedRelPath::new("works/w1/status-card.md")
            .unwrap()
            .as_str(),
        "works/w1/status-card.md"
    );
}

// Task: C002-T19
#[test]
fn managed_fs_anchors_open_and_rejects_leaf_symlink_and_fifo() {
    let (_d, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let target = ManagedRelPath::new("works/w1/output.md").unwrap();
    fs.ensure_dir(&lock, &ManagedRelPath::new("works/w1").unwrap())
        .unwrap();
    fs.write_new(&lock, &target, b"owned bytes").unwrap();
    assert!(fs.write_new(&lock, &target, b"replacement").is_err());
    let opened = fs.open_regular(&target).unwrap();
    assert_eq!(opened.read_bounded(128).unwrap(), b"owned bytes");
    fs.set_readonly(&lock, &opened).unwrap();
    assert_eq!(
        std::fs::metadata(home.root().as_path().join("works/w1/output.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o444
    );

    let outside = tempfile::tempdir().unwrap();
    let sentinel = sentinel(outside.path(), "outside-managed");
    let before = snapshot(&sentinel);
    std::fs::remove_file(home.root().as_path().join("works/w1/output.md")).unwrap();
    std::os::unix::fs::symlink(&sentinel, home.root().as_path().join("works/w1/output.md"))
        .unwrap();
    assert!(fs.open_regular(&target).is_err());
    assert_eq!(snapshot(&sentinel), before);

    let fifo_path = home.root().as_path().join("works/w1/input.fifo");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo_path)
            .status()
            .unwrap()
            .success()
    );
    let fifo = ManagedRelPath::new("works/w1/input.fifo").unwrap();
    assert!(fs.open_regular(&fifo).is_err());
}

// Task: C002-T19
#[test]
fn managed_rename_no_replace_and_remove_tree_stay_under_root() {
    let (_d, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let parent = ManagedRelPath::new("pending/test/payload").unwrap();
    fs.ensure_dir(&lock, &parent).unwrap();
    let source = ManagedRelPath::new("pending/test/payload/source").unwrap();
    let target = ManagedRelPath::new("pending/test/payload/target").unwrap();
    fs.write_new(&lock, &source, b"source").unwrap();
    fs.write_new(&lock, &target, b"target").unwrap();
    assert!(fs.rename_new(&lock, &source, &target).is_err());
    assert_eq!(
        fs.open_regular(&source).unwrap().read_bounded(32).unwrap(),
        b"source"
    );
    assert_eq!(
        fs.open_regular(&target).unwrap().read_bounded(32).unwrap(),
        b"target"
    );
    fs.write_atomic(&lock, &target, b"replacement").unwrap();
    assert_eq!(
        fs.open_regular(&target).unwrap().read_bounded(32).unwrap(),
        b"replacement"
    );

    let outside = tempfile::tempdir().unwrap();
    let sentinel_path = sentinel(outside.path(), "atomic-target");
    let sentinel_before = snapshot(&sentinel_path);
    std::fs::remove_file(home.root().as_path().join("pending/test/payload/target")).unwrap();
    std::os::unix::fs::symlink(
        &sentinel_path,
        home.root().as_path().join("pending/test/payload/target"),
    )
    .unwrap();
    fs.write_atomic(&lock, &target, b"replaced symlink entry")
        .unwrap();
    assert_eq!(snapshot(&sentinel_path), sentinel_before);
    assert_eq!(
        fs.open_regular(&target).unwrap().read_bounded(64).unwrap(),
        b"replaced symlink entry"
    );

    fs.remove_owned_tree(&lock, &ManagedRelPath::new("pending/test").unwrap())
        .unwrap();
    assert!(fs.open_regular(&source).is_err());
}

// Task: C002-T19
#[test]
fn managed_rename_rejects_symlink_source_without_moving_or_touching_target() {
    let outside = tempfile::tempdir().unwrap();
    let sentinel_path = sentinel(outside.path(), "rename-source");
    let sentinel_before = snapshot(&sentinel_path);
    let (_d, home) = temp_home();
    let lock = home.acquire_lock().unwrap();
    let fs = ManagedFs::open_existing(&home).unwrap();
    let directory = ManagedRelPath::new("pending/rename-test").unwrap();
    fs.ensure_dir(&lock, &directory).unwrap();
    let source = ManagedRelPath::new("pending/rename-test/source").unwrap();
    let target = ManagedRelPath::new("pending/rename-test/target").unwrap();
    std::os::unix::fs::symlink(&sentinel_path, home.root().as_path().join(source.as_str()))
        .unwrap();

    assert!(fs.rename_new(&lock, &source, &target).is_err());
    assert!(
        std::fs::symlink_metadata(home.root().as_path().join(source.as_str()))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!home.root().as_path().join(target.as_str()).exists());
    assert_eq!(snapshot(&sentinel_path), sentinel_before);
}

// 引用 WorkService 避免未使用告警的兜底（service 在多个用例中使用）。
#[allow(dead_code)]
fn _svc_ref(s: &WorkService) {
    let _ = s.list();
}
