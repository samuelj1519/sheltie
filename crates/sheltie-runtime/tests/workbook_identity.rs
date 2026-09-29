//! C002-T05：Workbook 身份、复制核验、只读根与初始化（O03/O06/N04/N10/§5.3）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::{NodeId, WorkId};
use sheltie_runtime::{Error, Home, StartArgs, WorkService, WorkbookRepo};

fn two_step_args() -> StartArgs {
    StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: [(
            "topic".to_string(),
            sheltie_runtime::request::InputValue::Literal {
                text: "t".to_string(),
            },
        )]
        .into_iter()
        .collect(),
    }
}

/// 已装目录被改后：verify 报 tampered，start 不再静默接受（O03）。
// Task: C002-T05
#[test]
fn load_rejects_tampered_registered_digest() {
    let (_d, home, svc) = home_with_example("two-step");
    let f =
        Path::new(home.workbook_dir("two-step", "1.0.0").as_str()).join("instructions/outline.md");
    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::write(&f, "被人改了").unwrap();

    let rows = svc_status_repo(&home).verify(None).unwrap();
    assert_eq!(rows[0].status, sheltie_runtime::VerifyStatus::Tampered);
    let err = svc.start(two_step_args(), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookTampered, "{err:?}");
    // 拒绝发生在任何物化之前。
    assert!(
        !home.works_dir().as_path().exists()
            || std::fs::read_dir(home.works_dir().as_path())
                .unwrap()
                .count()
                == 0
    );
}

fn svc_status_repo(home: &Home) -> WorkbookRepo {
    WorkbookRepo::new(home.clone())
}

/// 清理前核归属：被改过的已装目录拒绝 remove，行保留。
// Task: C002-T05
#[test]
fn remove_refuses_tampered_directory() {
    let (_d, home) = temp_home();
    let r = repo(&home);
    r.add(&abs(&example_dir("two-step")), None).unwrap();
    let f =
        Path::new(home.workbook_dir("two-step", "1.0.0").as_str()).join("instructions/outline.md");
    std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::write(&f, "改了").unwrap();

    let err = r.remove("two-step", "1.0.0", None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookTampered, "{err:?}");
    // 行还在。
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let rows: i64 = connection
        .query_row("SELECT COUNT(*) FROM workbooks", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 1);
}

/// Finder 宿主元数据按 §5.3 准确拒绝并点名；不落任何行或最终目录。
// Task: C002-T05
#[test]
fn ds_store_rejected_by_name_at_add() {
    let src = tempfile::tempdir().unwrap();
    let dst = copy_example("two-step", src.path());
    std::fs::write(dst.join(".DS_Store"), b"finder junk").unwrap();

    let (_d, home) = temp_home();
    let r = repo(&home);
    let err = r.add(&abs(&dst), None).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(".DS_Store"), "{msg}");
    assert!(
        !home.store_path().as_path().exists(),
        "非法 add 不创建Store"
    );
    assert!(!home.lock_path().as_path().exists(), "非法 add 不创建锁");
    assert!(!home.workbook_dir("two-step", "1.0.0").as_path().exists());
}

/// 新管理根上 `self install` 直接可用：建根、store.db 与 bin（O06）。
// Task: C002-T05
#[test]
fn self_install_on_new_home_creates_root_store_and_bin() {
    let (_d, home) = temp_home();
    sheltie_runtime::selfmgmt::install(&home).unwrap();
    assert!(home.store_path().as_path().exists());
    assert!(home.bin_dir().join_segment("sheltie").as_path().exists());
    // 幂等：再来一次返回 already_installed。
    let again = sheltie_runtime::selfmgmt::install(&home).unwrap();
    assert!(again.already_installed);
}

/// 只读打开不存在的库：NOT_FOUND 且不创建任何目录（GF-30）。
// Task: C002-T05
#[test]
fn readonly_open_never_creates_home() {
    let dir = tempfile::tempdir().unwrap();
    let home = Home::at(abs(dir.path()));
    assert!(matches!(
        WorkService::new(home.clone()).list(),
        Err(Error::NotFound { .. })
    ));
    let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert!(entries.is_empty(), "只读不得建目录：{entries:?}");
}

