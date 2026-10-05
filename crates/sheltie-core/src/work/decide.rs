//! The state machine's sole entry `decide`, with one private function per command.
//! Rules correspond to specs/contracts/protocol.md §3.

use std::collections::BTreeMap;

use super::command::{Command, Context, Decision, Effect, ObservedFile, Reply};
use super::next::legal_next;
use super::render::{render_brief, render_stats_json};
use super::start::validate_start_inputs;
use super::state::{
    Approval, ArtifactRef, Attempt, AttemptStatus, BlockedReason, Occurrence, WorkState, WorkStatus,
};
use crate::digest::Sha256Hex;
use crate::error::{Error, Result};
use crate::flow::{Graph, InputSource, NodeDef};
use crate::ids::{AttemptId, NodeId};
use crate::path::AbsPath;
use crate::text::Summary;
use crate::workbook::HostRequire;

/// Apply one command to a Work.
///
/// None state is legal only for Command::Start; other commands first pass guard_not_terminal,
/// then dispatch to private handlers; Decision.state is the complete new state.
pub fn decide(
    state: Option<&WorkState>,
    graph: &Graph,
    cmd: &Command,
    ctx: &Context,
) -> Result<Decision> {
    match (state, cmd) {
        (None, Command::Start { .. }) => decide_start(graph, cmd, ctx),
        (None, other) => Err(Error::InvalidRequest {
            reason: format!("{} requires an existing Work", other.name()),
        }),
        (Some(_), Command::Start { .. }) => Err(Error::InvalidRequest {
            reason: "Work already exists; cannot start again".to_string(),
        }),
        (Some(state), cmd) => {
            guard_not_terminal(state)?;
            match cmd {
                Command::Start { .. } => unreachable!("Handled above"),
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
                Command::ReplaceAttempt {
                    attempt,
                    reason,
                    observed_inputs,
                    instruction_text,
                } => decide_replace(
                    state,
                    graph,
                    attempt,
                    reason,
                    observed_inputs,
                    instruction_text,
                    ctx,
                ),
                Command::ApproveGate { node } => decide_approve(state, graph, node, ctx),
                Command::Cancel => decide_cancel(state, ctx),
            }
        }
    }
}

/// Terminal Works reject all commands with Error::WorkTerminal.
fn guard_not_terminal(state: &WorkState) -> Result<()> {
    if state.status.is_terminal() {
        return Err(Error::WorkTerminal {
            status: state.status.to_string(),
        });
    }
    Ok(())
}

fn increment_blocked_count(state: &mut WorkState) -> Result<()> {
    state.blocked_count =
        state
            .blocked_count
            .checked_add(1)
            .ok_or_else(|| Error::InvalidRequest {
                reason: "Cumulative blocking count overflow".to_string(),
            })?;
    Ok(())
}

/// work start steps 2, 6, and 7: require exact equality of input keys and all graph start.<key> references
/// (missing/extra keys yield InputMissing); initialize current = entry#1, visits[entry] = 1,
/// status = Active, Reply::Started, and RefreshStatusCard.
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
            reason: "decide_start accepts only work start".to_string(),
        });
    };

    // Require all start.<key> references to equal supplied keys (GF-30 shared validation);
    // runtime preflight and workbook show share the result without independent recomputation.
    validate_start_inputs(graph, inputs.keys())?;

    let entry = graph.entry().clone();
    let mut visits = BTreeMap::new();
    visits.insert(entry.clone(), 1);

    // Host resource list: all Workbook requires in manifest declaration order, preserving
    // version/digest/source (protocol work start step 8), including resources unused by this graph.
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
        blocked_count: 0,
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

