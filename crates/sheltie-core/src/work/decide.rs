//! 状态机的唯一入口 `decide`，以及每个命令一个私有函数。
//! 细则按 `specs/contracts/protocol.md` §3 逐条对应。

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::command::{Command, Context, Decision, Effect, ObservedFile, Reply};
use super::next::legal_next;
use super::render::{render_brief, render_stats_json};
use super::state::{
    Approval, ArtifactRef, Attempt, AttemptStatus, BlockedReason, Occurrence, WorkState, WorkStatus,
};
use crate::digest::Sha256Hex;
use crate::error::{Error, Result};
use crate::flow::{Graph, InputSource};
use crate::ids::{AttemptId, NodeId};
use crate::path::AbsPath;
use crate::text::Summary;

/// 对一个 Work 应用一个命令。
///
/// `state` 为 `None` 只在 `Command::Start` 时合法。其他命令先过 `guard_not_terminal`，
/// 再分派到对应的私有函数。返回的 `Decision.state` 是完整的新状态。
pub fn decide(
    state: Option<&WorkState>,
    graph: &Graph,
    cmd: &Command,
    ctx: &Context,
) -> Result<Decision> {
    match (state, cmd) {
        (None, Command::Start { .. }) => decide_start(graph, cmd, ctx),
        (None, other) => Err(Error::InvalidRequest {
            reason: format!("{} 要求 Work 已存在", other.name()),
        }),
        (Some(_), Command::Start { .. }) => Err(Error::InvalidRequest {
            reason: "Work 已存在，不能再 start".to_string(),
        }),
        (Some(state), cmd) => {
            guard_not_terminal(state)?;
            match cmd {
                Command::Start { .. } => unreachable!("上面已处理"),
                Command::BeginAttempt {
                    node,
                    observed_inputs,
                    instruction_text,
                } => decide_begin(state, graph, node, observed_inputs, instruction_text, ctx),
                Command::SubmitAttempt {
                    attempt,
                    summary,
                    observed_outputs,
                } => decide_submit(state, graph, attempt, summary, observed_outputs, ctx),
                Command::FailAttempt { attempt, reason } => {
                    decide_fail(state, graph, attempt, reason, ctx)
                }
                Command::ApproveGate { node } => decide_approve(state, graph, node, ctx),
                Command::Cancel => decide_cancel(state, ctx),
            }
        }
    }
}

/// 终态 Work 拒绝一切命令：`Error::WorkTerminal`。
fn guard_not_terminal(state: &WorkState) -> Result<()> {
    if state.status.is_terminal() {
        return Err(Error::WorkTerminal {
            status: state.status.to_string(),
        });
    }
    Ok(())
}

/// `work start` 第 2、6、7 步：核对起始输入键集合与图里全部 `start.<key>` 引用完全相等
/// （缺与多都是 `Error::InputMissing`），建初始状态 `current = entry#1`、`visits[entry] = 1`、
/// `status = Active`，回复 `Reply::Started`，效果 `RefreshStatusCard`。
fn decide_start(graph: &Graph, cmd: &Command, ctx: &Context) -> Result<Decision> {
    let Command::Start {
        work_id,
        name,
        workbook,
        flow,
        work_dir,
        inputs,
    } = cmd
    else {
        return Err(Error::InvalidRequest {
            reason: "decide_start 只接受 work start".to_string(),
        });
    };

    // 图里全部 start.<key> 引用，必须与给出的键集合完全相等。
    let mut wanted: BTreeSet<String> = BTreeSet::new();
    for node in graph.nodes() {
        for decl in &node.inputs {
            if let InputSource::Start { key } = &decl.from {
                wanted.insert(key.clone());
            }
        }
    }
    let missing: Vec<String> = wanted
        .iter()
        .filter(|k| !inputs.contains_key(*k))
        .cloned()
        .collect();
    let extra: Vec<String> = inputs
        .keys()
        .filter(|k| !wanted.contains(*k))
        .cloned()
        .collect();
    if !missing.is_empty() || !extra.is_empty() {
        return Err(Error::InputMissing { missing, extra });
    }

    let entry = graph.entry().clone();
    let mut visits = BTreeMap::new();
    visits.insert(entry.clone(), 1);

    // 宿主资源清单：Workbook 声明的**全部** requires，按 manifest 声明顺序，原样带上
    // version / digest / source（protocol.md work start 第 8 步；不止本图用到的那些）。
    let requires = graph.requires().to_vec();

    let state = WorkState {
        work_id: work_id.clone(),
        name: name.clone(),
        workbook: workbook.clone(),
        flow: flow.clone(),
        work_dir: work_dir.clone(),
        inputs: inputs.clone(),
        status: WorkStatus::Active,
        current: Occurrence { node: entry, n: 1 },
        visits,
        attempts: Vec::new(),
        approvals: Vec::new(),
        created_at: ctx.now.clone(),
        updated_at: ctx.now.clone(),
    };

    Ok(Decision {
        state,
        effects: vec![Effect::RefreshStatusCard],
        reply: Reply::Started {
            work_id: work_id.clone(),
            work_dir: work_dir.clone(),
            requires,
        },
    })
}

