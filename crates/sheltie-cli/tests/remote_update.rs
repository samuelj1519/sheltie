//! C002-T31：HTTP来源的URL固定与失败保全；只替换curl传输边界，不联网。
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::Env;
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BASE: &str = "https://fixture.invalid/releases";
const PAYLOAD: &[u8] = b"remote pinned 9.9.9 payload\n";
fn expected_platform() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "aarch64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "x86_64-unknown-linux-gnu"
    }
}
const CURL: &str = r#"#!/usr/bin/env python3
import json,sys,os
from pathlib import Path
cfg=json.loads(Path(os.environ['FAKE_CURL_CONFIG']).read_text())
url=sys.argv[-1]
log=Path(os.environ['FAKE_CURL_LOG'])
previous=log.read_text().splitlines() if log.exists() else []
with log.open('a') as f:f.write(url+'\n')
if cfg['mode']=='manifest_failure' and url.endswith('dist-manifest.json') or cfg['mode']=='asset_failure' and not url.endswith('dist-manifest.json'):
 print('controlled curl failure',file=sys.stderr);sys.exit(22)
if cfg['mode'] in ['stderr_exact','stderr_over']:sys.stderr.buffer.write(b'x'*(1024*1024+(cfg['mode']=='stderr_over')));sys.stderr.buffer.flush()
platform=cfg['platform']
version='9.9.9'
if url.endswith('/latest/download/dist-manifest.json'):
 if any('/latest/download/' in old for old in previous):version='8.8.8'
elif not url.startswith(cfg['base']+'/download/v9.9.9/'):
 print('unexpected URL '+url,file=sys.stderr);sys.exit(22)
name='sheltie-'+version+'-'+platform
if url.endswith('dist-manifest.json'):
 digest='dec2fc95c56b937d66a20cf64c623531d22fdd4def9a52dfadd4ebec7a60a694' if version=='9.9.9' else 'e86b13466a0a635fbea915871a23ec4e888579f8aa1f233b6526fa0d8181ca73'
 data=json.dumps({'version':version,'assets':[{'platform':platform,'name':name,'sha256':digest}]}).encode()
 if cfg['mode'] in ['stdout_exact','stdout_over']:data+=b' '*(32*1024*1024+(cfg['mode']=='stdout_over')-len(data))
 sys.stdout.buffer.write(data)
elif url==cfg['base']+'/download/v9.9.9/'+name:
 sys.stdout.buffer.write(b'remote pinned 9.9.9 payload\n')
else:
 print('unexpected asset '+url,file=sys.stderr);sys.exit(22)
