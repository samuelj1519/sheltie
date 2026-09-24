//! 任务书与状态卡的渲染。格式见 `specs/contracts/protocol.md` §4、§6。
//! 快照测试的期望文件在 `snapshots/` 下，由 T01 手写，是标准答案。

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::next::{NextOp, legal_next};
use super::state::{Attempt, AttemptStatus, BlockedReason, Timestamp, WorkState, WorkStatus};
use crate::flow::Graph;
use crate::ids::WorkId;

/// 渲染一份 Attempt 的任务书（协议 §4）。快照 `snapshots/*brief*.snap` 是逐字节的标准答案。
///
/// 逐行格式（`\n` 分隔，末尾一个换行）：
///
/// ```text
/// # 任务书：<node.title>
/// <空行>
/// Work: <work_id>（<name>）
/// 节点: <node>#<n>，第 <retry> 次尝试
/// 来自: <occ>（<kind> 边）            入口为「来自: 入口」
/// 执行者: agent（<tier>）             human 为「执行者: human」
/// <空行>
/// ## 输入
/// <空行>
/// | 名称 | 路径 | sha256 |
/// | --- | --- | --- |
/// | <name> | <绝对路径> | <64 位摘要> |        按节点 inputs[] 声明顺序
/// | <name> | 尚无 | |                         可选且未绑定
/// <空行>
/// ## 需要的宿主资源                           节点 requires 非空时才有此节
/// <空行>
/// | 类型 | 名称 | 版本 | 说明 |
/// | --- | --- | --- | --- |
/// | <kind> | <name> | <version 或 -> | 请确认你的宿主已装此资源；未装请停下并告知用户 |
/// <空行>
/// ## 说明
/// <空行>
/// <instruction_text 原文，去掉末尾换行后再补一个换行>
/// <空行>
/// ## 输出要求
/// <空行>
/// | 名称 | 写到 | 必需 | 上限 |
/// | --- | --- | --- | --- |
/// | <name> | <attempt_dir>/<path> | 是/否 | <human_size(max_bytes)> |
/// <空行>
/// 完成后不要自己修改输入文件。回复协调者时用几句话说明结论，并列出你写了哪些输出文件。
/// ```
///
/// human 执行者时最后一段换成：
///
/// ```text
/// 写完输出文件后，在终端运行：
/// <空行>
///     sheltie attempt submit <work_id> --attempt <attempt_id> --summary "<一句话结论>"
/// ```
///
/// 没有输入时「## 输入」节只有表头两行。`human_size`：能整除 1 MiB 写 `N MiB`，能整除 1 KiB 写 `N KiB`，否则 `N B`。
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
    out.push_str(&format!("# 任务书：{}\n\n", node.title));
    out.push_str(&format!("Work: {}（{}）\n", state.work_id, state.name));
    out.push_str(&format!(
        "节点: {}，第 {} 次尝试\n",
        attempt.occurrence(),
        attempt.id.retry
    ));
    match &attempt.entered_from {
        None => out.push_str("来自: 入口\n"),
        Some((occ, kind)) => {
            out.push_str(&format!("来自: {occ}（{} 边）\n", kind.as_str()));
        }
    }
    match node.executor {
        crate::flow::Executor::Agent => {
            let tier = node.tier.unwrap_or_default();
            out.push_str(&format!("执行者: agent（{}）\n", tier.as_str()));
        }
        crate::flow::Executor::Human => out.push_str("执行者: human\n"),
    }
    out.push('\n');

    out.push_str("## 输入\n\n");
    out.push_str("| 名称 | 路径 | sha256 |\n");
    out.push_str("| --- | --- | --- |\n");
    for decl in &node.inputs {
        match attempt.inputs.get(&decl.name).and_then(|o| o.as_ref()) {
            Some(r) => out.push_str(&format!(
                "| {} | {} | {} |\n",
                decl.name,
                r.path,
                r.sha256.as_str()
            )),
            None => out.push_str(&format!("| {} | 尚无 | |\n", decl.name)),
        }
    }

    if !node.requires.is_empty() {
        out.push_str("\n## 需要的宿主资源\n\n");
        out.push_str("| 类型 | 名称 | 版本 | 说明 |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for (kind, name) in &node.requires {
            // 版本从 Workbook 的 requires 声明里查（合同 §4 模板）；没声明版本就写 `-`。
            let version = graph
                .requires()
                .iter()
                .find(|r| r.kind == *kind && r.name == *name)
                .and_then(|r| r.version.as_deref())
                .unwrap_or("-");
            out.push_str(&format!(
                "| {} | {} | {} | 请确认你的宿主已装此资源；未装请停下并告知用户 |\n",
                kind.as_str(),
                name,
                version
            ));
        }
    }

    out.push_str("\n## 说明\n\n");
    out.push_str(instruction_text.trim_end_matches('\n'));
    out.push('\n');

    let attempt_dir = state.attempt_dir(&attempt.id);
    out.push_str("\n## 输出要求\n\n");
    out.push_str("| 名称 | 写到 | 必需 | 上限 |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for decl in &node.outputs {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            decl.name,
            attempt_dir.join(&decl.path),
            if decl.required { "是" } else { "否" },
            human_size(decl.max_bytes)
        ));
    }

    out.push('\n');
    match node.executor {
        crate::flow::Executor::Agent => {
            out.push_str("完成后不要自己修改输入文件。回复协调者时用几句话说明结论，并列出你写了哪些输出文件。\n");
        }
        crate::flow::Executor::Human => {
            out.push_str("写完输出文件后，在终端运行：\n\n");
            out.push_str(&format!(
                "    sheltie attempt submit {} --attempt {} --summary \"<一句话结论>\"\n",
                state.work_id, attempt.id
            ));
        }
    }
    out
}

