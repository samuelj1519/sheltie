//! C002-T03: real CLI output-path scenarios; outputs/brief.md and stats.json declarations
//! do not collide with engine files (workbook §3.2); workbook add rejects ancestor/case aliases.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

/// Single-node Workbook with outputs/brief.md and nested notes/sub/x.md outputs.
fn make_workbook(env: &Env, outputs: &str) -> std::path::PathBuf {
    let src = env.dir.path().join("wb");
    let _ = std::fs::remove_dir_all(&src);
    std::fs::create_dir_all(src.join("flows")).unwrap();
    std::fs::write(
        src.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"wb\"\nversion = \"1.0.0\"\nname = \"Output paths\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        src.join("flows/default.toml"),
        format!(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = {{ text = \"Do the task.\" }}\noutputs = [{outputs}]\n"
        ),
    )
    .unwrap();
    src
}

// Task: C002-T03
#[test]
fn add_rejects_ancestor_and_folded_alias_output_paths_at_cli() {
    let env = Env::new();
    for (a, b) in [
        ("out", "out/sub"),
        ("OUT.md", "out.md"),
        ("Out", "out/x.md"),
    ] {
        let src = make_workbook(
            &env,
            &format!("{{ name = \"p\", path = \"{a}\" }}, {{ name = \"q\", path = \"{b}\" }}"),
        );
        let (v, code) = env.fail(&["workbook", "add", src.to_str().unwrap()]);
        assert_eq!(code, 1, "{a} and {b}");
        assert_eq!(v["error"]["code"], "FLOW_INVALID", "{a} and {b}");
        assert!(
            v["error"]["detail"]["path"]
                .as_str()
                .unwrap()
                .starts_with("nodes[0].outputs"),
            "{a} and {b}：{}",
            v["error"]["detail"]["path"]
        );
    }
}

// Task: C002-T03
#[test]
fn worker_outputs_named_brief_and_stats_write_and_submit_via_real_chain() {
    let env = Env::new();
    let src = make_workbook(
        &env,
        "{ name = \"doc\", path = \"outputs/brief.md\" }, \
         { name = \"stats\", path = \"stats.json\" }, \
         { name = \"nested\", path = \"notes/sub/x.md\" }",
    );
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let wid = env.start("wb", &[]);
    let begun = env.begin(&wid, "only");

    // Write each brief-declared output, separate from engine brief files, then submit successfully.
    let brief_path = std::path::PathBuf::from(begun["data"]["brief_path"].as_str().unwrap());
    for (_, path) in begun["data"]["outputs"].as_object().unwrap() {
        let p = std::path::PathBuf::from(path.as_str().unwrap());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, format!("output for {}", p.display())).unwrap();
    }
    let submitted = env.ok(&[
        "attempt",
        "submit",
        &wid,
        "--attempt",
        "only#1.0",
        "--summary",
        "Completed",
    ]);
    assert_eq!(
        submitted["data"]["work_status"],
        serde_json::json!({"kind": "succeeded"})
    );
    assert_eq!(submitted["data"]["outputs"].as_object().unwrap().len(), 3);

    // Engine brief remains; worker outputs/brief.md is a distinct nonoverlapping path.
    assert!(brief_path.is_file());
    let outputs_brief = std::path::PathBuf::from(begun["data"]["outputs"]["doc"].as_str().unwrap());
    assert_ne!(brief_path, outputs_brief);
    assert!(outputs_brief.is_file());
    assert!(
        std::path::PathBuf::from(begun["data"]["outputs"]["stats"].as_str().unwrap()).is_file()
    );
}