/// attempt begin steps 1-5.
///
/// 1. node must appear in a legal_next BeginAttempt item; otherwise IllegalNext
///    (next contains each action's to_command_line result).
/// 2. If node differs from current, increment visits, set current = node#n, and record entered_from.
///    Same-node retry increments number and retains entered_from.
/// 3. `bind_inputs`。
/// 4. Create a Running Attempt with started_at = ctx.now.
/// 5. Reply::AttemptBegun, WriteBrief via render_brief(.., instruction_text), and RefreshStatusCard.
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
        reason: format!("Node {node} is absent from the graph"),
    })?;

    let mut new_state = state.clone();
    let (occ_n, number, entered_from) = if node != &state.current.node {
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
        let number = next_attempt_number(prev)?;
        let entered_from = prev.and_then(|a| a.entered_from.clone());
        (state.current.n, number, entered_from)
    };
    let attempt_id = AttemptId::new(node.clone(), occ_n, number);

    let bound = bind_inputs(state, graph, node, observed_inputs)?;

    let attempt = Attempt {
        id: attempt_id.clone(),
        status: AttemptStatus::Running,
        entered_from,
        inputs: bound,
        outputs: BTreeMap::new(),
        summary: None,
        fail_reason: None,
        replacement_reason: None,
        started_at: ctx.now.clone(),
        ended_at: None,
    };
    new_state.attempts.push(attempt);
    new_state.updated_at = ctx.now.clone();

    let delivery =
        prepare_attempt_delivery(&mut new_state, graph, def, &attempt_id, instruction_text)?;
    Ok(Decision {
        state: new_state,
        effects: delivery.effects,
        reply: Reply::AttemptBegun {
            attempt: attempt_id,
            brief_path: delivery.brief_path,
            output_dir: delivery.output_dir,
            inputs: delivery.inputs,
            outputs: delivery.outputs,
            requires: delivery.requires,
        },
    })
}

fn next_attempt_number(previous: Option<&Attempt>) -> Result<u32> {
    match previous {
        None => Ok(0),
        Some(previous) => previous
            .id
            .number
            .checked_add(1)
            .ok_or_else(|| Error::InvalidRequest {
                reason: "Attempt sequence overflow".to_string(),
            }),
    }
}

pub fn replacement_input_paths_for(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
) -> Result<BTreeMap<String, Option<AbsPath>>> {
    guard_not_terminal(state)?;
    let record = state
        .attempt(attempt)
        .ok_or_else(|| Error::AttemptNotFound {
            attempt: attempt.clone(),
        })?;
    if record.status != AttemptStatus::Running {
        return Err(Error::AttemptNotRunning {
            attempt: attempt.clone(),
        });
    }
    if state.status != WorkStatus::Active
        || state.latest_attempt_of_current().map(|record| &record.id) != Some(attempt)
    {
        return Err(Error::IllegalNext {
            requested: format!("attempt replace {attempt}"),
            next: legal_next(state, graph)
                .iter()
                .map(|operation| operation.to_command_line(&state.work_id))
                .collect(),
        });
    }
    if state.has_replacement_of(&state.current) {
        return Err(Error::ReplacementsExhausted {
            attempt: attempt.clone(),
        });
    }
    let node = graph
        .node(&attempt.node)
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("Node {} is absent from the graph", attempt.node),
        })?;
    if record.inputs.len() != node.inputs().len() {
        return Err(Error::InvalidRequest {
            reason: "Original Attempt input keys differ from declarations".to_string(),
        });
    }
    node.inputs()
        .iter()
        .map(|declaration| {
            let reference =
                record
                    .inputs
                    .get(declaration.name())
                    .ok_or_else(|| Error::InvalidRequest {
                        reason: format!("Original Attempt is missing input {}", declaration.name()),
                    })?;
            let path = if matches!(declaration.source(), InputSource::EngineStats) {
                None
            } else {
                reference.as_ref().map(|reference| reference.path.clone())
            };
            Ok((declaration.name().to_string(), path))
        })
        .collect()
}

