//! 把 `FlowDef` 编译成 `Graph`。九条规则见 `specs/contracts/workbook.md` §4。
//!
//! 每条规则一个私有函数，按顺序调用，任一失败整体拒绝。
//! 错误一律 `Error::FlowInvalid { rule: "<编号>", path, reason }`。

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::def::{EdgeDef, FlowDef};
use super::graph::{Graph, ResourceIndex};
use crate::error::Result;
use crate::ids::NodeId;
use crate::workbook::Manifest;

/// 编译入口。全部规则通过才返回 `Graph`。
pub fn compile(def: &FlowDef, manifest: &Manifest, res: &ResourceIndex) -> Result<Graph> {
    let out_edges = build_out_edges(&def.edges);
    check_rule_1(def)?;
    check_rule_2(def)?;
    check_rule_3(def, &out_edges)?;
    check_rule_4(def, &out_edges)?;
    check_rule_5(def, &out_edges)?;
    check_rule_6(def)?;
    check_rule_7(def, res)?;
    check_rule_8(def, manifest)?;
    check_rule_9(def)?;
    Ok(Graph::from_checked(
        def.entry.clone(),
        def.nodes.clone(),
        def.edges.clone(),
    ))
}

/// 邻接表：`from -> [to]`。规则 2 之前边可能有重复或未知节点，这里不校验，只收集。
fn build_out_edges(edges: &[EdgeDef]) -> BTreeMap<NodeId, Vec<NodeId>> {
    let mut map: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
    for e in edges {
        map.entry(e.from.clone()).or_default().push(e.to.clone());
    }
    map
}

/// 从 `start` 沿出边能到达的全部节点（含自身）。BFS。
#[allow(unused_variables)]
pub(crate) fn reachable_from(
    out_edges: &BTreeMap<NodeId, Vec<NodeId>>,
    start: &NodeId,
) -> BTreeSet<NodeId> {
    let _ = VecDeque::<NodeId>::new();
    todo!("T05")
}

/// 规则 1：所有 ID 合规、唯一；节点 id 不是保留字（`def::RESERVED_NODE_IDS`：`start`、`resource`、`engine`）；`entry` 存在。
/// 节点数 1..=64，边数 ≤ 256。
#[allow(unused_variables)]
fn check_rule_1(def: &FlowDef) -> Result<()> {
    todo!("T05")
}

/// 规则 2：每条边两端存在、不自环、`(from, to)` 不重复。
#[allow(unused_variables)]
fn check_rule_2(def: &FlowDef) -> Result<()> {
    todo!("T05")
}

/// 规则 3：从 `entry` 出发每个节点可达。
#[allow(unused_variables)]
fn check_rule_3(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    todo!("T05")
}

/// 规则 4：至少一个没有出边的节点。
#[allow(unused_variables)]
fn check_rule_4(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    todo!("T05")
}

/// 规则 5：`inputs[].from` 的 `Node` 来源存在、不是自己、输出名存在，且从被引用节点能到达本节点。
/// 另：`Start`、`Resource`、`EngineStats` 来源上 `required = false` 拒绝。
#[allow(unused_variables)]
fn check_rule_5(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    todo!("T05")
}

/// 规则 6：`gate = true` 的节点 `instruction` 不得是空白文本。
#[allow(unused_variables)]
fn check_rule_6(def: &FlowDef) -> Result<()> {
    todo!("T05")
}

/// 规则 7：`instruction.file` 存在、≤ 64 KiB、UTF-8；`resource.<path>` 输入存在、≤ 32 MiB（编码不限）。
#[allow(unused_variables)]
fn check_rule_7(def: &FlowDef, res: &ResourceIndex) -> Result<()> {
    todo!("T05")
}

/// 规则 8：节点 `requires[]` 每项在 manifest 里有声明；同一节点内不重复。
#[allow(unused_variables)]
fn check_rule_8(def: &FlowDef, manifest: &Manifest) -> Result<()> {
    todo!("T05")
}

