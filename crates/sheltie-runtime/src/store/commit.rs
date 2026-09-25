//! 一次写事务。顺序见 `specs/contracts/storage.md` §2。

use rusqlite::OptionalExtension;
use sheltie_core::ids::WorkId;
use sheltie_core::work::{Principal, Timestamp, WorkState};

use super::Store;
use crate::error::{Error, Result};

/// 一次写事务的全部输入。
#[derive(Debug, Clone)]
pub struct CommitInput {
    /// `workbook add` 之类不涉及 Work 的写操作为 `None`。
    pub work_id: Option<WorkId>,
    /// `start` 为 `None`（插入新行）；其他写操作为调用前读到的 revision。
    pub expected_revision: Option<u64>,
    /// 新状态。`None` 表示本次不改 `works` 表。
    pub state: Option<WorkState>,
    pub request_id: String,
    /// canonical JSON 载荷的 sha256。
    pub payload_hash: String,
    /// 序列化好的响应，重放时原样返回。
    pub reply_json: String,
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
    /// 同 `request_id` 同载荷重放，返回原响应。
    Replayed { reply_json: String },
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
        crate::failpoint::maybe_exit("before_commit");
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let prior: Option<(String, String)> = tx
            .query_row(
                "SELECT payload_hash, reply_json FROM requests WHERE request_id = ?1",
                [&input.request_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some((hash, reply)) = prior {
            // 两种命中都直接返回，事务在 drop 时回滚。
            if hash == input.payload_hash {
                return Ok(CommitOutcome::Replayed { reply_json: reply });
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
            let actual = actual.unwrap_or(0) as u64;
            if actual != expected {
                return Err(Error::RevisionConflict { expected, actual });
            }
        }
        // start（expected_revision 为 None）插入 revision 1；其他写操作在期望值上加一。
        let mut revision = 0u64;
        if let Some(state) = &input.state {
            revision = input.expected_revision.map_or(1, |r| r + 1);
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
                    revision as i64,
                    state.status.column(),
                    state_json,
                    state.created_at.as_str(),
                    state.updated_at.as_str(),
                ],
            )?;
        }
        tx.execute(
            "INSERT INTO audit (work_id, revision, request_id, principal, command_json, at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                input
                    .work_id
                    .as_ref()
                    .map_or_else(String::new, |w| w.as_str().to_string()),
                revision as i64,
                input.request_id,
                input.principal.0,
                input.command_json,
                input.at.as_str(),
            ],
        )?;
        tx.execute(
            "INSERT INTO requests (request_id, work_id, payload_hash, reply_json, at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                input.request_id,
                input.work_id.as_ref().map(|w| w.as_str()),
                input.payload_hash,
                input.reply_json,
                input.at.as_str(),
            ],
        )?;
        tx.commit()?;
        Ok(CommitOutcome::Committed { revision })
    }
}
