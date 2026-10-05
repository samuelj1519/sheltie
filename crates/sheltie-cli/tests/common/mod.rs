//! Shared CLI end-to-end support: temporary roots, commands, JSON envelopes, and next-action execution.
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

    /// One sheltie --home <tmp> --json ... command.
    pub fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::cargo_bin("sheltie").unwrap();
        c.args(["--home", &self.home(), "--json"]).args(args);
        c
    }

    /// Text mode.
    pub fn cmd_text(&self, args: &[&str]) -> Command {
        let mut c = Command::cargo_bin("sheltie").unwrap();
        c.args(["--home", &self.home()]).args(args);
        c
    }

    /// Run a command, require success, and decode its response envelope.
    pub fn ok(&self, args: &[&str]) -> Value {
        let out = self.cmd(args).output().unwrap();
        assert!(
            out.status.success(),
            "Command failed: {args:?}\nexit: {:?}\nstdout: {}\nstderr: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["ok"], true);
        v
    }

    /// Run a command, require failure, and return envelope/exit code.
    pub fn fail(&self, args: &[&str]) -> (Value, i32) {
        let out = self.cmd(args).output().unwrap();
        assert!(
            !out.status.success(),
            "Command unexpectedly succeeded: {args:?}"
        );
        let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        (v, out.status.code().unwrap_or(-1))
    }

    pub fn add_example(&self, name: &str) -> Value {
        self.ok(&["workbook", "add", example_dir(name).to_str().unwrap()])
    }

    /// work start, returning work_id.
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

    /// Write every declared output in the brief's output directory, then submit.
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

    /// Find and run next's attempt begin <node> command, proving next actions are executable.
    pub fn follow_begin(&self, envelope: &Value, node: &str) -> Value {
        let item = envelope["next"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == node)
            .unwrap_or_else(|| panic!("next has no begin {node}: {}", envelope["next"]));
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

/// Remove a frozen lookup while retaining the original directory and its permissions.
pub fn retain_frozen_copy(frozen: &Path, retained: &Path) {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let parent = frozen.parent().unwrap();
    assert_eq!(retained.parent(), Some(parent));
    let parent_permissions = std::fs::metadata(parent).unwrap().permissions();
    let parent_mode = parent_permissions.mode();
    let original = std::fs::metadata(frozen).unwrap();
    let frozen_permissions = original.permissions();
    let frozen_mode = frozen_permissions.mode();
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(parent_mode | 0o700)).unwrap();
    // The macOS 14 CI fixture move needs write access to this source directory.
    std::fs::set_permissions(frozen, std::fs::Permissions::from_mode(frozen_mode | 0o200)).unwrap();
    let moved = std::fs::rename(frozen, retained);
    let restore_path = if moved.is_ok() { retained } else { frozen };
    std::fs::set_permissions(restore_path, frozen_permissions).unwrap();
    std::fs::set_permissions(parent, parent_permissions).unwrap();
    assert_eq!(
        std::fs::metadata(parent).unwrap().permissions().mode(),
        parent_mode
    );
    let restored = std::fs::metadata(restore_path).unwrap();
    assert_eq!(restored.permissions().mode(), frozen_mode);
    assert_eq!(
        (restored.dev(), restored.ino()),
        (original.dev(), original.ino())
    );
    moved.unwrap();
    assert!(!frozen.exists(), "the frozen lookup must remain missing");
}

#[derive(Debug, PartialEq, Eq)]
pub struct TreeObject {
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub links: u64,
    pub bytes: Option<Vec<u8>>,
}

pub fn tree_objects(
    root: &Path,
    scopes: &[&str],
) -> std::collections::BTreeMap<PathBuf, TreeObject> {
    use std::os::unix::fs::MetadataExt;
    fn visit(
        root: &Path,
        path: &Path,
        objects: &mut std::collections::BTreeMap<PathBuf, TreeObject>,
    ) {
        let metadata = std::fs::symlink_metadata(path).unwrap();
        assert!(
            metadata.is_file() || metadata.is_dir(),
            "unexpected fixture object: {path:?}"
        );
        objects.insert(
            path.strip_prefix(root).unwrap().to_owned(),
            TreeObject {
                device: metadata.dev(),
                inode: metadata.ino(),
                mode: metadata.mode(),
                links: metadata.nlink(),
                bytes: metadata.is_file().then(|| std::fs::read(path).unwrap()),
            },
        );
        if metadata.is_dir() {
            for child in std::fs::read_dir(path).unwrap() {
                visit(root, &child.unwrap().path(), objects);
            }
        }
    }
    let mut objects = std::collections::BTreeMap::new();
    for scope in scopes {
        let path = root.join(scope);
        if path.exists() {
            visit(root, &path, &mut objects);
        }
    }
    objects
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
