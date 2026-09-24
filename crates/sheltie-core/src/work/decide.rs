//! 状态机的唯一入口 `decide`，以及每个命令一个私有函数。
//! 细则按 `specs/contracts/protocol.md` §3 逐条对应。

use std::collections::BTreeMap;

use super::command::{Command, Context, Decision, ObservedFile};
use super::state::{ArtifactRef, WorkState, WorkStatus};
use crate::error::{Error, Result};
use crate::flow::Graph;
use crate::ids::{AttemptId, NodeId};
use crate::path::AbsPath;

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
            reason: format!("{} 需要已存在的 Work", other.name()),
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
#[allow(unused_variables)]
fn guard_not_terminal(state: &WorkState) -> Result<()> {
    todo!("T09")
}

/// `work start` 第 2、6、7 步：核对起始输入键集合与图里全部 `start.<key>` 引用完全相等
/// （缺与多都是 `Error::InputMissing`），建初始状态 `current = entry#1`、`visits[entry] = 1`、
/// `status = Active`，回复 `Reply::Started`，效果 `RefreshStatusCard`。
#[allow(unused_variables)]
fn decide_start(graph: &Graph, cmd: &Command, ctx: &Context) -> Result<Decision> {
    todo!("T06")
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
#[allow(unused_variables)]
fn decide_begin(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
    observed_inputs: &BTreeMap<String, Option<ObservedFile>>,
    instruction_text: &str,
    ctx: &Context,
) -> Result<Decision> {
    todo!("T07")
}

/// 按节点的 `inputs[]` 逐条绑定。
///
/// - `Start { key }`：取 `state.inputs[key]`；观察摘要不符 `Error::ArtifactModified`。
/// - `Resource { path }`：路径 `state.workbook_dir().join(path)`；观察必须存在且摘要是它的摘要（首次绑定时以观察为准记录）。
/// - `Node { node, output }`：取 `latest_succeeded_of(node)` 的 `outputs[output]`；
///   没有时 `required` 为真报 `Error::InputUnavailable`，否则绑 `None`；
///   有时观察摘要必须等于记录，否则 `Error::ArtifactModified`。
#[allow(unused_variables)]
fn bind_inputs(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
    observed_inputs: &BTreeMap<String, Option<ObservedFile>>,
) -> Result<BTreeMap<String, Option<ArtifactRef>>> {
    todo!("T07")
}

/// `attempt submit` 第 1 到 6 步。
///
/// 1. Attempt 必须 `Running`，否则 `Error::AttemptNotRunning`。
/// 2. `summary` ≤ 4096 字节，否则 `Error::SummaryTooLong`。
/// 3. `check_outputs`。任一失败整体 `Err`，状态不变。
/// 4. 记录输出 `ArtifactRef`，`status = Succeeded`，`ended_at = ctx.now`。
/// 5. `status_after_success` 定 Work 状态（含 `gate`）。
/// 6. 回复 `Reply::AttemptSubmitted`；效果 `SealOutputs`、`RefreshStatusCard`。
#[allow(unused_variables)]
fn decide_submit(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    summary: &str,
    observed_outputs: &BTreeMap<String, Option<ObservedFile>>,
    ctx: &Context,
) -> Result<Decision> {
    todo!("T08")
}

/// 对照节点 `outputs[]` 校验观察：`required` 且缺 → `Error::OutputMissing`；
/// 超 `max_bytes` → `Error::OutputTooLarge`；可选且缺 → 跳过。返回要封存的引用。
#[allow(unused_variables)]
fn check_outputs(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    observed_outputs: &BTreeMap<String, Option<ObservedFile>>,
) -> Result<BTreeMap<String, ArtifactRef>> {
    todo!("T08")
}

/// 节点 Attempt 成功（或门槛刚批准）后 Work 的状态，按协议 `attempt submit` 第 5 步的顺序：
/// `consider_gate && node.gate && !approved` → `Blocked(Gate)`；无出边 → `Succeeded`；
/// 每条出边目标都 `visits >= max_visits` → `Blocked(NoLegalEdge)`；否则 `Active`。
#[allow(unused_variables)]
fn status_after_success(state: &WorkState, graph: &Graph, consider_gate: bool) -> WorkStatus {
    todo!("T08")
}

/// `attempt fail`：Attempt 必须 `Running`；`status = Failed`，记 `fail_reason`（≤ 4096）；
/// `retry == max_retries` 时 Work → `Blocked(RetriesExhausted)`。效果 `RefreshStatusCard`。
#[allow(unused_variables)]
fn decide_fail(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
    reason: &str,
    ctx: &Context,
) -> Result<Decision> {
    todo!("T08")
}

/// `gate approve`：Work 必须 `Blocked(Gate)` 且 `node == current.node`，否则 `Error::IllegalNext`。
/// 记 `Approval`，再用 `status_after_success(.., consider_gate = false)` 定状态。效果 `RefreshStatusCard`。
#[allow(unused_variables)]
fn decide_approve(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
    ctx: &Context,
) -> Result<Decision> {
    todo!("T09")
}

/// `work cancel`：`status = Cancelled`，`Running` 的 Attempt 保持原样。效果 `RefreshStatusCard`。
#[allow(unused_variables)]
fn decide_cancel(state: &WorkState, ctx: &Context) -> Result<Decision> {
    todo!("T09")
}

/// runtime 在 `attempt begin` 前调用：本节点每个输入当前应观察的路径。
/// `None` 表示可选输入的上游尚无产出，不用观察。
/// 规则与 `bind_inputs` 相同，只是不比摘要。
#[allow(unused_variables)]
pub fn input_paths_for(
    state: &WorkState,
    graph: &Graph,
    node: &NodeId,
) -> Result<BTreeMap<String, Option<AbsPath>>> {
    todo!("T07")
}

/// runtime 在 `attempt submit` 前调用：本 Attempt 每个声明输出的目标路径 `attempt_dir/<path>`。
#[allow(unused_variables)]
pub fn output_paths_for(
    state: &WorkState,
    graph: &Graph,
    attempt: &AttemptId,
) -> Result<BTreeMap<String, AbsPath>> {
    todo!("T08")
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

    // ── T06 Start ─────────────────────────────────────────────

    #[test]
    #[ignore = "T06"]
    fn t06_start_sets_current_to_entry_occurrence_1() {
        let mut fx = Fixture::article_review();
        let d = fx.start(&[("topic", "hello")]).unwrap();
        assert_eq!(d.state.current.node.as_str(), "draft");
        assert_eq!(d.state.current.n, 1);
        assert_eq!(d.state.visits_of(&node("draft")), 1);
        assert_eq!(d.state.status, WorkStatus::Active);
        assert!(d.state.attempts.is_empty());
    }

    #[test]
    #[ignore = "T06"]
    fn t06_start_rejects_missing_start_input_key() {
        let mut fx = Fixture::article_review();
        assert!(
            matches!(fx.start(&[]), Err(Error::InputMissing { missing, .. }) if missing == vec!["topic".to_string()])
        );
    }

    #[test]
    #[ignore = "T06"]
    fn t06_start_rejects_extra_start_input_key() {
        let mut fx = Fixture::article_review();
        assert!(
            matches!(fx.start(&[("topic", "a"), ("bonus", "b")]), Err(Error::InputMissing { extra, .. }) if extra == vec!["bonus".to_string()])
        );
    }

    #[test]
    #[ignore = "T06"]
    fn t06_start_next_is_begin_entry_and_cancel() {
        let mut fx = Fixture::article_review();
        let d = fx.start(&[("topic", "hello")]).unwrap();
        let next = legal_next(&d.state, &fx.graph);
        assert_eq!(next.len(), 2);
        assert!(
            matches!(&next[0], NextOp::BeginAttempt { node, edge: None, .. } if node.as_str() == "draft")
        );
        assert_eq!(next[1], NextOp::Cancel);
    }

    #[test]
    #[ignore = "T06"]
    fn t06_start_records_workbook_ref_and_frozen_inputs() {
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

    // ── T07 BeginAttempt ──────────────────────────────────────

    #[test]
    #[ignore = "T07"]
    fn t07_begin_on_entry_creates_running_attempt_with_frozen_inputs() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_begin_rejects_node_not_in_next() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_begin_via_edge_increments_visits_and_occurrence() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_begin_records_entered_from_occurrence_and_edge_kind() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_retry_keeps_entered_from_of_first_attempt() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_begin_filters_edges_whose_target_hit_max_visits() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_begin_rejects_modified_upstream_artifact() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.tamper_output("draft#1.0", "article");
        assert!(
            matches!(fx.begin("review"), Err(Error::ArtifactModified { input, .. }) if input == "article")
        );
    }

    #[test]
    #[ignore = "T07"]
    fn t07_begin_rejects_upstream_without_succeeded_attempt() {
        // 用一张自造的图：b 必需 a 的输出，但有边 a -> b 且 a 从未成功。构造方法见 testkit。
        let mut fx = Fixture::two_step_with_required_input_but_edge_before_success();
        assert!(matches!(
            fx.begin("second"),
            Err(Error::InputUnavailable { .. })
        ));
    }

    #[test]
    #[ignore = "T07"]
    fn t07_begin_leaves_optional_input_unbound_when_upstream_has_no_attempt() {
        let mut fx = Fixture::spec_dev().started_with(&[("request", "r"), ("project", "/p")]);
        let d = fx.begin("spec").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert_eq!(a.inputs["decision"], None);
        assert!(a.inputs["request"].is_some());
    }

    #[test]
    #[ignore = "T07"]
    fn t07_begin_binds_optional_input_when_upstream_succeeded_later() {
        let mut fx = Fixture::spec_dev().started_with(&[("request", "r"), ("project", "/p")]);
        fx.run_spec_dev_to_plan_review_returning("修改规格");
        let d = fx.begin("spec").unwrap();
        let a = d.state.latest_attempt_of_current().unwrap();
        assert!(a.inputs["decision"].is_some());
    }

    #[test]
    #[ignore = "T07"]
    fn t07_begin_after_failed_attempt_increments_retry_not_occurrence() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_begin_binds_resource_input_under_frozen_workbook_dir() {
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

    #[test]
    #[ignore = "T07"]
    fn t07_begin_reply_lists_node_requires() {
        let mut fx = Fixture::with_requires();
        let d = fx.begin("only").unwrap();
        assert!(
            matches!(d.reply, Reply::AttemptBegun { requires, .. } if requires == vec!["skill:company-api".to_string()])
        );
    }

    #[test]
    #[ignore = "T07"]
    fn t07_begin_emits_write_brief_effect() {
        let mut fx = Fixture::article_review().started();
        let d = fx.begin("draft").unwrap();
        assert!(d.effects.iter().any(|e| matches!(e, Effect::WriteBrief { path, content } if path.as_str().ends_with("attempts/draft/1/0/brief.md") && content.contains("# 任务书"))));
        assert!(d.effects.contains(&Effect::RefreshStatusCard));
    }

    // ── T08 Submit / Fail ─────────────────────────────────────

    #[test]
    #[ignore = "T08"]
    fn t08_submit_marks_attempt_succeeded_and_records_outputs() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_submit_rejects_when_attempt_not_running() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        assert!(matches!(
            fx.submit_ok("draft#1.0", "again"),
            Err(Error::AttemptNotRunning { .. })
        ));
    }

    #[test]
    #[ignore = "T08"]
    fn t08_submit_rejects_summary_over_4096_bytes() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        assert!(matches!(
            fx.submit_ok("draft#1.0", &"x".repeat(4097)),
            Err(Error::SummaryTooLong { .. })
        ));
    }

    #[test]
    #[ignore = "T08"]
    fn t08_submit_rejects_missing_required_output() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_submit_accepts_missing_optional_output() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_submit_rejects_output_over_max_bytes() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_submit_on_gate_node_blocks_work() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        let d = fx.submit_ok("notes#1.0", "写好了").unwrap();
        assert_eq!(d.state.status, WorkStatus::Blocked(BlockedReason::Gate));
    }

    #[test]
    #[ignore = "T08"]
    fn t08_submit_on_terminal_node_succeeds_work() {
        let mut fx = Fixture::two_step().started_with(&[("topic", "t")]);
        fx.begin("outline").unwrap();
        fx.submit_ok("outline#1.0", "ok").unwrap();
        fx.begin("summary").unwrap();
        let d = fx.submit_ok("summary#1.0", "ok").unwrap();
        assert_eq!(d.state.status, WorkStatus::Succeeded);
    }

    #[test]
    #[ignore = "T08"]
    fn t08_submit_on_terminal_gate_node_blocks_not_succeeds() {
        let mut fx = Fixture::single_gated_terminal();
        fx.begin("only").unwrap();
        let d = fx.submit_ok("only#1.0", "ok").unwrap();
        assert_eq!(d.state.status, WorkStatus::Blocked(BlockedReason::Gate));
    }

    #[test]
    #[ignore = "T08"]
    fn t08_submit_when_every_out_edge_target_hit_max_visits_blocks_no_legal_edge() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_fail_marks_attempt_failed_and_allows_retry() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_fail_at_max_retries_blocks_work() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_next_after_success_lists_out_edges_with_kind() {
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

    #[test]
    #[ignore = "T08"]
    fn t08_next_when_blocked_gate_has_only_approve_and_cancel() {
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

    #[test]
    #[ignore = "T09"]
    fn t09_approve_unblocks_and_records_principal_and_time() {
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

    #[test]
    #[ignore = "T09"]
    fn t09_approve_rejects_when_not_blocked_on_gate() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        assert!(matches!(
            fx.approve("notes"),
            Err(Error::IllegalNext { .. })
        ));
    }

    #[test]
    #[ignore = "T09"]
    fn t09_approve_rejects_wrong_node() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "ok").unwrap();
        assert!(matches!(
            fx.approve("archive"),
            Err(Error::IllegalNext { .. })
        ));
    }

    #[test]
    #[ignore = "T09"]
    fn t09_approve_on_terminal_node_succeeds_work() {
        let mut fx = Fixture::single_gated_terminal();
        fx.begin("only").unwrap();
        fx.submit_ok("only#1.0", "ok").unwrap();
        let d = fx.approve("only").unwrap();
        assert_eq!(d.state.status, WorkStatus::Succeeded);
    }

    #[test]
    #[ignore = "T09"]
    fn t09_cancel_from_active_and_blocked() {
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

    #[test]
    #[ignore = "T09"]
    fn t09_terminal_work_rejects_every_command_with_work_terminal() {
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
}