/// `attempt begin` 第 1 到 5 步。
///
/// 1. `node` 必须出现在 `legal_next` 的某个 `BeginAttempt` 项里，否则 `Error::IllegalNext`
///    （`next` 字段填每项 `to_command_line` 的结果）。
/// 2. 若 `node != current.node`：`visits[node] += 1`，`current = node#n`，记下 `entered_from`。
///    同节点重试：`retry + 1`，沿用上次的 `entered_from`。
/// 3. `bind_inputs`。
/// 4. 新建 `Running` 的 Attempt，`started_at = ctx.now`。
/// 5. 回复 `Reply::AttemptBegun`；效果 `WriteBrief`（内容用 `render_brief(.., instruction_text)`）与 `RefreshStatusCard`。
fn decide_begin(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
    observed_inputs: &BTreeMap<String, Option<ObservedFile>>,
    instruction_text: &str,
    ctx: &Context,
) -> Result<Decision> {
    let next = legal_next(state, graph);
    let illegal = || Error::IllegalNext {
        requested: format!("attempt begin {node}"),
        next: next
            .iter()
            .map(|op| op.to_command_line(&state.work_id))
            .collect(),
    };
    if !next.iter().any(|op| op.is_begin_of(node)) {
        return Err(illegal());
    }
    let def = graph.node(node).ok_or_else(|| Error::InvalidRequest {
        reason: format!("节点 {node} 不在图里"),
    })?;

    let mut new_state = state.clone();
    let (occ_n, retry, entered_from) = if node != &state.current.node {
        let edge = graph
            .out_edges(&state.current.node)
            .iter()
            .find(|e| e.to == *node)
            .ok_or_else(illegal)?;
        *new_state.visits.entry(node.clone()).or_insert(0) += 1;
        let n = new_state.visits_of(node);
        new_state.current = Occurrence {
            node: node.clone(),
            n,
        };
        (n, 0u32, Some((state.current.clone(), edge.kind)))
    } else {
        let prev = state.latest_attempt_of_current();
        let retry = prev.map(|a| a.id.retry + 1).unwrap_or(0);
        let entered_from = prev.and_then(|a| a.entered_from.clone());
        (state.current.n, retry, entered_from)
    };
    let attempt_id = AttemptId::new(node.clone(), occ_n, retry);

    let (bound, stats) = bind_inputs(state, graph, node, &attempt_id, observed_inputs)?;

    let mut effects = Vec::new();
    // 一次 begin 只写一个 stats.json（多个 engine.stats 输入共享同一份内容）。
    if let Some((artifact, content)) = stats {
        effects.push(Effect::WriteFile {
            path: artifact.path,
            content,
        });
    }

    let attempt = Attempt {
        id: attempt_id.clone(),
        status: AttemptStatus::Running,
        entered_from,
        inputs: bound.clone(),
        outputs: BTreeMap::new(),
        summary: None,
        fail_reason: None,
        started_at: ctx.now.clone(),
        ended_at: None,
    };
    new_state.attempts.push(attempt.clone());
    new_state.updated_at = ctx.now.clone();

    let attempt_dir = new_state.attempt_dir(&attempt_id);
    let brief_path = attempt_dir.join_segment("brief.md");
    effects.push(Effect::WriteBrief {
        path: brief_path.clone(),
        content: render_brief(&new_state, graph, &attempt, instruction_text),
    });
    effects.push(Effect::RefreshStatusCard);

    let inputs = bound
        .iter()
        .map(|(k, v)| (k.clone(), v.as_ref().map(|r| r.path.clone())))
        .collect();
    let mut outputs = BTreeMap::new();
    for decl in &def.outputs {
        outputs.insert(decl.name.clone(), attempt_dir.join(&decl.path));
    }
    // 节点引用的 `kind:name` 换成 manifest 里的那条声明，按节点里的顺序。
    // 编译期已保证每条引用都能在 manifest 找到（flow::compile）。
    let requires = def
        .requires
        .iter()
        .filter_map(|(kind, name)| {
            graph
                .requires()
                .iter()
                .find(|r| r.kind == *kind && r.name == *name)
                .cloned()
        })
        .collect();

    Ok(Decision {
        state: new_state,
        effects,
        reply: Reply::AttemptBegun {
            attempt: attempt_id,
            brief_path,
            output_dir: attempt_dir,
            inputs,
            outputs,
            requires,
        },
    })
}

/// 按节点的 `inputs[]` 逐条绑定。
///
/// - `Start { key }`：取 `state.inputs[key]`；观察摘要不符 `Error::ArtifactModified`。
/// - `Resource { path }`：路径 `state.workbook_dir().join(path)`；观察必须存在且摘要是它的摘要（首次绑定时以观察为准记录）。
/// - `EngineStats`：不看观察。用 `render::render_stats_json(state, graph)` 生成内容，路径 `attempt_dir(attempt_id)/stats.json`，
///   摘要 `Sha256Hex::of_bytes(内容)`；一次 begin 只算一份，返回值把它带回去给 `decide_begin` 追加一个 `Effect::WriteFile`。
/// - `Node { node, output }`：取 `latest_succeeded_of(node)` 的 `outputs[output]`；
///   没有时 `required` 为真报 `Error::InputUnavailable`，否则绑 `None`；
///   有时观察摘要必须等于记录，否则 `Error::ArtifactModified`。
///
/// 返回逐条绑定；有 `engine.stats` 输入时一并带回那份 stats.json 产物（内容与摘要同源）。
type BoundInputs = BTreeMap<String, Option<ArtifactRef>>;
type StatsArtifact = (ArtifactRef, String);

fn bind_inputs(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
    attempt_id: &AttemptId,
    observed_inputs: &BTreeMap<String, Option<ObservedFile>>,
) -> Result<(BoundInputs, Option<StatsArtifact>)> {
    let def = graph.node(node).ok_or_else(|| Error::InvalidRequest {
        reason: format!("节点 {node} 不在图里"),
    })?;
    let mut bound = BTreeMap::new();
    let mut stats: Option<StatsArtifact> = None;
    for decl in &def.inputs {
        let observed = observed_inputs.get(&decl.name).and_then(|o| o.as_ref());
        let artifact = match &decl.from {
            InputSource::Start { key } => {
                let want = state.inputs.get(key).ok_or_else(|| Error::InputMissing {
                    missing: vec![key.clone()],
                    extra: Vec::new(),
                })?;
                let obs = observed.ok_or_else(|| Error::ArtifactModified {
                    input: decl.name.clone(),
                    path: want.path.as_str().to_string(),
                })?;
                if !obs.matches(want) {
                    return Err(Error::ArtifactModified {
                        input: decl.name.clone(),
                        path: want.path.as_str().to_string(),
                    });
                }
                Some(want.clone())
            }
            InputSource::Resource { path } => {
                let full = state.workbook_dir().join(path);
                let obs = observed.ok_or_else(|| Error::ArtifactModified {
                    input: decl.name.clone(),
                    path: full.as_str().to_string(),
                })?;
                Some(ArtifactRef {
                    path: full,
                    sha256: obs.sha256.clone(),
                    bytes: obs.bytes,
                })
            }
            InputSource::EngineStats => {
                // 一次 begin 只算一份 stats.json，多个 engine.stats 输入共享它。
                if stats.is_none() {
                    stats = Some(engine_stats_artifact(state, graph, attempt_id)?);
                }
                stats.as_ref().map(|(artifact, _)| artifact.clone())
            }
            InputSource::Node { node: src, output } => match state
                .latest_succeeded_of(src)
                .and_then(|a| a.outputs.get(output))
            {
                Some(want) => {
                    let obs = observed.ok_or_else(|| Error::ArtifactModified {
                        input: decl.name.clone(),
                        path: want.path.as_str().to_string(),
                    })?;
                    if !obs.matches(want) {
                        return Err(Error::ArtifactModified {
                            input: decl.name.clone(),
                            path: want.path.as_str().to_string(),
                        });
                    }
                    Some(want.clone())
                }
                None => {
                    if decl.required {
                        return Err(Error::InputUnavailable {
                            input: decl.name.clone(),
                            node: src.clone(),
                        });
                    }
                    None
                }
            },
        };
        bound.insert(decl.name.clone(), artifact);
    }
    Ok((bound, stats))
}