fn decide_replace(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    reason: &str,
    observed_inputs: &BTreeMap<String, Option<ObservedFile>>,
    instruction_text: &str,
    ctx: &Context,
) -> Result<Decision> {
    let paths = replacement_input_paths_for(state, graph, attempt)?;
    let reason = Summary::new(reason, "reason").map_err(|_| Error::SummaryTooLong {
        max: Summary::max_bytes(),
        actual: reason.len(),
    })?;
    if paths.keys().ne(observed_inputs.keys()) {
        return Err(Error::InvalidRequest {
            reason: "Replacement observation keys differ from original declarations".to_string(),
        });
    }
    let old = state
        .attempt(attempt)
        .ok_or_else(|| Error::AttemptNotFound {
            attempt: attempt.clone(),
        })?;
    let node = graph
        .node(&attempt.node)
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("Node {} is absent from the graph", attempt.node),
        })?;
    let number = next_attempt_number(Some(old))?;
    let new_id = AttemptId::new(attempt.node.clone(), attempt.occurrence, number);
    for declaration in node.inputs() {
        let observed = observed_inputs
            .get(declaration.name())
            .and_then(Option::as_ref);
        if matches!(declaration.source(), InputSource::EngineStats) {
            if observed.is_some() {
                return Err(Error::InvalidRequest {
                    reason: "Replacement must not supply engine.stats file observations"
                        .to_string(),
                });
            }
        } else {
            match old.inputs.get(declaration.name()).and_then(Option::as_ref) {
                Some(reference) => {
                    if !observed.is_some_and(|file| {
                        file.path == reference.path
                            && file.sha256 == reference.sha256
                            && file.bytes == reference.bytes
                    }) {
                        return Err(Error::ArtifactModified {
                            input: declaration.name().to_string(),
                            path: reference.path.to_string(),
                        });
                    }
                }
                None if observed.is_none() => {}
                None => {
                    return Err(Error::InvalidRequest {
                        reason: format!(
                            "Unbound input {} must not be rebound during replacement",
                            declaration.name()
                        ),
                    });
                }
            }
        }
    }
    let mut new_state = state.clone();
    let old_mut = new_state
        .attempt_mut(attempt)
        .ok_or_else(|| Error::AttemptNotFound {
            attempt: attempt.clone(),
        })?;
    old_mut.status = AttemptStatus::Superseded;
    old_mut.ended_at = Some(ctx.now.clone());
    old_mut.replacement_reason = Some(reason);
    new_state.attempts.push(Attempt {
        id: new_id.clone(),
        status: AttemptStatus::Running,
        entered_from: old.entered_from.clone(),
        inputs: old.inputs.clone(),
        outputs: BTreeMap::new(),
        summary: None,
        fail_reason: None,
        replacement_reason: None,
        started_at: ctx.now.clone(),
        ended_at: None,
    });
    new_state.updated_at = ctx.now.clone();
    let delivery =
        prepare_attempt_delivery(&mut new_state, graph, node, &new_id, instruction_text)?;
    Ok(Decision {
        state: new_state,
        effects: delivery.effects,
        reply: Reply::AttemptReplaced {
            replaced_attempt: attempt.clone(),
            attempt: new_id,
            brief_path: delivery.brief_path,
            output_dir: delivery.output_dir,
            inputs: delivery.inputs,
            outputs: delivery.outputs,
            requires: delivery.requires,
        },
    })
}

struct AttemptDelivery {
    effects: Vec<Effect>,
    brief_path: AbsPath,
    output_dir: AbsPath,
    inputs: BTreeMap<String, Option<AbsPath>>,
    outputs: BTreeMap<String, AbsPath>,
    requires: Vec<HostRequire>,
}

