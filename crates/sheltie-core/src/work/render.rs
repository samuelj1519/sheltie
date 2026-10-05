//! Brief and status-card rendering; see `specs/contracts/protocol.md` §4 and §6.
//! Snapshot expectations in `snapshots/` were handwritten in T01 as independent oracles.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::next::{NextOp, legal_next};
use super::state::{
    ArtifactRef, Attempt, AttemptStatus, BlockedReason, Timestamp, WorkState, WorkStatus,
};
use crate::flow::{Graph, InputSource};
use crate::ids::WorkId;
use crate::path::AbsPath;

/// Render an Attempt brief (protocol §4). Handwritten `snapshots/*brief*.snap`
/// files are independent byte-for-byte expectations.
///
/// Lines use `\n`, including one final newline. The header names the node, Work,
/// Occurrence, zero-based Attempt number, incoming Occurrence and edge (or entry),
/// executor, and agent tier. Inputs follow node declaration order and carry absolute
/// paths and 64-digit SHA-256 digests. Missing optional inputs name their upstream
/// node; an empty input set still has the two table header rows.
///
/// Show host resources only for nonempty node requires. Look up each resource's
/// version in Workbook declarations, defaulting to `-`, and tell the worker to
/// confirm host installation or stop and inform the user. Copy instruction text
/// verbatim after trimming trailing newlines, then append one newline. Output rows
/// name their destination, required yes/no flag, and byte limit. Exact MiB/KiB
/// multiples use `N MiB`/`N KiB`; other sizes use `N B`.
///
/// Agent briefs end with instructions to leave input files unchanged, summarize the
/// conclusion for the coordinator, and list written outputs. Human briefs instead
/// give the `sheltie attempt submit` command with a one-sentence summary placeholder.
pub fn render_brief(
    state: &WorkState,
    graph: &Graph,
    attempt: &Attempt,
    instruction_text: &str,
) -> String {
    let Some(node) = graph.node(&attempt.id.node) else {
        return String::new();
    };
    let mut out = String::new();
    out.push_str(&format!("# Brief: {}\n\n", node.title));
    out.push_str(&format!("Work: {} ({})\n", state.work_id, state.name));
    out.push_str(&format!(
        "Node: {}, Attempt {}\n",
        attempt.occurrence(),
        attempt.id.number
    ));
    match &attempt.entered_from {
        None => out.push_str("From: entry\n"),
        Some((occ, kind)) => {
            out.push_str(&format!("From: {occ} ({} edge)\n", kind.as_str()));
        }
    }
    match node.executor {
        crate::flow::Executor::Agent => {
            let tier = node.tier.unwrap_or_default();
            out.push_str(&format!("Executor: agent ({})\n", tier.as_str()));
        }
        crate::flow::Executor::Human => out.push_str("Executor: human\n"),
    }
    out.push('\n');

    out.push_str("## Inputs\n\n");
    out.push_str("| Name | Path | sha256 |\n");
    out.push_str("| --- | --- | --- |\n");
    for decl in &node.inputs {
        match attempt.inputs.get(&decl.name).and_then(|o| o.as_ref()) {
            Some(r) => out.push_str(&format!(
                "| {} | {} | {} |\n",
                decl.name,
                r.path,
                r.sha256.as_str()
            )),
            // Protocol §4: identify the upstream node for unbound optional inputs.
            // Only Node inputs may be optional (compile rule 5); other sources are bound before begin.
            None => match &decl.from {
                InputSource::Node { node, .. } => out.push_str(&format!(
                    "| {} | Not available (upstream {} has not produced output) | |\n",
                    decl.name, node
                )),
                _ => out.push_str(&format!("| {} | Not available | |\n", decl.name)),
            },
        }
    }

    if !node.requires.is_empty() {
        out.push_str("\n## Required host resources\n\n");
        out.push_str("| Kind | Name | Version | Instructions |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for (kind, name) in &node.requires {
            // Look up the version in Workbook requires (protocol §4); use `-` when omitted.
            let version = graph
                .requires()
                .iter()
                .find(|r| r.kind == *kind && r.name == *name)
                .and_then(|r| r.version.as_deref())
                .unwrap_or("-");
            // Name the actual resource kind: skill, agent, or mcp (protocol §4).
            out.push_str(&format!(
                "| {} | {} | {} | Confirm your host has this {} installed; otherwise stop and inform the user |\n",
                kind.as_str(),
                name,
                version,
                kind.as_str()
            ));
        }
    }

    out.push_str("\n## Instructions\n\n");
    out.push_str(instruction_text.trim_end_matches('\n'));
    out.push('\n');

    let attempt_dir = state.attempt_dir(&attempt.id);
    out.push_str("\n## Output requirements\n\n");
    out.push_str("| Name | Write to | Required | Limit |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for decl in &node.outputs {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            decl.name,
            crate::work::layout::output_path(&attempt_dir, &decl.path),
            if decl.required { "yes" } else { "no" },
            human_size(decl.max_bytes)
        ));
    }

    out.push('\n');
    match node.executor {
        crate::flow::Executor::Agent => {
            out.push_str("Do not modify input files after finishing. Summarize your conclusion for the coordinator and list the output files you wrote.\n");
        }
        crate::flow::Executor::Human => {
            out.push_str("After writing the output files, run in your terminal:\n\n");
            out.push_str(&format!(
                "    sheltie attempt submit {} --attempt {} --summary \"<one-sentence conclusion>\"\n",
                state.work_id, attempt.id
            ));
        }
    }
    out
}

