//! cli 端到端测试共用：临时管理根、跑命令、解析 JSON 响应封装、按 `next` 走。
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use serde_json::Value;
#[path = "../../../sheltie-runtime/tests/common/owned_tempdir.rs"]
mod owned_tempdir;
pub use owned_tempdir::OwnedTempDir;

#[cfg(feature = "failpoint")]
pub mod process;

pub mod store;

pub struct Env {
    pub dir: OwnedTempDir,
}

impl Env {
    pub fn new() -> Self {
        Self {
            dir: OwnedTempDir::new(),
        }
    }

    pub fn home(&self) -> String {
        self.dir.path().to_str().unwrap().to_string()
    }

    /// 一条 `sheltie --home <tmp> --json ...` 命令。
    pub fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::cargo_bin("sheltie").unwrap();
        c.args(["--home", &self.home(), "--json"]).args(args);
        c
    }

    /// 文本模式。
    pub fn cmd_text(&self, args: &[&str]) -> Command {
        let mut c = Command::cargo_bin("sheltie").unwrap();
        c.args(["--home", &self.home()]).args(args);
        c
    }

    /// 跑命令，要求成功，返回解析后的响应封装。
    pub fn ok(&self, args: &[&str]) -> Value {
        let out = self.cmd(args).output().unwrap();
        assert!(
            out.status.success(),
            "命令失败：{args:?}\nexit: {:?}\nstdout: {}\nstderr: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["ok"], true);
        v
    }

    /// 跑命令，要求失败，返回响应封装与退出码。
    pub fn fail(&self, args: &[&str]) -> (Value, i32) {
        let out = self.cmd(args).output().unwrap();
        assert!(!out.status.success(), "命令意外成功：{args:?}");
        let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        (v, out.status.code().unwrap_or(-1))
    }

    pub fn add_example(&self, name: &str) -> Value {
        self.ok(&["workbook", "add", example_dir(name).to_str().unwrap()])
    }

    /// `work start`，返回 `work_id`。
    pub fn start(&self, workbook: &str, inputs: &[(&str, &str)]) -> String {
        let mut args = vec!["work", "start", "--workbook", workbook, "--flow", "default"];
        let owned: Vec<String> = inputs.iter().map(|(k, v)| format!("{k}={v}")).collect();
        for o in &owned {
            args.push("--input");
            args.push(o);
        }
        let v = self.ok(&args);
        v["data"]["work_id"].as_str().unwrap().to_string()
    }

    pub fn begin(&self, work: &str, node: &str) -> Value {
        self.ok(&["attempt", "begin", work, "--node", node])
    }

    /// 在任务书给的输出目录写出全部声明输出，然后提交。
    pub fn submit_all(&self, work: &str, begun: &Value, summary: &str) -> Value {
        let attempt = begun["data"]["attempt"].as_str().unwrap().to_string();
        for (_, path) in begun["data"]["outputs"].as_object().unwrap() {
            let p = Path::new(path.as_str().unwrap());
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, format!("output for {attempt}\n")).unwrap();
        }
        self.ok(&[
            "attempt",
            "submit",
            work,
            "--attempt",
            &attempt,
            "--summary",
            summary,
        ])
    }

    pub fn status(&self, work: &str) -> Value {
        self.ok(&["work", "status", work])
    }

    /// 从 `next` 里找一项 `attempt begin <node>` 并按它拼命令跑。证明 `next` 可执行。
    pub fn follow_begin(&self, envelope: &Value, node: &str) -> Value {
        let item = envelope["next"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == node)
            .unwrap_or_else(|| panic!("next 里没有 begin {node}：{}", envelope["next"]));
        let work = item["args"]["work"].as_str().unwrap();
        self.ok(&["attempt", "begin", work, "--node", node])
    }

    pub fn work_dir(&self, work: &str) -> PathBuf {
        self.dir.path().join("works").join(work)
    }

    pub fn workbook_dir(&self, id: &str, version: &str) -> PathBuf {
        self.dir.path().join("workbooks").join(id).join(version)
    }
}

pub fn example_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
        .canonicalize()
        .unwrap()
}

pub fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let target = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

pub fn make_writable(p: &Path) {
    std::fs::set_permissions(p, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
}

pub fn next_ops(v: &Value) -> Vec<String> {
    v["next"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["op"].as_str().unwrap().to_string())
        .collect()
}

pub fn next_begin_nodes(v: &Value) -> Vec<String> {
    v["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["op"] == "attempt begin")
        .map(|n| n["args"]["node"].as_str().unwrap().to_string())
        .collect()
}

pub fn copy_example(name: &str, into: &Path) -> PathBuf {
    let target = into.join(name);
    copy_dir(&example_dir(name), &target);
    target
}
