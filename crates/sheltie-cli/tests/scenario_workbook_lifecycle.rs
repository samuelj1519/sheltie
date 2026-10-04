//! T22：Workbook 生命周期场景。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

// Task: T22
#[test]
fn remove_in_use_workbook_is_rejected_with_work_list() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let (e, _) = env.fail(&["workbook", "remove", "two-step@1.0.0"]);
    assert_eq!(e["error"]["code"], "WORKBOOK_IN_USE");
    assert_eq!(e["error"]["detail"]["works"], serde_json::json!([wid]));
}

// Task: T22
#[test]
fn remove_after_work_succeeds_then_status_still_renders() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let b1 = env.begin(&wid, "outline");
    let s1 = env.submit_all(&wid, &b1, "ok");
    let b2 = env.follow_begin(&s1, "summary");
    env.submit_all(&wid, &b2, "ok");
    env.ok(&["workbook", "remove", "two-step@1.0.0"]);
    let st = env.status(&wid);
    assert_eq!(st["data"]["status"]["kind"], "succeeded");
    assert_eq!(
        st["data"]["done"],
        serde_json::json!(["outline#1", "summary#1"])
    );
}

// Task: T22
#[test]
fn editing_repository_copy_does_not_change_running_work_brief() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let f = env
        .workbook_dir("two-step", "1.0.0")
        .join("instructions/outline.md");
    make_writable(&f);
    std::fs::write(&f, "被改过").unwrap();
    let b = env.begin(&wid, "outline");
    let brief = std::fs::read_to_string(b["data"]["brief_path"].as_str().unwrap()).unwrap();
    assert!(brief.contains("列一份提纲"));
}

// Task: T22
#[test]
fn add_second_version_marks_it_latest_and_start_defaults_to_it() {
    let env = Env::new();
    env.add_example("two-step");
    let src = env.dir.path().join("v2");
    copy_dir(&example_dir("two-step"), &src);
    let m = std::fs::read_to_string(src.join("workbook.toml"))
        .unwrap()
        .replace("1.0.0", "2.0.0");
    std::fs::write(src.join("workbook.toml"), m).unwrap();
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let list = env.ok(&["workbook", "list"]);
    let rows = list["data"].as_array().unwrap();
    let latest: Vec<&str> = rows
        .iter()
        .filter(|r| r["latest"] == true)
        .map(|r| r["version"].as_str().unwrap())
        .collect();
    assert_eq!(latest, vec!["2.0.0"]);
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
    assert_eq!(v["data"]["workbook"]["version"], "2.0.0");
}

// Task: C002-T21
#[cfg(unix)]
#[test]
fn verify_rejects_installed_workbook_root_symlink() {
    let env = Env::new();
    env.add_example("two-step");
    let installed = env.workbook_dir("two-step", "1.0.0");
    let saved = env.dir.path().join("original-workbook");
    let before = std::fs::read(installed.join("workbook.toml")).unwrap();
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(
        installed.parent().unwrap(),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    std::fs::set_permissions(&installed, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::rename(&installed, &saved).unwrap();
    std::os::unix::fs::symlink(&saved, &installed).unwrap();

    let (_error, exit) = env.fail(&["workbook", "verify", "two-step@1.0.0"]);
    assert_eq!(exit, 1);
    assert_eq!(std::fs::read(saved.join("workbook.toml")).unwrap(), before);
}

// Task: C002-T21
#[cfg(unix)]
#[test]
fn start_rejects_installed_workbook_parent_symlink() {
    let env = Env::new();
    env.add_example("two-step");
    let installed_id_dir = env
        .workbook_dir("two-step", "1.0.0")
        .parent()
        .unwrap()
        .to_path_buf();
    let original = env.dir.path().join("original-workbook-id-dir");
    let sentinel = installed_id_dir.join("1.0.0/workbook.toml");
    let sentinel_bytes = std::fs::read(sentinel).unwrap();
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(&installed_id_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::rename(&installed_id_dir, &original).unwrap();
    std::os::unix::fs::symlink(&original, &installed_id_dir).unwrap();

    let (_error, exit) = env.fail(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    assert_eq!(exit, 1);
    assert_eq!(
        std::fs::read(original.join("1.0.0/workbook.toml")).unwrap(),
        sentinel_bytes
    );
}

// Task: C002-T21
#[test]
fn workbook_add_accepts_current_directory_dot_path() {
    let env = Env::new();
    let source = example_dir("two-step");
    let output = env
        .cmd(&["workbook", "add", "."])
        .current_dir(&source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "`workbook add .` 应接受词法点段：{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["data"]["id"], "two-step");
}
