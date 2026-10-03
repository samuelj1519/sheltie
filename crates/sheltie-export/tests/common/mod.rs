#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub fn binaries() -> &'static (PathBuf, PathBuf) {
    static BINS: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();
    BINS.get_or_init(|| {
        if let (Ok(engine), Ok(exporter)) = (
            std::env::var("SHELTIE_TEST_ENGINE_BINARY"),
            std::env::var("SHELTIE_TEST_EXPORT_BINARY"),
        ) {
            let pair = (PathBuf::from(engine), PathBuf::from(exporter));
            assert!(pair.0.is_absolute() && pair.0.is_file());
            assert!(pair.1.is_absolute() && pair.1.is_file());
            return pair;
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        let out = Command::new(env!("CARGO"))
            .current_dir(root)
            .env("RUSTC_WRAPPER", "")
            .env(
                "CARGO_TARGET_DIR",
                "/private/tmp/sheltie-c006-binary-tests-target",
            )
            .args([
                "build",
                "-p",
                "sheltie-cli",
                "-p",
                "sheltie-export",
                "--all-features",
                "--message-format=json",
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let mut engine = None;
        let mut exporter = None;
        for line in out
            .stdout
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let value: Value = serde_json::from_slice(line).unwrap();
            if value["reason"] != "compiler-artifact" {
                continue;
            }
            let Some(executable) = value["executable"].as_str() else {
                continue;
            };
            match value["target"]["name"].as_str() {
                Some("sheltie") => engine = Some(PathBuf::from(executable)),
                Some("sheltie-export") => exporter = Some(PathBuf::from(executable)),
                _ => (),
            }
        }
        (engine.unwrap(), exporter.unwrap())
    })
}

pub fn engine(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(&binaries().0)
        .args(["--home", home.to_str().unwrap(), "--json"])
        .args(args)
        .output()
        .unwrap()
}

pub fn ok(home: &Path, args: &[&str]) -> Value {
    let out = engine(home, args);
    assert!(
        out.status.success(),
        "{args:?}: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["ok"], true);
    value
}

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub home: PathBuf,
    pub parent: PathBuf,
    pub work: String,
    pub revision: u64,
    pub result: Value,
    pub binary: Vec<u8>,
}

impl Fixture {
    pub fn complete() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let home = base.join("home");
        let parent = base.join("copies");
        let workbook = base.join("workbook");
        std::fs::create_dir(&parent).unwrap();
        std::fs::create_dir(&workbook).unwrap();
        std::fs::create_dir(workbook.join("flows")).unwrap();
        std::fs::write(workbook.join("workbook.toml"), "schema='workbook/v1'\nid='byte-fixture'\nversion='1.0.0'\nname='原字节夹具'\nflows=['flows/default.toml']\n").unwrap();
        std::fs::write(workbook.join("flows/default.toml"), "schema='flow/v1'\nid='default'\nentry='make'\n[[nodes]]\nid='make'\ntitle='成果'\nexecutor='agent'\ninstruction={text='按声明生成成果'}\ninputs=[{name='task',from='start.task',result=true}]\noutputs=[{name='binary',path='final.bin',result=true},{name='empty',path='empty.bin',result=true}]\n").unwrap();
        ok(&home, &["workbook", "add", workbook.to_str().unwrap()]);
        let started = ok(
            &home,
            &[
                "work",
                "start",
                "--workbook",
                "byte-fixture",
                "--flow",
                "default",
                "--input",
                "task=原目标",
            ],
        );
        let work = started["data"]["work_id"].as_str().unwrap().to_string();
        let begun = ok(&home, &["attempt", "begin", &work, "--node", "make"]);
        let binary = vec![0, 255, 128, b'a', b'\n', 0, b'z'];
        std::fs::write(
            begun["data"]["outputs"]["binary"].as_str().unwrap(),
            &binary,
        )
        .unwrap();
        std::fs::write(begun["data"]["outputs"]["empty"].as_str().unwrap(), []).unwrap();
        ok(
            &home,
            &[
                "attempt",
                "submit",
                &work,
                "--attempt",
                "make#1.0",
                "--summary",
                "产物完成",
            ],
        );
        let result = ok(&home, &["work", "result", &work])["data"].clone();
        let revision = result["revision"].as_u64().unwrap();
        Self {
            dir,
            home,
            parent,
            work,
            revision,
            result,
            binary,
        }
    }

    pub fn export(&self) -> std::process::Output {
        Command::new(&binaries().1)
            .args([
                "--sheltie",
                binaries().0.to_str().unwrap(),
                "--home",
                self.home.to_str().unwrap(),
                "--work",
                &self.work,
                "--to",
                self.parent.to_str().unwrap(),
                "--json",
            ])
            .output()
            .unwrap()
    }
}