fn prepare_attempt_delivery(
    state: &mut WorkState,
    graph: &Graph,
    node: &NodeDef,
    attempt_id: &AttemptId,
    instruction_text: &str,
) -> Result<AttemptDelivery> {
    let index = state.attempts.len() - 1;
    let mut effects = Vec::new();
    if node
        .inputs()
        .iter()
        .any(|input| matches!(input.source(), InputSource::EngineStats))
    {
        // Statistics include the new Attempt; all stats slots share the same exact bytes.
        let (reference, content) = engine_stats_artifact(state, graph, attempt_id)?;
        for declaration in node
            .inputs()
            .iter()
            .filter(|input| matches!(input.source(), InputSource::EngineStats))
        {
            state.attempts[index]
                .inputs
                .insert(declaration.name().to_string(), Some(reference.clone()));
        }
        effects.push(Effect::WriteFile {
            path: reference.path,
            content,
        });
    }
    let attempt = &state.attempts[index];
    let directory = state.attempt_dir(attempt_id);
    let brief_path = crate::work::layout::brief_path(&directory);
    effects.push(Effect::WriteBrief {
        path: brief_path.clone(),
        content: render_brief(state, graph, attempt, instruction_text),
    });
    effects.push(Effect::RefreshStatusCard);
    let inputs = attempt
        .inputs
        .iter()
        .map(|(name, reference)| {
            (
                name.clone(),
                reference.as_ref().map(|reference| reference.path.clone()),
            )
        })
        .collect();
    let outputs = node
        .outputs
        .iter()
        .map(|output| {
            (
                output.name.clone(),
                crate::work::layout::output_path(&directory, &output.path),
            )
        })
        .collect();
    let requires = graph
        .node_requires(node.id())
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("Node {} is absent from the graph", node.id()),
        })?;
    Ok(AttemptDelivery {
        effects,
        brief_path,
        output_dir: crate::work::layout::outputs_dir(&directory),
        inputs,
        outputs,
        requires,
    })
}

/// Bind node inputs[] in declaration order.
///
/// - Start { key }: use state.inputs[key]; mismatched observed digest yields ArtifactModified.
/// - Resource { path }: use workbook_dir().join(path); require an observation and record its digest on first binding.
/// - EngineStats ignores observations and initially binds None; after creating the Attempt,
///   prepare_attempt_delivery generates statistics including it and fills the binding.
/// - Node { node, output }: use latest_succeeded_of(node).outputs[output];
///   if absent, required inputs yield InputUnavailable, optional inputs bind None;
///   if present, observed digest must match the record, otherwise ArtifactModified.
///
/// Return bindings, with None placeholders for EngineStats.
type BoundInputs = BTreeMap<String, Option<ArtifactRef>>;

