//! Work 的运行时状态。整份 `WorkState` 作为一列 JSON 持久化（存储合同 §1.2）。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::digest::Sha256Hex;
use crate::flow::EdgeKind;
use crate::ids::{AttemptId, FlowId, NodeId, WorkId, WorkName, WorkbookId};
use crate::path::AbsPath;
use crate::text::Summary;

/// RFC 3339 UTC 时间串，由 runtime 传入。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub String);

impl Timestamp {
    /// 取日期部分 `YYYY-MM-DD`，用于 `work_id`。
    pub fn day(&self) -> &str {
        self.0.get(..10).unwrap_or(&self.0)
    }
}

/// 操作者身份。MVP 是操作系统用户名。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Principal(pub String);

/// Work 绑定的 Workbook 版本与内容摘要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkbookRef {
    pub id: WorkbookId,
    pub version: String,
    pub digest: Sha256Hex,
}

/// 一个按字节冻结的文件引用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub path: AbsPath,
    pub sha256: Sha256Hex,
    pub bytes: u64,
}

/// 节点第 `n` 次被到达。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Occurrence {
    pub node: NodeId,
    pub n: u32,
}

impl std::fmt::Display for Occurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.node, self.n)
    }
}

/// Attempt 只有执行事实。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    Running,
    Succeeded,
    Failed,
}

impl AttemptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// 一次执行尝试。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attempt {
    pub id: AttemptId,
    pub status: AttemptStatus,
    /// 从哪个 Occurrence 经哪种边到达；入口为 `None`。重试沿用第一次的值。
    pub entered_from: Option<(Occurrence, EdgeKind)>,
    /// 开工时冻结。`None` 只出现在 `required = false` 且上游尚无产出。
    pub inputs: BTreeMap<String, Option<ArtifactRef>>,
    /// 提交时封存。`Running` 时为空。
    pub outputs: BTreeMap<String, ArtifactRef>,
    pub summary: Option<Summary>,
    pub fail_reason: Option<Summary>,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
}

impl Attempt {
    pub fn occurrence(&self) -> Occurrence {
        Occurrence {
            node: self.id.node.clone(),
            n: self.id.occurrence,
        }
    }
}

/// 门槛批准记录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approval {
    pub node: NodeId,
    pub occurrence: u32,
    pub by: Principal,
    pub at: Timestamp,
}

/// Work 为什么受阻。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockedReason {
    Gate,
    RetriesExhausted,
    NoLegalEdge,
}

impl BlockedReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gate => "gate",
            Self::RetriesExhausted => "retries_exhausted",
            Self::NoLegalEdge => "no_legal_edge",
        }
    }
}

/// Work 状态。没有 `Failed`：重试耗尽后唯一出路是取消。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "reason")]
pub enum WorkStatus {
    Active,
    Blocked(BlockedReason),
    Succeeded,
    Cancelled,
}

impl WorkStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Cancelled)
    }

    /// 数据库 `status` 列用的字面量。
    pub fn column(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Blocked(_) => "blocked",
            Self::Succeeded => "succeeded",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for WorkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked(reason) => write!(f, "blocked({})", reason.as_str()),
            other => f.write_str(other.column()),
        }
    }
}

/// 一个 Work 的全部状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkState {
    pub work_id: WorkId,
    pub name: WorkName,
    pub workbook: WorkbookRef,
    pub flow: FlowId,
    /// `~/.sheltie/works/<work_id>`。所有 Attempt 目录与冻结副本都在它下面。
    pub work_dir: AbsPath,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub status: WorkStatus,
    /// 当前所在 Occurrence。终态后保留最后一个。
    pub current: Occurrence,
    pub visits: BTreeMap<NodeId, u32>,
    pub attempts: Vec<Attempt>,
    pub approvals: Vec<Approval>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl WorkState {
    /// 冻结副本目录 `work_dir/workbook`。
    pub fn workbook_dir(&self) -> AbsPath {
        self.work_dir.join_segment("workbook")
    }

    /// Attempt 目录 `work_dir/attempts/<node>/<n>/<retry>`。
    pub fn attempt_dir(&self, id: &AttemptId) -> AbsPath {
        self.work_dir
            .join_segment("attempts")
            .join_segment(id.node.as_str())
            .join_segment(&id.occurrence.to_string())
            .join_segment(&id.retry.to_string())
    }

    /// 状态卡路径 `work_dir/status-card.md`。
    pub fn status_card_path(&self) -> AbsPath {
        self.work_dir.join_segment("status-card.md")
    }

    pub fn attempt(&self, id: &AttemptId) -> Option<&Attempt> {
        self.attempts.iter().find(|a| &a.id == id)
    }

    pub fn attempt_mut(&mut self, id: &AttemptId) -> Option<&mut Attempt> {
        self.attempts.iter_mut().find(|a| &a.id == id)
    }

    /// 当前 Occurrence 的最新 Attempt。
    pub fn latest_attempt_of_current(&self) -> Option<&Attempt> {
        self.attempts
            .iter()
            .rev()
            .find(|a| a.occurrence() == self.current)
    }

    /// 某节点最近一次 `Succeeded` 的 Attempt。
    pub fn latest_succeeded_of(&self, node: &NodeId) -> Option<&Attempt> {
        self.attempts
            .iter()
            .rev()
            .find(|a| &a.id.node == node && a.status == AttemptStatus::Succeeded)
    }

    pub fn visits_of(&self, node: &NodeId) -> u32 {
        self.visits.get(node).copied().unwrap_or(0)
    }
}
