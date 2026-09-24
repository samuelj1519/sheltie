//! 只读查询。不改状态、不建目录。

use sheltie_core::ids::WorkId;
use sheltie_core::work::WorkState;

use super::Store;
use crate::error::Result;

/// `workbooks` 表一行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbookRow {
    pub id: String,
    pub version: String,
    pub digest: String,
    pub dir: String,
    pub added_at: String,
}

/// `works` 表一行（`state_json` 已解码）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRow {
    pub revision: u64,
    pub state: WorkState,
}

impl Store {
    /// 读一个 Work。不存在报 `NotFound`；`state_json` 解不出报 `StoreCorrupt`。
    #[allow(unused_variables)]
    pub fn load_work(&self, id: &WorkId) -> Result<WorkRow> {
        todo!("T13")
    }

    /// 全部 Work，按 `work_id` 升序。
    pub fn list_works(&self) -> Result<Vec<WorkRow>> {
        todo!("T13")
    }

    /// 按前缀找 `work_id`。返回全部匹配，由调用方判断唯一性。
    #[allow(unused_variables)]
    pub fn find_works_by_prefix(&self, prefix: &str) -> Result<Vec<WorkId>> {
        todo!("T13")
    }

    /// 全部已装 Workbook，按 `(id, version)` 升序。
    pub fn list_workbooks(&self) -> Result<Vec<WorkbookRow>> {
        todo!("T13")
    }

    /// 某 id 的全部版本，按字面升序。
    #[allow(unused_variables)]
    pub fn workbook_versions(&self, id: &str) -> Result<Vec<WorkbookRow>> {
        todo!("T13")
    }

    /// 插入一行 Workbook。主键冲突报 `WorkbookExists`。独立事务（Workbook 入库不经 `commit`）。
    #[allow(unused_variables)]
    pub fn insert_workbook(&self, row: &WorkbookRow) -> Result<()> {
        todo!("T14")
    }

    /// 删一行 Workbook。不存在报 `NotFound`。
    #[allow(unused_variables)]
    pub fn delete_workbook(&self, id: &str, version: &str) -> Result<()> {
        todo!("T15")
    }
}
