//! C002-T12：spec-dev 单任务与整体交付闭环的五组回归。
//!
//! 真实 CLI 走 Workbook 流程，项目是各测试自建的独立临时 Git 仓库。模拟 worker 按
//! `workbooks/spec-dev` 说明书的约定写产物（任务基线/候选提交、批准摘要、占位体任务编号），
//! 核对规则用本文件的手写 oracle 实现（verify.md 的核对条款），期望值不调用生产 helper。
//! 真实 agent 交付质量留给 C002-T16。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command as Sh;

use common::*;
use serde_json::Value;

#[path = "common/spec_dev_replan.rs"]
mod replan;
use replan::verified_table;

// ── 独立临时 Git 仓库 ─────────────────────────────────────────────────────

struct Proj {
    root: PathBuf,
}

impl Proj {
    fn init(env: &Env, name: &str) -> Proj {
        let root = env.dir.path().join(name);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(
            root.join("gate.sh"),
            "#!/bin/sh\necho 'fixture gate passed'\nexit 0\n",
        )
        .unwrap();
        let proj = Proj { root };
        proj.git(&["init", "-q"]);
        proj.git(&["config", "user.email", "sim@example.com"]);
        proj.git(&["config", "user.name", "sim"]);
        proj.git(&["config", "commit.gpgsign", "false"]);
        // 宿主钩子（pre-commit 等）不进夹具：空目录当 hooksPath，提交只属于这个临时仓库。
        std::fs::create_dir_all(proj.root.join("hooks-empty")).unwrap();
        proj.git(&["config", "core.hooksPath", "hooks-empty"]);
        proj.git(&["add", "-A"]);
        proj.git(&["commit", "-q", "-m", "chore: 起点"]);
        proj
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn git(&self, args: &[&str]) -> String {
        // 仓库发现只认 cwd 里的临时仓库。宿主钩子进程会留给子孙 GIT_DIR/GIT_INDEX_FILE
        // 之类的变量；它们一旦生效，git 会忽略 cwd 去操作宿主仓库——夹具提交会写坏宿主的
        // 暂存区。这里把这类变量全部摘掉。
        let out = Sh::new("git")
            .args(args)
            .current_dir(&self.root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_OBJECT_DIRECTORY")
            .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_NAMESPACE")
            .env_remove("GIT_CEILING_DIRECTORIES")
            .env_remove("GIT_DISCOVERY_ACROSS_FILESYSTEM")
            .env_remove("GIT_CONFIG_GLOBAL")
            .env_remove("GIT_CONFIG_SYSTEM")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} 失败：{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
    }

    /// 跑门禁（方案「门禁」一节抄进项目的命令）。
    fn gate(&self) {
        let out = Sh::new("sh")
            .arg("gate.sh")
            .current_dir(&self.root)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "门禁失败：{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn capture_gate(&self, base: &str, candidate: &str, evidence: &Path) {
        let output = Sh::new("sh")
            .arg("gate.sh")
            .current_dir(&self.root)
            .output()
            .unwrap();
        let stdout = evidence.with_extension("stdout");
        let stderr = evidence.with_extension("stderr");
        std::fs::write(&stdout, &output.stdout).unwrap();
        std::fs::write(&stderr, &output.stderr).unwrap();
        std::fs::write(evidence, format!(
            "命令: sh gate.sh\n目录: {}\n退出码: {}\n提交: {candidate}\n基线: {base}\n文件: {}\nstdout: {}\nstdout_sha256: {}\nstderr: {}\nstderr_sha256: {}\n",
            self.root.display(), output.status.code().unwrap(), self.diff_names(base, candidate).join(","),
            stdout.display(), sha256_file(&stdout), stderr.display(), sha256_file(&stderr)
        )).unwrap();
        println!(
            "SPEC_DEV_RAW_GATE {}",
            serde_json::json!({
                "argv": ["sh", "gate.sh"], "cwd": self.root,
                "exit": output.status.code(), "baseline": base, "candidate": candidate,
                "files": self.diff_names(base, candidate),
                "stdout_bytes": output.stdout, "stderr_bytes": output.stderr,
                "manifest": evidence,
            })
        );
        assert!(
            output.status.success(),
            "fixture gate failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// 提交全部改动，返回候选提交哈希。
    fn commit(&self, msg: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
        self.head()
    }

    /// `git diff --name-only <基线>..<提交>`。
    fn diff_names(&self, base: &str, head: &str) -> Vec<String> {
        let range = format!("{base}..{head}");
        self.git(&["diff", "--name-only", &range])
            .lines()
            .map(|l| l.to_string())
            .collect()
    }

    /// `git diff <基线>..<提交> -- <文件>` 里的实际增删行（去掉 diff 头）。
    fn diff_lines(&self, base: &str, head: &str, file: &str) -> Vec<String> {
        let range = format!("{base}..{head}");
        self.git(&["diff", &range, "--", file])
            .lines()
            .filter(|l| {
                (l.starts_with('+') || l.starts_with('-'))
                    && !l.starts_with("+++")
                    && !l.starts_with("---")
            })
            .map(|l| l.to_string())
            .collect()
    }
}

// ── 独立 oracle：摘要、约定文本解析与核对规则 ────────────────────────────

/// 与说明书一致：`shasum -a 256`（Linux 是 `sha256sum`）。不走生产摘要代码。
fn sha256_file(path: &Path) -> String {
    let p = path.to_str().unwrap();
    if let Ok(out) = Sh::new("shasum").args(["-a", "256", p]).output() {
        if out.status.success() {
            return first_field(&String::from_utf8_lossy(&out.stdout));
        }
    }
    let out = Sh::new("sha256sum").arg(p).output().unwrap();
    assert!(out.status.success(), "sha256sum 失败：{p}");
    first_field(&String::from_utf8_lossy(&out.stdout))
}

fn first_field(text: &str) -> String {
    text.split_whitespace().next().unwrap().to_string()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

/// 约定文本里的「键: 值」行（change / report / decision 的元数据行）。
fn field<'a>(text: &'a str, key: &str) -> &'a str {
    let prefix = format!("{key}:");
    text.lines()
        .find_map(|l| l.strip_prefix(prefix.as_str()))
        .map(|v| v.trim())
        .unwrap_or_else(|| panic!("缺「{key}:」行"))
}

/// plan.md「基线」那一行的原始基线：`原始基线（…）：`<完整哈希>``。
fn plan_baseline(plan_text: &str) -> String {
    let line = plan_text
        .lines()
        .find(|l| l.starts_with("原始基线（"))
        .expect("plan.md 缺「基线」的原始基线行");
    let hash = line.rsplit('`').nth(1).expect("原始基线行缺反引号里的哈希");
    assert_eq!(hash.len(), 40, "原始基线应是完整哈希：{hash}");
    hash.to_string()
}

/// 占位体行的归属标记：行尾 `// Tnn` 或 `# Tnn`。
fn line_task(line: &str) -> Option<String> {
    for sep in ["//", "#"] {
        if let Some(idx) = line.rfind(sep) {
            let tag = line[idx + sep.len()..].trim();
            if let Some(rest) = tag.strip_prefix('T') {
                if rest.len() == 2 && rest.bytes().all(|b| b.is_ascii_digit()) {
                    return Some(tag.to_string());
                }
            }
        }
    }
    None
}

/// verify.md 第 6 步：占位体只查本任务。返回 (本任务残留行, 没标编号的行)。
/// 约定两种写法：Rust 占位 + `// Tnn`、Python 占位 + `# Tnn`（骨架规则「其他语言类推」）。
fn placeholder_left(text: &str, own: &str) -> (Vec<String>, Vec<String>) {
    let rust_stub = concat!("todo", "!()");
    let mut own_left = Vec::new();
    let mut unmarked = Vec::new();
    for line in text.lines() {
        if !(line.contains(rust_stub) || line.contains("raise NotImplementedError")) {
            continue;
        }
        match line_task(line) {
            Some(task) if task == own => own_left.push(line.to_string()),
            Some(_) => {}
            None => unmarked.push(line.to_string()),
        }
    }
    (own_left, unmarked)
}

/// verify.md「怎么验」第 1 步：批准记录的两个摘要必须等于绑定输入文件的 shasum。
/// 对不上返回文件侧的真实摘要（写进「发现」）。
fn check_approval(
    decision_text: &str,
    spec_path: &Path,
    plan_path: &Path,
) -> Result<(), (String, String)> {
    let got_spec = sha256_file(spec_path);
    let got_plan = sha256_file(plan_path);
    if field(decision_text, "批准的规格") == got_spec
        && field(decision_text, "批准的方案") == got_plan
    {
        Ok(())
    } else {
        Err((got_spec, got_plan))
    }
}

/// verify.md 第 5 步：本任务（含修复轮次）的改动都在「只改哪些文件」里。返回越界文件。
fn out_of_scope(diff: &[String], whitelist: &[&str]) -> Vec<String> {
    diff.iter()
        .filter(|f| !whitelist.contains(&f.as_str()))
        .cloned()
        .collect()
}

/// 实现者动作：删掉禁用标记那一行，其余字节不动。
fn unskip(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if !line.contains("@pytest.mark.skip") {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

// ── 约定文本的写法（模拟 worker 按说明书产出） ──────────────────────────

const SPEC_MD: &str = "# 规格：报表页导出 CSV\n\n## 目标\n\n筛选结果可以导出成 CSV 文件。\n\n## 验收标准\n\n- 导出的 CSV 只含当前筛选结果。\n";

/// plan.md 按模板写。「基线」那一行只由 `baseline` 决定：改方案时传同一个字符串=原样抄。
fn plan_md(baseline: &str, revision: &str) -> String {
    format!(
        "# 技术方案：报表页导出 CSV\n\n\
         ## 现状\n\n报表模块现在只输出 JSON。\n\n\
         ## 改法\n\n- src/export.py：导出入口（T01）\n- src/encode.py：CSV 编码（T02）\n\n\
         ## 门禁\n\n每个任务做完都要全部通过。命令在项目根目录运行。\n\n```bash\nsh gate.sh\n```\n\n\
         ## 基线\n\n原始基线（Work 开始时的 `git rev-parse HEAD`；改方案从上一版原样抄这一行，不重设）：`{baseline}`\n\n\
         ## 风险\n\n- 编码细节可能返工。\n\n\
         ## 修订记录\n\n{revision}"
    )
}

fn tasks_md(items: &[(&str, &str, &str)]) -> String {
    let mut s = String::from("# 任务清单\n\n");
    for (id, files, tests) in items {
        s.push_str(&format!(
            "## {id}\n\n- 只改哪些文件：{files}\n- 要变绿的测试：{tests}\n- 做完能观察到什么：对应测试变绿\n\n"
        ));
    }
    s
}

/// plan-review 的 decision.md：批准记录绑定获批版本，可附条件。
fn approval_md(spec_sha: &str, plan_sha: &str, conditions: &str) -> String {
    format!("通过\n批准的规格: {spec_sha}\n批准的方案: {plan_sha}\n\n{conditions}\n")
}

/// escalate 的 decision.md：四个词之一，加人的意见。
fn escalation_md(first: &str, opinions: &str) -> String {
    format!("{first}\n{opinions}\n")
}

/// implement 的 change.md（第一行 `完成 Tnn` / `卡住 Tnn`）。
fn change_done(
    task: &str,
    baseline: &str,
    commit: &str,
    fix_round: u32,
    files: &[&str],
    note: &str,
) -> String {
    let mut s = format!(
        "完成 {task}\n基线: {baseline}\n提交: {commit}\n修复轮次: {fix_round}\n启用的测试: 1\n改动文件:\n"
    );
    for f in files {
        s.push_str(&format!("- {f}\n"));
    }
    s.push_str("门禁:\n- sh gate.sh: 通过\n");
    s.push_str(&format!("备注: {note}\n"));
    s
}

fn change_stuck(task: &str, note: &str) -> String {
    format!(
        "卡住 {task}\n基线: 无\n提交: 无\n修复轮次: 0\n启用的测试: 0\n改动文件:\n门禁:\n备注: {note}\n"
    )
}

/// fix 的 change.md（第一行 `修复完成` / `卡住`）。
fn fix_done(task: &str, baseline: &str, commit: &str, fix_round: u32, note: &str) -> String {
    format!(
        "修复完成\n针对: 验证报告\n任务: {task}\n基线: {baseline}\n提交: {commit}\n修复轮次: {fix_round}\n改动文件:\n门禁:\n- sh gate.sh: 通过\n备注: {note}\n"
    )
}

/// verify / review 的 report.md：第一行定路线，三行元数据原样抄 change。
fn verify_report(first: &str, task: &str, fix_round: u32, baseline: &str, checks: &str) -> String {
    format!(
        "{first}\n任务: {task}\n修复轮次: {fix_round}\n基线: {baseline}\n\n## 检查项\n\n{checks}\n"
    )
}

// ── 真实 CLI 走法 ────────────────────────────────────────────────────────

fn spec_dev_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../workbooks/spec-dev")
        .canonicalize()
        .unwrap()
}

fn start_spec_dev(env: &Env, proj: &Proj) -> String {
    env.ok(&["workbook", "add", spec_dev_dir().to_str().unwrap()]);
    env.start(
        "spec-dev",
        &[
            ("request", "报表页加导出 CSV"),
            ("project", proj.root().to_str().unwrap()),
        ],
    )
}

/// 按名写声明输出并提交；没显式给内容的输出写占位文本。
fn submit_outputs(
    env: &Env,
    wid: &str,
    begun: &Value,
    outputs: &[(&str, &str)],
    summary: &str,
) -> Value {
    let attempt = begun["data"]["attempt"].as_str().unwrap().to_string();
    let declared = begun["data"]["outputs"].as_object().unwrap();
    for (output, input) in [("reviewed-plan", "plan"), ("reviewed-tasks", "tasks")] {
        if let Some(destination) = declared.get(output).and_then(Value::as_str) {
            let destination = Path::new(destination);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(input_path(begun, input), destination).unwrap();
        }
    }
    for (name, content) in outputs {
        let path = declared
            .get(*name)
            .unwrap_or_else(|| panic!("{attempt} 没声明输出 {name}"))
            .as_str()
            .unwrap();
        let p = Path::new(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
    env.ok(&[
        "attempt",
        "submit",
        wid,
        "--attempt",
        &attempt,
        "--summary",
        summary,
    ])
}

/// 任务书输入表里绑定到的文件。
fn input_path(begun: &Value, name: &str) -> PathBuf {
    let v = begun["data"]["inputs"]
        .as_object()
        .unwrap()
        .get(name)
        .unwrap_or_else(|| panic!("没有输入 {name}"));
    assert!(!v.is_null(), "输入 {name} 未绑定");
    PathBuf::from(v.as_str().unwrap())
}

fn brief_text(begun: &Value) -> String {
    read(Path::new(begun["data"]["brief_path"].as_str().unwrap()))
}

/// 任务书输出表里某输出的目标路径（提交后从磁盘读回）。
fn output_file(begun: &Value, name: &str) -> PathBuf {
    PathBuf::from(
        begun["data"]["outputs"]
            .as_object()
            .unwrap()
            .get(name)
            .unwrap()
            .as_str()
            .unwrap(),
    )
}

/// 规划三步：spec → plan（「基线」记原始基线）→ plan-review 通过。
/// 返回 (批准记录文本)。
fn plan_and_approve(env: &Env, wid: &str, proj: &Proj, baseline: &str, conditions: &str) -> String {
    plan_and_approve_with_hash(env, wid, proj, baseline, conditions, None)
}

fn plan_and_approve_with_hash(
    env: &Env,
    wid: &str,
    proj: &Proj,
    baseline: &str,
    conditions: &str,
    plan_hash: Option<&str>,
) -> String {
    // 写方案时项目还没有任何提交：「基线」记的就是 Work 开始时的 HEAD。
    assert_eq!(proj.head(), baseline);
    let s0 = env.begin(wid, "spec");
    submit_outputs(env, wid, &s0, &[("spec", SPEC_MD)], "规格写好");
    let b1 = env.begin(wid, "plan");
    assert_eq!(b1["data"]["attempt"], "plan#1.0");
    let plan = format!(
        "{}\n继承来源: 无\n\n{}",
        plan_md(baseline, "- 2026-09-28 初版\n"),
        verified_table(&[])
    );
    let tasks = tasks_md(&[
        (
            "T01 导出入口",
            "src/export.py、tests/test_export.py",
            "test_export",
        ),
        (
            "T02 编码处理",
            "src/encode.py、tests/test_encode.py",
            "test_encode",
        ),
    ]);
    let tasks = format!(
        "{tasks}\n原始基线: {baseline}\n继承来源: 无\n\n{}",
        verified_table(&[])
    );
    submit_outputs(
        env,
        wid,
        &b1,
        &[("plan", &plan), ("tasks", &tasks)],
        "方案写好",
    );
    let pr = env.begin(wid, "plan-review");
    let decision = approval_md(
        &sha256_file(&input_path(&pr, "spec")),
        &plan_hash
            .map(str::to_string)
            .unwrap_or_else(|| sha256_file(&input_path(&pr, "plan"))),
        conditions,
    );
    submit_outputs(env, wid, &pr, &[("decision", &decision)], "批准");
    decision
}

/// 搭骨架：核批准版本 → 写占位文件 → 骨架提交 → 交 scaffold.md。
fn do_scaffold(env: &Env, wid: &str, proj: &Proj, files: &[(&str, &str)]) -> Value {
    let b = env.begin(wid, "scaffold");
    let approval_path = b["data"]["inputs"]["escalation"]
        .as_str()
        .map(PathBuf::from)
        .filter(|path| read(path).contains("更正原审批:"))
        .unwrap_or_else(|| input_path(&b, "decision"));
    assert_eq!(
        check_approval(
            &read(&approval_path),
            &input_path(&b, "spec"),
            &input_path(&b, "plan")
        ),
        Ok(())
    );
    for (path, content) in files {
        std::fs::write(proj.root().join(path), content).unwrap();
    }
    proj.gate();
    let head = proj.commit("feat: 骨架\n\nTask: scaffold\nAgent: sim");
    let md = format!("完成\n骨架提交: {head}\n单任务测试命令: pytest tests/ -k <任务>\n");
    submit_outputs(env, wid, &b, &[("scaffold", &md)], "骨架完成")
}

/// 两个任务各占各的文件的骨架（G1、G4a 用）。
fn scaffold_two_files(env: &Env, wid: &str, proj: &Proj) -> Value {
    do_scaffold(
        env,
        wid,
        proj,
        &[
            ("src/export.py", EXPORT_PY_STUB),
            ("tests/test_export.py", TEST_EXPORT_PY_SKIPPED),
            ("src/encode.py", ENCODE_PY_STUB),
            ("tests/test_encode.py", TEST_ENCODE_PY_SKIPPED),
        ],
    )
}

const EXPORT_PY_STUB: &str = "def export(rows):\n    raise NotImplementedError  # T01\n";
const EXPORT_PY_DONE: &str = "def export(rows):\n    return \",\".join(rows)\n";
const ENCODE_PY_STUB: &str = "def encode(rows):\n    raise NotImplementedError  # T02\n";
const TEST_EXPORT_PY_SKIPPED: &str = "import pytest\n\nfrom src.export import export\n\n\n@pytest.mark.skip(reason=\"T01\")\ndef test_export():\n    assert export([\"a\"]) == \"a\"\n";
const TEST_ENCODE_PY_SKIPPED: &str = "import pytest\n\nfrom src.encode import encode\n\n\n@pytest.mark.skip(reason=\"T02\")\ndef test_encode():\n    assert encode([\"a\"]) == \"a\"\n";

/// verify.md 对干净完成的 T01 的核对（手写 oracle）。
fn assert_t01_clean(proj: &Proj, base: &str, commit: &str, v: &Value) {
    let diff = proj.diff_names(base, commit);
    assert_eq!(diff, ["src/export.py", "tests/test_export.py"]);
    assert!(out_of_scope(&diff, &["src/export.py", "tests/test_export.py"]).is_empty());
    // 测试文件只有删禁用标记这一行。
    assert_eq!(
        proj.diff_lines(base, commit, "tests/test_export.py"),
        ["-@pytest.mark.skip(reason=\"T01\")"]
    );
    // 占位体：本任务编号清零，没标编号的不许有。
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/export.py")), "T01");
    assert!(own_left.is_empty(), "本任务占位体残留：{own_left:?}");
    assert!(unmarked.is_empty(), "占位体没标任务编号：{unmarked:?}");
    // 批准版本对得上（decision 是 plan-review 绑定的那一版的记录）。
    assert_eq!(
        check_approval(
            &read(&input_path(v, "decision")),
            &input_path(v, "spec"),
            &input_path(v, "plan"),
        ),
        Ok(())
    );
}

// ── 五组闭环 ─────────────────────────────────────────────────────────────

// Task: C002-T12
#[test]
fn two_task_loops_scope_each_task_diff_to_its_own_files() {
    let env = Env::new();
    let proj = Proj::init(&env, "g1");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(
        &env,
        &wid,
        &proj,
        &baseline0,
        "条件：导出文件名固定 export.csv。",
    );
    let sc = scaffold_two_files(&env, &wid, &proj);
    let scaffold_commit = proj.head();

    // T01：只改自己的两个文件。
    let im1 = env.follow_begin(&sc, "implement");
    assert_eq!(im1["data"]["attempt"], "implement#1.0");
    assert!(brief_text(&im1).contains("来自: scaffold#1（main 边）"));
    assert!(
        input_path(&im1, "decision")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    let t01_base = proj.head();
    std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_DONE).unwrap();
    std::fs::write(
        proj.root().join("tests/test_export.py"),
        unskip(TEST_EXPORT_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t01_commit = proj.commit("feat(export): 导出入口\n\nTask: T01\nAgent: sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "完成 T01");

    let v1 = env.follow_begin(&s1, "verify");
    assert_eq!(v1["data"]["attempt"], "verify#1.0");
    assert!(brief_text(&v1).contains("来自: implement#1（main 边）"));
    assert!(
        input_path(&v1, "change")
            .to_str()
            .unwrap()
            .ends_with("/attempts/implement/occurrence-001/attempt-000/outputs/change.md")
    );
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report(
        "通过，下一任务 T02",
        "T01",
        0,
        &t01_base,
        "- 批准版本: 对得上\n- 改动范围: 全在白名单\n- 占位体: 本任务无残留\n- 条件: 导出文件名固定 export.csv，成立",
    );
    let s2 = submit_outputs(&env, &wid, &v1, &[("report", &r1)], "通过，下一任务 T02");

    // T02：这次忘了填占位体，走一轮修复；范围始终只算 T02 自己的文件。
    let im2 = env.follow_begin(&s2, "implement");
    assert_eq!(im2["data"]["attempt"], "implement#2.0");
    assert!(brief_text(&im2).contains("来自: verify#1（main 边）"));
    // 上一轮验证报告按路径绑给 implement（意见正文不进任务书）。
    let report_in = input_path(&im2, "report");
    assert!(
        report_in
            .to_str()
            .unwrap()
            .ends_with("/attempts/verify/occurrence-001/attempt-000/outputs/report.md")
    );
    assert!(brief_text(&im2).contains(&format!("| report | {} | ", report_in.display())));
    let t02_base = proj.head();
    assert_eq!(t02_base, t01_commit);
    std::fs::write(
        proj.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t02_commit = proj.commit("feat(encode): 编码处理\n\nTask: T02\nAgent: sim");
    let change2 = change_done(
        "T02",
        &t02_base,
        &t02_commit,
        0,
        &["src/encode.py", "tests/test_encode.py"],
        "占位体还没填。",
    );
    let s3 = submit_outputs(&env, &wid, &im2, &[("change", &change2)], "完成 T02");

    let v2 = env.follow_begin(&s3, "verify");
    let diff2 = proj.diff_names(&t02_base, &t02_commit);
    assert_eq!(diff2, ["tests/test_encode.py"]);
    let (own_left, _) = placeholder_left(&read(&proj.root().join("src/encode.py")), "T02");
    assert_eq!(own_left, ["    raise NotImplementedError  # T02"]);
    let r2 = verify_report(
        "不通过",
        "T02",
        0,
        &t02_base,
        "- 改动范围: 全在白名单\n- 占位体: 本任务还有 1 个没填",
    );
    let s4 = submit_outputs(&env, &wid, &v2, &[("report", &r2)], "不通过");

    let fx = env.follow_begin(&s4, "fix");
    assert!(brief_text(&fx).contains("来自: verify#2（branch 边）"));
    std::fs::write(
        proj.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    proj.gate();
    let fix_commit = proj.commit("fix(encode): 补上实现\n\nTask: T02\nAgent: sim");
    let fix_md = fix_done("T02", &t02_base, &fix_commit, 1, "占位体已填。");
    let s5 = submit_outputs(&env, &wid, &fx, &[("change", &fix_md)], "修复完成");

    // 修复轮次并进同一个任务基线的范围：基线..修复提交 = 两个文件，没有 T01 的。
    let v3 = env.follow_begin(&s5, "verify");
    assert_eq!(v3["data"]["attempt"], "verify#3.0");
    assert!(brief_text(&v3).contains("来自: fix#1（re_review 边）"));
    assert!(
        input_path(&v3, "fix_change")
            .to_str()
            .unwrap()
            .ends_with("/attempts/fix/occurrence-001/attempt-000/outputs/change.md")
    );
    let fix_change_text = read(&input_path(&v3, "fix_change"));
    assert_eq!(field(&fix_change_text, "修复轮次"), "1");
    assert_eq!(field(&fix_change_text, "基线"), t02_base);
    assert_eq!(field(&fix_change_text, "提交"), fix_commit);
    let diff3 = proj.diff_names(&t02_base, &fix_commit);
    assert_eq!(diff3, ["src/encode.py", "tests/test_encode.py"]);
    assert!(out_of_scope(&diff3, &["src/encode.py", "tests/test_encode.py"]).is_empty());
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/encode.py")), "T02");
    assert!(own_left.is_empty() && unmarked.is_empty());

    // O09 的失败模式：骨架以来的累计范围会把 T01 的文件算成 T02 的越界。
    let cumulative = proj.diff_names(&scaffold_commit, &fix_commit);
    assert_eq!(
        cumulative,
        [
            "src/encode.py",
            "src/export.py",
            "tests/test_encode.py",
            "tests/test_export.py"
        ]
    );
    assert!(!diff3.iter().any(|f| f == "src/export.py"));

    let r3 = verify_report(
        "通过，全部完成",
        "T02",
        1,
        &t02_base,
        "- 批准版本: 对得上\n- 改动范围: 含修复轮次共两个文件，全在白名单\n- 占位体: 本任务无残留",
    );
    let s6 = submit_outputs(&env, &wid, &v3, &[("report", &r3)], "通过，全部完成");
    // 全部任务完成后，下一步是整体审查（G5 在那里核原始基线）。
    assert!(next_begin_nodes(&s6).contains(&"review".to_string()));
}

// Task: C002-T12
#[test]
fn out_of_whitelist_file_in_second_task_is_flagged() {
    let env = Env::new();
    let proj = Proj::init(&env, "g1n");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "条件：无。");
    let sc = scaffold_two_files(&env, &wid, &proj);

    // T01 干净完成（与正例同一路径）。
    let im1 = env.follow_begin(&sc, "implement");
    let t01_base = proj.head();
    std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_DONE).unwrap();
    std::fs::write(
        proj.root().join("tests/test_export.py"),
        unskip(TEST_EXPORT_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t01_commit = proj.commit("feat(export): 导出入口\n\nTask: T01\nAgent: sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "完成 T01");
    let v1 = env.follow_begin(&s1, "verify");
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report("通过，下一任务 T02", "T01", 0, &t01_base, "- 全部对得上");
    let s2 = submit_outputs(&env, &wid, &v1, &[("report", &r1)], "通过，下一任务 T02");

    // T02 单条件反例：多改了一个白名单外的文件（T01 的 export.py）。
    let im2 = env.follow_begin(&s2, "implement");
    let t02_base = proj.head();
    std::fs::write(
        proj.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    std::fs::write(
        proj.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    std::fs::write(
        proj.root().join("src/export.py"),
        "def export(rows):\n    return \";\".join(rows)\n",
    )
    .unwrap();
    proj.gate();
    let t02_commit = proj.commit("feat(encode): 编码处理\n\nTask: T02\nAgent: sim");
    let change2 = change_done(
        "T02",
        &t02_base,
        &t02_commit,
        0,
        &["src/encode.py", "tests/test_encode.py"],
        "",
    );
    submit_outputs(&env, &wid, &im2, &[("change", &change2)], "完成 T02");

    // 验证者只看基线..提交：越界文件被抓出来。
    let v2 = env.begin(&wid, "verify");
    let diff = proj.diff_names(&t02_base, &t02_commit);
    assert_eq!(
        diff,
        ["src/encode.py", "src/export.py", "tests/test_encode.py"]
    );
    assert_eq!(
        out_of_scope(&diff, &["src/encode.py", "tests/test_encode.py"]),
        ["src/export.py"]
    );
    let r2 = verify_report(
        "不通过",
        "T02",
        0,
        &t02_base,
        "- 改了不该改的文件：src/export.py",
    );
    let s3 = submit_outputs(&env, &wid, &v2, &[("report", &r2)], "不通过");
    // 报告原文能让修复者不问就动手。
    let report_text = read(&output_file(&v2, "report"));
    assert_eq!(report_text.lines().next().unwrap(), "不通过");
    assert!(report_text.contains("改了不该改的文件：src/export.py"));
    assert_eq!(field(&report_text, "基线"), t02_base);
    // 不通过走修复。
    assert!(next_begin_nodes(&s3).contains(&"fix".to_string()));
}

// Task: C002-T12
#[test]
fn shared_file_keeps_future_task_placeholders() {
    let env = Env::new();
    let proj = Proj::init(&env, "g2");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "条件：无。");

    // 骨架：一个共用源码文件里两个任务各一个占位体，测试文件各归各的任务。
    let sc = do_scaffold(
        &env,
        &wid,
        &proj,
        &[
            (
                "src/feature.py",
                "def step_one():\n    raise NotImplementedError  # T01\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
            ),
            (
                "tests/test_step_one.py",
                "import pytest\n\nfrom src.feature import step_one\n\n\n@pytest.mark.skip(reason=\"T01\")\ndef test_step_one():\n    assert step_one() == 1\n",
            ),
            (
                "tests/test_step_two.py",
                "import pytest\n\nfrom src.feature import step_two\n\n\n@pytest.mark.skip(reason=\"T02\")\ndef test_step_two():\n    assert step_two() == 2\n",
            ),
        ],
    );

    // T01 只填自己的占位体，T02 的原样留着。
    let im = env.follow_begin(&sc, "implement");
    // 本组同样核 brief 输入绑定：批准记录按路径绑给 implement。
    let decision = input_path(&im, "decision");
    assert!(
        decision
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(brief_text(&im).contains(&format!("| decision | {} | ", decision.display())));
    let base = proj.head();
    std::fs::write(
        proj.root().join("src/feature.py"),
        "def step_one():\n    return 1\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
    )
    .unwrap();
    std::fs::write(
        proj.root().join("tests/test_step_one.py"),
        unskip(
            "import pytest\n\nfrom src.feature import step_one\n\n\n@pytest.mark.skip(reason=\"T01\")\ndef test_step_one():\n    assert step_one() == 1\n",
        ),
    )
    .unwrap();
    proj.gate();
    let commit = proj.commit("feat(feature): 步骤一\n\nTask: T01\nAgent: sim");
    let change = change_done(
        "T01",
        &base,
        &commit,
        0,
        &["src/feature.py", "tests/test_step_one.py"],
        "",
    );
    let s = submit_outputs(&env, &wid, &im, &[("change", &change)], "完成 T01");

    // 验证只查本任务的占位体：未来任务的占位合法留存，不判不通过。
    let v = env.follow_begin(&s, "verify");
    let change_in = input_path(&v, "change");
    assert!(
        change_in
            .to_str()
            .unwrap()
            .ends_with("/attempts/implement/occurrence-001/attempt-000/outputs/change.md")
    );
    assert!(brief_text(&v).contains(&format!("| change | {} | ", change_in.display())));
    let content = read(&proj.root().join("src/feature.py"));
    let (own_left, unmarked) = placeholder_left(&content, "T01");
    assert!(own_left.is_empty(), "本任务占位体残留：{own_left:?}");
    assert!(unmarked.is_empty());
    assert!(
        content.contains("raise NotImplementedError  # T02"),
        "未来任务的占位体必须留着：{content}"
    );
    let diff = proj.diff_names(&base, &commit);
    assert_eq!(diff, ["src/feature.py", "tests/test_step_one.py"]);
    let r = verify_report(
        "通过，下一任务 T02",
        "T01",
        0,
        &base,
        "- 改动范围: 全在白名单\n- 占位体: 本任务无残留，T02 的占位按骨架规则留存",
    );
    submit_outputs(&env, &wid, &v, &[("report", &r)], "通过，下一任务 T02");
}

// Task: C002-T12
#[test]
fn own_task_placeholder_left_behind_is_flagged() {
    let env = Env::new();
    let proj = Proj::init(&env, "g2n");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "条件：无。");
    let sc = do_scaffold(
        &env,
        &wid,
        &proj,
        &[(
            "src/feature.py",
            "def step_one():\n    raise NotImplementedError  # T01\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
        )],
    );

    // 单条件反例：把 T02 的占位体填了，本任务的还留着。
    let im = env.follow_begin(&sc, "implement");
    let decision = input_path(&im, "decision");
    assert!(
        decision
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    let base = proj.head();
    std::fs::write(
        proj.root().join("src/feature.py"),
        "def step_one():\n    raise NotImplementedError  # T01\n\n\ndef step_two():\n    return 2\n",
    )
    .unwrap();
    proj.gate();
    let commit = proj.commit("feat(feature): 步骤二\n\nTask: T01\nAgent: sim");
    let change = change_done("T01", &base, &commit, 0, &["src/feature.py"], "");
    let s = submit_outputs(&env, &wid, &im, &[("change", &change)], "完成 T01");

    let v = env.follow_begin(&s, "verify");
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/feature.py")), "T01");
    assert_eq!(own_left, ["    raise NotImplementedError  # T01"]);
    assert!(unmarked.is_empty());
    let r = verify_report(
        "不通过",
        "T01",
        0,
        &base,
        "- 占位体没填完：src/feature.py 的 step_one 还是 T01 的占位体",
    );
    submit_outputs(&env, &wid, &v, &[("report", &r)], "不通过");
    assert_eq!(
        read(&output_file(&v, "report")).lines().next().unwrap(),
        "不通过"
    );
}

// Task: C002-T12
#[test]
fn unmarked_placeholder_is_flagged() {
    let env = Env::new();
    let proj = Proj::init(&env, "g2u");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "条件：无。");
    let sc = do_scaffold(
        &env,
        &wid,
        &proj,
        &[(
            "src/feature.py",
            // 与 G2n 同一动作，唯一不同的条件：T01 的占位体没带任务编号。
            "def step_one():\n    raise NotImplementedError\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
        )],
    );

    let im = env.follow_begin(&sc, "implement");
    let base = proj.head();
    std::fs::write(
        proj.root().join("src/feature.py"),
        "def step_one():\n    raise NotImplementedError\n\n\ndef step_two():\n    return 2\n",
    )
    .unwrap();
    proj.gate();
    let commit = proj.commit("feat(feature): 步骤二\n\nTask: T01\nAgent: sim");
    let change = change_done("T01", &base, &commit, 0, &["src/feature.py"], "");
    let s = submit_outputs(&env, &wid, &im, &[("change", &change)], "完成 T01");

    let v = env.follow_begin(&s, "verify");
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/feature.py")), "T01");
    // 没标编号的占位体归不了谁：本任务占位清单是空的，它照样判「不通过」。
    assert!(own_left.is_empty());
    assert_eq!(unmarked, ["    raise NotImplementedError"]);
    let r = verify_report(
        "不通过",
        "T01",
        0,
        &base,
        "- 占位体没标任务编号：src/feature.py 的 step_one 还是占位体",
    );
    submit_outputs(&env, &wid, &v, &[("report", &r)], "不通过");
    let report_text = read(&output_file(&v, "report"));
    assert_eq!(report_text.lines().next().unwrap(), "不通过");
    assert!(report_text.contains("占位体没标任务编号"));
}

// Task: C002-T12
#[test]
fn conditional_approval_binds_decision_into_scaffold_implement_verify() {
    let env = Env::new();
    let proj = Proj::init(&env, "g3");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    let conditions = "条件：导出文件名固定 export.csv。";
    plan_and_approve(&env, &wid, &proj, &baseline0, conditions);

    // scaffold / implement / verify 三个节点都绑同一份批准记录，逐个按绑定文件核摘要。
    let mut last = env.begin(&wid, "scaffold");
    let mut t01_base = String::new();
    for (idx, node) in ["scaffold", "implement", "verify"].iter().enumerate() {
        if idx > 0 {
            last = env.follow_begin(&last, node);
        }
        assert_eq!(last["data"]["node"].as_str().unwrap(), *node);
        let decision = input_path(&last, "decision");
        assert!(
            decision
                .to_str()
                .unwrap()
                .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md"),
            "{node} 绑的 decision：{decision:?}"
        );
        assert!(
            input_path(&last, "plan")
                .to_str()
                .unwrap()
                .ends_with("/attempts/plan/occurrence-001/attempt-000/outputs/plan.md"),
            "{node} 绑的 plan 不是获批的那一版"
        );
        assert!(
            input_path(&last, "spec")
                .to_str()
                .unwrap()
                .ends_with("/attempts/spec/occurrence-001/attempt-000/outputs/spec.md"),
            "{node} 绑的 spec 不是获批的那一版"
        );
        let brief = brief_text(&last);
        assert!(
            brief.contains(&format!("| decision | {} | ", decision.display())),
            "{node}"
        );
        // 批准核对对得上；条件留在批准记录里，验证者逐条当验收。
        let decision_text = read(&decision);
        assert_eq!(
            check_approval(
                &decision_text,
                &input_path(&last, "spec"),
                &input_path(&last, "plan")
            ),
            Ok(())
        );
        assert!(decision_text.contains(conditions));
        // 节点接着要做活并提交，好让下一个节点跟上。
        match *node {
            "scaffold" => {
                std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_STUB).unwrap();
                std::fs::write(
                    proj.root().join("tests/test_export.py"),
                    TEST_EXPORT_PY_SKIPPED,
                )
                .unwrap();
                std::fs::write(proj.root().join("src/encode.py"), ENCODE_PY_STUB).unwrap();
                std::fs::write(
                    proj.root().join("tests/test_encode.py"),
                    TEST_ENCODE_PY_SKIPPED,
                )
                .unwrap();
                proj.gate();
                let head = proj.commit("feat: 骨架\n\nTask: scaffold\nAgent: sim");
                let md =
                    format!("完成\n骨架提交: {head}\n单任务测试命令: pytest tests/ -k <任务>\n");
                last = submit_outputs(&env, &wid, &last, &[("scaffold", &md)], "骨架完成");
            }
            "implement" => {
                t01_base = proj.head();
                std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_DONE).unwrap();
                std::fs::write(
                    proj.root().join("tests/test_export.py"),
                    unskip(TEST_EXPORT_PY_SKIPPED),
                )
                .unwrap();
                proj.gate();
                let commit = proj.commit("feat(export): 导出入口\n\nTask: T01\nAgent: sim");
                let change = change_done(
                    "T01",
                    &t01_base,
                    &commit,
                    0,
                    &["src/export.py", "tests/test_export.py"],
                    "",
                );
                last = submit_outputs(&env, &wid, &last, &[("change", &change)], "完成 T01");
            }
            _ => {
                let report_path = output_file(&last, "report");
                let report = verify_report(
                    "通过，下一任务 T02",
                    "T01",
                    0,
                    &t01_base,
                    &format!("- 批准版本: 对得上\n- 条件作为验收项：{conditions} 成立"),
                );
                last = submit_outputs(
                    &env,
                    &wid,
                    &last,
                    &[("report", &report)],
                    "通过，下一任务 T02",
                );
                // 条件真的进了验证报告（不是只写在批准记录里）。
                assert!(read(&report_path).contains(conditions));
            }
        }
    }
    // 三个节点都核过批准版本；验证第一行把流程交给下一个任务。
    assert!(next_begin_nodes(&last).contains(&"implement".to_string()));
}

// Task: C002-T12
#[test]
fn stale_approval_hash_makes_worker_stop() {
    let env = Env::new();
    let proj = Proj::init(&env, "g3n");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    let s0 = env.begin(&wid, "spec");
    submit_outputs(&env, &wid, &s0, &[("spec", SPEC_MD)], "规格写好");

    // plan#1，随后 plan-review#1 记下它的摘要并要求改方案。
    let b1 = env.begin(&wid, "plan");
    let plan1 = plan_md(&baseline0, "- 2026-09-28 初版\n");
    submit_outputs(
        &env,
        &wid,
        &b1,
        &[
            ("plan", &plan1),
            (
                "tasks",
                &tasks_md(&[
                    (
                        "T01 导出入口",
                        "src/export.py、tests/test_export.py",
                        "test_export",
                    ),
                    (
                        "T02 编码处理",
                        "src/encode.py、tests/test_encode.py",
                        "test_encode",
                    ),
                ]),
            ),
        ],
        "方案写好",
    );
    let pr1 = env.begin(&wid, "plan-review");
    let plan1_sha = sha256_file(&input_path(&pr1, "plan"));
    let spec_sha = sha256_file(&input_path(&pr1, "spec"));
    let d1 = format!("修改方案\n批准的规格: {spec_sha}\n批准的方案: {plan1_sha}\n\n任务拆错了。\n");
    let s1 = submit_outputs(&env, &wid, &pr1, &[("decision", &d1)], "修改方案");

    // plan#2（「基线」原样抄），plan-review#2 的批准记录还写着 plan#1 的摘要——人写错了。
    let b2 = env.follow_begin(&s1, "plan");
    assert_eq!(b2["data"]["attempt"], "plan#2.0");
    let plan2 = plan_md(&baseline0, "- 2026-09-28 初版\n- 2026-09-28 改法重排\n");
    let s2 = submit_outputs(
        &env,
        &wid,
        &b2,
        &[
            ("plan", &plan2),
            (
                "tasks",
                &tasks_md(&[(
                    "T01 编码处理",
                    "src/encode.py、tests/test_encode.py",
                    "test_encode",
                )]),
            ),
        ],
        "方案重排",
    );
    let pr2 = env.follow_begin(&s2, "plan-review");
    assert_eq!(pr2["data"]["attempt"], "plan-review#2.0");
    assert!(
        input_path(&pr2, "plan")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan/occurrence-002/attempt-000/outputs/plan.md")
    );
    let d2 = format!("通过\n批准的规格: {spec_sha}\n批准的方案: {plan1_sha}\n\n条件：无。\n");
    let s3 = submit_outputs(&env, &wid, &pr2, &[("decision", &d2)], "批准");

    // scaffold 开工先核版本：绑的是 plan#2，批准记录是 plan#1 的摘要 → 停下。
    let sb = env.follow_begin(&s3, "scaffold");
    assert_eq!(sb["data"]["attempt"], "scaffold#1.0");
    let decision_text = read(&input_path(&sb, "decision"));
    assert_eq!(field(&decision_text, "批准的方案"), plan1_sha);
    let err = check_approval(
        &decision_text,
        &input_path(&sb, "spec"),
        &input_path(&sb, "plan"),
    )
    .unwrap_err();
    assert_eq!(err.1, sha256_file(&input_path(&sb, "plan")));
    assert_ne!(err.1, plan1_sha, "摘要不同才叫旧批准失效");
    // 只改「批准的方案」一个条件就恢复——差别只有哈希。
    let corrected = approval_md(&spec_sha, &err.1, "条件：无。");
    assert_eq!(
        check_approval(
            &corrected,
            &input_path(&sb, "spec"),
            &input_path(&sb, "plan")
        ),
        Ok(())
    );

    // 失败停止路径：按卡住处理，不按没批过的方案开工，项目一个提交都不加。
    let head_before = proj.head();
    let stuck = format!(
        "卡住\n备注: 批准的规格 {}，批准的方案 {}，绑定 plan 文件的摘要 {}；旧批准已失效。\n",
        spec_sha, plan1_sha, err.1
    );
    let s4 = submit_outputs(&env, &wid, &sb, &[("scaffold", &stuck)], "卡住");
    assert_eq!(proj.head(), head_before, "卡住时不许有骨架提交");
    assert_eq!(head_before, baseline0);
    assert!(next_begin_nodes(&s4).contains(&"escalate".to_string()));
}

// Task: C002-T12
#[test]
fn verify_escalation_continue_binds_opinion_on_return() {
    let env = Env::new();
    let proj = Proj::init(&env, "g4a");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "条件：无。");
    let sc = scaffold_two_files(&env, &wid, &proj);

    let im1 = env.follow_begin(&sc, "implement");
    let t01_base = proj.head();
    std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_DONE).unwrap();
    std::fs::write(
        proj.root().join("tests/test_export.py"),
        unskip(TEST_EXPORT_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t01_commit = proj.commit("feat(export): 导出入口\n\nTask: T01\nAgent: sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "完成 T01");

    // 验证要授权：这不是修复能解决的，报告「不通过，需要人」。
    let v1 = env.follow_begin(&s1, "verify");
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report(
        "不通过，需要人",
        "T01",
        0,
        &t01_base,
        "- 验证要跑 `./gate.sh --with-network`，需要授权",
    );
    let s2 = submit_outputs(&env, &wid, &v1, &[("report", &r1)], "不通过，需要人");
    assert!(next_begin_nodes(&s2).contains(&"escalate".to_string()));

    // 人「继续」并给授权；意见正文只进 decision 文件。
    let es = env.follow_begin(&s2, "escalate");
    let opinion = "允许运行 ./gate.sh --with-network，范围只此一条。";
    let d = escalation_md("继续", opinion);
    let s3 = submit_outputs(&env, &wid, &es, &[("decision", &d)], "继续");
    let verify_op = s3["next"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == "verify")
        .expect("escalate 继续后 next 里应有 verify");
    assert_eq!(verify_op["edge"], "back");

    // 回到 verify：来自 escalate 的 back 边，绑到人的意见。
    let v2 = env.follow_begin(&s3, "verify");
    assert_eq!(v2["data"]["attempt"], "verify#2.0");
    let brief = brief_text(&v2);
    assert!(brief.contains("来自: escalate#1（back 边）"));
    let escalation = input_path(&v2, "escalation");
    assert!(
        escalation
            .to_str()
            .unwrap()
            .ends_with("/attempts/escalate/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(brief.contains(&format!("| escalation | {} | ", escalation.display())));
    // 批准记录照旧绑定并核对；意见正文不进任务书，只在绑定文件里。
    assert!(
        input_path(&v2, "decision")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(!brief.contains(opinion));
    assert_eq!(read(&escalation), d);

    let r2 = verify_report(
        "通过，下一任务 T02",
        "T01",
        0,
        &t01_base,
        "- 按人的授权补跑了带网络的门禁，通过",
    );
    submit_outputs(&env, &wid, &v2, &[("report", &r2)], "通过，下一任务 T02");
}

// Task: C002-T12
#[test]
fn scaffold_escalation_continue_binds_opinion_on_return() {
    let env = Env::new();
    let proj = Proj::init(&env, "g4b");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "条件：无。");

    // 骨架撞上方案与现有代码的冲突，按卡住处理。
    let sb1 = env.begin(&wid, "scaffold");
    assert!(brief_text(&sb1).contains("来自: plan-review#1（main 边）"));
    let stuck = "卡住\n备注: 方案的改法与 src/legacy.py 现有导出接口冲突，要人裁决。\n";
    let s1 = submit_outputs(&env, &wid, &sb1, &[("scaffold", stuck)], "卡住");
    assert_eq!(proj.head(), baseline0, "卡住时不许有骨架提交");
    let escalate_op = s1["next"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == "escalate")
        .expect("scaffold 卡住后 next 里应有 escalate");
    assert_eq!(escalate_op["edge"], "branch");

    // 人「继续」并亲手改了接口。
    let es = env.follow_begin(&s1, "escalate");
    let opinion = "我已亲手改好 src/legacy.py 的导出接口，按新签名搭骨架。";
    let d = escalation_md("继续", opinion);
    let s2 = submit_outputs(&env, &wid, &es, &[("decision", &d)], "继续");
    let scaffold_op = s2["next"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == "scaffold")
        .expect("escalate 继续后 next 里应有 scaffold");
    assert_eq!(scaffold_op["edge"], "back");

    // 回到 scaffold：来自 escalate 的 back 边，绑到人的意见再搭。
    let sb2 = env.follow_begin(&s2, "scaffold");
    assert_eq!(sb2["data"]["attempt"], "scaffold#2.0");
    let brief = brief_text(&sb2);
    assert!(brief.contains("来自: escalate#1（back 边）"));
    let escalation = input_path(&sb2, "escalation");
    assert!(
        escalation
            .to_str()
            .unwrap()
            .ends_with("/attempts/escalate/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(brief.contains(&format!("| escalation | {} | ", escalation.display())));
    assert!(!brief.contains(opinion));
    assert_eq!(read(&escalation), d);

    // 按人的意见搭完并提交。
    std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_STUB).unwrap();
    std::fs::write(
        proj.root().join("tests/test_export.py"),
        TEST_EXPORT_PY_SKIPPED,
    )
    .unwrap();
    proj.gate();
    let head = proj.commit("feat: 骨架\n\nTask: scaffold\nAgent: sim");
    let md = format!("完成\n骨架提交: {head}\n单任务测试命令: pytest tests/ -k <任务>\n");
    submit_outputs(&env, &wid, &sb2, &[("scaffold", &md)], "骨架完成");
}

// Task: C002-T12
#[test]
fn replan_after_first_task_keeps_original_baseline_for_final_review() {
    let env = Env::new();
    let proj = Proj::init(&env, "g5");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "条件：无。");
    let sc = scaffold_two_files(&env, &wid, &proj);

    // 任务 1 完成。
    let im1 = env.follow_begin(&sc, "implement");
    let t01_base = proj.head();
    std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_DONE).unwrap();
    std::fs::write(
        proj.root().join("tests/test_export.py"),
        unskip(TEST_EXPORT_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t01_commit = proj.commit("feat(export): 导出入口\n\nTask: T01\nAgent: sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "完成 T01");
    let v1 = env.follow_begin(&s1, "verify");
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report("通过，下一任务 T02", "T01", 0, &t01_base, "- 全部对得上");
    let s2 = submit_outputs(&env, &wid, &v1, &[("report", &r1)], "通过，下一任务 T02");

    // 任务 2 卡住，人改方案。
    let im2 = env.follow_begin(&s2, "implement");
    let s3 = submit_outputs(
        &env,
        &wid,
        &im2,
        &[(
            "change",
            &change_stuck("T02", "任务拆错了，编码要先加配置项。"),
        )],
        "卡住 T02",
    );
    let es = env.follow_begin(&s3, "escalate");
    let s4 = submit_outputs(
        &env,
        &wid,
        &es,
        &[(
            "decision",
            &escalation_md("改方案", "任务拆错了，重排任务清单。"),
        )],
        "改方案",
    );

    // plan#2：「基线」那一行从上一版原样抄，不重设。
    let b2 = env.follow_begin(&s4, "plan");
    assert_eq!(b2["data"]["attempt"], "plan#2.0");
    let replan_head = proj.head();
    assert_eq!(replan_head, t01_commit);
    let plan2 = plan_md(
        &baseline0,
        "- 2026-09-28 初版\n- 2026-09-28 任务重排，编码保留为 T02\n",
    );
    let s5 = submit_outputs(
        &env,
        &wid,
        &b2,
        &[
            ("plan", &plan2),
            (
                "tasks",
                &tasks_md(&[(
                    "T02 编码处理",
                    "src/encode.py、tests/test_encode.py",
                    "test_encode",
                )]),
            ),
        ],
        "方案重排",
    );
    let pr2 = env.follow_begin(&s5, "plan-review");
    let d2 = approval_md(
        &sha256_file(&input_path(&pr2, "spec")),
        &sha256_file(&input_path(&pr2, "plan")),
        "条件：无。",
    );
    let s6 = submit_outputs(&env, &wid, &pr2, &[("decision", &d2)], "批准");

    // 重排后的骨架与实现：剩下的活是新清单的 T02。
    let sb2 = env.follow_begin(&s6, "scaffold");
    let s7 = submit_outputs(
        &env,
        &wid,
        &sb2,
        &[("scaffold", "完成\n骨架提交: 同一签名，任务编号按新清单\n")],
        "骨架完成",
    );
    let im3 = env.follow_begin(&s7, "implement");
    assert_eq!(im3["data"]["attempt"], "implement#3.0");
    let t02_base = proj.head();
    std::fs::write(
        proj.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    std::fs::write(
        proj.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t02_commit = proj.commit("feat(encode): 编码处理\n\nTask: T02\nAgent: sim");
    let change3 = change_done(
        "T02",
        &t02_base,
        &t02_commit,
        0,
        &["src/encode.py", "tests/test_encode.py"],
        "",
    );
    let s8 = submit_outputs(&env, &wid, &im3, &[("change", &change3)], "完成 T02");
    let v2 = env.follow_begin(&s8, "verify");
    let r2 = verify_report("通过，全部完成", "T02", 0, &t02_base, "- 全部对得上");
    let s9 = submit_outputs(&env, &wid, &v2, &[("report", &r2)], "通过，全部完成");

    // 最终审查绑的是 plan#2，「基线」仍是 Work 开始时的 HEAD。
    let rv = env.follow_begin(&s9, "review");
    let plan_bound = read(&input_path(&rv, "plan"));
    assert!(
        input_path(&rv, "plan")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan/occurrence-002/attempt-000/outputs/plan.md")
    );
    assert_eq!(plan_baseline(&plan_bound), baseline0);
    assert_eq!(
        baseline0,
        proj.git(&["rev-list", "--max-parents=0", "HEAD"])
    );

    // 正例：整体范围从原始基线算，改方案前完成的任务 1 还在里面。
    let whole = proj.diff_names(&baseline0, &proj.head());
    assert_eq!(
        whole,
        [
            "src/encode.py",
            "src/export.py",
            "tests/test_encode.py",
            "tests/test_export.py"
        ]
    );
    assert!(whole.iter().any(|f| f == "src/export.py"));

    // 单条件反例：改方案时若把「基线」重设成 plan#2 当时的 HEAD，任务 1 就从范围里消失。
    let reset = proj.diff_names(&replan_head, &proj.head());
    assert_eq!(reset, ["src/encode.py", "tests/test_encode.py"]);
    assert!(!reset.iter().any(|f| f == "src/export.py"));
}
