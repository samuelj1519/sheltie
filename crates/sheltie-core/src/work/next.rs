//! Legal next actions; table in `specs/architecture.md` §3.1.

use serde::{Deserialize, Serialize};

use super::state::{AttemptStatus, BlockedReason, WorkState, WorkStatus};
use crate::flow::{EdgeKind, Executor, Graph, Tier};
use crate::ids::{AttemptId, NodeId, WorkId};

/// One legal next action, directly renderable as a command line (protocol §5 next item).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op", deny_unknown_fields)]
pub enum NextOp {
    /// `attempt begin`: include edge when entering another node; omit it on entry or same-node retry.
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
    #[serde(rename = "attempt replace")]
    ReplaceAttempt { attempt: AttemptId },
    #[serde(rename = "gate approve")]
    ApproveGate { node: NodeId },
    #[serde(rename = "work cancel")]
    Cancel,
}

impl NextOp {
    /// Command-line forms:
    /// `sheltie attempt begin <work> --node <node>`、`sheltie attempt submit <work> --attempt <id> --summary "<one-sentence conclusion>"`、
    /// `sheltie attempt fail <work> --attempt <id> --reason "<reason>"`、`sheltie gate approve <work> --node <node>`、
    /// `sheltie work cancel <work>`。
    pub fn to_command_line(&self, work_id: &WorkId) -> String {
        match self {
            Self::BeginAttempt { node, .. } => {
                format!("sheltie attempt begin {work_id} --node {node}")
            }
            Self::SubmitAttempt { attempt } => format!(
                "sheltie attempt submit {work_id} --attempt {attempt} --summary \"<one-sentence conclusion>\""
            ),
            Self::FailAttempt { attempt } => {
                format!("sheltie attempt fail {work_id} --attempt {attempt} --reason \"<reason>\"")
            }
            Self::ReplaceAttempt { attempt } => {
                format!(
                    "sheltie attempt replace {work_id} --attempt {attempt} --reason \"<reason>\""
                )
            }
            Self::ApproveGate { node } => format!("sheltie gate approve {work_id} --node {node}"),
            Self::Cancel => format!("sheltie work cancel {work_id}"),
        }
    }

    /// Whether this action enters or retries node `node`.
    pub fn is_begin_of(&self, node: &NodeId) -> bool {
        matches!(self, Self::BeginAttempt { node: n, .. } if n == node)
    }
}

/// Compute current legal next actions.
///
/// Branch on state.status and the current Occurrence's latest Attempt:
/// - Terminal: empty.
/// - `Blocked(Gate)`：`gate approve <current>`、`work cancel`。
/// - `Blocked(RetriesExhausted | NoLegalEdge)`：`work cancel`。
/// - No current Attempt, or latest Failed with retries remaining: `attempt begin <current>` without edge, and work cancel.
/// - Latest Running: attempt submit, attempt fail, work cancel; include attempt replace if not already replaced in this Occurrence.
/// - Latest Succeeded (no gate or approved): every outgoing current -> to with visits below max_visits
///   yields attempt begin <to> with edge; also include work cancel.
///
/// T06 covers terminal/no-Attempt branches; T07 Succeeded/retryable Failed; T08 Running/Blocked.
pub fn legal_next(state: &WorkState, graph: &Graph) -> Vec<NextOp> {
    match state.status {
        WorkStatus::Active => match state.latest_attempt_of_current() {
            // First Attempt in this Occurrence: entry and retries begin at the current node without an edge.
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
                AttemptStatus::Running => {
                    let mut ops = vec![
                        NextOp::SubmitAttempt {
                            attempt: attempt.id.clone(),
                        },
                        NextOp::FailAttempt {
                            attempt: attempt.id.clone(),
                        },
                    ];
                    if !state.has_replacement_of(&state.current) {
                        ops.push(NextOp::ReplaceAttempt {
                            attempt: attempt.id.clone(),
                        });
                    }
                    ops.push(NextOp::Cancel);
                    ops
                }
                AttemptStatus::Failed => {
                    // Retry the current node without edge while retries remain; otherwise only work cancel remains.
                    let mut ops = Vec::with_capacity(2);
                    if let Some(def) = graph.node(&state.current.node) {
                        if state.failed_count_of(&state.current) <= def.max_retries as usize {
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
                    // Each outgoing target below max_visits yields begin; also include work cancel.
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
                AttemptStatus::Superseded => vec![NextOp::Cancel],
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