/// 规则 9：`executor = human` 的节点 `tier` 为 `None`。
#[allow(unused_variables)]
fn check_rule_9(def: &FlowDef) -> Result<()> {
    todo!("T05")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::flow::parse::parse_flow;
    use crate::testkit;

    fn rule_of(err: Error) -> &'static str {
        match err {
            Error::FlowInvalid { rule, .. } => rule,
            other => panic!("不是 FlowInvalid：{other:?}"),
        }
    }

    /// 用 article-review 样例作为合法基线，改一处得到反例。
    fn compile_text(flow_text: &str) -> Result<Graph> {
        let def = parse_flow(flow_text)?;
        compile(
            &def,
            &testkit::article_review_manifest(),
            &testkit::article_review_resources(),
        )
    }

    fn base() -> &'static str {
        testkit::ARTICLE_REVIEW_FLOW
    }

    #[test]
    #[ignore = "T05"]
    fn t05_compiles_article_review_example() {
        let g = compile_text(base()).unwrap();
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 3);
        assert!(g.is_terminal(&NodeId::new("publish").unwrap()));
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_entry_not_a_node() {
        assert_eq!(
            rule_of(
                compile_text(&base().replace("entry = \"draft\"", "entry = \"nope\"")).unwrap_err()
            ),
            "1"
        );
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_node_id_start_or_resource() {
        let text = format!(
            "{}\n[[nodes]]\nid = \"start\"\ntitle = \"S\"\nexecutor = \"agent\"\ninstruction = {{ text = \"s\" }}\n[[edges]]\nfrom = \"publish\"\nto = \"start\"\nkind = \"main\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "1");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_self_loop_edge() {
        let text = format!(
            "{}\n[[edges]]\nfrom = \"publish\"\nto = \"publish\"\nkind = \"back\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "2");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_duplicate_from_to() {
        let text = format!(
            "{}\n[[edges]]\nfrom = \"draft\"\nto = \"review\"\nkind = \"branch\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "2");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_unreachable_node() {
        let text = format!(
            "{}\n[[nodes]]\nid = \"island\"\ntitle = \"I\"\nexecutor = \"agent\"\ninstruction = {{ text = \"i\" }}\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "3");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_graph_without_terminal_node() {
        let text = format!(
            "{}\n[[edges]]\nfrom = \"publish\"\nto = \"draft\"\nkind = \"back\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "4");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_input_from_unknown_output() {
        let text = base().replace("from = \"draft.article\"", "from = \"draft.nope\"");
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "5");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_input_from_node_that_cannot_reach_consumer() {
        // draft 引用 publish 的输出：publish 是终点，到不了 draft。
        let text = base().replace(
            "inputs  = [{ name = \"topic\", from = \"start.topic\" }]",
            "inputs  = [{ name = \"topic\", from = \"publish.final\" }]",
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "5");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_optional_input_on_start_or_resource_source() {
        let text = base().replace(
            "{ name = \"topic\", from = \"start.topic\" }",
            "{ name = \"topic\", from = \"start.topic\", required = false }",
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "5");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_node_id_engine() {
        let text = format!(
            "{}\n[[nodes]]\nid = \"engine\"\ntitle = \"E\"\nexecutor = \"agent\"\ninstruction = {{ text = \"e\" }}\n[[edges]]\nfrom = \"publish\"\nto = \"engine\"\nkind = \"main\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "1");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_optional_engine_stats_input() {
        let text = base().replace(
            "{ name = \"topic\", from = \"start.topic\" }",
            "{ name = \"topic\", from = \"start.topic\" }, { name = \"stats\", from = \"engine.stats\", required = false }",
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "5");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_gate_node_with_empty_text() {
        let text = base().replace(
            "instruction = { text = \"阅读审查通过的文章",
            "gate = true\ninstruction = { text = \"   ",
        );
        // 上面把 publish 的说明文本改成空白并加 gate；parse 已拒绝空白文本时也算规则 6 的前置。
        assert!(matches!(
            compile_text(&text),
            Err(Error::FlowInvalid { .. })
        ));
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_missing_instruction_file() {
        let text = base().replace("instructions/review.md", "instructions/missing.md");
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "7");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_non_utf8_instruction() {
        let def = parse_flow(base()).unwrap();
        let mut res = testkit::article_review_resources();
        if let Some(meta) = res
            .files
            .get_mut(&crate::path::RelPath::new("instructions/draft.md").unwrap())
        {
            meta.is_utf8 = false;
        }
        let err = compile(&def, &testkit::article_review_manifest(), &res).unwrap_err();
        assert_eq!(rule_of(err), "7");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_missing_resource_input_file() {
        let text = base().replace("resources/review-checklist.md", "resources/nope.md");
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "7");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_accepts_binary_resource_input() {
        let def = parse_flow(base()).unwrap();
        let mut res = testkit::article_review_resources();
        if let Some(meta) = res
            .files
            .get_mut(&crate::path::RelPath::new("resources/review-checklist.md").unwrap())
        {
            meta.is_utf8 = false;
        }
        assert!(compile(&def, &testkit::article_review_manifest(), &res).is_ok());
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_node_require_not_declared_in_manifest() {
        let text = base().replace(
            "max_visits = 3",
            "max_visits = 3\nrequires = [\"skill:ghost\"]",
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "8");
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_duplicate_node_require() {
        let def = parse_flow(&base().replace(
            "max_visits = 3",
            "max_visits = 3\nrequires = [\"skill:a\", \"skill:a\"]",
        ))
        .unwrap();
        let mut manifest = testkit::article_review_manifest();
        manifest.requires.push(crate::workbook::HostRequire {
            kind: crate::workbook::RequireKind::Skill,
            name: "a".into(),
            version: None,
            digest: None,
            source: None,
        });
        assert_eq!(
            rule_of(compile(&def, &manifest, &testkit::article_review_resources()).unwrap_err()),
            "8"
        );
    }

    #[test]
    #[ignore = "T05"]
    fn t05_rejects_tier_on_human_node() {
        let text = base().replace(
            "executor = \"human\"",
            "executor = \"human\"\ntier = \"strong\"",
        );
        // parse 可能先拒绝（tier 与 human 同时出现）；无论哪层，都必须是 FlowInvalid。
        assert!(matches!(
            compile_text(&text),
            Err(Error::FlowInvalid { .. })
        ));
    }

    proptest::proptest! {
        #[test]
        #[ignore = "T05"]
        fn t05_proptest_compile_never_panics(
            n in 2usize..=8,
            edges in proptest::collection::vec((0usize..8, 0usize..8), 0..16),
        ) {
            let def = testkit::random_flow(n, &edges);
            let manifest = testkit::article_review_manifest();
            let res = ResourceIndex::default();
            match compile(&def, &manifest, &res) {
                Ok(_) => {}
                Err(Error::FlowInvalid { .. }) => {}
                Err(other) => panic!("意外错误：{other:?}"),
            }
        }
    }
}