/// `N MiB` / `N KiB` / `N B`。0 写 `0 B`。
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

/// 渲染状态卡（协议 §6）。快照 `snapshots/*status_card*.snap` 是标准答案。
///
/// ```text
/// # Work <work_id>（<name>）
/// <空行>
/// workbook: <id>@<version>   flow: <flow>   status: <status>     status 用 WorkStatus 的 Display，如 blocked(gate)
/// current: <occ>
/// done: <occ>, <occ>                                              全部 Succeeded 的 Occurrence，按提交先后；没有写「无」
/// pending: <node>, <node>                                         从未到达的节点，按图声明顺序；没有写「无」
/// visits: draft 2/3, review 1/3, publish 0/1                      按图声明顺序，visits/max_visits
/// blocked: <reason>: <说明>                                       只在 Blocked 时有；说明见协议 §6
/// <空行>
/// ## 最近一次尝试
/// <空行>
/// <attempt_id> <status>                                           attempts.last()；没有任何 Attempt 写「无」
/// summary: <summary>                                              仅 Succeeded 且有摘要
/// reason: <fail_reason>                                           仅 Failed 且有原因
/// outputs:                                                        仅 Succeeded 且有输出；每个输出一行，缩进两格
///   <name> → <path> (sha256 <64 位>, <human_size(bytes)>)
/// <空行>
/// ## 合法下一步
/// <空行>
/// - <to_command_line>                                             每项一行；终态时写「- 无」
/// ```
///
/// `blocked` 行的说明：`gate: <occ> 需要 gate approve`；`retries_exhausted: <occ>`；
/// `no_legal_edge: <occ> 的全部出边目标已达 max_visits`。
pub fn render_status_card(state: &WorkState, graph: &Graph) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Work {}（{}）\n\n", state.work_id, state.name));
    out.push_str(&format!(
        "workbook: {}@{}   flow: {}   status: {}\n",
        state.workbook.id, state.workbook.version, state.flow, state.status
    ));
    out.push_str(&format!("current: {}\n", state.current));

    let done = done_occurrences(state);
    out.push_str(&format!(
        "done: {}\n",
        if done.is_empty() {
            "无".to_string()
        } else {
            done.join(", ")
        }
    ));
    let pending = pending_nodes(state, graph);
    out.push_str(&format!(
        "pending: {}\n",
        if pending.is_empty() {
            "无".to_string()
        } else {
            pending.join(", ")
        }
    ));
    let visits = visit_items(state, graph)
        .into_iter()
        .map(|(n, m)| format!("{n} {m}"))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("visits: {visits}\n"));
    if let Some(line) = blocked_line(state) {
        out.push_str(&line);
        out.push('\n');
    }

    out.push_str("\n## 最近一次尝试\n\n");
    match state.attempts.last() {
        None => out.push_str("无\n"),
        Some(a) => {
            out.push_str(&format!("{} {}\n", a.id, a.status.as_str()));
            match a.status {
                AttemptStatus::Succeeded => {
                    if let Some(s) = &a.summary {
                        out.push_str(&format!("summary: {}\n", s.as_str()));
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
                    if let Some(s) = &a.fail_reason {
                        out.push_str(&format!("reason: {}\n", s.as_str()));
                    }
                }
                AttemptStatus::Running => {}
            }
        }
    }

    out.push_str("\n## 合法下一步\n\n");
    let next = legal_next(state, graph);
    if next.is_empty() {
        out.push_str("- 无\n");
    } else {
        for op in &next {
            out.push_str(&format!("- {}\n", op.to_command_line(&state.work_id)));
        }
    }
    out
}

