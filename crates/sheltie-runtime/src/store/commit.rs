//! 一次写事务。顺序见 `specs/contracts/storage.md` §2。

use rusqlite::OptionalExtension;
use sheltie_core::ids::WorkId;
use sheltie_core::work::{Principal, Timestamp, WorkState};

use super::Store;
use crate::error::{Error, Result};

/// 一次写事务的全部输入（存储合同 §2.3）。
#[derive(Debug, Clone)]
pub struct CommitInput {
    /// `workbook add/remove` 之类不涉及 Work 的写操作为 `None`。
    pub work_id: Option<WorkId>,
    /// `start` 为 `None`（插入新行）；其他写操作为调用前读到的 revision。
    pub expected_revision: Option<u64>,
    /// 新状态。`None` 表示本次不改 `works` 表。
    pub state: Option<WorkState>,
    /// 本事务要插入的 `workbooks` 行（add）。
    pub workbook_insert: Option<super::WorkbookRow>,
    /// 本事务要删除的 `workbooks` 行（remove）。
    pub workbook_delete: Option<(String, String)>,
    pub request_id: String,
    /// `RequestIntent` canonical JSON 的 sha256（§2.1）。
    pub intent_hash: String,
    /// 提交时的完整 `ResponseSnapshot`，重放时原样返回（cli-result/v2）。
    pub reply_json: String,
    /// 效果登记（§3.2）。
    pub effects_json: String,
    pub principal: Principal,
    /// 去掉大字段后的 Command JSON，进审计表。
    pub command_json: String,
    pub at: Timestamp,
}

/// 事务结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitOutcome {
    /// 写入成功，`revision` 是提交后的值（不涉及 Work 时为 0）。
    Committed { revision: u64 },
    /// 同 `request_id` 同意图重放：返回原响应快照与效果登记；`published` 是原请求
    /// 的效果完成标记，调用方据此决定是否还需要恢复。
    Replayed {
        reply_json: String,
        effects_json: String,
        published: bool,
    },
}

impl Store {
    /// 执行一次写事务。
    ///
    /// ```text
    /// BEGIN IMMEDIATE
    ///   查 requests：命中且 hash 相同 → ROLLBACK，Replayed；命中且不同 → ROLLBACK，RequestConflict
    ///   若 expected_revision 有值：查 works.revision，不等 → ROLLBACK，RevisionConflict
    ///   若 state 有值：UPDATE 或 INSERT works（revision + 1，status 列从 state 派生）
    ///   INSERT audit（work_id 为 None 时用空串）
    ///   INSERT requests
    /// COMMIT
    /// ```
    /// 事务内不读文件、不算摘要。
    pub fn commit(&self, input: CommitInput) -> Result<CommitOutcome> {
        if let Some(state) = &input.state {
            let id = input.work_id.as_ref().ok_or_else(|| Error::StoreCorrupt {
                detail: "写入 WorkState 时缺 work_id".to_string(),
            })?;
            if &state.work_id != id {
                return Err(Error::StoreCorrupt {
                    detail: format!(
                        "写入目标 {id} 与 state_json.work_id {} 不一致",
                        state.work_id
                    ),
                });
            }
            state
                .validate_persisted()
                .map_err(|detail| Error::StoreCorrupt {
                    detail: format!("写入 WorkState 不合法：{detail}"),
                })?;
        }
        crate::failpoint::maybe_exit("before_commit");
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let prior: Option<(String, String, String, bool)> = tx
            .query_row(
                "SELECT intent_hash, reply_json, effects_json, published FROM requests WHERE request_id = ?1",
                [&input.request_id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get::<_, i64>(3)? != 0,
                    ))
                },
            )
            .optional()?;
        if let Some((hash, reply, effects, published)) = prior {
            // 两种命中都直接返回，事务在 drop 时回滚。
            if hash == input.intent_hash {
                return Ok(CommitOutcome::Replayed {
                    reply_json: reply,
                    effects_json: effects,
                    published,
                });
            }
            return Err(Error::RequestConflict {
                request_id: input.request_id,
            });
        }
        if let Some(expected) = input.expected_revision {
            let actual: Option<i64> = match &input.work_id {
                Some(id) => tx
                    .query_row(
                        "SELECT revision FROM works WHERE work_id = ?1",
                        [id.as_str()],
                        |r| r.get(0),
                    )
                    .optional()?,
                None => None,
            };
            let actual = match actual {
                Some(value) => u64::try_from(value)
                    .ok()
                    .filter(|revision| *revision > 0)
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("Work revision {value} 不合法"),
                    })?,
                None => 0,
            };
            if actual != expected {
                return Err(Error::RevisionConflict { expected, actual });
            }
        }
        // start（expected_revision 为 None）插入 revision 1；其他写操作在期望值上加一。
        let mut revision = 0u64;
        if let Some(state) = &input.state {
            revision = match input.expected_revision {
                None => 1,
                Some(previous) => previous.checked_add(1).ok_or_else(|| Error::StoreCorrupt {
                    detail: "Work revision 无法安全递增".to_string(),
                })?,
            };
            let db_revision = i64::try_from(revision).map_err(|_| Error::StoreCorrupt {
                detail: "Work revision 超过 SQLite INTEGER 上限".to_string(),
            })?;
            let state_json = serde_json::to_string(state).map_err(|e| Error::StoreCorrupt {
                detail: format!("序列化 WorkState 失败：{e}"),
            })?;
            let work_id = input
                .work_id
                .as_ref()
                .map_or_else(String::new, |w| w.as_str().to_string());
            tx.execute(
                "INSERT INTO works (work_id, revision, status, state_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(work_id) DO UPDATE SET
                   revision = excluded.revision,
                   status = excluded.status,
                   state_json = excluded.state_json,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    work_id,
                    db_revision,
                    state.status.column(),
                    state_json,
                    state.created_at.as_str(),
                    state.updated_at.as_str(),
                ],
            )?;
        }
        if let Some(row) = &input.workbook_insert {
            tx.execute(
                "INSERT INTO workbooks (id, version, digest, dir, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
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
        }
        if let Some((id, version)) = &input.workbook_delete {
            let n = tx.execute(
                "DELETE FROM workbooks WHERE id = ?1 AND version = ?2",
                rusqlite::params![id, version],
            )?;
            if n == 0 {
                return Err(Error::NotFound {
                    what: format!("Workbook {id}@{version}"),
                });
            }
        }
        let audit_revision = i64::try_from(revision).map_err(|_| Error::StoreCorrupt {
            detail: "审计 revision 超过 SQLite INTEGER 上限".to_string(),
        })?;
        tx.execute(
            "INSERT INTO audit (work_id, revision, request_id, principal, command_json, at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                input
                    .work_id
                    .as_ref()
                    .map_or_else(String::new, |w| w.as_str().to_string()),
                audit_revision,
                input.request_id,
                input.principal.0,
                input.command_json,
                input.at.as_str(),
            ],
        )?;
        tx.execute(
            "INSERT INTO requests (request_id, intent_hash, work_id, reply_json, effects_json, published, at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
            rusqlite::params![
                input.request_id,
                input.intent_hash,
                input.work_id.as_ref().map(|w| w.as_str()),
                input.reply_json,
                input.effects_json,
                input.at.as_str(),
            ],
        )?;
        tx.commit()?;
        Ok(CommitOutcome::Committed { revision })
    }
}
