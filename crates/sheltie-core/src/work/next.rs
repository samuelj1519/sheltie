//! 合法下一步。表见 `specs/architecture.md` §3.1。

use serde::{Deserialize, Serialize};

use super::state::{AttemptStatus, BlockedReason, WorkState, WorkStatus};
use crate::flow::{EdgeKind, Executor, Graph, Tier};
use crate::ids::{AttemptId, NodeId, WorkId};

/// 一项可执行的下一步。能直接拼成命令行（协议 §5 的 `next` 项）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum NextOp {
    /// `attempt begin`。进入另一节点时带 `edge`；同节点重试或首次开工时为 `None`。
    #[serde(rename = "attempt begin")]
    BeginAttempt {
        node: NodeId,
        edge: Option<EdgeKind>,
        executor: Executor,
        tier: Option<Tier>,
    },
    #[serde(rename = "attempt submit")]
    SubmitAttempt { attempt: AttemptId },
    #[serde(rename = "attempt fail")]
    FailAttempt { attempt: AttemptId },
    #[serde(rename = "gate approve")]
    ApproveGate { node: NodeId },
    #[serde(rename = "work cancel")]
    Cancel,
}

impl NextOp {
    /// 命令行形式：
    /// `sheltie attempt begin <work> --node <node>`、`sheltie attempt submit <work> --attempt <id> --summary "<一句话结论>"`、
    /// `sheltie attempt fail <work> --attempt <id> --reason "<原因>"`、`sheltie gate approve <work> --node <node>`、
    /// `sheltie work cancel <work>`。
    pub fn to_command_line(&self, work_id: &WorkId) -> String {
        match self {
            Self::BeginAttempt { node, .. } => {
                format!("sheltie attempt begin {work_id} --node {node}")
            }
            Self::SubmitAttempt { attempt } => format!(
                "sheltie attempt submit {work_id} --attempt {attempt} --summary \"<一句话结论>\""
            ),
            Self::FailAttempt { attempt } => {
                format!("sheltie attempt fail {work_id} --attempt {attempt} --reason \"<原因>\"")
            }
            Self::ApproveGate { node } => format!("sheltie gate approve {work_id} --node {node}"),
            Self::Cancel => format!("sheltie work cancel {work_id}"),
        }
    }

    /// 这项是不是「进入或重试节点 `node`」。
    pub fn is_begin_of(&self, node: &NodeId) -> bool {
        matches!(self, Self::BeginAttempt { node: n, .. } if n == node)
    }
}

/// 计算当前合法下一步。
///
/// 按 `state.status` 与当前 Occurrence 的最新 Attempt 分支：
/// - 终态：空。
/// - `Blocked(Gate)`：`gate approve <current>`、`work cancel`。
/// - `Blocked(RetriesExhausted | NoLegalEdge)`：`work cancel`。
/// - 当前 Occurrence 无 Attempt，或最新 `Failed` 且 `retry < max_retries`：`attempt begin <current>`（`edge = None`）、`work cancel`。
/// - 最新 `Running`：`attempt submit`、`attempt fail`、`work cancel`。
/// - 最新 `Succeeded`（无门槛或已批准）：每条出边 `current -> to` 且 `visits[to] < max_visits[to]`
///   给一项 `attempt begin <to>` 带 `edge`；再加 `work cancel`。
///
/// T06 填「终态」与「无 Attempt」；T07 填「Succeeded」与「Failed 可重试」；T08 填「Running」与「Blocked」。
pub fn legal_next(state: &WorkState, graph: &Graph) -> Vec<NextOp> {
    if state.status.is_terminal() {
        return Vec::new();
    }
    match state.status {
        WorkStatus::Active => match state.latest_attempt_of_current() {
            // 首次到达本 Occurrence：进入或重试都从 begin 当前节点开始，不带边。
            None => {
                let mut ops = Vec::with_capacity(2);
                if let Some(def) = graph.node(&state.current.node) {
                    ops.push(NextOp::BeginAttempt {
                        node: state.current.node.clone(),
                        edge: None,
                        executor: def.executor,
                        tier: def.tier,
                    });
                }
                ops.push(NextOp::Cancel);
                ops
            }
            Some(attempt) => match attempt.status {
                AttemptStatus::Running => vec![
                    NextOp::SubmitAttempt {
                        attempt: attempt.id.clone(),
                    },
                    NextOp::FailAttempt {
                        attempt: attempt.id.clone(),
                    },
                    NextOp::Cancel,
                ],
                AttemptStatus::Failed => {
                    // 还能重试就再 begin 当前节点（不带边）；否则只剩取消。
                    let mut ops = Vec::with_capacity(2);
                    if let Some(def) = graph.node(&state.current.node) {
                        if attempt.id.retry < def.max_retries {
                            ops.push(NextOp::BeginAttempt {
                                node: state.current.node.clone(),
                                edge: None,
                                executor: def.executor,
                                tier: def.tier,
                            });
                        }
                    }
                    ops.push(NextOp::Cancel);
                    ops
                }
                AttemptStatus::Succeeded => {
                    // 每条出边目标未达 max_visits 就是一项 begin；再加取消。
                    let mut ops = Vec::with_capacity(4);
                    for edge in graph.out_edges(&state.current.node) {
                        let Some(def) = graph.node(&edge.to) else {
                            continue;
                        };
                        if state.visits_of(&edge.to) < def.max_visits {
                            ops.push(NextOp::BeginAttempt {
                                node: edge.to.clone(),
                                edge: Some(edge.kind),
                                executor: def.executor,
                                tier: def.tier,
                            });
                        }
                    }
                    ops.push(NextOp::Cancel);
                    ops
                }
            },
        },
        WorkStatus::Blocked(reason) => {
            let mut ops = Vec::with_capacity(2);
            if reason == BlockedReason::Gate {
                ops.push(NextOp::ApproveGate {
                    node: state.current.node.clone(),
                });
            }
            ops.push(NextOp::Cancel);
            ops
        }
        WorkStatus::Succeeded | WorkStatus::Cancelled => Vec::new(),
    }
}