/// `--json` 用的状态卡。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    pub last_attempt: Option<LastAttemptJson>,
    pub next: Vec<NextOp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LastAttemptJson {
    pub attempt: String,
    pub status: String,
    pub summary: Option<String>,
    pub outputs: BTreeMap<String, String>,
}

/// 状态卡的结构化形式。字段与文本卡一致。
pub fn status_card_json(state: &WorkState, graph: &Graph) -> StatusCardJson {
    StatusCardJson {
        work_id: state.work_id.clone(),
        name: state.name.to_string(),
        workbook: format!("{}@{}", state.workbook.id, state.workbook.version),
        flow: state.flow.to_string(),
        status: state.status,
        current: state.current.to_string(),
        done: done_occurrences(state),
        pending: pending_nodes(state, graph),
        visits: visit_items(state, graph).into_iter().collect(),
        last_attempt: state.attempts.last().map(|a| LastAttemptJson {
            attempt: a.id.to_string(),
            status: a.status.as_str().to_string(),
            summary: a.summary.as_ref().map(|s| s.as_str().to_string()),
            outputs: a
                .outputs
                .iter()
                .map(|(name, r)| (name.clone(), r.path.to_string()))
                .collect(),
        }),
        next: legal_next(state, graph),
    }
}

/// 全部 Succeeded 的 Occurrence，按提交先后。
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

/// 从未到达的节点 id，按图声明顺序。
fn pending_nodes(state: &WorkState, graph: &Graph) -> Vec<String> {
    graph
        .nodes()
        .filter(|d| state.visits_of(&d.id) == 0)
        .map(|d| d.id.as_str().to_string())
        .collect()
}

/// `(node, "n/m")`，按图声明顺序。
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

/// `blocked` 行；非 `Blocked` 为 `None`。
fn blocked_line(state: &WorkState) -> Option<String> {
    let occ = state.current.to_string();
    match state.status {
        WorkStatus::Blocked(BlockedReason::Gate) => {
            Some(format!("blocked: gate: {occ} 需要 gate approve"))
        }
        WorkStatus::Blocked(BlockedReason::RetriesExhausted) => {
            Some(format!("blocked: retries_exhausted: {occ}"))
        }
        WorkStatus::Blocked(BlockedReason::NoLegalEdge) => Some(format!(
            "blocked: no_legal_edge: {occ} 的全部出边目标已达 max_visits"
        )),
        _ => None,
    }
}

