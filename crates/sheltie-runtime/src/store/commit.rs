//! 一次写事务。顺序见 `specs/contracts/storage.md` §2。

use sheltie_core::ids::WorkId;
use sheltie_core::work::{Principal, Timestamp, WorkState};

use super::Store;
use crate::error::Result;

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
    #[allow(unused_variables)]
    pub fn commit(&self, input: CommitInput) -> Result<CommitOutcome> {
        crate::failpoint::maybe_exit("before_commit");
        todo!("T13")
    }
}
