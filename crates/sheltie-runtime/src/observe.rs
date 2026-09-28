//! 对文件系统的只读观察。这是 core 拿到「事实」的唯一通道。
//! 全部经 `fsx::SafeFile`：同一句柄上核对身份、限额、读取与摘要。

use sheltie_core::flow::ResourceIndex;
use sheltie_core::path::AbsPath;
use sheltie_core::work::{ObservedFile, Principal, Timestamp};

use crate::error::Result;
use crate::fsx::{ExternalReadFile, ExternalReadTree, MAX_FILE_BYTES};

/// 观察一个文件：`SafeFile` 句柄核对身份（拒绝软链、目录、硬链），再在句柄上
/// 限额读取并算 sha256 与字节数。不存在返回 `Error::NotFound`。
pub fn observe_file(path: &AbsPath) -> Result<ObservedFile> {
    let f = ExternalReadFile::open_regular(path)?;
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
    let tree = ExternalReadTree::open(dir)?;
    build_resource_index_from_tree(&tree)
}

pub(crate) fn build_resource_index_from_tree(tree: &ExternalReadTree) -> Result<ResourceIndex> {
    let (_, resources, _) =
        crate::workbook_digest::inspect_tree_v2(tree, &std::collections::BTreeMap::new())?;
    Ok(resources)
}

/// 当前操作主体：发起进程的真实 OS 身份（D-036）。unix 上取 effective uid 对应的
/// 账户名；查不到账户条目或名称不是 UTF-8 时记 `uid:<数值>`，不落到猜测值。
/// 依赖是维护中的同 API 分支 `uzers`（见 D-036 勘误）。
/// 完全不读 `USER`/`USERNAME`——环境变量由调用方任意可设。
pub fn principal() -> Principal {
    #[cfg(unix)]
    {
        let uid = uzers::get_effective_uid();
        match uzers::get_user_by_uid(uid) {
            Some(user) => match user.name().to_str() {
                Some(name) => Principal(name.to_string()),
                None => Principal(format!("uid:{uid}")),
            },
            None => Principal(format!("uid:{uid}")),
        }
    }
    #[cfg(not(unix))]
    {
        Principal("unknown".to_string())
    }
}

/// 当前 UTC 时间，秒精度。格式化在 core 的 `Timestamp::from_unix_secs`。
pub fn now() -> Timestamp {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Timestamp::from_unix_secs(secs)
}
