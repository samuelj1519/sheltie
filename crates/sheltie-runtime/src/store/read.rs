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

/// 请求快照的元数据；与效果载荷和完成标记分别解码。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestMetadata {
    pub intent_hash: String,
    pub reply_json: String,
    pub work_id: Option<String>,
    pub at: String,
}

impl RequestMetadata {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            intent_hash: row.get(0)?,
            reply_json: row.get(1)?,
            work_id: row.get(2)?,
            at: row.get(3)?,
        })
    }

    pub(crate) fn check_row(&self, request_id: &str, row: &RequestRow) -> Result<()> {
        if self.intent_hash != row.intent_hash
            || self.reply_json != row.reply_json
            || self.work_id != row.work_id
            || self.at != row.at
        {
            return Err(Error::StoreCorrupt {
                detail: format!("请求 {request_id} 的元数据在校验期间改变"),
            });
        }
        Ok(())
    }
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
    pub principal: String,
}

/// `works` 表一行（`state_json` 已解码）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRow {
    pub revision: u64,
    pub state: WorkState,
}

#[derive(Debug)]
pub(crate) struct WorkReadRequest {
    pub(crate) request_id: String,
    pub(crate) row: RequestRow,
    pub(crate) audit: AuditRow,
}

#[derive(Debug)]
pub(crate) struct WorkReadBundle {
    pub(crate) work: WorkRow,
    pub(crate) requests: Vec<WorkReadRequest>,
}