/// 事实视图（协议 `work stats`）：每个节点被到达几次、尝试几次、失败几次、平均耗时、从哪些节点经哪种边进来。
/// 只是对 `WorkState` 的计数，不含任何判断。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StatsJson {
    pub work_id: WorkId,
    pub status: WorkStatus,
    /// 从 `created_at` 到 `updated_at` 的秒数。
    pub total_seconds: u64,
    pub blocked_count: u32,
    pub approvals: u32,
    /// 按图声明顺序。
    pub nodes: Vec<NodeStatsJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeStatsJson {
    pub node: String,
    pub visits: u32,
    pub max_visits: u32,
    pub attempts: u32,
    pub failed: u32,
    /// 已结束 Attempt 的平均秒数；没有则 0。
    pub avg_seconds: u64,
    /// `"<from>×<n>"` 或 `"entry×<n>"`，按首次出现顺序。
    pub entered_via: Vec<String>,
}

/// 事实视图的结构化形式。`total_seconds` 与 `avg_seconds` 由 RFC 3339 时间串相减得到；解析不了的当 0。
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

        // entered_via 按 Occurrence 计（重试沿用同一来源），首次出现顺序。
        let mut order: Vec<String> = Vec::new();
        let mut counts: BTreeMap<String, u32> = BTreeMap::new();
        let mut seen_occ = BTreeSet::new();
        for a in &attempts {
            if !seen_occ.insert(a.id.occurrence) {
                continue;
            }
            let key = match &a.entered_from {
                Some((occ, _)) => occ.node.as_str().to_string(),
                None => "entry".to_string(),
            };
            if !counts.contains_key(&key) {
                order.push(key.clone());
            }
            *counts.entry(key).or_insert(0) += 1;
        }
        let entered_via = order
            .into_iter()
            .map(|k| format!("{k}×{}", counts[&k]))
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
            avg_seconds: ended_secs.checked_div(ended_n).unwrap_or(0),
            entered_via,
        });
    }
    StatsJson {
        work_id: state.work_id.clone(),
        status: state.status,
        total_seconds: secs_between(&state.created_at, &state.updated_at),
        blocked_count: count_blocks(state, graph),
        approvals: state.approvals.len() as u32,
        nodes,
    }
}

/// 被阻断过的次数：门槛成功、重试耗尽，加上当前 `no_legal_edge`。
fn count_blocks(state: &WorkState, graph: &Graph) -> u32 {
    let mut latest: BTreeMap<(crate::ids::NodeId, u32), &Attempt> = BTreeMap::new();
    for a in &state.attempts {
        latest.insert((a.id.node.clone(), a.id.occurrence), a);
    }
    let mut n = 0u32;
    for a in latest.values() {
        let Some(def) = graph.node(&a.id.node) else {
            continue;
        };
        match a.status {
            AttemptStatus::Succeeded if def.gate => n += 1,
            AttemptStatus::Failed if a.id.retry >= def.max_retries => n += 1,
            _ => {}
        }
    }
    if state.status == WorkStatus::Blocked(BlockedReason::NoLegalEdge) {
        n += 1;
    }
    n
}

/// RFC 3339（`…Z`）到秒；解析不了当 0。
fn secs_between(a: &Timestamp, b: &Timestamp) -> u64 {
    let (Some(x), Some(y)) = (rfc3339_secs(a), rfc3339_secs(b)) else {
        return 0;
    };
    (y - x).max(0) as u64
}

