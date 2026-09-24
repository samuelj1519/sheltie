//! Workbook 仓库：`add / list / load / remove / verify`。规则见 `specs/contracts/storage.md` §5 与协议 §3。

use serde::Serialize;
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{FlowDef, Graph};
use sheltie_core::ids::WorkId;
use sheltie_core::path::AbsPath;
use sheltie_core::workbook::Manifest;

use crate::error::Result;
use crate::home::Home;
use crate::store::{Store, WorkbookRow};

/// 单文件上限 32 MiB。
pub const MAX_FILE_BYTES: u64 = 33_554_432;
/// 目录总量上限 256 MiB。
pub const MAX_TOTAL_BYTES: u64 = 268_435_456;

/// `workbook add` 的返回。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Added {
    pub id: String,
    pub version: String,
    pub digest: Sha256Hex,
    pub flows: Vec<String>,
    pub requires: Vec<String>,
}

/// 装好的 Workbook：manifest、每张图、目录。
#[derive(Debug, Clone)]
pub struct LoadedWorkbook {
    pub manifest: Manifest,
    pub flows: Vec<(FlowDef, Graph)>,
    pub dir: AbsPath,
    pub digest: Sha256Hex,
}

impl LoadedWorkbook {
    pub fn flow(&self, id: &str) -> Option<&(FlowDef, Graph)> {
        self.flows.iter().find(|(f, _)| f.id.as_str() == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Removed {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyStatus {
    Ok,
    Tampered,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyRow {
    pub id: String,
    pub version: String,
    pub status: VerifyStatus,
}

/// 仓库句柄。
#[derive(Debug, Clone)]
pub struct WorkbookRepo {
    home: Home,
    store: Store,
}

impl WorkbookRepo {
    pub fn new(home: Home, store: Store) -> Self {
        Self { home, store }
    }

    /// `workbook add <dir>`（存储合同 §5 四步）。
    ///
    /// 1. 校验：`build_resource_index`、`parse_manifest`、每个 Flow `parse_flow` + `compile`。
    /// 2. 复制到 `staging/<uuid>/`（`copy_confined`），fsync，算 `digest_dir`。
    /// 3. `store.insert_workbook`；冲突则删 staging 并报 `WorkbookExists`。
    /// 4. rename 到 `workbooks/<id>/<version>/`，整棵置只读。
    #[allow(unused_variables)]
    pub fn add(&self, dir: &AbsPath) -> Result<Added> {
        todo!("T14")
    }

    pub fn list(&self) -> Result<Vec<WorkbookRow>> {
        self.store.list_workbooks()
    }

    /// 从已装目录（或任意目录，用于冻结副本）重新解析并编译。
    #[allow(unused_variables)]
    pub fn load_dir(&self, dir: &AbsPath) -> Result<LoadedWorkbook> {
        todo!("T14")
    }

    /// 按 `id` 与版本加载；`version` 为 `None` 取字面最高版本。不存在报 `NotFound`。
    #[allow(unused_variables)]
    pub fn load(&self, id: &str, version: Option<&str>) -> Result<LoadedWorkbook> {
        todo!("T14")
    }

    /// 目录摘要：全部文件按相对路径排序，拼 `路径\0内容` 后 sha256（协议 `workbook add`）。
    #[allow(unused_variables)]
    pub fn digest_dir(dir: &AbsPath) -> Result<Sha256Hex> {
        todo!("T14")
    }

    /// 受限复制：拒绝软链、硬链、非普通文件、含 `..`、单文件超 32 MiB、总量超 256 MiB。
    #[allow(unused_variables)]
    pub(crate) fn copy_confined(src: &AbsPath, dst: &AbsPath) -> Result<u64> {
        todo!("T14")
    }

    /// `workbook remove <id>@<version>`（协议细则三步）。
    #[allow(unused_variables)]
    pub fn remove(&self, id: &str, version: &str) -> Result<Removed> {
        todo!("T15")
    }

    /// 引用该版本且非终态的 Work。先按 `status` 列过滤，再解 `state_json` 核对。
    #[allow(unused_variables)]
    pub(crate) fn works_referencing(&self, id: &str, version: &str) -> Result<Vec<WorkId>> {
        todo!("T15")
    }

    /// `workbook verify`。`filter` 为 `Some((id, version))` 只核对一个。
    #[allow(unused_variables)]
    pub fn verify(&self, filter: Option<(&str, &str)>) -> Result<Vec<VerifyRow>> {
        todo!("T15")
    }
}
