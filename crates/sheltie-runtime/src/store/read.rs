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

/// `requests` 表一行的重核视图。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestRow {
    pub intent_hash: String,
    pub reply_json: String,
    pub effects_json: String,
    pub published: bool,
    pub work_id: Option<String>,
    pub at: String,
}

/// The single audit record that must own a persisted request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuditRow {
    pub work_id: String,
    pub revision: i64,
    pub command_json: String,
    pub at: String,
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
        let row: Option<(i64, String, String)> = conn
            .query_row(
                "SELECT revision, status, state_json FROM works WHERE work_id = ?1",
                [id.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((revision, status, state_json)) = row else {
            return Err(Error::NotFound {
                what: format!("Work {id}"),
            });
        };
        decode_row(id.as_str(), revision, &status, &state_json)
    }

    /// 全部 Work，按 `work_id` 升序。
    pub fn list_works(&self) -> Result<Vec<WorkRow>> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare("SELECT work_id, revision, status, state_json FROM works ORDER BY work_id")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (work_id, revision, status, state_json) = row?;
            out.push(decode_row(&work_id, revision, &status, &state_json)?);
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
    pub fn insert_workbook(&self, row: &WorkbookRow) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO workbooks (id, version, digest, dir, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![row.id, row.version, row.digest, row.dir, row.added_at],
        )
        .map_err(|e| match &e {
            rusqlite::Error::SqliteFailure(code, _)
                if code.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Error::WorkbookExists {
                    id: row.id.clone(),
                    version: row.version.clone(),
                }
            }
            _ => e.into(),
        })?;
        Ok(())
    }

    /// 删一行 Workbook。不存在报 `NotFound`。
    pub fn delete_workbook(&self, id: &str, version: &str) -> Result<()> {
        let conn = self.connect()?;
        let n = conn.execute(
            "DELETE FROM workbooks WHERE id = ?1 AND version = ?2",
            rusqlite::params![id, version],
        )?;
        if n == 0 {
            return Err(Error::NotFound {
                what: format!("Workbook {id}@{version}"),
            });
        }
        Ok(())
    }

    /// 查一个请求的 `(intent_hash, reply_json)`。无锁预检的重放查重用。
    pub(crate) fn lookup_request(&self, request_id: &str) -> Result<Option<(String, String)>> {
        let conn = self.connect()?;
        Ok(conn
            .query_row(
                "SELECT intent_hash, reply_json FROM requests WHERE request_id = ?1",
                [request_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }

    /// 锁内重核请求。
    pub(crate) fn inspect_request(&self, request_id: &str) -> Result<Option<RequestRow>> {
        let conn = self.connect()?;
        Ok(conn
            .query_row(
                "SELECT intent_hash, reply_json, effects_json, published, work_id, at
                 FROM requests WHERE request_id = ?1",
                [request_id],
                |r| {
                    let published = r.get::<_, i64>(3)?;
                    let published = match published {
                        0 => false,
                        1 => true,
                        _ => {
                            return Err(rusqlite::Error::FromSqlConversionFailure(
                                3,
                                rusqlite::types::Type::Integer,
                                Box::new(std::io::Error::other("published必须为0或1")),
                            ));
                        }
                    };
                    Ok(RequestRow {
                        intent_hash: r.get(0)?,
                        reply_json: r.get(1)?,
                        effects_json: r.get(2)?,
                        published,
                        work_id: r.get(4)?,
                        at: r.get(5)?,
                    })
                },
            )
            .optional()?)
    }

    /// 未完成效果的请求，按提交先后（`audit.seq` 递增）。恢复按这个顺序执行。
    pub(crate) fn unpublished_requests(&self) -> Result<Vec<(String, String)>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT r.request_id, r.effects_json FROM requests r
             WHERE r.published <> 1
             ORDER BY COALESCE(
               (SELECT MIN(a.seq) FROM audit a WHERE a.request_id = r.request_id),
               9223372036854775807
             ), r.request_id",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub(crate) fn audit_rows(&self, request_id: &str) -> Result<Vec<AuditRow>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT work_id, revision, command_json, at FROM audit WHERE request_id = ?1 ORDER BY seq",
        )?;
        let rows = stmt.query_map([request_id], |row| {
            Ok(AuditRow {
                work_id: row.get(0)?,
                revision: row.get(1)?,
                command_json: row.get(2)?,
                at: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 效果全部完成后置 `published = 1`。
    pub(crate) fn mark_published(&self, request_id: &str) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE requests SET published = 1 WHERE request_id = ?1",
            [request_id],
        )?;
        Ok(())
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

pub(crate) fn decode_row(
    work_id: &str,
    revision: i64,
    status: &str,
    state_json: &str,
) -> Result<WorkRow> {
    let id = WorkId::parse(work_id).map_err(|e| Error::StoreCorrupt {
        detail: format!("works 行 {work_id:?} 的 work_id 不合法：{e}"),
    })?;
    let revision = u64::try_from(revision)
        .ok()
        .filter(|revision| *revision > 0)
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("works 行 {id} 的 revision 必须大于零"),
        })?;
    let state: WorkState = serde_json::from_str(state_json).map_err(|e| Error::StoreCorrupt {
        detail: format!("works 行 {id} 的 state_json 解不开：{e}"),
    })?;
    if state.work_id != id {
        return Err(Error::StoreCorrupt {
            detail: format!("works 行 {id} 的 state_json.work_id 是 {}", state.work_id),
        });
    }
    if state.status.column() != status {
        return Err(Error::StoreCorrupt {
            detail: format!("works 行 {id} 的 status {status:?} 与 state_json.status 不一致"),
        });
    }
    state
        .validate_persisted()
        .map_err(|detail| Error::StoreCorrupt {
            detail: format!("works 行 {id} 的 state_json.{detail}"),
        })?;
    Ok(WorkRow { revision, state })
}