fn rfc3339_secs(ts: &Timestamp) -> Option<i64> {
    let s = ts.0.strip_suffix('Z')?;
    let s = s.split('.').next()?;
    let (d, t) = s.split_once('T')?;
    let mut dp = d.split('-');
    let y: i64 = dp.next()?.parse().ok()?;
    let mo: i64 = dp.next()?.parse().ok()?;
    let day: i64 = dp.next()?.parse().ok()?;
    let mut tp = t.split(':');
    let h: i64 = tp.next()?.parse().ok()?;
    let mi: i64 = tp.next()?.parse().ok()?;
    let sec: i64 = tp.next()?.parse().ok()?;
    let (y2, mp) = if mo > 2 { (y, mo - 3) } else { (y - 1, mo + 9) };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146097 + doe) * 86400 + h * 3600 + mi * 60 + sec)
}

/// 事实视图的文本表（协议 `work stats`）。快照 `*stats_table*.snap` 是标准答案。
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
    let s = render_stats_json(state, graph);
    let mut out = String::new();
    out.push_str(&format!("# Stats {}\n\n", s.work_id));
    out.push_str(&format!(
        "status: {}   total: {}s   blocked: {}   approvals: {}\n\n",
        s.status, s.total_seconds, s.blocked_count, s.approvals
    ));
    out.push_str("| node | visits | attempts | failed | avg | entered_via |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for n in &s.nodes {
        out.push_str(&format!(
            "| {} | {}/{} | {} | {} | {}s | {} |\n",
            n.node,
            n.visits,
            n.max_visits,
            n.attempts,
            n.failed,
            n.avg_seconds,
            n.entered_via.join(", ")
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::Fixture;
    use crate::work::next::NextOp;

    #[test]
    fn t10_brief_for_review_node() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        insta::assert_snapshot!(render_brief(fx.state(), &fx.graph, a, "审查这篇文章。"));
    }

    #[test]
    fn t10_brief_for_node_with_requires() {
        let mut fx = Fixture::with_requires();
        fx.begin("only").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        insta::assert_snapshot!(render_brief(
            fx.state(),
            &fx.graph,
            a,
            "用公司 API 做点事。"
        ));
    }

    #[test]
    fn t10_brief_for_node_without_requires_omits_section() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "写初稿。");
        assert!(!text.contains("需要的宿主资源"));
    }

    // ── M1 复核 O3：版本列来自 Workbook 的 requires 声明，没声明版本写 `-` ─────

    #[test]
    fn t10_brief_require_version_column_comes_from_manifest() {
        use crate::digest::Sha256Hex;
        use crate::flow::{ResourceIndex, compile, parse_flow};
        use crate::ids::{FlowId, NodeId, WorkId, WorkName};
        use crate::path::AbsPath;
        use crate::work::WorkbookRef;
        use crate::work::command::Command;
        use crate::work::decide::decide;
        use crate::workbook::parse_manifest;

        let manifest = parse_manifest(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"单节点\"\nflows = [\"flows/default.toml\"]\n[[requires]]\nkind = \"skill\"\nname = \"company-api\"\nversion = \"^1\"\n",
        )
        .unwrap();
        let def = parse_flow(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = { text = \"用公司 API 做点事。\" }\nrequires = [\"skill:company-api\"]\n",
        )
        .unwrap();
        let graph = compile(&def, &manifest, &ResourceIndex::default()).unwrap();
        let start = Command::Start {
            work_id: WorkId::new("2026-09-24", 1, &WorkName::normalize("t").unwrap()).unwrap(),
            name: WorkName::normalize("t").unwrap(),
            workbook: WorkbookRef {
                id: manifest.id.clone(),
                version: manifest.version.clone(),
                digest: Sha256Hex::of_bytes(b"fixture-workbook"),
            },
            flow: FlowId::new("default").unwrap(),
            work_dir: AbsPath::new("/sheltie-test/works/2026-09-24-001-t").unwrap(),
            inputs: BTreeMap::new(),
        };
        let d0 = decide(None, &graph, &start, &crate::testkit::ctx()).unwrap();
        let d1 = decide(
            Some(&d0.state),
            &graph,
            &Command::BeginAttempt {
                node: NodeId::new("only").unwrap(),
                observed_inputs: BTreeMap::new(),
                instruction_text: "用公司 API 做点事。".to_string(),
            },
            &crate::testkit::ctx(),
        )
        .unwrap();
        let a = d1.state.latest_attempt_of_current().unwrap();
        let text = render_brief(&d1.state, &graph, a, "用公司 API 做点事。");
        assert!(text.contains(
            "| skill | company-api | ^1 | 请确认你的宿主已装此资源；未装请停下并告知用户 |"
        ));
    }

    #[test]
    fn t10_brief_marks_unbound_optional_input_as_absent() {
        let mut fx = Fixture::spec_dev().started_with(&[("request", "r"), ("project", "/p")]);
        fx.begin("spec").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "写规格。");
        assert!(text.contains("| decision | 尚无"));
    }

    #[test]
    fn t10_brief_for_human_executor_ends_with_submit_command() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "通过").unwrap();
        fx.begin("publish").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        insta::assert_snapshot!(render_brief(fx.state(), &fx.graph, a, "确认可以发布。"));
    }

    #[test]
    fn t10_brief_shows_entered_from_line_or_entry() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let entry = render_brief(
            fx.state(),
            &fx.graph,
            fx.state().latest_attempt_of_current().unwrap(),
            "x",
        );
        assert!(entry.contains("来自: 入口"));
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        let via = render_brief(
            fx.state(),
            &fx.graph,
            fx.state().latest_attempt_of_current().unwrap(),
            "x",
        );
        assert!(via.contains("来自: draft#1（main 边）"));
    }

    #[test]
    fn t10_status_card_active_mid_flow() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        insta::assert_snapshot!(render_status_card(fx.state(), &fx.graph));
    }

    #[test]
    fn t10_status_card_blocked_on_gate() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "写好了").unwrap();
        insta::assert_snapshot!(render_status_card(fx.state(), &fx.graph));
    }

    #[test]
    fn t10_status_card_succeeded() {
        let mut fx = Fixture::two_step().started_with(&[("topic", "t")]);
        fx.begin("outline").unwrap();
        fx.submit_ok("outline#1.0", "提纲好了").unwrap();
        fx.begin("summary").unwrap();
        fx.submit_ok("summary#1.0", "摘要好了").unwrap();
        insta::assert_snapshot!(render_status_card(fx.state(), &fx.graph));
    }

    #[test]
    fn t10_next_op_renders_begin_with_node_flag() {
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

    #[test]
    fn t10_next_op_begin_carries_executor_and_tier() {
        let fx = Fixture::article_review().started();
        let json = serde_json::to_value(&legal_next(fx.state(), &fx.graph)[0]).unwrap();
        assert_eq!(json["op"], "attempt begin");
        assert_eq!(json["executor"], "agent");
        assert_eq!(json["tier"], "standard");
    }

    #[test]
    fn t10_stats_table_mid_flow() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        insta::assert_snapshot!(render_stats(fx.state(), &fx.graph));
    }

    #[test]
    fn t10_stats_json_counts_visits_failures_and_entered_via() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "崩").unwrap();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.1", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "不通过").unwrap();
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
        assert_eq!(draft.entered_via, vec!["entry×1", "review×1"]);
        assert_eq!(s.nodes[1].entered_via, vec!["draft×1"]);
        assert_eq!(s.nodes[2].attempts, 0);
        assert_eq!(s.blocked_count, 0);
    }

    #[test]
    fn t10_status_card_lists_done_occurrences_in_order() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#2.0", "改了").unwrap();
        let card = status_card_json(fx.state(), &fx.graph);
        assert_eq!(card.done, vec!["draft#1", "review#1", "draft#2"]);
        assert_eq!(card.pending, vec!["publish"]);
        assert_eq!(card.current, "draft#2");
    }

    // ── M1 补测（阻断行、阻断计数、时间换算；夹具时钟固定，快照里全是 0s） ─────

    fn exhausted_draft() -> Fixture {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "一").unwrap();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.1", "二").unwrap();
        fx
    }

    fn no_legal_edge() -> Fixture {
        let mut fx = Fixture::article_review_with_review_max_visits_1().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        fx.submit_ok("review#1.0", "不通过").unwrap();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#2.0", "改了").unwrap();
        fx
    }

    #[test]
    fn t10_status_card_names_retries_exhausted_occurrence() {
        let fx = exhausted_draft();
        let card = render_status_card(fx.state(), &fx.graph);
        assert!(
            card.contains("blocked: retries_exhausted: draft#1\n"),
            "{card}"
        );
    }

    #[test]
    fn t10_status_card_explains_no_legal_edge() {
        let fx = no_legal_edge();
        let card = render_status_card(fx.state(), &fx.graph);
        assert!(
            card.contains("blocked: no_legal_edge: draft#2 的全部出边目标已达 max_visits"),
            "{card}"
        );
    }

    #[test]
    fn t10_stats_blocked_count_by_reason() {
        // 普通成功与可重试的失败都不算阻断。
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "一").unwrap();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 0);
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.1", "ok").unwrap();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 0);

        let fx = exhausted_draft();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);

        let fx = no_legal_edge();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);

        // 门槛成功算一次，批准之后仍算。
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "写好了").unwrap();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);
        fx.approve("notes").unwrap();
        assert_eq!(render_stats_json(fx.state(), &fx.graph).blocked_count, 1);
    }

    #[test]
    fn t10_secs_between_matches_independent_calendar_math() {
        // 期望值由 Python datetime 独立算出。
        let cases = [
            ("2026-02-28T23:59:30Z", "2026-03-01T00:00:30Z", 60),
            ("2024-02-28T00:00:00Z", "2024-03-01T00:00:00Z", 172_800),
            ("1999-12-31T23:59:59Z", "2000-03-01T00:00:00Z", 5_184_001),
            (
                "1970-01-01T00:00:00Z",
                "2026-09-24T03:04:05.678Z",
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
        for (a, b, want) in cases {
            let got = secs_between(&Timestamp(a.to_string()), &Timestamp(b.to_string()));
            assert_eq!(got, want, "{a} → {b}");
        }
        // 倒序夹到 0；不是 `…Z` 的形状解析不了，当 0。
        let ts = |s: &str| Timestamp(s.to_string());
        assert_eq!(
            secs_between(&ts("2026-01-01T00:00:09Z"), &ts("2026-01-01T00:00:00Z")),
            0
        );
        assert_eq!(
            secs_between(
                &ts("2026-01-01T00:00:00+08:00"),
                &ts("2026-01-01T00:00:09Z")
            ),
            0
        );
    }

    #[test]
    fn t10_stats_total_and_avg_use_timestamps() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.fail("draft#1.0", "一").unwrap();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.1", "ok").unwrap();
        fx.begin("review").unwrap();
        let mut state = fx.state().clone();
        let ts = |s: &str| Timestamp(s.to_string());
        state.created_at = ts("2026-09-24T03:00:00Z");
        state.updated_at = ts("2026-09-24T04:00:00Z");
        state.attempts[0].started_at = ts("2026-09-24T03:00:00Z");
        state.attempts[0].ended_at = Some(ts("2026-09-24T03:00:10Z"));
        state.attempts[1].started_at = ts("2026-09-24T03:01:00Z");
        state.attempts[1].ended_at = Some(ts("2026-09-24T03:01:30Z"));
        // review#1.0 仍在运行：不计入 avg。
        state.attempts[2].started_at = ts("2026-09-24T03:02:00Z");
        let stats = render_stats_json(&state, &fx.graph);
        assert_eq!(stats.total_seconds, 3600);
        assert_eq!(stats.nodes[0].avg_seconds, 20);
        assert_eq!(stats.nodes[1].avg_seconds, 0);
        assert!(render_stats(&state, &fx.graph).contains("total: 3600s"));
    }
}