impl Store {
    /// Both ownership indexes are read in one SQLite snapshot, including completed effects.
    pub(crate) fn read_work_bundle(&self, id: &WorkId) -> Result<WorkReadBundle> {
        use rusqlite::types::ValueRef;
        use std::collections::BTreeMap;

        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let work: Option<(i64, String, String)> = tx
            .query_row(
                "SELECT revision, status, state_json FROM works WHERE work_id = ?1",
                [id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some((revision, status, state_json)) = work else {
            return Err(Error::NotFound {
                what: format!("Work {id}"),
            });
        };
        let work = decode_row(id.as_str(), revision, &status, &state_json)?;
        crate::failpoint::rendezvous("read_bundle_after_work", id.as_str())
            .map_err(|error| Error::io(self.path.as_str(), error))?;

        let mut requests = BTreeMap::new();
        {
            let mut statement = tx.prepare(
                "SELECT request_id, intent_hash, reply_json, effects_json, published, work_id, at
                 FROM requests r WHERE r.work_id = ?1 OR EXISTS (
                   SELECT 1 FROM audit a WHERE a.request_id = r.request_id AND a.work_id = ?1
                 ) ORDER BY request_id",
            )?;
            let mut rows = statement.query([id.as_str()])?;
            while let Some(row) = rows.next()? {
                let request_id: String = row.get(0)?;
                let published = match row.get_ref(4)? {
                    ValueRef::Integer(0) => false,
                    ValueRef::Integer(1) => true,
                    _ => {
                        return Err(Error::StoreCorrupt {
                            detail: format!("请求 {request_id} 的published值无效"),
                        });
                    }
                };
                requests.insert(
                    request_id,
                    RequestRow {
                        intent_hash: row.get(1)?,
                        reply_json: row.get(2)?,
                        effects_json: row.get(3)?,
                        published,
                        work_id: row.get(5)?,
                        at: row.get(6)?,
                    },
                );
            }
        }
        let audits = {
            let mut statement = tx.prepare(
                "SELECT seq, request_id, work_id, revision, command_json, at, principal FROM audit a
                 WHERE a.work_id = ?1 OR EXISTS (
                   SELECT 1 FROM requests r WHERE r.request_id = a.request_id AND r.work_id = ?1
                 ) ORDER BY revision, seq",
            )?;
            statement
                .query_map([id.as_str()], audit_row_of)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        let mut complete = Vec::with_capacity(audits.len());
        let mut previous_seq = 0;
        for audit in audits {
            let request_id = &audit.request_id;
            let row = requests
                .remove(request_id)
                .ok_or_else(|| Error::StoreCorrupt {
                    detail: format!("Work {id} 的audit请求 {request_id} 缺失或重复"),
                })?;
            let expected_revision = u64::try_from(complete.len())
                .ok()
                .and_then(|value| value.checked_add(1));
            if row.work_id.as_deref() != Some(id.as_str())
                || audit.work_id != id.as_str()
                || u64::try_from(audit.revision).ok() != expected_revision
                || audit.seq <= previous_seq
                || row.at != audit.at
            {
                return Err(Error::StoreCorrupt {
                    detail: format!(
                        "Work {id} 的请求 {request_id} 与audit归属/revision/时间不一致"
                    ),
                });
            }
            previous_seq = audit.seq;
            complete.push(WorkReadRequest {
                request_id: request_id.clone(),
                row,
                audit,
            });
        }
        if !requests.is_empty() || u64::try_from(complete.len()).ok() != Some(work.revision) {
            return Err(Error::StoreCorrupt {
                detail: format!("Work {id} 的requests/audit闭包与revision不完整"),
            });
        }
        tx.commit()?;
        Ok(WorkReadBundle {
            work,
            requests: complete,
        })
    }

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

    pub(crate) fn inspect_request_metadata(
        &self,
        request_id: &str,
    ) -> Result<Option<RequestMetadata>> {
        let conn = self.connect()?;
        Ok(conn
            .query_row(
                "SELECT intent_hash, reply_json, work_id, at FROM requests WHERE request_id = ?1",
                [request_id],
                RequestMetadata::from_row,
            )
            .optional()?)
    }

    pub(crate) fn lookup_request_effects(&self, request_id: &str) -> Result<Option<String>> {
        let conn = self.connect()?;
        Ok(conn
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id = ?1",
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
                "SELECT intent_hash, reply_json, work_id, at, effects_json, published
                 FROM requests WHERE request_id = ?1",
                [request_id],
                |r| {
                    let published = r.get::<_, i64>(5)?;
                    let published = match published {
                        0 => false,
                        1 => true,
                        _ => {
                            return Err(rusqlite::Error::FromSqlConversionFailure(
                                5,
                                rusqlite::types::Type::Integer,
                                Box::new(std::io::Error::other("published必须为0或1")),
                            ));
                        }
                    };
                    let metadata = RequestMetadata::from_row(r)?;
                    Ok(RequestRow {
                        intent_hash: metadata.intent_hash,
                        reply_json: metadata.reply_json,
                        effects_json: r.get(4)?,
                        published,
                        work_id: metadata.work_id,
                        at: metadata.at,
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
            "SELECT seq, request_id, work_id, revision, command_json, at, principal FROM audit WHERE request_id = ?1 ORDER BY seq",
        )?;
        let rows = stmt.query_map([request_id], audit_row_of)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// 按事务顺序读取全部审计事件；caller校验归属，不用正确字段预筛损坏行。
    pub(crate) fn audit_history(&self) -> Result<Vec<AuditRow>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT seq, request_id, work_id, revision, command_json, at, principal
             FROM audit ORDER BY seq",
        )?;
        let rows = stmt.query_map([], audit_row_of)?;
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

fn audit_row_of(row: &rusqlite::Row<'_>) -> rusqlite::Result<AuditRow> {
    Ok(AuditRow {
        seq: row.get(0)?,
        request_id: row.get(1)?,
        work_id: row.get(2)?,
        revision: row.get(3)?,
        command_json: row.get(4)?,
        at: row.get(5)?,
        principal: row.get(6)?,
    })
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

#[cfg(test)]
mod consistency_tests {
    use super::*;

    // Task: C002-T48
    #[test]
    fn request_metadata_keeps_immutable_fields_bound_when_only_completion_rolls_over() {
        let metadata = RequestMetadata {
            intent_hash: "a".repeat(64),
            reply_json: "{\"request_id\":\"committed\",\"revision\":1}".into(),
            work_id: Some("2026-10-03-001-original".into()),
            at: "2026-10-03T00:00:00Z".into(),
        };
        let row = RequestRow {
            intent_hash: metadata.intent_hash.clone(),
            reply_json: metadata.reply_json.clone(),
            work_id: metadata.work_id.clone(),
            at: metadata.at.clone(),
            effects_json: "[]".into(),
            published: false,
        };
        for published in [false, true] {
            metadata
                .check_row(
                    "committed",
                    &RequestRow {
                        published,
                        ..row.clone()
                    },
                )
                .unwrap();
        }
        for field in ["intent", "reply", "work", "time"] {
            let mut changed = row.clone();
            match field {
                "intent" => changed.intent_hash = "b".repeat(64),
                "reply" => {
                    changed.reply_json = "{\"request_id\":\"committed\",\"revision\":2}".into()
                }
                "work" => changed.work_id = Some("2026-10-03-002-other".into()),
                "time" => changed.at = "2026-10-03T00:00:01Z".into(),
                _ => unreachable!(),
            }
            assert!(
                matches!(
                    metadata.check_row("committed", &changed),
                    Err(Error::StoreCorrupt { .. })
                ),
                "{field}"
            );
        }
    }
}
