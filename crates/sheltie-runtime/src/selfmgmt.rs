//! 二进制自管理：`self install | update | rollback | uninstall | version`。
//! 规则见 `specs/contracts/storage.md` §9 与协议 `self` 组。

use serde::Serialize;
use sheltie_core::path::AbsPath;

use crate::error::Result;
use crate::home::Home;

/// 发布清单的来源。默认 GitHub Releases；测试用环境变量 `SHELTIE_RELEASE_BASE` 指向本地目录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSource {
    pub base: String,
}

impl ReleaseSource {
    pub fn from_env() -> Self {
        let base = std::env::var("SHELTIE_RELEASE_BASE")
            .unwrap_or_else(|_| "https://github.com/Samuel-J/sheltie/releases".to_string());
        Self { base }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstallOutcome {
    pub installed_to: AbsPath,
    pub already_installed: bool,
    pub path_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateOutcome {
    pub from: String,
    pub to: String,
    pub up_to_date: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub platform: String,
    pub home: AbsPath,
    pub schema_version: i64,
}

/// 当前平台串 `<os>-<arch>`，与发布包命名一致。
pub fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// `self version`。只读，不需要管理根存在。
pub fn version_info(home: &Home) -> VersionInfo {
    VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform(),
        home: home.root().clone(),
        schema_version: crate::store::SCHEMA_VERSION,
    }
}

/// `self install`：把当前可执行文件复制到 `bin/sheltie`（先写 `tmp/` 再 rename）。
/// 已存在且字节相同 → `already_installed = true`。`modify_path = false` 时只返回提示，不写 rc。
#[allow(unused_variables)]
pub fn install(home: &Home, modify_path: bool) -> Result<InstallOutcome> {
    todo!("T20")
}

/// `self update`（存储合同 §9 五步）。`version` 为 `None` 取最新。
#[allow(unused_variables)]
pub fn update(home: &Home, source: &ReleaseSource, version: Option<&str>) -> Result<UpdateOutcome> {
    crate::failpoint::maybe_exit("update_between_renames");
    todo!("T20")
}

/// `self rollback`：`sheltie.prev` 换回来；`bin/sheltie` 缺失时直接挪回。没有 `.prev` 报 `NotFound`。
#[allow(unused_variables)]
pub fn rollback(home: &Home) -> Result<()> {
    todo!("T20")
}

/// `self uninstall`：默认只删 `bin/`；`purge` 时删整个管理根，`confirmed` 为假则报 `InvalidRequest`。
#[allow(unused_variables)]
pub fn uninstall(home: &Home, purge: bool, confirmed: bool) -> Result<Vec<AbsPath>> {
    todo!("T20")
}
