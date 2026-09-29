//! runtime 集成测试共用：临时管理根、样例目录、快捷构造。
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use sheltie_core::path::AbsPath;
use sheltie_runtime::{Error, Home, WorkService, WorkbookRepo};
use tempfile::TempDir;

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
        panic!("请求错误必须保留EFFECT_PENDING归属：{error:?}");
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
        panic!("请求错误必须保留EFFECT_PENDING归属：{error:?}");
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

/// 一个全新的临时管理根。返回 `TempDir` 保活。
pub fn temp_home() -> (TempDir, Home) {
    let dir = tempfile::tempdir().unwrap();
    let home = Home::at(abs(dir.path()));
    (dir, home)
}

/// 仓库里的 `examples/<name>` 目录。
pub fn example_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
        .canonicalize()
        .unwrap()
}

/// 把样例复制到临时目录，返回可写副本（便于篡改）。
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

/// 装好一个样例并返回服务。
pub fn home_with_example(name: &str) -> (TempDir, Home, WorkService) {
    let (dir, home) = temp_home();
    repo(&home).add(&abs(&example_dir(name)), None).unwrap();
    let svc = service(&home);
    (dir, home, svc)
}

pub fn start_two_step(svc: &WorkService) -> sheltie_runtime::Response {
    svc.start(
        sheltie_runtime::StartArgs {
            workbook_id: "two-step".into(),
            version: None,
            flow: "default".into(),
            name: None,
            inputs: [(
                "topic".to_string(),
                sheltie_runtime::request::InputValue::Literal {
                    text: "给新人介绍 Sheltie".to_string(),
                },
            )]
            .into_iter()
            .collect(),
        },
        None,
    )
    .unwrap()
}

pub fn work_id_of(resp: &sheltie_runtime::Response) -> sheltie_core::ids::WorkId {
    match &resp.reply {
        sheltie_core::work::Reply::Started { work_id, .. } => work_id.clone(),
        other => panic!("不是 Started：{other:?}"),
    }
}

/// 在 Attempt 输出目录写一个文件。
pub fn write_output(output_dir: &AbsPath, rel: &str, content: &str) {
    let p = Path::new(output_dir.as_str()).join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

pub fn output_dir_of(resp: &sheltie_runtime::Response) -> AbsPath {
    match &resp.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("不是 AttemptBegun：{other:?}"),
    }
}