/// `N MiB` / `N KiB` / `N B`; zero is `0 B`.
fn human_size(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    const KIB: u64 = 1024;
    if bytes == 0 {
        "0 B".to_string()
    } else if bytes % MIB == 0 {
        format!("{} MiB", bytes / MIB)
    } else if bytes % KIB == 0 {
        format!("{} KiB", bytes / KIB)
    } else {
        format!("{bytes} B")
    }
}

/// Render the status card (protocol §6). Handwritten `snapshots/*status_card*.snap`
/// files are independent expectations. The header names the Work and displays
/// workbook, flow, WorkStatus (such as blocked(gate)), and current Occurrence.
/// Successful Occurrences follow submission order; pending nodes and visit counts
/// follow graph declaration order. Empty lists use `none`.
///
/// Only blocked states have a blocked line: gate names the required gate approval,
/// retries_exhausted names the Occurrence, and no_legal_edge says every outgoing
/// target has reached max_visits. The current-task section gives the bound brief,
/// inputs, and draft outputs without claiming that drafts exist or are sealed.
///
/// The latest Attempt section gives status, successful summary and outputs, or
/// failure/replacement reason. Artifact lines include path, digest, and human-readable
/// size, indented by two spaces. Legal next actions use one command per line;
/// terminal states use `- none`. Lines use `\n`, including a final newline.
pub fn render_status_card(state: &WorkState, graph: &Graph) -> String {
    status_view(state, graph).render()
}

impl StatusView {
    /// Text rendering of the same fact view.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Work {} ({})\n\n", self.work_id, self.name));
        out.push_str(&format!(
            "workbook: {}   flow: {}   status: {}\n",
            self.workbook, self.flow, self.status
        ));
        out.push_str(&format!("current: {}\n", self.current));

        out.push_str(&format!(
            "done: {}\n",
            if self.done.is_empty() {
                "none".to_string()
            } else {
                self.done.join(", ")
            }
        ));
        out.push_str(&format!(
            "pending: {}\n",
            if self.pending.is_empty() {
                "none".to_string()
            } else {
                self.pending.join(", ")
            }
        ));
        let visits = self
            .visits
            .iter()
            .map(|(n, m)| format!("{n} {m}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("visits: {visits}\n"));
        if let Some(line) = &self.blocked {
            out.push_str(&format!("blocked: {line}\n"));
        }

        out.push_str("\n## Current task\n\n");
        match &self.resume {
            None => out.push_str("none\n"),
            Some(resume) => {
                out.push_str(&format!(
                    "attempt: {}\nbrief_path: {}\n",
                    resume.attempt, resume.brief_path
                ));
                out.push_str("inputs:\n");
                for (name, reference) in &resume.inputs {
                    match reference {
                        Some(reference) => out.push_str(&format!(
                            "  {} → {} (sha256 {}, {} B)\n",
                            name, reference.path, reference.sha256, reference.bytes,
                        )),
                        None => out.push_str(&format!("  {name} → Not available\n")),
                    }
                }
                if !resume.draft_outputs.is_empty() {
                    out.push_str("draft_outputs:\n");
                    for (name, path) in &resume.draft_outputs {
                        out.push_str(&format!("  {name} → {path}\n"));
                    }
                }
            }
        }

        out.push_str("\n## Latest Attempt\n\n");
        match &self.last_attempt {
            None => out.push_str("none\n"),
            Some(a) => {
                out.push_str(&format!("{} {}\n", a.attempt, a.status.as_str()));
                match a.status {
                    AttemptStatus::Succeeded => {
                        if let Some(s) = &a.summary {
                            out.push_str(&format!("summary: {s}\n"));
                        }
                        if !a.outputs.is_empty() {
                            out.push_str("outputs:\n");
                            for (name, r) in &a.outputs {
                                out.push_str(&format!(
                                    "  {} → {} (sha256 {}, {})\n",
                                    name,
                                    r.path,
                                    r.sha256.as_str(),
                                    human_size(r.bytes)
                                ));
                            }
                        }
                    }
                    AttemptStatus::Failed => {
                        if let Some(s) = &a.reason {
                            out.push_str(&format!("reason: {s}\n"));
                        }
                    }
                    AttemptStatus::Superseded => {
                        if let Some(s) = &a.reason {
                            out.push_str(&format!("reason: {s}\n"));
                        }
                    }
                    AttemptStatus::Running => {}
                }
            }
        }

        out.push_str("\n## Legal next actions\n\n");
        if self.next.is_empty() {
            out.push_str("- none\n");
        } else {
            for op in &self.next {
                out.push_str(&format!("- {}\n", op.to_command_line(&self.work_id)));
            }
        }
        out
    }
}

/// Fact view (GF-10/GF-29): text and JSON render the same facts; assemble failure reasons,
/// complete artifact references (path, digest, bytes), and `next` only once here.
/// T06 delivers pure implementation and independent tests; T07 connects persistent callers and response envelopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusView {
    pub work_id: WorkId,
    pub name: String,
    pub workbook: String,
    pub flow: String,
    pub status: WorkStatus,
    pub current: String,
    pub done: Vec<String>,
    pub pending: Vec<String>,
    pub visits: Vec<(String, String)>,
    /// Explanation shared with the text card, such as `gate: review#2 requires gate approve`; otherwise `None`.
    pub blocked: Option<String>,
    pub last_attempt: Option<LastAttemptView>,
    pub resume: Option<ResumeView>,
    pub next: Vec<NextOp>,
}

