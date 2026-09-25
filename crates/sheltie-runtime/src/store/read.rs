//! 只读查询。不改状态、不建目录。

use rusqlite::OptionalExtension;
use sheltie_core::ids::WorkId;
use sheltie_core::work::WorkState;

use super::Store;
use crate::error::{Error, Result};

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
    pub fn load_work(&self, id: &WorkId) -> Result<WorkRow> {
        let conn = self.connect()?;
        let row: Option<(i64, String)> = conn
            .query_row(
                "SELECT revision, state_json FROM works WHERE work_id = ?1",
                [id.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((revision, state_json)) = row else {
            return Err(Error::NotFound {
                what: format!("Work {id}"),
            });
        };
        Ok(WorkRow {
            revision: revision as u64,
            state: decode_state(id, &state_json)?,
        })
    }

    /// 全部 Work，按 `work_id` 升序。
    pub fn list_works(&self) -> Result<Vec<WorkRow>> {
        let conn = self.connect()?;
        let mut stmt =
            conn.prepare("SELECT work_id, revision, state_json FROM works ORDER BY work_id")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (work_id, revision, state_json) = row?;
            let id = WorkId::parse(&work_id).map_err(|e| Error::StoreCorrupt {
                detail: format!("works 表的 work_id {work_id:?} 解不开：{e}"),
            })?;
            out.push(WorkRow {
                revision: revision as u64,
                state: decode_state(&id, &state_json)?,
            });
        }
        Ok(out)
    }

    /// 按前缀找 `work_id`。返回全部匹配，由调用方判断唯一性。
    pub fn find_works_by_prefix(&self, prefix: &str) -> Result<Vec<WorkId>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT work_id FROM works WHERE work_id LIKE ?1 ESCAPE '\\'
                 ORDER BY work_id",
        )?;
        // 前缀里的 LIKE 通配符按字面匹配。
        let pattern = format!(
            "{}%",
            prefix
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let rows = stmt.query_map([pattern], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            let work_id = row?;
            out.push(WorkId::parse(&work_id).map_err(|e| Error::StoreCorrupt {
                detail: format!("works 表的 work_id {work_id:?} 解不开：{e}"),
            })?);
        }
        Ok(out)
    }

    /// 全部已装 Workbook，按 `(id, version)` 升序。
    pub fn list_workbooks(&self) -> Result<Vec<WorkbookRow>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, version, digest, dir, added_at FROM workbooks ORDER BY id, version",
        )?;
        let rows = stmt.query_map([], workbook_row_of)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 某 id 的全部版本，按字面升序。
    pub fn workbook_versions(&self, id: &str) -> Result<Vec<WorkbookRow>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, version, digest, dir, added_at FROM workbooks
                 WHERE id = ?1 ORDER BY version",
        )?;
        let rows = stmt.query_map([id], workbook_row_of)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
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

fn workbook_row_of(r: &rusqlite::Row<'_>) -> rusqlite::Result<WorkbookRow> {
    Ok(WorkbookRow {
        id: r.get(0)?,
        version: r.get(1)?,
        digest: r.get(2)?,
        dir: r.get(3)?,
        added_at: r.get(4)?,
    })
}

fn decode_state(id: &WorkId, state_json: &str) -> Result<WorkState> {
    serde_json::from_str(state_json).map_err(|e| Error::StoreCorrupt {
        detail: format!("Work {id} 的 state_json 解不开：{e}"),
    })
}
