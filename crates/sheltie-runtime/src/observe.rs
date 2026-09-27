//! 对文件系统的只读观察。这是 core 拿到「事实」的唯一通道。
//! 全部经 `fsx::SafeFile`：同一句柄上核对身份、限额、读取与摘要。

use std::collections::BTreeMap;

use sheltie_core::flow::{ResourceIndex, ResourceMeta};
use sheltie_core::path::{AbsPath, RelPath};
use sheltie_core::work::{ObservedFile, Principal, Timestamp};

use crate::error::{Error, Result};
use crate::fsx::{MAX_FILE_BYTES, SafeFile};

/// 观察一个文件：`SafeFile` 句柄核对身份（拒绝软链、目录、硬链），再在句柄上
/// 限额读取并算 sha256 与字节数。不存在返回 `Error::NotFound`。
pub fn observe_file(path: &AbsPath) -> Result<ObservedFile> {
    let f = SafeFile::open_regular(path)?;
    let (sha256, bytes) = f.sha256_bounded(MAX_FILE_BYTES)?;
    Ok(ObservedFile::new(path.clone(), sha256, bytes))
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
/// 键是相对 `dir` 的路径。每个文件经句柄限额读取，只为字节数与 UTF-8 判定，
/// 不把资源全文转成字符串（design §4）。
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
        let f = SafeFile::open_regular(&path)?;
        let bytes = f.read_bounded(MAX_FILE_BYTES)?;
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
/// 真实 OS 主体来源的替换（D-036）归 C002-T05。
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