/// `attempt submit` 第 1 到 6 步。
///
/// 1. Attempt 必须 `Running`，否则 `Error::AttemptNotRunning`。
/// 2. `summary` ≤ 4096 字节，否则 `Error::SummaryTooLong`。
/// 3. `check_outputs`。任一失败整体 `Err`，状态不变。
/// 4. 记录输出 `ArtifactRef`，`status = Succeeded`，`ended_at = ctx.now`。
/// 5. `status_after_success` 定 Work 状态（含 `gate`）。
/// 6. 回复 `Reply::AttemptSubmitted`；效果 `SealOutputs`、`RefreshStatusCard`。
fn decide_submit(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    summary: &str,
    observed_outputs: &BTreeMap<String, Option<ObservedFile>>,
    ctx: &Context,
) -> Result<Decision> {
    let not_running = || Error::AttemptNotRunning {
        attempt: attempt.clone(),
    };
    let prev = state.attempt(attempt).ok_or_else(not_running)?;
    if prev.status != AttemptStatus::Running {
        return Err(not_running());
    }
    // 协议 attempt submit 第 2 步：超限报 `SUMMARY_TOO_LONG`，不是通用的 `TEXT_TOO_LONG`。
    let summary_text = Summary::new(summary, "summary").map_err(|_| Error::SummaryTooLong {
        max: Summary::max_bytes(),
        actual: summary.len(),
    })?;
    let sealed = check_outputs(state, graph, attempt, observed_outputs)?;

    let mut new_state = state.clone();
    if let Some(a) = new_state.attempt_mut(attempt) {
        a.status = AttemptStatus::Succeeded;
        a.outputs = sealed.clone();
        a.summary = Some(summary_text);
        a.ended_at = Some(ctx.now.clone());
    }
    new_state.status = status_after_success(&new_state, graph, true);
    new_state.updated_at = ctx.now.clone();

    Ok(Decision {
        state: new_state,
        effects: vec![
            Effect::SealOutputs {
                paths: sealed.values().map(|r| r.path.clone()).collect(),
            },
            Effect::RefreshStatusCard,
        ],
        reply: Reply::AttemptSubmitted {
            attempt: attempt.clone(),
            outputs: sealed,
        },
    })
}

/// 对照节点 `outputs[]` 校验观察：`required` 且缺 → `Error::OutputMissing`；
/// 超 `max_bytes` → `Error::OutputTooLarge`；可选且缺 → 跳过。返回要封存的引用。
fn check_outputs(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    observed_outputs: &BTreeMap<String, Option<ObservedFile>>,
) -> Result<BTreeMap<String, ArtifactRef>> {
    let def = graph
        .node(&attempt.node)
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("节点 {} 不在图里", attempt.node),
        })?;
    let mut sealed = BTreeMap::new();
    for decl in &def.outputs {
        match observed_outputs.get(&decl.name).and_then(|o| o.as_ref()) {
            Some(obs) => {
                if obs.bytes > decl.max_bytes {
                    return Err(Error::OutputTooLarge {
                        output: decl.name.clone(),
                        max_bytes: decl.max_bytes,
                        actual: obs.bytes,
                    });
                }
                sealed.insert(decl.name.clone(), obs.clone().into_ref());
            }
            None => {
                if decl.required {
                    let path = state.attempt_dir(attempt).join(&decl.path);
                    return Err(Error::OutputMissing {
                        output: decl.name.clone(),
                        path: path.as_str().to_string(),
                    });
                }
            }
        }
    }
    Ok(sealed)
}

/// 节点 Attempt 成功（或门槛刚批准）后 Work 的状态，按协议 `attempt submit` 第 5 步的顺序：
/// `consider_gate && node.gate` → `Blocked(Gate)`；无出边 → `Succeeded`；
/// 每条出边目标都 `visits >= max_visits` → `Blocked(NoLegalEdge)`；否则 `Active`。
fn status_after_success(state: &WorkState, graph: &Graph, consider_gate: bool) -> WorkStatus {
    let Some(def) = graph.node(&state.current.node) else {
        return WorkStatus::Active;
    };
    if consider_gate && def.gate {
        return WorkStatus::Blocked(BlockedReason::Gate);
    }
    let outs = graph.out_edges(&state.current.node);
    if outs.is_empty() {
        return WorkStatus::Succeeded;
    }
    let all_maxed = outs.iter().all(|e| {
        graph
            .node(&e.to)
            .is_some_and(|t| state.visits_of(&e.to) >= t.max_visits)
    });
    if all_maxed {
        return WorkStatus::Blocked(BlockedReason::NoLegalEdge);
    }
    WorkStatus::Active
}