/// Latest Attempt facts, including failure reasons and complete artifact references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastAttemptView {
    pub attempt: String,
    pub status: AttemptStatus,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub outputs: BTreeMap<String, ArtifactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResumeView {
    pub attempt: String,
    pub brief_path: AbsPath,
    pub inputs: BTreeMap<String, Option<ArtifactRef>>,
    pub draft_outputs: BTreeMap<String, AbsPath>,
}

/// Assemble the fact view from state and graph.
pub fn status_view(state: &WorkState, graph: &Graph) -> StatusView {
    StatusView {
        work_id: state.work_id.clone(),
        name: state.name.to_string(),
        workbook: format!("{}@{}", state.workbook.id, state.workbook.version),
        flow: state.flow.to_string(),
        status: state.status,
        current: state.current.to_string(),
        done: done_occurrences(state),
        pending: pending_nodes(state, graph),
        visits: visit_items(state, graph),
        blocked: blocked_line(state),
        last_attempt: state.attempts.last().map(|a| LastAttemptView {
            attempt: a.id.to_string(),
            status: a.status,
            summary: a.summary.as_ref().map(|s| s.as_str().to_string()),
            reason: a
                .fail_reason
                .as_ref()
                .or(a.replacement_reason.as_ref())
                .map(|s| s.as_str().to_string()),
            outputs: a.outputs.clone(),
        }),
        resume: state.latest_attempt_of_current().map(|attempt| {
            let attempt_dir = state.attempt_dir(&attempt.id);
            let mut draft_outputs = BTreeMap::new();
            if attempt.status == AttemptStatus::Running {
                if let Some(node) = graph.node(&attempt.id.node) {
                    for output in &node.outputs {
                        draft_outputs.insert(
                            output.name.clone(),
                            crate::work::layout::output_path(&attempt_dir, &output.path),
                        );
                    }
                }
            }
            ResumeView {
                attempt: attempt.id.to_string(),
                brief_path: crate::work::layout::brief_path(&attempt_dir),
                inputs: attempt.inputs.clone(),
                draft_outputs,
            }
        }),
        next: legal_next(state, graph),
    }
}

/// Structured `StatusView` rendering for `--json`; `last_attempt.outputs` contains complete
/// artifact references (path, digest, bytes); failure reasons share the text card's source, and `next` matches
/// the protocol §5 response envelope exactly (O13).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatusCardJson {
    pub work_id: WorkId,
    pub name: String,
    pub workbook: String,
    pub flow: String,
    pub status: WorkStatus,
    pub current: String,
    pub done: Vec<String>,
    pub pending: Vec<String>,
    pub visits: BTreeMap<String, String>,
    pub blocked: Option<String>,
    pub last_attempt: Option<LastAttemptJson>,
    pub resume: Option<ResumeView>,
    pub next: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LastAttemptJson {
    pub attempt: String,
    pub status: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub outputs: BTreeMap<String, ArtifactRefJson>,
}

/// Complete artifact reference (protocol §6), sharing the text card's path, digest, and size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactRefJson {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

/// Structured status card; fields match the text card.
pub fn status_card_json(state: &WorkState, graph: &Graph) -> StatusCardJson {
    status_view(state, graph).into_json()
}

impl StatusView {
    /// Protocol JSON payload from the same fact view.
    pub fn into_json(self) -> StatusCardJson {
        let next = self
            .next
            .iter()
            .map(|op| next_item_json(&self.work_id, op))
            .collect();
        StatusCardJson {
            work_id: self.work_id,
            name: self.name,
            workbook: self.workbook,
            flow: self.flow,
            status: self.status,
            current: self.current,
            done: self.done,
            pending: self.pending,
            visits: self.visits.into_iter().collect(),
            blocked: self.blocked,
            last_attempt: self.last_attempt.map(|a| LastAttemptJson {
                attempt: a.attempt,
                status: a.status.as_str().to_string(),
                summary: a.summary,
                reason: a.reason,
                outputs: a
                    .outputs
                    .iter()
                    .map(|(name, r)| {
                        (
                            name.clone(),
                            ArtifactRefJson {
                                path: r.path.to_string(),
                                sha256: r.sha256.as_str().to_string(),
                                bytes: r.bytes,
                            },
                        )
                    })
                    .collect(),
            }),
            resume: self.resume,
            next,
        }
    }
}

/// Protocol §5 `next` item: `op`, `args` (including `work`), and, only for `attempt begin`,
/// `edge`, `executor`, and `tier`; the protocol has one canonical shape (O13).
pub fn next_item_json(work_id: &WorkId, op: &NextOp) -> serde_json::Value {
    match op {
        NextOp::BeginAttempt {
            node,
            edge,
            executor,
            tier,
        } => {
            let mut v = serde_json::json!({
                "op": "attempt begin",
                "args": { "work": work_id.as_str(), "node": node.as_str() },
            });
            if let Some(kind) = edge {
                v["edge"] = serde_json::json!(kind.as_str());
            }
            v["executor"] = serde_json::json!(executor.as_str());
            if let Some(t) = tier {
                v["tier"] = serde_json::json!(t.as_str());
            }
            v
        }
        NextOp::SubmitAttempt { attempt } => serde_json::json!({
            "op": "attempt submit",
            "args": { "work": work_id.as_str(), "attempt": attempt.to_string() },
        }),
        NextOp::FailAttempt { attempt } => serde_json::json!({
            "op": "attempt fail",
            "args": { "work": work_id.as_str(), "attempt": attempt.to_string() },
        }),
        NextOp::ReplaceAttempt { attempt } => serde_json::json!({
            "op": "attempt replace",
            "args": { "work": work_id.as_str(), "attempt": attempt.to_string() },
        }),
        NextOp::ApproveGate { node } => serde_json::json!({
            "op": "gate approve",
            "args": { "work": work_id.as_str(), "node": node.as_str() },
        }),
        NextOp::Cancel => serde_json::json!({
            "op": "work cancel",
            "args": { "work": work_id.as_str() },
        }),
    }
}