/// 真实 `sheltie` 二进制路径：问 cargo 要（crash.rs 的做法），不信相对 target 推断。
fn sheltie_bin() -> std::path::PathBuf {
    let out = std::process::Command::new("cargo")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args(["build", "-p", "sheltie-cli", "--message-format=json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cargo build 失败：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v["reason"] == "compiler-artifact"
            && v["target"]["name"] == "sheltie"
            && v["executable"].is_string()
        {
            let p = std::path::PathBuf::from(v["executable"].as_str().unwrap_or_default());
            assert!(p.exists(), "cargo 报的路径不存在：{}", p.display());
            return p;
        }
    }
    panic!("cargo 没报出 sheltie 的可执行文件路径");
}

fn account_name_oracle() -> String {
    let out = std::process::Command::new("id")
        .arg("-un")
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// 主体取真实 OS 身份：伪造 `USER` 的子进程里跑真实 CLI 开 Work，audit 主体既不等于
/// 伪造值、又与 `id -un` 的独立输出一致（D-036 的确认方式）。
// Task: C002-T05
#[test]
fn audit_principal_ignores_spoofed_user_env() {
    let (_d, home) = temp_home();
    repo(&home)
        .add(&abs(&example_dir("two-step")), None)
        .unwrap();

    let out = std::process::Command::new(sheltie_bin())
        .env("USER", "spoofed-auditor")
        .env("USERNAME", "spoofed-auditor")
        .env("SHELTIE_HOME", home.root().as_str())
        .args([
            "work",
            "start",
            "--workbook",
            "two-step",
            "--flow",
            "default",
            "--input",
            "topic=t",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "start 失败：{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let conn = rusqlite::Connection::open_with_flags(
        home.store_path().as_str(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let principal: String = conn
        .query_row(
            "SELECT principal FROM audit ORDER BY seq DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_ne!(principal, "spoofed-auditor");
    assert_eq!(principal, account_name_oracle());
}

/// in-process 读取同样不采信 USER（principal 不读环境变量；这里只验证它等于 oracle）。
// Task: C002-T05
#[test]
fn principal_matches_id_un_oracle() {
    assert_eq!(
        sheltie_runtime::observe::principal().0,
        account_name_oracle()
    );
}

/// 正例闭环：合法安装、冻结、终态查询；Workbook 删除后终态 Work 仍可完整 status。
// Task: C002-T05
#[test]
fn work_readable_after_workbook_removed() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    write_output(&output_dir_of(&begun), "outline.md", "提纲");
    // 提交后取消进入终态，再删 Workbook。
    svc.submit(
        &wid,
        &sheltie_core::ids::AttemptId::parse("outline#1.0").unwrap(),
        &sheltie_runtime::request::InputValue::Literal {
            text: "完成".to_string(),
        },
        None,
    )
    .unwrap();
    svc.cancel(&wid, None).unwrap();

    repo(&home).remove("two-step", "1.0.0", None).unwrap();
    let (card, json) = svc.status(&wid).unwrap();
    assert!(card.contains(&format!("# Work {wid}")));
    assert_eq!(json.status, sheltie_core::work::WorkStatus::Cancelled);
}

/// 只读位是减少误写，不是不可绕过：同用户 chmod 可改冻结副本，改后引擎按摘要拒绝；
/// 也不宣称只读位防篡改——防线是 `load` 的摘要核对。
// Task: C002-T05
#[test]
fn readonly_bits_reduce_accidents_but_digest_is_the_guard() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(), None).unwrap());
    let frozen = std::path::PathBuf::from(home.work_dir(&wid).as_str()).join("workbook");
    // 只读位在（含根 0555；N10）。
    let mode = std::fs::metadata(&frozen).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o555);
    // 同用户放开权限改字节。
    let mut m = std::fs::metadata(&frozen).unwrap().permissions();
    m.set_mode(0o755);
    std::fs::set_permissions(&frozen, m).unwrap();
    for entry in std::fs::read_dir(&frozen).unwrap().flatten() {
        if entry.path().is_file() {
            let mut fm = entry.metadata().unwrap().permissions();
            fm.set_mode(0o644);
            std::fs::set_permissions(entry.path(), fm).unwrap();
        }
    }
    std::fs::write(frozen.join("workbook.toml"), "改了").unwrap();
    // 独立重算：摘要确实变了，且与库里的记录不同。
    let now = WorkbookRepo::digest_dir(&abs(&frozen)).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let state_json: String = conn
        .query_row(
            "SELECT state_json FROM works WHERE work_id = ?1",
            [wid.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let recorded = serde_json::from_str::<sheltie_core::work::WorkState>(&state_json)
        .unwrap()
        .workbook
        .digest;
    assert_ne!(now, recorded, "篡改必须改变摘要");
    let err = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::StoreCorrupt, "{err:?}");
}

/// 让 dead-code 检查满意：helper 在多个用例间共享。
#[allow(dead_code)]
fn _wid_of(svc: &WorkService, args: StartArgs) -> WorkId {
    work_id_of(&svc.start(args, None).unwrap())
}
