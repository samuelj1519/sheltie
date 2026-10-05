//! Shared runtime integration support: temporary roots, example directories, and constructors.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use sheltie_core::path::AbsPath;
use sheltie_runtime::{Error, Home, WorkService, WorkbookRepo};
mod owned_tempdir;
pub use owned_tempdir::OwnedTempDir;
mod snapshot;
#[allow(unused_imports)]
pub use snapshot::{StoreRows, store_rows};

#[cfg(feature = "failpoint")]
use sheltie_runtime as runtime;
#[cfg(feature = "failpoint")]
pub mod init;

pub fn assert_effect_pending(
    error: Error,
    committed: bool,
    request_id: Option<&str>,
    pending_request_id: Option<&str>,
) -> serde_json::Value {
    let Error::EffectPending {
        committed: actual_committed,
        request_id: actual_request,
        pending_request_id: actual_pending,
        cause,
        original,
        pending_original,
        ..
    } = error
    else {
        panic!("Request errors must preserve EFFECT_PENDING ownership: {error:?}");
    };
    assert_eq!(actual_committed, committed);
    if let Some(request_id) = request_id {
        assert_eq!(actual_request, request_id);
    } else {
        assert!(!actual_request.is_empty());
    }
    assert_eq!(actual_pending.as_deref(), pending_request_id);
    assert_eq!(cause, sheltie_core::ErrorCode::StoreCorrupt);
    let (snapshot, other) = if committed {
        (original, pending_original)
    } else {
        (pending_original, original)
    };
    let snapshot: serde_json::Value = serde_json::from_str(snapshot.as_deref().unwrap()).unwrap();
    if snapshot["ok"] == true {
        assert_eq!(snapshot["data"]["replayed"], false);
        assert!(snapshot.get("next").unwrap().is_array());
    } else {
        assert!(snapshot.get("request_id").is_some());
    }
    assert!(other.is_none());
    snapshot
}

pub fn assert_effect_pending_without_original(
    error: Error,
    committed: bool,
    request_id: &str,
    pending_request_id: Option<&str>,
) {
    let Error::EffectPending {
        committed: actual_committed,
        request_id: actual_request,
        pending_request_id: actual_pending,
        cause,
        original,
        pending_original,
        ..
    } = error
    else {
        panic!("Request errors must preserve EFFECT_PENDING ownership: {error:?}");
    };
    assert_eq!(actual_committed, committed);
    assert_eq!(actual_request, request_id);
    assert_eq!(actual_pending.as_deref(), pending_request_id);
    assert_eq!(cause, sheltie_core::ErrorCode::StoreCorrupt);
    assert!(original.is_none());
    assert!(pending_original.is_none());
}

pub fn abs(p: &Path) -> AbsPath {
    AbsPath::new(p.to_str().unwrap()).unwrap()
}

/// Fresh temporary management root; return TempDir to keep it alive.
pub fn temp_home() -> (OwnedTempDir, Home) {
    let dir = OwnedTempDir::new();
    let home = Home::resolve(Some((abs(dir.path())).as_str())).unwrap();
    (dir, home)
}

/// Repository examples/<name> directory.
pub fn example_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
        .canonicalize()
        .unwrap()
}

/// Copy an example into a writable temporary directory for tampering tests.
pub fn copy_example(name: &str, into: &Path) -> PathBuf {
    let dst = into.join(name);
    copy_dir(&example_dir(name), &dst);
    dst
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

pub fn repo(home: &Home) -> WorkbookRepo {
    WorkbookRepo::new(home.clone())
}

pub fn service(home: &Home) -> WorkService {
    WorkService::new(home.clone())
}

pub fn sentinel(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, format!("sentinel-{name}")).unwrap();
    path
}

pub fn snapshot(path: &Path) -> (Vec<u8>, u32) {
    use std::os::unix::fs::PermissionsExt as _;
    (
        std::fs::read(path).unwrap(),
        std::fs::metadata(path).unwrap().permissions().mode(),
    )
}

/// Install an example and return its service.
pub fn home_with_example(name: &str) -> (OwnedTempDir, Home, WorkService) {
    let (dir, home) = temp_home();
    repo(&home).add(&abs(&example_dir(name)), None).unwrap();
    let svc = service(&home);
    (dir, home, svc)
}

pub fn lit(text: &str) -> sheltie_runtime::request::InputValue {
    sheltie_runtime::request::InputValue::Literal {
        text: text.to_string(),
    }
}

pub fn inputs_lit(
    pairs: &[(&str, &str)],
) -> std::collections::BTreeMap<String, sheltie_runtime::request::InputValue> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), lit(value)))
        .collect()
}

pub fn two_step_args(inputs: &[(&str, &str)]) -> sheltie_runtime::StartArgs {
    sheltie_runtime::StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: inputs_lit(inputs),
    }
}

pub fn start_two_step(svc: &WorkService) -> sheltie_runtime::Response {
    svc.start(two_step_args(&[("topic", "给新人介绍 Sheltie")]), None)
        .unwrap()
}

pub fn work_id_of(resp: &sheltie_runtime::Response) -> sheltie_core::ids::WorkId {
    match &resp.reply {
        sheltie_core::work::Reply::Started { work_id, .. } => work_id.clone(),
        other => panic!("Expected Started, got {other:?}"),
    }
}

/// Write a file in the Attempt output directory.
pub fn write_output(output_dir: &AbsPath, rel: &str, content: &str) {
    let p = Path::new(output_dir.as_str()).join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

pub fn output_dir_of(resp: &sheltie_runtime::Response) -> AbsPath {
    match &resp.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("Expected AttemptBegun, got {other:?}"),
    }
}

#[cfg(feature = "failpoint")]
pub struct RendezvousWorker<T> {
    releases: Vec<PathBuf>,
    worker: Option<std::thread::JoinHandle<T>>,
}

#[cfg(feature = "failpoint")]
impl<T> RendezvousWorker<T> {
    pub fn new(worker: std::thread::JoinHandle<T>, before: &Path, after: &Path) -> Self {
        Self {
            releases: vec![before.join("release"), after.join("release")],
            worker: Some(worker),
        }
    }

    pub fn single(worker: std::thread::JoinHandle<T>, rendezvous: &Path) -> Self {
        Self {
            releases: vec![rendezvous.join("release")],
            worker: Some(worker),
        }
    }

    pub fn wait(&self, message: &str) {
        use std::time::{Duration, Instant};
        let reached = self.releases[0].parent().unwrap().join("reached");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !reached.exists() {
            assert!(Instant::now() < deadline, "{message}");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    pub fn finish(&mut self) -> std::thread::Result<T> {
        for release in &self.releases {
            std::fs::write(release, b"release").unwrap();
        }
        let result = self.worker.take().unwrap().join();
        sheltie_runtime::failpoint::disarm_rendezvous().unwrap();
        result
    }
}

#[cfg(feature = "failpoint")]
impl<T> Drop for RendezvousWorker<T> {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            for release in &self.releases {
                let _ = std::fs::write(release, b"release");
            }
            let _ = worker.join();
            let _ = sheltie_runtime::failpoint::disarm_rendezvous();
        }
    }
}
