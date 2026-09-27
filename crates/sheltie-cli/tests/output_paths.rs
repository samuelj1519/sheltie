//! C002-T03：输出路径规则的 CLI 真实链路。声明 `outputs/brief.md`、`stats.json`
//! 不再与引擎文件比较（workbook 合同 §3.2）；祖先与大小写别名在 `workbook add` 拒绝。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

/// 一个单节点 Workbook：两个输出 `outputs/brief.md` 与嵌套 `notes/sub/x.md`。
fn make_workbook(env: &Env, outputs: &str) -> std::path::PathBuf {
    let src = env.dir.path().join("wb");
    let _ = std::fs::remove_dir_all(&src);
    std::fs::create_dir_all(src.join("flows")).unwrap();
    std::fs::write(
        src.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"wb\"\nversion = \"1.0.0\"\nname = \"输出路径\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        src.join("flows/default.toml"),
        format!(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = {{ text = \"做。\" }}\noutputs = [{outputs}]\n"
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
        assert_eq!(code, 1, "{a} 与 {b}");
        assert_eq!(v["error"]["code"], "FLOW_INVALID", "{a} 与 {b}");
        assert!(
            v["error"]["detail"]["path"]
                .as_str()
                .unwrap()
                .starts_with("nodes[0].outputs"),
            "{a} 与 {b}：{}",
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

    // 在任务书给的输出路径逐个写文件（引擎 brief 与它们不同路径），然后提交成功。
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
        "完成",
    ]);
    assert_eq!(
        submitted["data"]["work_status"],
        serde_json::json!({"kind": "succeeded"})
    );
    assert_eq!(submitted["data"]["outputs"].as_object().unwrap().len(), 3);

    // 引擎任务书仍在；worker 的 outputs/brief.md 与它不同路径，互不覆盖。
    assert!(brief_path.is_file());
    let outputs_brief = std::path::PathBuf::from(begun["data"]["outputs"]["doc"].as_str().unwrap());
    assert_ne!(brief_path, outputs_brief);
    assert!(outputs_brief.is_file());
    assert!(
        std::path::PathBuf::from(begun["data"]["outputs"]["stats"].as_str().unwrap()).is_file()
    );
}