/// All Succeeded Occurrences in submission order.
fn done_occurrences(state: &WorkState) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for a in &state.attempts {
        if a.status == AttemptStatus::Succeeded {
            let occ = a.occurrence();
            if seen.insert(occ.clone()) {
                out.push(occ.to_string());
            }
        }
    }
    out
}

/// Unvisited node IDs in graph declaration order.
fn pending_nodes(state: &WorkState, graph: &Graph) -> Vec<String> {
    graph
        .nodes()
        .filter(|d| state.visits_of(&d.id) == 0)
        .map(|d| d.id.as_str().to_string())
        .collect()
}

/// `(node, "n/m")` in graph declaration order.
fn visit_items(state: &WorkState, graph: &Graph) -> Vec<(String, String)> {
    graph
        .nodes()
        .map(|d| {
            (
                d.id.as_str().to_string(),
                format!("{}/{}", state.visits_of(&d.id), d.max_visits),
            )
        })
        .collect()
}

/// The `blocked` line; `None` for non-Blocked states.
/// Blocked explanation (protocol §6), shared with the text line without its `blocked: ` prefix;
/// `None` for non-Blocked states; the text card adds its own prefix.
fn blocked_line(state: &WorkState) -> Option<String> {
    let occ = state.current.to_string();
    match state.status {
        WorkStatus::Blocked(BlockedReason::Gate) => {
            Some(format!("gate: {occ} requires gate approve"))
        }
        WorkStatus::Blocked(BlockedReason::RetriesExhausted) => {
            Some(format!("retries_exhausted: {occ}"))
        }
        WorkStatus::Blocked(BlockedReason::NoLegalEdge) => Some(format!(
            "no_legal_edge: {occ} has all outgoing targets at max_visits"
        )),
        _ => None,
    }
}

/// Fact view (`work stats`): visits, attempts, failures, average duration, and incoming node/edge pairs per node.
/// Counts from `WorkState` without content judgments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatsJson {
    pub work_id: WorkId,
    pub status: WorkStatus,
    /// Seconds from `created_at` to `updated_at`.
    pub total_seconds: u64,
    pub blocked_count: u32,
    pub approvals: u32,
    /// In graph declaration order.
    pub nodes: Vec<NodeStatsJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeStatsJson {
    pub node: String,
    pub visits: u32,
    pub max_visits: u32,
    pub attempts: u32,
    pub failed: u32,
    pub superseded: u32,
    /// Average seconds for ended Attempts; zero when none exist.
    pub avg_seconds: u64,
    /// `"<from>(<edge>)×<n>"` or `"entry×<n>"`, in first-observed order.
    pub entered_via: Vec<String>,
    /// Structured data from the same source: `{from, edge, count}`; null from/edge means entry.
    pub entered_via_json: Vec<EnteredViaJson>,
}

/// One incoming-source fact (`work stats` JSON shape).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnteredViaJson {
    pub from: Option<String>,
    pub edge: Option<String>,
    pub count: u32,
}

/// Structured facts; derive total_seconds and avg_seconds by subtracting `Timestamp::unix_secs`.
pub fn render_stats_json(state: &WorkState, graph: &Graph) -> StatsJson {
    let mut nodes = Vec::new();
    for def in graph.nodes() {
        let attempts: Vec<&Attempt> = state
            .attempts
            .iter()
            .filter(|a| a.id.node == def.id)
            .collect();
        let failed = attempts
            .iter()
            .filter(|a| a.status == AttemptStatus::Failed)
            .count() as u32;
        let superseded = attempts
            .iter()
            .filter(|attempt| attempt.status == AttemptStatus::Superseded)
            .count() as u32;

        // Count entered_via by Occurrence (retries retain the same source), keyed by node and edge kind;
        // preserve first-observed order (N07: node-only keys would merge main and back).
        let mut order: Vec<(Option<String>, Option<String>)> = Vec::new();
        let mut counts: BTreeMap<(Option<String>, Option<String>), u32> = BTreeMap::new();
        let mut seen_occ = BTreeSet::new();
        for a in &attempts {
            if !seen_occ.insert(a.id.occurrence) {
                continue;
            }
            let key = match &a.entered_from {
                Some((occ, kind)) => (
                    Some(occ.node.as_str().to_string()),
                    Some(kind.as_str().to_string()),
                ),
                None => (None, None),
            };
            if !counts.contains_key(&key) {
                order.push(key.clone());
            }
            *counts.entry(key).or_insert(0) += 1;
        }
        let entered_via = order
            .iter()
            .map(|key| {
                let n = counts.get(key).copied().unwrap_or(0);
                match key {
                    (Some(f), Some(e)) => format!("{f}({e})×{n}"),
                    _ => format!("entry×{n}"),
                }
            })
            .collect();
        let entered_via_json = order
            .iter()
            .map(|(from, edge)| EnteredViaJson {
                from: from.clone(),
                edge: edge.clone(),
                count: counts
                    .get(&(from.clone(), edge.clone()))
                    .copied()
                    .unwrap_or(0),
            })
            .collect();

        let mut ended_secs = 0u64;
        let mut ended_n = 0u64;
        for a in &attempts {
            if let Some(end) = &a.ended_at {
                ended_secs += secs_between(&a.started_at, end);
                ended_n += 1;
            }
        }
        nodes.push(NodeStatsJson {
            node: def.id.as_str().to_string(),
            visits: state.visits_of(&def.id),
            max_visits: def.max_visits,
            attempts: attempts.len() as u32,
            failed,
            superseded,
            avg_seconds: ended_secs.checked_div(ended_n).unwrap_or(0),
            entered_via,
            entered_via_json,
        });
    }
    StatsJson {
        work_id: state.work_id.clone(),
        status: state.status,
        total_seconds: secs_between(&state.created_at, &state.updated_at),
        // Cumulative blocking is recorded by state transitions (GF-29/N07); cancellation does not decrease it,
        // and current status must not reconstruct history.
        blocked_count: state.blocked_count,
        approvals: state.approvals.len() as u32,
        nodes,
    }
}