fn bind_inputs(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
    observed_inputs: &BTreeMap<String, Option<ObservedFile>>,
) -> Result<BoundInputs> {
    let def = graph.node(node).ok_or_else(|| Error::InvalidRequest {
        reason: format!("Node {node} is absent from the graph"),
    })?;
    let mut bound = BTreeMap::new();
    for decl in &def.inputs {
        let observed = observed_inputs.get(&decl.name).and_then(|o| o.as_ref());
        let artifact = match &decl.from {
            InputSource::Start { key } => {
                let want = state.inputs.get(key).ok_or_else(|| Error::InputMissing {
                    missing: vec![key.clone()],
                    extra: Vec::new(),
                })?;
                Some(bind_frozen_input(&decl.name, want, observed)?)
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
            InputSource::EngineStats => None,
            InputSource::Node { node: src, output } => match state
                .latest_succeeded_of(src)
                .and_then(|a| a.outputs.get(output))
            {
                Some(want) => Some(bind_frozen_input(&decl.name, want, observed)?),
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
    Ok(bound)
}

fn bind_frozen_input(
    name: &str,
    reference: &ArtifactRef,
    observed: Option<&ObservedFile>,
) -> Result<ArtifactRef> {
    if !observed.is_some_and(|file| file.matches(reference)) {
        return Err(Error::ArtifactModified {
            input: name.to_string(),
            path: reference.path.as_str().to_string(),
        });
    }
    Ok(reference.clone())
}

/// attempt submit steps 1-6.
///
/// 1. Require Running, otherwise AttemptNotRunning.
/// 2. summary is at most 4096 bytes, otherwise SummaryTooLong.
/// 3. check_outputs; any failure rejects everything without changing state.
/// 4. Record output ArtifactRefs, status = Succeeded, ended_at = ctx.now.
/// 5. status_after_success sets Work status, including gate handling.
/// 6. Reply::AttemptSubmitted, SealOutputs, and RefreshStatusCard.
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
    // Protocol attempt submit step 2: oversized summaries yield SUMMARY_TOO_LONG, not generic TEXT_TOO_LONG.
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
    // Record cumulative blocking at occurrence (GF-29): gate and no_legal_edge each count once.
    if matches!(
        new_state.status,
        WorkStatus::Blocked(BlockedReason::Gate | BlockedReason::NoLegalEdge)
    ) {
        increment_blocked_count(&mut new_state)?;
    }
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

/// Validate observations against outputs[]: missing required outputs yield OutputMissing;
/// oversized outputs yield OutputTooLarge; skip absent optional outputs. Return references to seal.
fn check_outputs(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    observed_outputs: &BTreeMap<String, Option<ObservedFile>>,
) -> Result<BTreeMap<String, ArtifactRef>> {
    let def = graph
        .node(&attempt.node)
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("Node {} is absent from the graph", attempt.node),
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
                    let path =
                        crate::work::layout::output_path(&state.attempt_dir(attempt), &decl.path);
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

/// Work status after Attempt success or gate approval, in protocol attempt submit step 5 order:
/// consider_gate && node.gate -> Blocked(Gate); no outgoing edges -> Succeeded;
/// all outgoing targets exhausted -> Blocked(NoLegalEdge); otherwise Active.
fn success_status_at(
    graph: &Graph,
    node: &NodeId,
    consider_gate: bool,
    all_maxed: bool,
) -> WorkStatus {
    let Some(def) = graph.node(node) else {
        return WorkStatus::Active;
    };
    if consider_gate && def.gate {
        return WorkStatus::Blocked(BlockedReason::Gate);
    }
    if graph.is_terminal(node) {
        return WorkStatus::Succeeded;
    }
    if all_maxed {
        WorkStatus::Blocked(BlockedReason::NoLegalEdge)
    } else {
        WorkStatus::Active
    }
}

fn status_after_success(state: &WorkState, graph: &Graph, consider_gate: bool) -> WorkStatus {
    let all_maxed = graph.out_edges(&state.current.node).iter().all(|edge| {
        graph
            .node(&edge.to)
            .is_some_and(|target| state.visits_of(&edge.to) >= target.max_visits)
    });
    success_status_at(graph, &state.current.node, consider_gate, all_maxed)
}

/// Failure status depends only on actual failures in this Occurrence and the frozen retry limit.
fn status_after_failure(failures: usize, max_retries: u32) -> WorkStatus {
    if failures > max_retries as usize {
        WorkStatus::Blocked(BlockedReason::RetriesExhausted)
    } else {
        WorkStatus::Active
    }
}

/// Validate necessary historical-reply conditions without reconstructing history from current visits.
pub fn reply_status_matches(
    reply: &Reply,
    status: WorkStatus,
    state: &WorkState,
    graph: &Graph,
) -> bool {
    match reply {
        Reply::AttemptFailed { attempt } => {
            let Some((index, record)) = state
                .attempts
                .iter()
                .enumerate()
                .find(|(_, record)| &record.id == attempt)
            else {
                return false;
            };
            record.status == AttemptStatus::Failed
                && graph.node(&attempt.node).is_some_and(|node| {
                    let failures = state.attempts[..=index]
                        .iter()
                        .filter(|candidate| {
                            candidate.occurrence() == record.occurrence()
                                && candidate.status == AttemptStatus::Failed
                        })
                        .count();
                    status == status_after_failure(failures, node.max_retries)
                })
        }
        Reply::AttemptSubmitted { attempt, .. } => {
            graph.node(&attempt.node).is_some()
                && (status == success_status_at(graph, &attempt.node, true, false)
                    || status == success_status_at(graph, &attempt.node, true, true))
        }
        Reply::GateApproved { node, .. } => {
            graph.node(node).is_some()
                && (status == success_status_at(graph, node, false, false)
                    || status == success_status_at(graph, node, false, true))
        }
        Reply::Started { .. } | Reply::AttemptBegun { .. } | Reply::AttemptReplaced { .. } => {
            status == WorkStatus::Active
        }
        Reply::Cancelled => status == WorkStatus::Cancelled,
    }
}

/// attempt fail requires Running; set Failed and record fail_reason of at most 4096 bytes;
/// actual failures above max_retries block Work with RetriesExhausted. Effect: RefreshStatusCard.
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
    // Like submit step 2, oversized text yields SUMMARY_TOO_LONG.
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
    new_state.status =
        status_after_failure(new_state.failed_count_of(&prev.occurrence()), max_retries);
    if new_state.status == WorkStatus::Blocked(BlockedReason::RetriesExhausted) {
        increment_blocked_count(&mut new_state)?;
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

/// gate approve requires Blocked(Gate) and node == current.node, otherwise IllegalNext.
/// Record Approval, use status_after_success with consider_gate = false, and RefreshStatusCard.
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
    if matches!(
        new_state.status,
        WorkStatus::Blocked(BlockedReason::NoLegalEdge)
    ) {
        increment_blocked_count(&mut new_state)?;
    }
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

/// work cancel sets Cancelled, leaving Running Attempts unchanged; effect RefreshStatusCard.
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

/// Called by runtime before attempt begin; current observation paths for this node's inputs.
/// None means absent optional upstream output or engine.stats, generated by the engine without observation.
/// Same rules as bind_inputs, without digest comparison.
pub fn input_paths_for(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
) -> Result<BTreeMap<String, Option<AbsPath>>> {
    let def = graph.node(node).ok_or_else(|| Error::InvalidRequest {
        reason: format!("Node {node} is absent from the graph"),
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

/// Bind engine.stats by serializing render_stats_json to JSON at attempt_dir/engine/stats.json.
/// Reject serialization failure without fabricated stats; core has no storage errors, so use InvalidRequest.
fn engine_stats_artifact(
    state: &WorkState,
    graph: &Graph,
    attempt_id: &AttemptId,
) -> Result<(ArtifactRef, String)> {
    let stats = render_stats_json(state, graph);
    let content = serde_json::to_string(&stats).map_err(|e| Error::InvalidRequest {
        reason: format!("engine.stats serialization failed: {e}"),
    })?;
    // Engine files are separate from worker outputs (workbook contract §3.2): engine/stats.json.
    let path = crate::work::layout::engine_stats_path(&state.attempt_dir(attempt_id));
    let artifact = ArtifactRef {
        sha256: Sha256Hex::of_bytes(content.as_bytes()),
        bytes: content.len() as u64,
        path,
    };
    Ok((artifact, content))
}

/// Called by runtime before submit; each declared output's path at attempt_dir/outputs/<path>.
pub fn output_paths_for(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
) -> Result<BTreeMap<String, AbsPath>> {
    let def = graph
        .node(&attempt.node)
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("Node {} is absent from the graph", attempt.node),
        })?;
    let dir = state.attempt_dir(attempt);
    let mut paths = BTreeMap::new();
    for decl in &def.outputs {
        paths.insert(
            decl.name.clone(),
            crate::work::layout::output_path(&dir, &decl.path),
        );
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

    // Task: C005-T01
    #[test]
    fn attempt_number_allocation_checks_overflow_without_wraparound() {
        let mut fixture = Fixture::article_review().started();
        fixture.begin("draft").unwrap();
        let mut attempt = fixture.state().attempts[0].clone();
        assert_eq!(next_attempt_number(None).unwrap(), 0);
        assert_eq!(next_attempt_number(Some(&attempt)).unwrap(), 1);
        attempt.id.number = u32::MAX;
        assert!(matches!(
            next_attempt_number(Some(&attempt)),
            Err(Error::InvalidRequest { .. })
        ));
        assert_eq!(attempt.id.number, u32::MAX);
    }

    fn node(s: &str) -> NodeId {
        NodeId::new(s).unwrap()
    }

    /// Compile supplied manifest/flow text and start a Work without inputs; return graph and start decision.
    fn start_texts(manifest_text: &str, flow_text: &str) -> (Graph, Decision) {
        let mut fx = Fixture::from_texts(manifest_text, flow_text, &[]);
        let decision = fx.start(&[]).unwrap();
        (fx.graph, decision)
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

    // ── M1 review O1: work start returns all Workbook requires in manifest declaration order ─────

    // Task: T06
    #[test]
    fn start_requires_is_full_manifest_list_in_declaration_order() {
        let (_graph, d) = start_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"Single node\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"beta\"\n[[requires]]\nkind = \"mcp\"\nname = \"alpha\"\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = { text = \"Do this task.\" }\nrequires = [\"mcp:alpha\"]\n",
        );
        let Reply::Started { requires, .. } = d.reply else {
            panic!("Expected Reply::Started");
        };
        // Include unused skill:beta; preserve manifest declaration order without sorting.
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
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"Single node\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"company-api\"\nversion = \"2.1.0\"\ndigest = \"sha256:abababababababababababababababababababababababababababababababab\"\nsource = \"https://example.com/company-api\"\n[[requires]]\nkind = \"mcp\"\nname = \"db\"\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = { text = \"Do this task.\" }\n",
        );
        // Replies carry manifest declarations; absent fields are null, without omission or flattening to kind:name.
        // Reply digests are bare 64-digit hexadecimal; sha256: is only manifest notation.
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
        fx.submit_ok("draft#1.0", "Draft ready").unwrap();
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
        fx.fail("review#1.0", "Crashed").unwrap();
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
        // review.max_visits = 3; after the third review success draft is exhausted, while main remains legal.
        let mut fx = Fixture::article_review().started();
        for _ in 0..3 {
            fx.run_to_review_done_not_passing();
            if fx.state().visits_of(&node("draft")) < 3 {
                fx.begin("draft").unwrap();
                fx.submit_ok(
                    &format!("draft#{}.0", fx.state().visits_of(&node("draft"))),
                    "Revised",
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
        // Custom graph: b requires a output and has a -> b, but a never succeeded; see testkit.
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
        fx.run_spec_dev_to_plan_review_returning("Revise the specification");
        let d = fx.begin("spec").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert!(a.inputs["decision"].is_some());
    }

    // Task: T07
    #[test]
    fn begin_after_failed_attempt_increments_retry_not_occurrence() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "Crashed").unwrap();
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
            panic!("Expected Reply::AttemptBegun");
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
                .ends_with("attempts/only/occurrence-001/attempt-000/engine/stats.json")
        );
        let written = d.effects.iter().find_map(|e| match e {
            Effect::WriteFile { path, content } if path == &stats.path => Some(content.clone()),
            _ => None,
        });
        let content = written.expect("Expected a WriteFile effect");
        assert_eq!(
            stats.sha256,
            crate::digest::Sha256Hex::of_bytes(content.as_bytes())
        );
        assert_eq!(stats.bytes, content.len() as u64);
        assert!(content.contains("\"nodes\""));
        // stats.json includes the current Attempt (D-29).
        assert!(
            content.contains("\"attempts\":1"),
            "Current Attempt is included: {content}"
        );
    }

    // ── M1 review O6: engine.stats inputs share one stats.json, computed and written once ─────

    // Task: T07
    #[test]
    fn begin_with_two_engine_stats_inputs_writes_one_stats_json() {
        let (graph, d0) = start_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"Single node\"\nflows = [\"flows/default.toml\"]\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = { text = \"Do this task.\" }\ninputs = [{ name = \"s1\", from = \"engine.stats\" }, { name = \"s2\", from = \"engine.stats\" }]\noutputs = [{ name = \"out\", path = \"out.md\" }]\n",
        );
        let d = decide(
            Some(&d0.state),
            &graph,
            &Command::BeginAttempt {
                node: node("only"),
                observed_inputs: BTreeMap::new(),
                instruction_text: "Do this task.".to_string(),
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
        assert!(d.effects.iter().any(|e| matches!(e, Effect::WriteBrief { path, content } if path.as_str().ends_with("attempts/draft/occurrence-001/attempt-000/brief.md") && content.contains("# Brief"))));
        assert!(d.effects.contains(&Effect::RefreshStatusCard));
    }

    // ── T08 Submit / Fail ─────────────────────────────────────

    // ── M1 review: distinguish begin reply requires by both kind and name ─────

    // Task: T07
    #[test]
    fn begin_reply_requires_ignore_crossed_kind_name_pairs() {
        // Manifest declares skill:company-api and mcp:db; the node references only mcp:db.
        // Changing && to || would select a declaration matching only kind or name.
        let (graph, d0) = start_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"Single node\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"db\"\nversion = \"1.0\"\n[[requires]]\nkind = \"mcp\"\nname = \"company-api\"\nversion = \"2.0\"\n[[requires]]\nkind = \"mcp\"\nname = \"db\"\nversion = \"3.0\"\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = { text = \"Do this task.\" }\nrequires = [\"mcp:db\"]\n",
        );
        let d = decide(
            Some(&d0.state),
            &graph,
            &Command::BeginAttempt {
                node: node("only"),
                observed_inputs: BTreeMap::new(),
                instruction_text: "Do this task.".to_string(),
            },
            &testkit::ctx(),
        )
        .unwrap();
        let Reply::AttemptBegun { requires, .. } = d.reply else {
            panic!("Expected Reply::AttemptBegun");
        };
        assert_eq!(requires.len(), 1);
        assert_eq!(
            (
                requires[0].kind.as_str(),
                requires[0].name.as_str(),
                requires[0].version.as_deref()
            ),
            ("mcp", "db", Some("3.0"))
        );
    }

    // Task: T08
    #[test]
    fn submit_marks_attempt_succeeded_and_records_outputs() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let d = fx.submit_ok("draft#1.0", "Draft ready").unwrap();
        let a = d
            .state
            .attempt(&AttemptId::parse("draft#1.0").unwrap())
            .unwrap();
        assert_eq!(a.status, AttemptStatus::Succeeded);
        assert!(a.outputs.contains_key("article"));
        assert_eq!(a.summary.as_ref().unwrap().as_str(), "Draft ready");
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

    // Task: C002-T40
    #[test]
    fn submit_rejects_summary_over_4096_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        assert!(matches!(
            fx.submit_ok("draft#1.0", &"x".repeat(4097)),
            Err(Error::SummaryTooLong {
                max: 4096,
                actual: 4097
            })
        ));
        let d = fx.submit_ok("draft#1.0", &"a".repeat(4096)).unwrap();
        assert_eq!(
            d.state.attempts[0].summary.as_ref().unwrap().as_str().len(),
            4096
        );
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

    // Task: C002-T40
    #[test]
    fn submit_rejects_output_over_max_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let id = AttemptId::parse("draft#1.0").unwrap();
        let err = fx
            .submit_with(&id, "ok", &[("article", 262_145)])
            .unwrap_err();
        assert!(matches!(
            err,
            Error::OutputTooLarge {
                output,
                max_bytes: 262_144,
                actual: 262_145
            } if output == "article"
        ));
        let d = fx.submit_with(&id, "ok", &[("article", 262_144)]).unwrap();
        assert_eq!(d.state.attempts[0].outputs["article"].bytes, 262_144);
    }

    // Task: T08
    #[test]
    fn submit_on_gate_node_blocks_work() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        let d = fx.submit_ok("notes#1.0", "Written").unwrap();
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
        fx.submit_ok("review#1.0", "Rejected").unwrap();
        fx.begin("draft").unwrap();
        let d = fx.submit_ok("draft#2.0", "Revised").unwrap();
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
        let d = fx.fail("draft#1.0", "Timed out").unwrap();
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
        fx.fail("draft#1.0", "First").unwrap();
        fx.begin("draft").unwrap();
        let d = fx.fail("draft#1.1", "Second").unwrap();
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
        fx.submit_ok("review#1.0", "Reviewed").unwrap();
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
            "Cancellation must not fabricate Attempt completion"
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

    // ── M1 additional limit boundaries and retry paths ─────────────────────────

    // Task: T07
    #[test]
    fn retry_binds_engine_stats_under_retry_dir() {
        let mut fx = Fixture::with_engine_stats_input();
        fx.begin("only").unwrap();
        fx.fail("only#1.0", "Retry").unwrap();
        let d = fx.begin("only").unwrap();
        let stats = d.state.latest_attempt_of_current().unwrap().inputs["stats"]
            .clone()
            .unwrap();
        assert!(
            stats
                .path
                .as_str()
                .ends_with("attempts/only/occurrence-001/attempt-001/engine/stats.json")
        );
        assert!(
            d.effects
                .iter()
                .any(|e| matches!(e, Effect::WriteFile { path, .. } if path == &stats.path))
        );
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