/// `attempt fail`：Attempt 必须 `Running`；`status = Failed`，记 `fail_reason`（≤ 4096）；
/// `retry == max_retries` 时 Work → `Blocked(RetriesExhausted)`。效果 `RefreshStatusCard`。
fn decide_fail(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    reason: &str,
    ctx: &Context,
) -> Result<Decision> {
    let not_running = || Error::AttemptNotRunning {
        attempt: attempt.clone(),
    };
    let prev = state.attempt(attempt).ok_or_else(not_running)?;
    if prev.status != AttemptStatus::Running {
        return Err(not_running());
    }
    // 同 submit 第 2 步：超限报 `SUMMARY_TOO_LONG`。
    let reason_text = Summary::new(reason, "reason").map_err(|_| Error::SummaryTooLong {
        max: Summary::max_bytes(),
        actual: reason.len(),
    })?;
    let max_retries = graph
        .node(&attempt.node)
        .map(|d| d.max_retries)
        .unwrap_or(0);

    let mut new_state = state.clone();
    if let Some(a) = new_state.attempt_mut(attempt) {
        a.status = AttemptStatus::Failed;
        a.fail_reason = Some(reason_text);
        a.ended_at = Some(ctx.now.clone());
    }
    if attempt.retry >= max_retries {
        new_state.status = WorkStatus::Blocked(BlockedReason::RetriesExhausted);
    }
    new_state.updated_at = ctx.now.clone();

    Ok(Decision {
        state: new_state,
        effects: vec![Effect::RefreshStatusCard],
        reply: Reply::AttemptFailed {
            attempt: attempt.clone(),
        },
    })
}

/// `gate approve`：Work 必须 `Blocked(Gate)` 且 `node == current.node`，否则 `Error::IllegalNext`。
/// 记 `Approval`，再用 `status_after_success(.., consider_gate = false)` 定状态。效果 `RefreshStatusCard`。
fn decide_approve(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
    ctx: &Context,
) -> Result<Decision> {
    let next = legal_next(state, graph);
    let illegal = || Error::IllegalNext {
        requested: format!("gate approve {node}"),
        next: next
            .iter()
            .map(|op| op.to_command_line(&state.work_id))
            .collect(),
    };
    if state.status != WorkStatus::Blocked(BlockedReason::Gate) || node != &state.current.node {
        return Err(illegal());
    }

    let mut new_state = state.clone();
    new_state.approvals.push(Approval {
        node: node.clone(),
        occurrence: new_state.current.n,
        by: ctx.principal.clone(),
        at: ctx.now.clone(),
    });
    new_state.status = status_after_success(&new_state, graph, false);
    new_state.updated_at = ctx.now.clone();

    Ok(Decision {
        state: new_state,
        effects: vec![Effect::RefreshStatusCard],
        reply: Reply::GateApproved {
            node: node.clone(),
            occurrence: state.current.n,
        },
    })
}

/// `work cancel`：`status = Cancelled`，`Running` 的 Attempt 保持原样。效果 `RefreshStatusCard`。
fn decide_cancel(state: &WorkState, ctx: &Context) -> Result<Decision> {
    let mut new_state = state.clone();
    new_state.status = WorkStatus::Cancelled;
    new_state.updated_at = ctx.now.clone();

    Ok(Decision {
        state: new_state,
        effects: vec![Effect::RefreshStatusCard],
        reply: Reply::Cancelled,
    })
}

/// runtime 在 `attempt begin` 前调用：本节点每个输入当前应观察的路径。
/// `None` 表示可选输入的上游尚无产出，或来源是 `engine.stats`（引擎自己生成，不观察）。
/// 规则与 `bind_inputs` 相同，只是不比摘要。
pub fn input_paths_for(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
) -> Result<BTreeMap<String, Option<AbsPath>>> {
    let def = graph.node(node).ok_or_else(|| Error::InvalidRequest {
        reason: format!("节点 {node} 不在图里"),
    })?;
    let mut paths = BTreeMap::new();
    for decl in &def.inputs {
        let path = match &decl.from {
            InputSource::Start { key } => state.inputs.get(key).map(|r| r.path.clone()),
            InputSource::Resource { path } => Some(state.workbook_dir().join(path)),
            InputSource::EngineStats => None,
            InputSource::Node { node: src, output } => state
                .latest_succeeded_of(src)
                .and_then(|a| a.outputs.get(output))
                .map(|r| r.path.clone()),
        };
        paths.insert(decl.name.clone(), path);
    }
    Ok(paths)
}

/// `engine.stats` 绑定：`render_stats_json` 序列化成 JSON，写到 `attempt_dir/stats.json`。
/// 序列化失败就报错，不造一份假 stats（core 没有存储类错误码，先归入 `InvalidRequest`）。
fn engine_stats_artifact(
    state: &WorkState,
    graph: &Graph,
    attempt_id: &AttemptId,
) -> Result<(ArtifactRef, String)> {
    let stats = render_stats_json(state, graph);
    let content = serde_json::to_string(&stats).map_err(|e| Error::InvalidRequest {
        reason: format!("engine.stats 序列化失败：{e}"),
    })?;
    let path = state.attempt_dir(attempt_id).join_segment("stats.json");
    let artifact = ArtifactRef {
        sha256: Sha256Hex::of_bytes(content.as_bytes()),
        bytes: content.len() as u64,
        path,
    };
    Ok((artifact, content))
}