/// Time difference in seconds, clamped to zero; Timestamp construction already validates syntax.
fn secs_between(a: &Timestamp, b: &Timestamp) -> u64 {
    (b.unix_secs() - a.unix_secs()).max(0) as u64
}

/// Text fact table (`work stats`); handwritten `*stats_table*.snap` files are independent expectations.
///
/// ```text
/// # Stats <work_id>
///
/// status: <status>   total: <Ns>   blocked: <n>   approvals: <n>
///
/// | node | visits | attempts | failed | avg | entered_via |
/// | --- | --- | --- | --- | --- | --- |
/// | draft | 2/3 | 2 | 0 | 0s | entry×1, review×1 |
/// ```
pub fn render_stats(state: &WorkState, graph: &Graph) -> String {
    render_stats_json(state, graph).render()
}

impl StatsJson {
    /// Render assembled statistics without recounting state.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Stats {}\n\n", self.work_id));
        out.push_str(&format!(
            "status: {}   total: {}s   blocked: {}   approvals: {}\n\n",
            self.status, self.total_seconds, self.blocked_count, self.approvals
        ));
        out.push_str("| node | visits | attempts | failed | superseded | avg | entered_via |\n");
        out.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
        for n in &self.nodes {
            out.push_str(&format!(
                "| {} | {}/{} | {} | {} | {} | {}s | {} |\n",
                n.node,
                n.visits,
                n.max_visits,
                n.attempts,
                n.failed,
                n.superseded,
                n.avg_seconds,
                n.entered_via.join(", ")
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::Fixture;
    use crate::work::next::NextOp;

    // Task: C004-T01
    #[test]
    fn resume_without_current_attempt_is_null_in_text_and_json() {
        let fixture = Fixture::article_review().started();
        assert!(
            status_view(fixture.state(), &fixture.graph)
                .resume
                .is_none()
        );
        assert_eq!(
            serde_json::to_value(status_card_json(fixture.state(), &fixture.graph)).unwrap()["resume"],
            serde_json::Value::Null
        );
        assert!(
            render_status_card(fixture.state(), &fixture.graph)
                .contains("## Current task\n\nnone\n")
        );
    }

    // Task: C004-T01
    #[test]
    fn resume_running_attempt_has_frozen_inputs_and_declared_draft_paths() {
        let mut fixture = Fixture::article_review().started();
        fixture.begin("draft").unwrap();
        let card = status_card_json(fixture.state(), &fixture.graph);
        let resume = card.resume.unwrap();
        assert_eq!(resume.attempt, "draft#1.0");
        assert_eq!(
            resume.brief_path.as_str(),
            "/tmp/sheltie-test/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/brief.md"
        );
        assert_eq!(resume.inputs["review"], None);
        let topic = resume.inputs["topic"].as_ref().unwrap();
        assert_eq!(
            topic.path.as_str(),
            "/tmp/sheltie-test/works/2026-09-24-001-t/start-inputs/topic"
        );
        assert_eq!(
            topic.sha256.as_str(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert_eq!(topic.bytes, 5);
        assert_eq!(
            resume.draft_outputs["article"].as_str(),
            "/tmp/sheltie-test/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/outputs/article.md"
        );
        let text = render_status_card(fixture.state(), &fixture.graph);
        assert!(text.contains("attempt: draft#1.0"));
        assert!(text.contains(resume.brief_path.as_str()));
        assert!(text.contains("review → Not available"));
        assert!(text.contains(&format!("{} (sha256 {}, 5 B)", topic.path, topic.sha256)));
        assert!(text.contains("draft_outputs:"));
        assert!(text.contains(resume.draft_outputs["article"].as_str()));
    }

    // Task: C004-T01
    #[test]
    fn resume_current_attempt_keeps_brief_and_inputs_after_end_but_removes_drafts() {
        let mut fixture = Fixture::article_review().started();
        fixture.begin("draft").unwrap();
        fixture.fail("draft#1.0", "Interrupted").unwrap();
        let failed = status_card_json(fixture.state(), &fixture.graph)
            .resume
            .unwrap();
        assert_eq!(failed.attempt, "draft#1.0");
        assert!(failed.draft_outputs.is_empty());
        fixture.begin("draft").unwrap();
        let running = status_card_json(fixture.state(), &fixture.graph)
            .resume
            .unwrap();
        assert_eq!(running.attempt, "draft#1.1");
        assert!(running.brief_path.as_str().contains("attempt-001/brief.md"));
        fixture.submit_ok("draft#1.1", "Finished").unwrap();
        let succeeded = status_card_json(fixture.state(), &fixture.graph)
            .resume
            .unwrap();
        assert_eq!(succeeded.inputs, running.inputs);
        assert_eq!(succeeded.brief_path, running.brief_path);
        assert!(succeeded.draft_outputs.is_empty());
        assert!(!render_status_card(fixture.state(), &fixture.graph).contains("draft_outputs:"));
    }

    // Task: T10
    #[test]
    fn brief_for_review_node() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        insta::assert_snapshot!(
            "brief_for_review_node",
            render_brief(fx.state(), &fx.graph, a, "Review this article.")
        );
    }

    // Task: T10
    #[test]
    fn brief_for_node_with_requires() {
        let mut fx = Fixture::with_requires();
        fx.begin("only").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        insta::assert_snapshot!(
            "brief_for_node_with_requires",
            render_brief(fx.state(), &fx.graph, a, "Use the company API.")
        );
    }

    // Task: T10
    #[test]
    fn brief_for_node_without_requires_omits_section() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "Write a draft.");
        assert!(!text.contains("Required host resources"));
        assert!(text.contains("From: entry"));
    }

    // ── M1 review O3: versions come from Workbook requires; omitted versions use `-` ─────

    // Task: T10
    #[test]
    fn brief_require_row_takes_version_from_manifest_and_names_kind() {
        let mut fx = Fixture::from_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"Single node\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"company-api\"\nversion = \"^1\"\n[[requires]]\nkind = \"mcp\"\nname = \"db\"\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = { text = \"Use the company API.\" }\nrequires = [\"skill:company-api\", \"mcp:db\"]\n",
            &[],
        )
        .started_with(&[]);
        fx.begin("only").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "Use the company API.");
        // Versions come from manifest declarations, defaulting to `-`; instructions name the actual kind (protocol §4).
        assert!(text.contains(
            "| skill | company-api | ^1 | Confirm your host has this skill installed; otherwise stop and inform the user |\n\
             | mcp | db | - | Confirm your host has this mcp installed; otherwise stop and inform the user |\n"
        ));
    }

