//! 对文件系统的只读观察。这是 core 拿到「事实」的唯一通道。

use std::collections::BTreeMap;

use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{ResourceIndex, ResourceMeta};
use sheltie_core::path::{AbsPath, RelPath};
use sheltie_core::work::{ObservedFile, Principal, Timestamp};

use crate::error::{Error, Result};

/// 观察一个文件：必须是普通文件（`symlink_metadata`，不跟随软链）；算 sha256 与字节数。
/// 不存在返回 `Error::NotFound`；是软链或目录返回 `Error::InvalidRequest`。
pub fn observe_file(path: &AbsPath) -> Result<ObservedFile> {
    let meta = std::fs::symlink_metadata(path.as_path()).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Error::NotFound {
            what: path.to_string(),
        },
        _ => Error::io(path.as_str(), e),
    })?;
    let ft = meta.file_type();
    if ft.is_symlink() {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 是符号链接"),
        });
    }
    if ft.is_dir() {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 是目录"),
        });
    }
    if !ft.is_file() {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 不是普通文件"),
        });
    }
    let bytes = std::fs::read(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
    Ok(ObservedFile::new(
        path.clone(),
        Sha256Hex::of_bytes(&bytes),
        bytes.len() as u64,
    ))
}

/// 观察一个文件，不存在时返回 `Ok(None)`，其他错误照常返回。
pub fn observe_optional(path: &AbsPath) -> Result<Option<ObservedFile>> {
    match observe_file(path) {
        Ok(f) => Ok(Some(f)),
        Err(crate::Error::NotFound { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 递归列出目录下全部普通文件，建 `ResourceIndex`。跳过目录，拒绝软链（返回错误）。
/// 键是相对 `dir` 的路径。`is_utf8` 通过读全文判断。
pub fn build_resource_index(dir: &AbsPath) -> Result<ResourceIndex> {
    let mut files = BTreeMap::new();
    walk_dir(dir, dir, &mut files)?;
    Ok(ResourceIndex { files })
}

fn walk_dir(
    root: &AbsPath,
    dir: &AbsPath,
    files: &mut BTreeMap<RelPath, ResourceMeta>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir.as_path()).map_err(|e| Error::io(dir.as_str(), e))? {
        let entry = entry.map_err(|e| Error::io(dir.as_str(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = dir.join_segment(&name);
        let ft = entry.file_type().map_err(|e| Error::io(path.as_str(), e))?;
        if ft.is_symlink() {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 是符号链接"),
            });
        }
        if ft.is_dir() {
            walk_dir(root, &path, files)?;
            continue;
        }
        if !ft.is_file() {
            continue;
        }
        let bytes = std::fs::read(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
        let rel = RelPath::new(
            path.as_path()
                .strip_prefix(root.as_path())
                .map_err(|e| Error::io(root.as_str(), std::io::Error::other(e.to_string())))?
                .to_string(),
        )?;
        files.insert(
            rel,
            ResourceMeta {
                bytes: bytes.len() as u64,
                is_utf8: std::str::from_utf8(&bytes).is_ok(),
            },
        );
    }
    Ok(())
}

/// 当前操作系统用户名，取 `USER` 或 `USERNAME`，都没有则 `unknown`。
pub fn principal() -> Principal {
    let name = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    Principal(name)
}

/// 当前 UTC 时间，秒精度。格式化在 core 的 `Timestamp::from_unix_secs`。
pub fn now() -> Timestamp {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Timestamp::from_unix_secs(secs)
}