/// runtime 在 `attempt submit` 前调用：本 Attempt 每个声明输出的目标路径 `attempt_dir/<path>`。
pub fn output_paths_for(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
) -> Result<BTreeMap<String, AbsPath>> {
    let def = graph
        .node(&attempt.node)
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("节点 {} 不在图里", attempt.node),
        })?;
    let dir = state.attempt_dir(attempt);
    let mut paths = BTreeMap::new();
    for decl in &def.outputs {
        paths.insert(decl.name.clone(), dir.join(&decl.path));
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::EdgeKind;
    use crate::ids::WorkName;
    use crate::testkit::{self, Fixture};
    use crate::work::next::NextOp;
    use crate::work::state::{AttemptStatus, BlockedReason};
    use crate::work::{Effect, Reply, legal_next};

    fn node(s: &str) -> NodeId {
        NodeId::new(s).unwrap()
    }

    /// 用给定 manifest 与 flow 文本编图并起一个 Work（无起始输入），返回图与 start 的决策。
    fn start_texts(manifest_text: &str, flow_text: &str) -> (Graph, Decision) {
        let manifest = crate::workbook::parse_manifest(manifest_text).unwrap();
        let def = crate::flow::parse_flow(flow_text).unwrap();
        let graph =
            crate::flow::compile(&def, &manifest, &crate::flow::ResourceIndex::default()).unwrap();
        let cmd = Command::Start {
            work_id: crate::ids::WorkId::new("2026-09-24", 1, &WorkName::normalize("t").unwrap())
                .unwrap(),
            name: WorkName::normalize("t").unwrap(),
            workbook: crate::work::WorkbookRef {
                id: manifest.id.clone(),
                version: manifest.version.clone(),
                digest: Sha256Hex::of_bytes(b"fixture-workbook"),
            },
            flow: crate::ids::FlowId::new("default").unwrap(),
            work_dir: AbsPath::new("/sheltie-test/works/2026-09-24-001-t").unwrap(),
            inputs: BTreeMap::new(),
        };
        let d = decide(None, &graph, &cmd, &testkit::ctx()).unwrap();
        (graph, d)
    }

    // ── T06 Start ─────────────────────────────────────────────

    // Task: T06
    #[test]
    fn start_sets_current_to_entry_occurrence_1() {
        let mut fx = Fixture::article_review();
        let d = fx.start(&[("topic", "hello")]).unwrap();
        assert_eq!(d.state.current.node.as_str(), "draft");
        assert_eq!(d.state.current.n, 1);
        assert_eq!(d.state.visits_of(&node("draft")), 1);
        assert_eq!(d.state.status, WorkStatus::Active);
        assert!(d.state.attempts.is_empty());
    }

    // Task: T06
    #[test]
    fn start_rejects_missing_start_input_key() {
        let mut fx = Fixture::article_review();
        assert!(
            matches!(fx.start(&[]), Err(Error::InputMissing { missing, .. }) if missing == vec!["topic".to_string()])
        );
    }

    // Task: T06
    #[test]
    fn start_rejects_extra_start_input_key() {
        let mut fx = Fixture::article_review();
        assert!(
            matches!(fx.start(&[("topic", "a"), ("bonus", "b")]), Err(Error::InputMissing { extra, .. }) if extra == vec!["bonus".to_string()])
        );
    }

    // Task: T06
    #[test]
    fn start_next_is_begin_entry_and_cancel() {
        let mut fx = Fixture::article_review();
        let d = fx.start(&[("topic", "hello")]).unwrap();
        let next = legal_next(&d.state, &fx.graph);
        assert_eq!(next.len(), 2);
        assert!(
            matches!(&next[0], NextOp::BeginAttempt { node, edge: None, .. } if node.as_str() == "draft")
        );
        assert_eq!(next[1], NextOp::Cancel);
    }

    // Task: T06
    #[test]
    fn start_records_workbook_ref_and_frozen_inputs() {
        let mut fx = Fixture::article_review();
        let d = fx.start(&[("topic", "hello")]).unwrap();
        assert_eq!(d.state.workbook.id.as_str(), "article-review");
        assert_eq!(d.state.inputs.len(), 1);
        assert_eq!(
            d.state.inputs["topic"].sha256,
            crate::digest::Sha256Hex::of_bytes(b"hello")
        );
        assert_eq!(d.state.name, WorkName::normalize("t").unwrap());
        assert!(matches!(d.reply, Reply::Started { .. }));
    }

    // ── M1 复核 O1：work start 回复 Workbook 全量 requires，按 manifest 声明顺序 ─────

    // Task: T06
    #[test]
    fn start_requires_is_full_manifest_list_in_declaration_order() {
        let (_graph, d) = start_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"单节点\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"beta\"\n[[requires]]\nkind = \"mcp\"\nname = \"alpha\"\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = { text = \"做这一件事。\" }\nrequires = [\"mcp:alpha\"]\n",
        );
        let Reply::Started { requires, .. } = d.reply else {
            panic!("应当是 Reply::Started");
        };
        // skill:beta 没有任何节点引用，也必须在清单里；顺序是 manifest 声明顺序（不是排序后的）。
        let ids: Vec<_> = requires
            .iter()
            .map(|r| (r.kind.as_str(), r.name.as_str()))
            .collect();
        assert_eq!(ids, vec![("skill", "beta"), ("mcp", "alpha")]);
    }

    // Task: T06
    #[test]
    fn start_requires_carry_manifest_declaration_as_is() {
        let (_graph, d) = start_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"单节点\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"company-api\"\nversion = \"2.1.0\"\ndigest = \"sha256:abababababababababababababababababababababababababababababababab\"\nsource = \"https://example.com/company-api\"\n[[requires]]\nkind = \"mcp\"\nname = \"db\"\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = { text = \"做这一件事。\" }\n",
        );
        // 回复里的每一项就是 manifest 的那条声明：没声明的字段是 null，不省略、不拼成 `kind:name`。
        // digest 和引擎其他回复一样是裸 64 位十六进制；`sha256:` 前缀只是 manifest 的书写格式。
        let reply = serde_json::to_value(&d.reply).unwrap();
        assert_eq!(
            reply["requires"],
            serde_json::json!([
                {
                    "kind": "skill",
                    "name": "company-api",
                    "version": "2.1.0",
                    "digest": "abababababababababababababababababababababababababababababababab",
                    "source": "https://example.com/company-api"
                },
                { "kind": "mcp", "name": "db", "version": null, "digest": null, "source": null }
            ])
        );
    }

    // ── T07 BeginAttempt ──────────────────────────────────────

    // Task: T07
    #[test]
    fn begin_on_entry_creates_running_attempt_with_frozen_inputs() {
        let mut fx = Fixture::article_review().started();
        let d = fx.begin("draft").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert_eq!(a.status, AttemptStatus::Running);
        assert_eq!(a.id.to_string(), "draft#1.0");
        assert_eq!(
            a.inputs["topic"].as_ref().unwrap().sha256,
            fx.state().inputs["topic"].sha256
        );
        assert_eq!(a.entered_from, None);
    }

    // Task: T07
    #[test]
    fn begin_rejects_node_not_in_next() {
        let mut fx = Fixture::article_review().started();
        let err = fx.begin("review").unwrap_err();
        match err {
            Error::IllegalNext { requested, next } => {
                assert!(requested.contains("review"));
                assert_eq!(
                    next,
                    legal_next(fx.state(), &fx.graph)
                        .iter()
                        .map(|n| n.to_command_line(&fx.state().work_id))
                        .collect::<Vec<_>>()
                );
            }
            other => panic!("{other:?}"),
        }
    }

    // Task: T07
    #[test]
    fn begin_via_edge_increments_visits_and_occurrence() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "初稿完成").unwrap();
        let d = fx.begin("review").unwrap();
        assert_eq!(d.state.current.to_string(), "review#1");
        assert_eq!(d.state.visits_of(&node("review")), 1);
        let a = d.state.latest_attempt_of_current().unwrap();
        assert_eq!(
            a.entered_from,
            Some((testkit::occ("draft", 1), EdgeKind::Main))
        );
    }

    // Task: T07
    #[test]
    fn begin_records_entered_from_occurrence_and_edge_kind() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        let d = fx.begin("draft").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert_eq!(a.id.to_string(), "draft#2.0");
        assert_eq!(
            a.entered_from,
            Some((testkit::occ("review", 1), EdgeKind::Back))
        );
    }

    // Task: T07
    #[test]
    fn retry_keeps_entered_from_of_first_attempt() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.fail("review#1.0", "崩了").unwrap();
        let d = fx.begin("review").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert_eq!(a.id.to_string(), "review#1.1");
        assert_eq!(
            a.entered_from,
            Some((testkit::occ("draft", 1), EdgeKind::Main))
        );
    }

    // Task: T07
    #[test]
    fn begin_filters_edges_whose_target_hit_max_visits() {
        // review.max_visits = 3；到第三次 review 成功后，back 边的目标 draft（max_visits 3）已满，main 边仍在。
        let mut fx = Fixture::article_review().started();
        for _ in 0..3 {
            fx.run_to_review_done_not_passing();
            if fx.state().visits_of(&node("draft")) < 3 {
                fx.begin("draft").unwrap();
                fx.submit_ok(
                    &format!("draft#{}.0", fx.state().visits_of(&node("draft"))),
                    "改了",
                )
                .unwrap();
            }
        }
        let next = legal_next(fx.state(), &fx.graph);
        assert!(!next.iter().any(|n| n.is_begin_of(&node("draft"))));
        assert!(next.iter().any(|n| n.is_begin_of(&node("publish"))));
    }

    // Task: T07
    #[test]
    fn begin_rejects_modified_upstream_artifact() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.tamper_output("draft#1.0", "article");
        assert!(
            matches!(fx.begin("review"), Err(Error::ArtifactModified { input, .. }) if input == "article")
        );
    }

    // Task: T07
    #[test]
    fn begin_rejects_upstream_without_succeeded_attempt() {
        // 用一张自造的图：b 必需 a 的输出，但有边 a -> b 且 a 从未成功。构造方法见 testkit。
        let mut fx = Fixture::two_step_with_required_input_but_edge_before_success();
        assert!(matches!(
            fx.begin("second"),
            Err(Error::InputUnavailable { .. })
        ));
    }

    // Task: T07
    #[test]
    fn begin_leaves_optional_input_unbound_when_upstream_has_no_attempt() {
        let mut fx = Fixture::spec_dev().started_with(&[("request", "r"), ("project", "/p")]);
        let d = fx.begin("spec").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert_eq!(a.inputs["decision"], None);
        assert!(a.inputs["request"].is_some());
    }

    // Task: T07
    #[test]
    fn begin_binds_optional_input_when_upstream_succeeded_later() {
        let mut fx = Fixture::spec_dev().started_with(&[("request", "r"), ("project", "/p")]);
        fx.run_spec_dev_to_plan_review_returning("修改规格");
        let d = fx.begin("spec").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert!(a.inputs["decision"].is_some());
    }

    // Task: T07
    #[test]
    fn begin_after_failed_attempt_increments_retry_not_occurrence() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "崩").unwrap();
        let d = fx.begin("draft").unwrap();
        assert_eq!(
            d.state.latest_attempt_of_current().unwrap().id.to_string(),
            "draft#1.1"
        );
        assert_eq!(d.state.visits_of(&node("draft")), 1);
    }

    // Task: T07
    #[test]
    fn begin_binds_resource_input_under_frozen_workbook_dir() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        let d = fx.begin("review").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        let checklist = a.inputs["checklist"].as_ref().unwrap();
        assert!(
            checklist
                .path
                .as_str()
                .starts_with(fx.state().workbook_dir().as_str())
        );
        assert!(
            checklist
                .path
                .as_str()
                .ends_with("resources/review-checklist.md")
        );
    }

    // Task: T07
    #[test]
    fn begin_reply_lists_node_requires() {
        let mut fx = Fixture::with_requires();
        let d = fx.begin("only").unwrap();
        let Reply::AttemptBegun { requires, .. } = d.reply else {
            panic!("应当是 Reply::AttemptBegun");
        };
        assert_eq!(requires.len(), 1);
        assert_eq!(requires[0].kind.as_str(), "skill");
        assert_eq!(requires[0].name, "company-api");
    }

    // Task: T07
    #[test]
    fn begin_binds_engine_stats_and_emits_write_file() {
        let mut fx = Fixture::with_engine_stats_input();
        let d = fx.begin("only").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        let stats = a.inputs["stats"].as_ref().unwrap();
        assert!(
            stats
                .path
                .as_str()
                .ends_with("attempts/only/1/0/stats.json")
        );
        let written = d.effects.iter().find_map(|e| match e {
            Effect::WriteFile { path, content } if path == &stats.path => Some(content.clone()),
            _ => None,
        });
        let content = written.expect("应有 WriteFile 效果");
        assert_eq!(
            stats.sha256,
            crate::digest::Sha256Hex::of_bytes(content.as_bytes())
        );
        assert_eq!(stats.bytes, content.len() as u64);
        assert!(content.contains("\"nodes\""));
    }

    // ── M1 复核 O6：多个 engine.stats 输入共享一份 stats.json，只算一次、只写一次 ─────

    // Task: T07
    #[test]
    fn begin_with_two_engine_stats_inputs_writes_one_stats_json() {
        let (graph, d0) = start_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"单节点\"\nflows = [\"flows/default.toml\"]\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = { text = \"做这一件事。\" }\ninputs = [{ name = \"s1\", from = \"engine.stats\" }, { name = \"s2\", from = \"engine.stats\" }]\noutputs = [{ name = \"out\", path = \"out.md\" }]\n",
        );
        let d = decide(
            Some(&d0.state),
            &graph,
            &Command::BeginAttempt {
                node: node("only"),
                observed_inputs: BTreeMap::new(),
                instruction_text: "做这一件事。".to_string(),
            },
            &testkit::ctx(),
        )
        .unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        let s1 = a.inputs["s1"].as_ref().unwrap();
        let s2 = a.inputs["s2"].as_ref().unwrap();
        assert_eq!(s1, s2);
        assert_eq!(
            d.effects
                .iter()
                .filter(|e| matches!(e, Effect::WriteFile { .. }))
                .count(),
            1
        );
    }

    // Task: T07
    #[test]
    fn begin_emits_write_brief_effect() {
        let mut fx = Fixture::article_review().started();
        let d = fx.begin("draft").unwrap();
        assert!(d.effects.iter().any(|e| matches!(e, Effect::WriteBrief { path, content } if path.as_str().ends_with("attempts/draft/1/0/brief.md") && content.contains("# 任务书"))));
        assert!(d.effects.contains(&Effect::RefreshStatusCard));
    }

    // ── T08 Submit / Fail ─────────────────────────────────────

    // Task: T08
    #[test]
    fn submit_marks_attempt_succeeded_and_records_outputs() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let d = fx.submit_ok("draft#1.0", "初稿完成").unwrap();
        let a = d
            .state
            .attempt(&AttemptId::parse("draft#1.0").unwrap())
            .unwrap();
        assert_eq!(a.status, AttemptStatus::Succeeded);
        assert!(a.outputs.contains_key("article"));
        assert_eq!(a.summary.as_ref().unwrap().as_str(), "初稿完成");
        assert!(
            d.effects
                .iter()
                .any(|e| matches!(e, Effect::SealOutputs { paths } if paths.len() == 1))
        );
    }

    // Task: T08
    #[test]
    fn submit_rejects_when_attempt_not_running() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        assert!(matches!(
            fx.submit_ok("draft#1.0", "again"),
            Err(Error::AttemptNotRunning { .. })
        ));
    }

    // Task: T08
    #[test]
    fn submit_rejects_summary_over_4096_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        assert!(matches!(
            fx.submit_ok("draft#1.0", &"x".repeat(4097)),
            Err(Error::SummaryTooLong { .. })
        ));
    }

    // Task: T08
    #[test]
    fn submit_rejects_missing_required_output() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let err = fx
            .submit_with(&AttemptId::parse("draft#1.0").unwrap(), "ok", &[])
            .unwrap_err();
        assert!(matches!(err, Error::OutputMissing { output, .. } if output == "article"));
        assert_eq!(
            fx.state().latest_attempt_of_current().unwrap().status,
            AttemptStatus::Running
        );
    }

    // Task: T08
    #[test]
    fn submit_accepts_missing_optional_output() {
        let mut fx = Fixture::with_optional_output();
        fx.begin("only").unwrap();
        let d = fx
            .submit_with(
                &AttemptId::parse("only#1.0").unwrap(),
                "ok",
                &[("must", 10)],
            )
            .unwrap();
        assert!(!d.state.attempts[0].outputs.contains_key("maybe"));
    }

    // Task: T08
    #[test]
    fn submit_rejects_output_over_max_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let err = fx
            .submit_with(
                &AttemptId::parse("draft#1.0").unwrap(),
                "ok",
                &[("article", 262_145)],
            )
            .unwrap_err();
        assert!(matches!(
            err,
            Error::OutputTooLarge {
                max_bytes: 262_144,
                ..
            }
        ));
    }

    // Task: T08
    #[test]
    fn submit_on_gate_node_blocks_work() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        let d = fx.submit_ok("notes#1.0", "写好了").unwrap();
        assert_eq!(d.state.status, WorkStatus::Blocked(BlockedReason::Gate));
    }

    // Task: T08
    #[test]
    fn submit_on_terminal_node_succeeds_work() {
        let mut fx = Fixture::two_step().started_with(&[("topic", "t")]);
        fx.begin("outline").unwrap();
        fx.submit_ok("outline#1.0", "ok").unwrap();
        fx.begin("summary").unwrap();
        let d = fx.submit_ok("summary#1.0", "ok").unwrap();
        assert_eq!(d.state.status, WorkStatus::Succeeded);
    }

    // Task: T08
    #[test]
    fn submit_on_terminal_gate_node_blocks_not_succeeds() {
        let mut fx = Fixture::single_gated_terminal();
        fx.begin("only").unwrap();
        let d = fx.submit_ok("only#1.0", "ok").unwrap();
        assert_eq!(d.state.status, WorkStatus::Blocked(BlockedReason::Gate));
    }

    // Task: T08
    #[test]
    fn submit_when_every_out_edge_target_hit_max_visits_blocks_no_legal_edge() {
        let mut fx = Fixture::article_review_with_review_max_visits_1().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "不通过").unwrap();
        fx.begin("draft").unwrap();
        let d = fx.submit_ok("draft#2.0", "改了").unwrap();
        assert_eq!(
            d.state.status,
            WorkStatus::Blocked(BlockedReason::NoLegalEdge)
        );
    }

    // Task: T08
    #[test]
    fn fail_marks_attempt_failed_and_allows_retry() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let d = fx.fail("draft#1.0", "超时").unwrap();
        assert_eq!(d.state.attempts[0].status, AttemptStatus::Failed);
        assert_eq!(d.state.status, WorkStatus::Active);
        assert!(
            legal_next(&d.state, &fx.graph)
                .iter()
                .any(|n| n.is_begin_of(&node("draft")))
        );
    }

    // Task: T08
    #[test]
    fn fail_at_max_retries_blocks_work() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "一").unwrap();
        fx.begin("draft").unwrap();
        let d = fx.fail("draft#1.1", "二").unwrap();
        assert_eq!(
            d.state.status,
            WorkStatus::Blocked(BlockedReason::RetriesExhausted)
        );
        assert_eq!(legal_next(&d.state, &fx.graph), vec![NextOp::Cancel]);
    }

    // Task: T08
    #[test]
    fn next_after_success_lists_out_edges_with_kind() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "看完了").unwrap();
        let next = legal_next(fx.state(), &fx.graph);
        assert!(
            matches!(&next[0], NextOp::BeginAttempt { node, edge: Some(EdgeKind::Main), .. } if node.as_str() == "publish")
        );
        assert!(
            matches!(&next[1], NextOp::BeginAttempt { node, edge: Some(EdgeKind::Back), .. } if node.as_str() == "draft")
        );
        assert_eq!(next[2], NextOp::Cancel);
    }

    // Task: T08
    #[test]
    fn next_when_blocked_gate_has_only_approve_and_cancel() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "ok").unwrap();
        let next = legal_next(fx.state(), &fx.graph);
        assert_eq!(
            next,
            vec![
                NextOp::ApproveGate {
                    node: node("notes")
                },
                NextOp::Cancel
            ]
        );
    }

    // ── T09 Approve / Cancel ──────────────────────────────────

    // Task: T09
    #[test]
    fn approve_unblocks_and_records_principal_and_time() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "ok").unwrap();
        let d = fx.approve("notes").unwrap();
        assert_eq!(d.state.status, WorkStatus::Active);
        assert_eq!(d.state.approvals[0].by, testkit::principal());
        assert_eq!(d.state.approvals[0].at, testkit::now());
        assert!(
            legal_next(&d.state, &fx.graph)
                .iter()
                .any(|n| n.is_begin_of(&node("archive")))
        );
    }

    // Task: T09
    #[test]
    fn approve_rejects_when_not_blocked_on_gate() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        assert!(matches!(
            fx.approve("notes"),
            Err(Error::IllegalNext { .. })
        ));
    }

    // Task: T09
    #[test]
    fn approve_rejects_wrong_node() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "ok").unwrap();
        assert!(matches!(
            fx.approve("archive"),
            Err(Error::IllegalNext { .. })
        ));
    }

    // Task: T09
    #[test]
    fn approve_on_terminal_node_succeeds_work() {
        let mut fx = Fixture::single_gated_terminal();
        fx.begin("only").unwrap();
        fx.submit_ok("only#1.0", "ok").unwrap();
        let d = fx.approve("only").unwrap();
        assert_eq!(d.state.status, WorkStatus::Succeeded);
    }

    // Task: T09
    #[test]
    fn cancel_from_active_and_blocked() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let d = fx.cancel().unwrap();
        assert_eq!(d.state.status, WorkStatus::Cancelled);
        assert_eq!(
            d.state.attempts[0].status,
            AttemptStatus::Running,
            "取消不伪造 Attempt 结束"
        );

        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "ok").unwrap();
        assert_eq!(fx.cancel().unwrap().state.status, WorkStatus::Cancelled);
    }

    // Task: T09
    #[test]
    fn terminal_work_rejects_every_command_with_work_terminal() {
        let mut fx = Fixture::article_review().started();
        fx.cancel().unwrap();
        assert!(matches!(fx.begin("draft"), Err(Error::WorkTerminal { .. })));
        assert!(matches!(fx.cancel(), Err(Error::WorkTerminal { .. })));
        assert!(matches!(
            fx.approve("draft"),
            Err(Error::WorkTerminal { .. })
        ));
        assert!(legal_next(fx.state(), &fx.graph).is_empty());
    }

    // ── M1 补测（上限边界与重试路径） ─────────────────────────

    // Task: T07
    #[test]
    fn retry_binds_engine_stats_under_retry_dir() {
        let mut fx = Fixture::with_engine_stats_input();
        fx.begin("only").unwrap();
        fx.fail("only#1.0", "再来").unwrap();
        let d = fx.begin("only").unwrap();
        let stats = d.state.latest_attempt_of_current().unwrap().inputs["stats"]
            .clone()
            .unwrap();
        assert!(
            stats
                .path
                .as_str()
                .ends_with("attempts/only/1/1/stats.json")
        );
        assert!(
            d.effects
                .iter()
                .any(|e| matches!(e, Effect::WriteFile { path, .. } if path == &stats.path))
        );
    }

    // Task: T08
    #[test]
    fn submit_accepts_summary_of_exactly_4096_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let d = fx.submit_ok("draft#1.0", &"a".repeat(4096)).unwrap();
        assert_eq!(
            d.state.attempts[0].summary.as_ref().unwrap().as_str().len(),
            4096
        );
    }

    // Task: T08
    #[test]
    fn submit_accepts_output_of_exactly_max_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let id = AttemptId::parse("draft#1.0").unwrap();
        let d = fx.submit_with(&id, "ok", &[("article", 262_144)]).unwrap();
        assert_eq!(d.state.attempts[0].outputs["article"].bytes, 262_144);
    }

    // Task: T08
    #[test]
    fn fail_reason_limit_is_4096_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        assert!(matches!(
            fx.fail("draft#1.0", &"a".repeat(4097)),
            Err(Error::SummaryTooLong {
                max: 4096,
                actual: 4097
            })
        ));
        let d = fx.fail("draft#1.0", &"a".repeat(4096)).unwrap();
        assert_eq!(d.state.attempts[0].status, AttemptStatus::Failed);
    }
}
