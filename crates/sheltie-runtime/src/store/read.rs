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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestEffectRow {
    pub request_id: String,
    pub published: bool,
    pub effects_json: String,
    pub work_id: Option<String>,
}

/// The single audit record that must own a persisted request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuditRow {
    pub seq: i64,
    pub request_id: String,
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
        let mut stmt = conn.prepare("SELECT work_id FROM works ORDER BY work_id")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            let work_id = row?;
            if !work_id.starts_with(prefix) {
                continue;
            }
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
    #[cfg(test)]
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

    /// Read only the idempotency identity. A malformed reply payload must not hide the fact
    /// that the request was already committed or bypass REQUEST_CONFLICT precedence.
    pub(crate) fn lookup_request_hash(&self, request_id: &str) -> Result<Option<String>> {
        let conn = self.connect()?;
        Ok(conn
            .query_row(
                "SELECT intent_hash FROM requests WHERE request_id = ?1",
                [request_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub(crate) fn lookup_request_work(&self, request_id: &str) -> Result<Option<Option<String>>> {
        let conn = self.connect()?;
        Ok(conn
            .query_row(
                "SELECT work_id FROM requests WHERE request_id = ?1",
                [request_id],
                |row| row.get(0),
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

    /// Request IDs in commit order. Recovery opens each row separately so one malformed
    /// `effects_json` can be attributed to its owner instead of aborting this index read.
    pub(crate) fn unpublished_request_ids(&self) -> Result<Vec<String>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT r.request_id FROM requests r
             WHERE r.published <> 1
             ORDER BY COALESCE(
               (SELECT MIN(a.seq) FROM audit a WHERE a.request_id = r.request_id),
               9223372036854775807
             ), r.request_id",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// All request effect rows, including completed requests that may still own cleanup metadata.
    /// A single malformed row aborts the index so cleanup cannot infer an object is unreferenced.
    pub(crate) fn all_request_effect_rows(&self) -> Result<Vec<RequestEffectRow>> {
        self.request_effect_rows(
            "SELECT request_id, published, effects_json, work_id FROM requests ORDER BY request_id",
            [],
        )
    }

    pub(crate) fn work_start_effect_rows(&self, work: &WorkId) -> Result<Vec<RequestEffectRow>> {
        self.request_effect_rows(
            "SELECT DISTINCT r.request_id, r.published, r.effects_json, r.work_id
             FROM requests r JOIN audit a ON a.request_id = r.request_id
             WHERE r.work_id = ?1 AND a.revision = 1 ORDER BY r.request_id",
            [work.as_str()],
        )
    }

    fn request_effect_rows(
        &self,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> Result<Vec<RequestEffectRow>> {
        use rusqlite::types::ValueRef;

        let conn = self.connect()?;
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query(params)?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let request_id: String = row.get(0)?;
            let published = match row.get_ref(1)? {
                ValueRef::Integer(0) => false,
                ValueRef::Integer(1) => true,
                _ => {
                    return Err(Error::StoreCorrupt {
                        detail: format!("请求 {request_id} 的published值无效"),
                    });
                }
            };
            let effects_json =
                match row.get_ref(2)? {
                    ValueRef::Text(bytes) => std::str::from_utf8(bytes)
                        .map(str::to_string)
                        .map_err(|error| Error::StoreCorrupt {
                            detail: format!("请求 {request_id} 的effects_json不是UTF-8：{error}"),
                        })?,
                    _ => {
                        return Err(Error::StoreCorrupt {
                            detail: format!("请求 {request_id} 的effects_json不是TEXT"),
                        });
                    }
                };
            out.push(RequestEffectRow {
                request_id,
                published,
                effects_json,
                work_id: row.get(3)?,
            });
        }
        Ok(out)
    }

    pub(crate) fn audit_rows(&self, request_id: &str) -> Result<Vec<AuditRow>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT seq, request_id, work_id, revision, command_json, at FROM audit WHERE request_id = ?1 ORDER BY seq",
        )?;
        let rows = stmt.query_map([request_id], |row| {
            Ok(AuditRow {
                seq: row.get(0)?,
                request_id: row.get(1)?,
                work_id: row.get(2)?,
                revision: row.get(3)?,
                command_json: row.get(4)?,
                at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 按事务顺序读取全部审计事件；caller校验归属，不用正确字段预筛损坏行。
    pub(crate) fn audit_history(&self) -> Result<Vec<AuditRow>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT seq, request_id, work_id, revision, command_json, at
             FROM audit ORDER BY seq",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AuditRow {
                seq: row.get(0)?,
                request_id: row.get(1)?,
                work_id: row.get(2)?,
                revision: row.get(3)?,
                command_json: row.get(4)?,
                at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 效果全部完成后置 `published = 1`。
    pub(crate) fn mark_published(&self, request_id: &str) -> Result<()> {
        let conn = self.connect()?;
        let changed = conn.execute(
            "UPDATE requests SET published = 1 WHERE request_id = ?1",
            [request_id],
        )?;
        if changed != 1 {
            return Err(Error::StoreCorrupt {
                detail: format!("请求 {request_id} 发布标记应更新1行，实际更新{changed}行"),
            });
        }
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