    // ── M1 review: distinguish declarations by both kind and name ─────

    // Task: T10
    #[test]
    fn brief_require_version_ignores_crossed_kind_name_pairs() {
        // skill:db and mcp:db share a name but differ in kind; the node references mcp:db.
        // Changing && to || would incorrectly select skill:db version 1.0 first.
        let mut fx = Fixture::from_texts(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"Single node\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"db\"\nversion = \"1.0\"\n[[requires]]\nkind = \"mcp\"\nname = \"db\"\nversion = \"3.0\"\n",
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = { text = \"Do this task.\" }\nrequires = [\"mcp:db\"]\n",
            &[],
        )
        .started_with(&[]);
        fx.begin("only").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "Do this task.");
        assert!(
            text.contains("| mcp | db | 3.0 |"),
            "Version must come from the mcp:db declaration:\n{text}"
        );
    }

    // Task: T10
    #[test]
    fn brief_marks_unbound_optional_input_as_absent() {
        let mut fx = Fixture::spec_dev().started_with(&[("request", "r"), ("project", "/p")]);
        fx.begin("spec").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "Write the specification.");
        assert!(text.contains(
            "| decision | Not available (upstream plan-review has not produced output) | |"
        ));
    }

    // Task: T10
    #[test]
    fn brief_for_human_executor_ends_with_submit_command() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "Approved").unwrap();
        fx.begin("publish").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        insta::assert_snapshot!(
            "brief_for_human_executor_ends_with_submit_command",
            render_brief(fx.state(), &fx.graph, a, "Confirm publication is approved.")
        );
    }

    // Task: C005-T02
    #[test]
    fn status_card_active_mid_flow() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        insta::assert_snapshot!(
            "status_card_active_mid_flow",
            render_status_card(fx.state(), &fx.graph)
        );
    }

    // Task: T10
    #[test]
    fn status_card_blocked_on_gate() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "Written").unwrap();
        insta::assert_snapshot!(
            "status_card_blocked_on_gate",
            render_status_card(fx.state(), &fx.graph)
        );
    }

    // Task: T10
    #[test]
    fn status_card_succeeded() {
        let mut fx = Fixture::two_step().started_with(&[("topic", "t")]);
        fx.begin("outline").unwrap();
        fx.submit_ok("outline#1.0", "Outline ready").unwrap();
        fx.begin("summary").unwrap();
        fx.submit_ok("summary#1.0", "Summary ready").unwrap();
        insta::assert_snapshot!(
            "status_card_succeeded",
            render_status_card(fx.state(), &fx.graph)
        );
    }

    // Task: T10
    #[test]
    fn next_op_renders_begin_with_node_flag() {
        let fx = Fixture::article_review().started();
        let next = legal_next(fx.state(), &fx.graph);
        assert_eq!(
            next[0].to_command_line(&fx.state().work_id),
            format!("sheltie attempt begin {} --node draft", fx.state().work_id)
        );
        assert_eq!(
            NextOp::Cancel.to_command_line(&fx.state().work_id),
            format!("sheltie work cancel {}", fx.state().work_id)
        );
    }

    // Task: T10
    #[test]
    fn next_op_begin_carries_executor_and_tier() {
        let fx = Fixture::article_review().started();
        let json = serde_json::to_value(&legal_next(fx.state(), &fx.graph)[0]).unwrap();
        assert_eq!(json["op"], "attempt begin");
        assert_eq!(json["executor"], "agent");
        assert_eq!(json["tier"], "standard");
    }

    // Task: T10
    #[test]
    fn stats_table_mid_flow() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        insta::assert_snapshot!("stats_table_mid_flow", render_stats(fx.state(), &fx.graph));
    }

    // Task: T10
    #[test]
    fn stats_json_counts_visits_failures_and_entered_via() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "Crashed").unwrap();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.1", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "Rejected").unwrap();
        fx.begin("draft").unwrap();
        let s = render_stats_json(fx.state(), &fx.graph);
        let draft = &s.nodes[0];
        assert_eq!(
            (
                draft.node.as_str(),
                draft.visits,
                draft.attempts,
                draft.failed
            ),
            ("draft", 2, 3, 1)
        );
        // The work stats contract requires edge kinds: draft(back)×1.
        assert_eq!(draft.entered_via, vec!["entry×1", "review(back)×1"]);
        assert_eq!(s.nodes[1].entered_via, vec!["draft(main)×1"]);
        assert_eq!(s.nodes[2].attempts, 0);
        assert_eq!(s.blocked_count, 0);
    }

    // Task: T10
    #[test]
    fn status_card_lists_done_occurrences_in_order() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#2.0", "Revised").unwrap();
        let card = status_card_json(fx.state(), &fx.graph);
        assert_eq!(card.done, vec!["draft#1", "review#1", "draft#2"]);
        assert_eq!(card.pending, vec!["publish"]);
        assert_eq!(card.current, "draft#2");
    }

    // ── M1 additional blocked-line, blocking-count, and timestamp coverage; snapshots use a fixed clock and 0s ─────

    fn exhausted_draft() -> Fixture {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "First").unwrap();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.1", "Second").unwrap();
        fx
    }

    fn no_legal_edge() -> Fixture {
        let mut fx = Fixture::article_review_with_review_max_visits_1().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "Rejected").unwrap();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#2.0", "Revised").unwrap();
        fx
    }

    // Task: T10
    #[test]
    fn status_card_names_retries_exhausted_occurrence() {
        let fx = exhausted_draft();
        let card = render_status_card(fx.state(), &fx.graph);
        assert!(
            card.contains("blocked: retries_exhausted: draft#1\n"),
            "{card}"
        );
    }

    // Task: T10
    #[test]
    fn status_card_explains_no_legal_edge() {
        let fx = no_legal_edge();
        let card = render_status_card(fx.state(), &fx.graph);
        assert!(
            card.contains("blocked: no_legal_edge: draft#2 has all outgoing targets at max_visits"),
            "{card}"
        );
    }

    // Task: T10
    #[test]
    fn stats_blocked_count_by_reason() {
        // Ordinary success and retryable failure do not count as blocking.
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "First").unwrap();
        assert_eq!(fx.state().blocked_count, 0);
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 0);
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.1", "ok").unwrap();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 0);

        let fx = exhausted_draft();
        assert_eq!(fx.state().blocked_count, 1);
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);

        let fx = no_legal_edge();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);

        // Gate success counts once, retaining the count after approval.
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "Written").unwrap();
        assert_eq!(fx.state().blocked_count, 1);
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);
        fx.approve("notes").unwrap();
        assert_eq!(
            fx.state().blocked_count,
            1,
            "Approval does not erase prior blocking facts"
        );
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);
    }

    // Task: T10
    #[test]
    fn secs_between_matches_independent_calendar_math() {
        // Expected values are independently calculated with Python datetime.
        let cases = [
            ("2026-02-28T23:59:30Z", "2026-03-01T00:00:30Z", 60),
            ("2024-02-28T00:00:00Z", "2024-03-01T00:00:00Z", 172_800),
            ("1999-12-31T23:59:59Z", "2000-03-01T00:00:00Z", 5_184_001),
            (
                "1970-01-01T00:00:00Z",
                "2026-09-24T03:04:05Z",
                1_790_219_045,
            ),
            ("1900-02-28T00:00:00Z", "1900-03-01T00:00:00Z", 86_400),
            ("2025-12-31T23:59:59Z", "2026-01-01T00:00:09Z", 10),
            (
                "1600-01-01T00:00:00Z",
                "2400-12-31T23:59:59Z",
                25_277_183_999,
            ),
        ];
        let ts = |s: &str| Timestamp::parse(s).unwrap();
        for (a, b, want) in cases {
            assert_eq!(secs_between(&ts(a), &ts(b)), want, "{a} → {b}");
        }
        // Clamp reversed time to zero; Timestamp::parse rejects invalid syntax before this function.
        assert_eq!(
            secs_between(&ts("2026-01-01T00:00:09Z"), &ts("2026-01-01T00:00:00Z")),
            0
        );
    }

    // ── C002-T06: fact view and cumulative blocking (O13/N07) ───────────────────

    // Task: C002-T06
    #[test]
    fn blocked_count_survives_cancel_after_no_legal_edge() {
        // NoLegalEdge counts once; cancellation changes state without reverting recorded facts (GF-29/N07).
        let mut fx = no_legal_edge();
        assert_eq!(fx.state().blocked_count, 1);
        fx.cancel().unwrap();
        assert_eq!(
            fx.state().blocked_count,
            1,
            "Cancellation must not decrease cumulative blocking"
        );
        let stats = render_stats_json(fx.state(), &fx.graph);
        assert_eq!(stats.blocked_count, 1);
    }

    // Task: C002-T06
    #[test]
    fn status_json_carries_reason_and_full_artifact_refs() {
        // JSON carries the failure reason (O13); missing or incorrect fields fail this test.
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "Worker crashed").unwrap();
        let card = status_card_json(fx.state(), &fx.graph);
        let last = card.last_attempt.as_ref().unwrap();
        assert_eq!(last.reason.as_deref(), Some("Worker crashed"));
        assert_eq!(last.status, "failed");

        // Successful outputs contain complete artifact references with path, sha256, and bytes.
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let d = fx.submit_ok("draft#1.0", "Completed").unwrap();
        let sealed = d.state.attempts[0].outputs["article"].clone();
        let card = status_card_json(&d.state, &fx.graph);
        let out = &card.last_attempt.as_ref().unwrap().outputs["article"];
        assert_eq!(out.path, sealed.path.to_string());
        assert_eq!(out.sha256, sealed.sha256.as_str());
        assert_eq!(out.bytes, sealed.bytes);
        // Handwritten expected fields must exist; removing digest/bytes fails equality above.
        assert_eq!(out.sha256.len(), 64);
        assert!(out.bytes > 0);

        // Blocked explanations share one source in both formats.
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "Written").unwrap();
        let card = status_card_json(fx.state(), &fx.graph);
        assert_eq!(
            card.blocked.as_deref(),
            Some("gate: notes#1 requires gate approve")
        );
    }

    // Task: C002-T06
    #[test]
    fn next_items_use_single_protocol_shape() {
        // Status-card data.next matches protocol §5 top-level next: op/args/work, plus
        // edge/executor/tier on begin items (O13 inconsistent-shape repair).
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        let card = status_card_json(fx.state(), &fx.graph);
        let next = &card.next;
        assert!(next.len() >= 2);
        for item in next {
            assert!(item["op"].is_string(), "{item}");
            assert!(item["args"]["work"].is_string(), "{item}");
        }
        let begin = next
            .iter()
            .find(|i| i["op"] == "attempt begin" && i["args"]["node"] == "draft")
            .unwrap();
        assert_eq!(begin["edge"], "back");
        assert_eq!(begin["executor"], "agent");
        assert_eq!(begin["tier"], "standard");
        // Matches text-card command lines from the same facts.
        let text = render_status_card(fx.state(), &fx.graph);
        assert!(text.contains("sheltie attempt begin"), "{text}");
    }

    // Task: C002-T06
    #[test]
    fn entered_via_distinguishes_edges_and_entry() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        // Reenter draft after rejection through the back edge from review#1.
        fx.begin("draft").unwrap();
        let stats = render_stats_json(fx.state(), &fx.graph);
        let draft = &stats.nodes[0];
        // Text and structured forms share facts: entry has no from/edge, while back retains its edge kind.
        assert_eq!(draft.entered_via, vec!["entry×1", "review(back)×1"]);
        assert_eq!(
            draft.entered_via_json,
            vec![
                EnteredViaJson {
                    from: None,
                    edge: None,
                    count: 1
                },
                EnteredViaJson {
                    from: Some("review".to_string()),
                    edge: Some("back".to_string()),
                    count: 1,
                },
            ]
        );
        assert_eq!(
            stats.nodes[1].entered_via,
            vec!["draft(main)×1"],
            "review enters through main; preserve the edge kind"
        );
    }

    // Task: C002-T06
    #[test]
    fn text_and_json_agree_on_hand_written_state() {
        // Independently handwritten state: nonzero timestamps, failure reason, and multiple outputs; compare every rendered field.
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let mut state = fx.state().clone();
        let ts = |s: &str| Timestamp::parse(s).unwrap();
        state.created_at = ts("2026-09-24T01:00:00Z");
        state.updated_at = ts("2026-09-24T02:30:00Z");
        state.attempts[0].started_at = ts("2026-09-24T01:10:00Z");
        state.attempts[0].fail_reason =
            Some(crate::text::Summary::new("Timed out", "reason").unwrap());
        state.attempts[0].status = AttemptStatus::Failed;
        state.attempts[0].ended_at = Some(ts("2026-09-24T01:25:00Z"));
        state.blocked_count = 2;

        let text = render_status_card(&state, &fx.graph);
        assert!(text.contains("reason: Timed out"), "{text}");
        let stats = render_stats_json(&state, &fx.graph);
        assert_eq!(stats.total_seconds, 5400);
        assert_eq!(stats.nodes[0].avg_seconds, 900);
        assert_eq!(stats.blocked_count, 2);
        assert!(render_stats(&state, &fx.graph).contains("total: 5400s"));
        assert!(render_stats(&state, &fx.graph).contains("blocked: 2"));
    }

    // Task: T10
    #[test]
    fn stats_total_and_avg_use_timestamps() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "First").unwrap();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.1", "ok").unwrap();
        fx.begin("review").unwrap();
        let mut state = fx.state().clone();
        let ts = |s: &str| Timestamp::parse(s).unwrap();
        state.created_at = ts("2026-09-24T03:00:00Z");
        state.updated_at = ts("2026-09-24T04:00:00Z");
        state.attempts[0].started_at = ts("2026-09-24T03:00:00Z");
        state.attempts[0].ended_at = Some(ts("2026-09-24T03:00:10Z"));
        state.attempts[1].started_at = ts("2026-09-24T03:01:00Z");
        state.attempts[1].ended_at = Some(ts("2026-09-24T03:01:30Z"));
        // review#1.0 is still running, so exclude it from avg.
        state.attempts[2].started_at = ts("2026-09-24T03:02:00Z");
        let stats = render_stats_json(&state, &fx.graph);
        assert_eq!(stats.total_seconds, 3600);
        assert_eq!(stats.nodes[0].avg_seconds, 20);
        assert_eq!(stats.nodes[1].avg_seconds, 0);
        assert!(render_stats(&state, &fx.graph).contains("total: 3600s"));
    }
}