"#;
struct Transport {
    _dir: tempfile::TempDir,
    tool: PathBuf,
    config: PathBuf,
    log: PathBuf,
}
impl Transport {
    fn new(mode: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let tool = dir.path().join("tools");
        std::fs::create_dir(&tool).unwrap();
        let executable = tool.join("curl");
        std::fs::write(&executable, CURL).unwrap();
        std::fs::set_permissions(executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let config = dir.path().join("config.json");
        std::fs::write(
            &config,
            json!({"mode":mode,"base":BASE,"platform":expected_platform()}).to_string(),
        )
        .unwrap();
        let log = dir.path().join("requests.txt");
        Self {
            _dir: dir,
            tool,
            config,
            log,
        }
    }
    fn update(&self, env: &Env, version: Option<&str>) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_sheltie"));
        cmd.args(["--home", &env.home(), "--json", "self", "update"]);
        if let Some(version) = version {
            cmd.args(["--version", version]);
        }
        cmd.env("SHELTIE_RELEASE_BASE", BASE)
            .env("FAKE_CURL_CONFIG", &self.config)
            .env("FAKE_CURL_LOG", &self.log)
            .env(
                "PATH",
                format!("{}:{}", self.tool.display(), std::env::var("PATH").unwrap()),
            );
        cmd.output().unwrap()
    }
    fn requests(&self) -> Vec<String> {
        std::fs::read_to_string(&self.log)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}
fn read(home: &Path, name: &str) -> Vec<u8> {
    std::fs::read(home.join(name)).unwrap()
}

// Task: C002-T31
#[test]
fn remote_update_pins_latest_and_explicit_tags_and_accepts_exact_stderr_limit() {
    for (mode, version) in [
        ("normal", None),
        ("normal", Some("9.9.9")),
        ("stderr_exact", None),
    ] {
        let env = Env::new();
        env.ok(&["self", "install"]);
        let home = env.dir.path();
        let old = read(home, "bin/sheltie");
        let store = read(home, "store.db");
        let transport = Transport::new(mode);
        let out = transport.update(&env, version);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        let response: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(response["data"]["to"], "9.9.9");
        let mut expected = Vec::new();
        if version.is_none() {
            expected.push(format!("{BASE}/latest/download/dist-manifest.json"));
        }
        expected.push(format!("{BASE}/download/v9.9.9/dist-manifest.json"));
        expected.push(format!(
            "{BASE}/download/v9.9.9/sheltie-9.9.9-{}",
            expected_platform()
        ));
        assert_eq!(transport.requests(), expected);
        assert_eq!(read(home, "bin/sheltie"), PAYLOAD);
        assert_eq!(read(home, "bin/sheltie.prev"), old);
        assert_eq!(read(home, "store.db"), store);
    }
}

// Task: C002-T31
#[test]
fn failed_remote_manifest_or_asset_preserves_binaries_store_and_cleans_its_tmp() {
    for mode in ["manifest_failure", "asset_failure"] {
        let env = Env::new();
        env.ok(&["self", "install"]);
        let home = env.dir.path();
        std::fs::write(home.join("bin/sheltie.prev"), b"previous independent bytes").unwrap();
        let old = read(home, "bin/sheltie");
        let prev = read(home, "bin/sheltie.prev");
        let store = read(home, "store.db");
        let transport = Transport::new(mode);
        let out = transport.update(&env, None);
        assert!(!out.status.success());
        let response: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(response["error"]["code"], "UPDATE_UNAVAILABLE");
        assert_eq!(read(home, "bin/sheltie"), old);
        assert_eq!(read(home, "bin/sheltie.prev"), prev);
        assert_eq!(read(home, "store.db"), store);
        assert_eq!(std::fs::read_dir(home.join("tmp")).unwrap().count(), 0);
    }
}

// Task: C002-T32
#[test]
fn remote_update_accepts_exact_stream_limits_and_preserves_state_on_one_more_byte() {
    for mode in ["stdout_exact", "stdout_over", "stderr_exact", "stderr_over"] {
        let env = Env::new();
        env.ok(&["self", "install"]);
        let home = env.dir.path();
        std::fs::write(home.join("bin/sheltie.prev"), b"previous independent bytes").unwrap();
        let old = read(home, "bin/sheltie");
        let previous = read(home, "bin/sheltie.prev");
        let store = read(home, "store.db");
        let transport = Transport::new(mode);
        let output = transport.update(&env, Some("9.9.9"));
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        if mode.ends_with("exact") {
            assert!(output.status.success(), "{mode}: {response}");
            assert_eq!(read(home, "bin/sheltie"), PAYLOAD);
            assert_eq!(read(home, "bin/sheltie.prev"), old);
            assert_eq!(
                transport.requests(),
                vec![
                    format!("{BASE}/download/v9.9.9/dist-manifest.json"),
                    format!(
                        "{BASE}/download/v9.9.9/sheltie-9.9.9-{}",
                        expected_platform()
                    )
                ]
            );
        } else {
            assert!(!output.status.success(), "{mode}");
            assert_eq!(response["error"]["code"], "UPDATE_UNAVAILABLE");
            assert_eq!(read(home, "bin/sheltie"), old);
            assert_eq!(read(home, "bin/sheltie.prev"), previous);
            assert_eq!(
                transport.requests(),
                vec![format!("{BASE}/download/v9.9.9/dist-manifest.json")]
            );
        }
        assert_eq!(read(home, "store.db"), store);
        assert_eq!(std::fs::read_dir(home.join("tmp")).unwrap().count(), 0);
    }
}
