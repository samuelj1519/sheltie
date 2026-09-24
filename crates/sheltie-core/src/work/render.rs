//! 任务书与状态卡的渲染。格式见 `specs/contracts/protocol.md` §4、§6。
//! 快照测试的期望文件在 `snapshots/` 下，由 T01 手写，是标准答案。

use std::collections::BTreeMap;

use serde::Serialize;

use super::next::{NextOp, legal_next};
use super::state::{Attempt, WorkState, WorkStatus};
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
#[allow(unused_variables)]
pub fn render_brief(
    state: &WorkState,
    graph: &Graph,
    attempt: &Attempt,
    instruction_text: &str,
) -> String {
    todo!("T10")
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
#[allow(unused_variables)]
pub fn render_status_card(state: &WorkState, graph: &Graph) -> String {
    todo!("T10")
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
#[allow(unused_variables)]
pub fn status_card_json(state: &WorkState, graph: &Graph) -> StatusCardJson {
    let _ = legal_next;
    todo!("T10")
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
#[allow(unused_variables)]
pub fn render_stats_json(state: &WorkState, graph: &Graph) -> StatsJson {
    todo!("T10")
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
#[allow(unused_variables)]
pub fn render_stats(state: &WorkState, graph: &Graph) -> String {
    todo!("T10")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::Fixture;
    use crate::work::next::NextOp;

    #[test]
    #[ignore = "T10"]
    fn t10_brief_for_review_node() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        fx.submit_ok("draft#1.0", "ok").unwrap();
        fx.begin("review").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        insta::assert_snapshot!(render_brief(fx.state(), &fx.graph, a, "审查这篇文章。"));
    }

    #[test]
    #[ignore = "T10"]
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
    #[ignore = "T10"]
    fn t10_brief_for_node_without_requires_omits_section() {
        let mut fx = Fixture::article_review().started();
        fx.begin("draft").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "写初稿。");
        assert!(!text.contains("需要的宿主资源"));
    }

    #[test]
    #[ignore = "T10"]
    fn t10_brief_marks_unbound_optional_input_as_absent() {
        let mut fx = Fixture::spec_dev().started_with(&[("request", "r"), ("project", "/p")]);
        fx.begin("spec").unwrap();
        let a = fx.state().latest_attempt_of_current().unwrap();
        let text = render_brief(fx.state(), &fx.graph, a, "写规格。");
        assert!(text.contains("| decision | 尚无"));
    }

    #[test]
    #[ignore = "T10"]
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
    #[ignore = "T10"]
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
    #[ignore = "T10"]
    fn t10_status_card_active_mid_flow() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        insta::assert_snapshot!(render_status_card(fx.state(), &fx.graph));
    }

    #[test]
    #[ignore = "T10"]
    fn t10_status_card_blocked_on_gate() {
        let mut fx = Fixture::gated_release().started_with(&[("version", "1.0")]);
        fx.begin("notes").unwrap();
        fx.submit_ok("notes#1.0", "写好了").unwrap();
        insta::assert_snapshot!(render_status_card(fx.state(), &fx.graph));
    }

    #[test]
    #[ignore = "T10"]
    fn t10_status_card_succeeded() {
        let mut fx = Fixture::two_step().started_with(&[("topic", "t")]);
        fx.begin("outline").unwrap();
        fx.submit_ok("outline#1.0", "提纲好了").unwrap();
        fx.begin("summary").unwrap();
        fx.submit_ok("summary#1.0", "摘要好了").unwrap();
        insta::assert_snapshot!(render_status_card(fx.state(), &fx.graph));
    }

    #[test]
    #[ignore = "T10"]
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
    #[ignore = "T10"]
    fn t10_next_op_begin_carries_executor_and_tier() {
        let fx = Fixture::article_review().started();
        let json = serde_json::to_value(&legal_next(fx.state(), &fx.graph)[0]).unwrap();
        assert_eq!(json["op"], "attempt begin");
        assert_eq!(json["executor"], "agent");
        assert_eq!(json["tier"], "standard");
    }

    #[test]
    #[ignore = "T10"]
    fn t10_stats_table_mid_flow() {
        let mut fx = Fixture::article_review().started();
        fx.run_to_review_done_not_passing();
        fx.begin("draft").unwrap();
        insta::assert_snapshot!(render_stats(fx.state(), &fx.graph));
    }

    #[test]
    #[ignore = "T10"]
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
    #[ignore = "T10"]
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
}
