//! 对文件系统的只读观察。这是 core 拿到「事实」的唯一通道。

use sheltie_core::flow::ResourceIndex;
use sheltie_core::path::AbsPath;
use sheltie_core::work::{ObservedFile, Principal, Timestamp};

use crate::error::Result;

/// 观察一个文件：必须是普通文件（`symlink_metadata`，不跟随软链）；算 sha256 与字节数。
/// 不存在返回 `Error::NotFound`；是软链或目录返回 `Error::InvalidRequest`。
#[allow(unused_variables)]
pub fn observe_file(path: &AbsPath) -> Result<ObservedFile> {
    todo!("T12")
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
#[allow(unused_variables)]
pub fn build_resource_index(dir: &AbsPath) -> Result<ResourceIndex> {
    todo!("T12")
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
