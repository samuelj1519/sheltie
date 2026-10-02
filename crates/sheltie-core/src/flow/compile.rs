//! 把 `FlowDef` 编译成 `Graph`。九条规则见 `specs/contracts/workbook.md` §4。
//!
//! 每条规则一个私有函数，按顺序调用，任一失败整体拒绝。
//! 错误一律 `Error::FlowInvalid { rule: "<编号>", path, reason }`。

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::def::{
    EdgeDef, Executor, FlowDef, InputSource, Instruction, NodeDef, RESERVED_NODE_IDS,
};
use super::graph::{Graph, ResourceIndex};
use crate::error::{Error, Result};
use crate::ids::NodeId;
use crate::workbook::Manifest;

/// `resource.<path>` 绑进来的文件上限 32 MiB，编码不限。
const RESOURCE_MAX_BYTES: u64 = 32 * 1024 * 1024;

fn invalid(rule: &'static str, path: impl Into<String>, reason: impl Into<String>) -> Error {
    Error::FlowInvalid {
        rule,
        path: path.into(),
        reason: reason.into(),
    }
}

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
        manifest.requires.clone(),
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
pub(crate) fn reachable_from(
    out_edges: &BTreeMap<NodeId, Vec<NodeId>>,
    start: &NodeId,
) -> BTreeSet<NodeId> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    seen.insert(start.clone());
    queue.push_back(start.clone());
    while let Some(n) = queue.pop_front() {
        for to in out_edges.get(&n).into_iter().flatten() {
            if seen.insert(to.clone()) {
                queue.push_back(to.clone());
            }
        }
    }
    seen
}

/// 规则 1：所有 ID 合规、唯一；节点 id 不是保留字（`def::RESERVED_NODE_IDS`：`start`、`resource`、`engine`）；`entry` 存在。
/// 节点数 1..=64，边数 ≤ 256。
fn check_rule_1(def: &FlowDef) -> Result<()> {
    if def.nodes.is_empty() || def.nodes.len() > FlowDef::MAX_NODES {
        return Err(invalid(
            "1",
            "nodes",
            format!("节点数必须在 1..={} 之间", FlowDef::MAX_NODES),
        ));
    }
    if def.edges.len() > FlowDef::MAX_EDGES {
        return Err(invalid(
            "1",
            "edges",
            format!("边数不得超过 {}", FlowDef::MAX_EDGES),
        ));
    }
    let mut seen = BTreeSet::new();
    for (i, node) in def.nodes.iter().enumerate() {
        let path = format!("nodes[{i}].id");
        if RESERVED_NODE_IDS.contains(&node.id.as_str()) {
            return Err(invalid(
                "1",
                path,
                format!("节点 id 不得是保留字 {}", node.id),
            ));
        }
        if !seen.insert(node.id.clone()) {
            return Err(invalid("1", path, "节点 id 重复"));
        }
    }
    if def.node(&def.entry).is_none() {
        return Err(invalid(
            "1",
            "entry",
            format!("入口 {} 不是节点 id", def.entry),
        ));
    }
    Ok(())
}

/// 规则 2：每条边两端存在、不自环、`(from, to)` 不重复。
fn check_rule_2(def: &FlowDef) -> Result<()> {
    let mut seen = BTreeSet::new();
    for (i, e) in def.edges.iter().enumerate() {
        let path = format!("edges[{i}]");
        if e.from == e.to {
            return Err(invalid("2", path.clone(), "不得自环"));
        }
        if def.node(&e.from).is_none() {
            return Err(invalid(
                "2",
                path.clone(),
                format!("from {} 不是节点 id", e.from),
            ));
        }
        if def.node(&e.to).is_none() {
            return Err(invalid(
                "2",
                path.clone(),
                format!("to {} 不是节点 id", e.to),
            ));
        }
        if !seen.insert((e.from.clone(), e.to.clone())) {
            return Err(invalid("2", path, "同一条 (from, to) 出现多次"));
        }
    }
    Ok(())
}

/// 规则 3：从 `entry` 出发每个节点可达。
fn check_rule_3(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    let reachable = reachable_from(out_edges, &def.entry);
    for (i, node) in def.nodes.iter().enumerate() {
        if !reachable.contains(&node.id) {
            return Err(invalid(
                "3",
                format!("nodes[{i}].id"),
                format!("从入口 {} 到不了 {}", def.entry, node.id),
            ));
        }
    }
    Ok(())
}

/// 规则 4：至少一个没有出边的节点。
fn check_rule_4(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    let has_terminal = def
        .nodes
        .iter()
        .any(|n| out_edges.get(&n.id).is_none_or(Vec::is_empty));
    if !has_terminal {
        return Err(invalid(
            "4",
            "edges",
            "没有任何终点节点（每个节点都有出边）",
        ));
    }
    Ok(())
}

/// 规则 5：`inputs[].from` 的 `Node` 来源存在、不是自己、输出名存在，且从被引用节点能到达本节点；
/// 被引用的输出 `required = false` 时，本输入也必须 `required = false`（下游不得把可选输出当必需输入）。
/// 另：`Start`、`Resource`、`EngineStats` 来源上 `required = false` 拒绝。
fn check_rule_5(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        for (j, input) in node.inputs.iter().enumerate() {
            let path = format!("nodes[{i}].inputs[{j}].from");
            match &input.from {
                InputSource::Start { .. }
                | InputSource::Resource { .. }
                | InputSource::EngineStats => {
                    // 这三种来源永远存在，写 required = false 没有意义。
                    if !input.required {
                        return Err(invalid(
                            "5",
                            path,
                            "start、resource 与 engine.stats 来源不得声明 required = false",
                        ));
                    }
                }
                InputSource::Node { node: src, output } => {
                    if src == &node.id {
                        return Err(invalid("5", path.clone(), "来源节点不得是自己"));
                    }
                    let src_def = def.node(src).ok_or_else(|| {
                        invalid("5", path.clone(), format!("来源节点 {src} 不存在"))
                    })?;
                    let src_out = src_def.output(output).ok_or_else(|| {
                        invalid(
                            "5",
                            path.clone(),
                            format!("节点 {src} 没有声明输出 {output}"),
                        )
                    })?;
                    // 可选输出不得被下游当必需输入（合同 §3.2、§4 规则 5）。
                    if !src_out.required && input.required {
                        return Err(invalid(
                            "5",
                            path,
                            format!("来源输出 {src}.{output} 是可选的，本输入不得 required = true"),
                        ));
                    }
                    if !reachable_from(out_edges, src).contains(&node.id) {
                        return Err(invalid(
                            "5",
                            path,
                            format!("从 {src} 沿边走不到 {}", node.id),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// 规则 6：`gate = true` 的节点 `instruction` 不得是空白文本。
fn check_rule_6(def: &FlowDef) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        if !node.gate {
            continue;
        }
        if let Instruction::Text(text) = &node.instruction {
            if text.trim().is_empty() {
                return Err(invalid(
                    "6",
                    format!("nodes[{i}].instruction"),
                    "gate 节点的说明文本不得是空白",
                ));
            }
        }
    }
    Ok(())
}

/// 规则 7：`instruction.file` 存在、≤ 64 KiB、UTF-8；`resource.<path>` 输入存在、≤ 32 MiB（编码不限）。
fn check_rule_7(def: &FlowDef, res: &ResourceIndex) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        if let Instruction::File(path) = &node.instruction {
            let at = format!("nodes[{i}].instruction.file");
            let meta = res
                .get(path)
                .ok_or_else(|| invalid("7", at.clone(), format!("文件 {path} 不存在")))?;
            if meta.bytes > NodeDef::FILE_MAX_BYTES {
                return Err(invalid(
                    "7",
                    at.clone(),
                    format!("文件超过 {} 字节", NodeDef::FILE_MAX_BYTES),
                ));
            }
            if !meta.is_utf8 {
                return Err(invalid("7", at, "说明文件必须是 UTF-8"));
            }
        }
        for (j, input) in node.inputs.iter().enumerate() {
            if let InputSource::Resource { path } = &input.from {
                let at = format!("nodes[{i}].inputs[{j}].from");
                let meta = res
                    .get(path)
                    .ok_or_else(|| invalid("7", at.clone(), format!("文件 {path} 不存在")))?;
                if meta.bytes > RESOURCE_MAX_BYTES {
                    return Err(invalid(
                        "7",
                        at,
                        format!("文件超过 {RESOURCE_MAX_BYTES} 字节"),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// 规则 8：节点 `requires[]` 每项在 manifest 里有声明；同一节点内不重复。
fn check_rule_8(def: &FlowDef, manifest: &Manifest) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        let mut seen = BTreeSet::new();
        for (j, (kind, name)) in node.requires.iter().enumerate() {
            let path = format!("nodes[{i}].requires[{j}]");
            if !seen.insert((*kind, name.clone())) {
                return Err(invalid("8", path.clone(), "同一节点内 requires 重复"));
            }
            if manifest.find_require(*kind, name).is_none() {
                return Err(invalid(
                    "8",
                    path,
                    format!("workbook.toml 里没有声明 {}:{}", kind.as_str(), name),
                ));
            }
        }
    }
    Ok(())
}

/// 规则 9：`executor = human` 的节点 `tier` 为 `None`。
fn check_rule_9(def: &FlowDef) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        if node.executor == Executor::Human && node.tier.is_some() {
            return Err(invalid(
                "9",
                format!("nodes[{i}].tier"),
                "human 节点不得声明 tier",
            ));
        }
    }
    Ok(())
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

    /// 把 article-review 样例当合法基线，改一处得到反例。
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

    // Task: T05
    #[test]
    fn compiles_article_review_example() {
        let g = compile_text(base()).unwrap();
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 3);
        assert!(g.is_terminal(&NodeId::new("publish").unwrap()));
        assert!(!g.is_terminal(&NodeId::new("draft").unwrap()));
        assert!(!g.is_terminal(&NodeId::new("review").unwrap()));
    }

    // Task: T05
    #[test]
    fn rejects_entry_not_a_node() {
        assert_eq!(
            rule_of(
                compile_text(&base().replace("entry = \"draft\"", "entry = \"nope\"")).unwrap_err()
            ),
            "1"
        );
    }

    // Task: T05
    #[test]
    fn rejects_node_id_start_or_resource() {
        let text = format!(
            "{}\n[[nodes]]\nid = \"start\"\ntitle = \"S\"\nexecutor = \"agent\"\ninstruction = {{ text = \"s\" }}\n[[edges]]\nfrom = \"publish\"\nto = \"start\"\nkind = \"main\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "1");
    }

    // Task: T05
    #[test]
    fn rejects_self_loop_edge() {
        let text = format!(
            "{}\n[[edges]]\nfrom = \"publish\"\nto = \"publish\"\nkind = \"back\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "2");
    }

    // Task: T05
    #[test]
    fn rejects_duplicate_from_to() {
        let text = format!(
            "{}\n[[edges]]\nfrom = \"draft\"\nto = \"review\"\nkind = \"branch\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "2");
    }

    // Task: T05
    #[test]
    fn rejects_unreachable_node() {
        let text = format!(
            "{}\n[[nodes]]\nid = \"island\"\ntitle = \"I\"\nexecutor = \"agent\"\ninstruction = {{ text = \"i\" }}\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "3");
    }

    // Task: T05
    #[test]
    fn rejects_graph_without_terminal_node() {
        let text = format!(
            "{}\n[[edges]]\nfrom = \"publish\"\nto = \"draft\"\nkind = \"back\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "4");
    }

    // Task: T05
    #[test]
    fn rejects_input_from_unknown_output() {
        let text = base().replace("from = \"draft.article\"", "from = \"draft.nope\"");
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "5");
    }

    // Task: T05
    #[test]
    fn rejects_input_from_node_that_cannot_reach_consumer() {
        // draft 引用 publish 的输出：publish 是终点，到不了 draft。
        // 夹具按样例字节同步：draft 的输入是多行数组，替换针对 topic 那条输入。
        let text = base().replace(
            "{ name = \"topic\", from = \"start.topic\" }",
            "{ name = \"topic\", from = \"publish.final\" }",
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "5");
    }

    // Task: C002-T40
    #[test]
    fn rejects_optional_input_on_start_or_resource_source() {
        for declaration in [
            "{ name = \"topic\", from = \"start.topic\" }",
            "{ name = \"checklist\", from = \"resource.resources/review-checklist.md\" }",
        ] {
            let optional = declaration.replace(" }", ", required = false }");
            let text = base().replace(declaration, &optional);
            assert_eq!(
                rule_of(compile_text(&text).unwrap_err()),
                "5",
                "{declaration}"
            );
        }
    }

    // Task: T05
    #[test]
    fn rejects_node_id_engine() {
        let text = format!(
            "{}\n[[nodes]]\nid = \"engine\"\ntitle = \"E\"\nexecutor = \"agent\"\ninstruction = {{ text = \"e\" }}\n[[edges]]\nfrom = \"publish\"\nto = \"engine\"\nkind = \"main\"\n",
            base()
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "1");
    }

    // Task: T05
    #[test]
    fn rejects_optional_engine_stats_input() {
        let text = base().replace(
            "{ name = \"topic\", from = \"start.topic\" }",
            "{ name = \"topic\", from = \"start.topic\" }, { name = \"stats\", from = \"engine.stats\", required = false }",
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "5");
    }

    // Task: T05
    #[test]
    fn rejects_gate_node_with_empty_text() {
        let text = base().replace(
            "instruction = { text = \"阅读审查通过的文章，确认可以发布。把最终版复制到 final.md。\" }",
            "gate = true\ninstruction = { text = \"   \" }",
        );
        // 上面把 publish 的说明文本改成空白并加 gate；parse 已拒绝空白文本时也算规则 6 的前置。
        assert!(matches!(
            compile_text(&text),
            Err(Error::FlowInvalid { .. })
        ));
    }

    // Task: T05
    #[test]
    fn rejects_missing_instruction_file() {
        let text = base().replace("instructions/review.md", "instructions/missing.md");
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "7");
    }

    // Task: T05
    #[test]
    fn rejects_non_utf8_instruction() {
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

    // Task: T05
    #[test]
    fn rejects_missing_resource_input_file() {
        let text = base().replace("resources/review-checklist.md", "resources/nope.md");
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "7");
    }

    // Task: T05
    #[test]
    fn accepts_binary_resource_input() {
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

    // Task: T05
    #[test]
    fn rejects_node_require_not_declared_in_manifest() {
        let text = base().replace(
            "max_visits = 3",
            "max_visits = 3\nrequires = [\"skill:ghost\"]",
        );
        assert_eq!(rule_of(compile_text(&text).unwrap_err()), "8");
    }

    // Task: T05
    #[test]
    fn rejects_duplicate_node_require() {
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

    // Task: T05
    #[test]
    fn rejects_tier_on_human_node() {
        let text = base().replace(
            "executor = \"human\"",
            "executor = \"human\"\ntier = \"strong\"",
        );
        // §3.2 字段规则在 parse 层先拒，rule 是 "parse"，走不到规则 9。
        assert!(matches!(
            compile_text(&text),
            Err(Error::FlowInvalid { rule: "parse", .. })
        ));
        // 直接改 FlowDef 绕过 parse，钉住编译规则 9 本身。
        let mut def = parse_flow(base()).unwrap();
        let human = def
            .nodes
            .iter_mut()
            .find(|n| n.id.as_str() == "publish")
            .unwrap();
        human.tier = Some(crate::flow::def::Tier::Strong);
        let err = compile(
            &def,
            &testkit::article_review_manifest(),
            &testkit::article_review_resources(),
        )
        .unwrap_err();
        assert_eq!(rule_of(err), "9");
    }

    proptest::proptest! {
        // Task: T05
        #[test]
        fn proptest_compile_never_panics(
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

    // ── M1 补测（规则 1 与规则 7 的数量与大小上限；终点判定的反例） ─────

    /// `n` 个节点的链 `n0 -> n1 -> …`，再补前向边（`i < j`，不成环、不重复）凑够 `edges` 条。
    fn chain_with_edges(n: usize, edges: usize) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = (1..n).map(|i| (i - 1, i)).collect();
        'fill: for i in 0..n {
            for j in (i + 2)..n {
                if out.len() >= edges {
                    break 'fill;
                }
                out.push((i, j));
            }
        }
        out
    }

    fn compile_def(def: &crate::flow::FlowDef) -> Result<Graph> {
        compile(
            def,
            &testkit::article_review_manifest(),
            &ResourceIndex::default(),
        )
    }

    // Task: T05
    #[test]
    fn node_count_limit_is_64() {
        assert_eq!(
            compile_def(&testkit::random_flow(64, &chain_with_edges(64, 63)))
                .unwrap()
                .node_count(),
            64
        );
        assert_eq!(
            rule_of(compile_def(&testkit::random_flow(65, &chain_with_edges(65, 64))).unwrap_err()),
            "1"
        );
    }

    // Task: T05
    #[test]
    fn edge_count_limit_is_256() {
        let at = chain_with_edges(64, 256);
        assert_eq!(at.len(), 256);
        assert_eq!(
            compile_def(&testkit::random_flow(64, &at))
                .unwrap()
                .edge_count(),
            256
        );
        let over = chain_with_edges(64, 257);
        assert_eq!(
            rule_of(compile_def(&testkit::random_flow(64, &over)).unwrap_err()),
            "1"
        );
    }

    fn resources_with(path: &str, bytes: u64) -> ResourceIndex {
        use crate::flow::graph::ResourceMeta;
        use crate::path::RelPath;

        let mut res = testkit::article_review_resources();
        res.files.insert(
            RelPath::new(path).unwrap(),
            ResourceMeta {
                bytes,
                is_utf8: true,
            },
        );
        res
    }

    fn compile_with(res: &ResourceIndex) -> Result<Graph> {
        compile(
            &parse_flow(base()).unwrap(),
            &testkit::article_review_manifest(),
            res,
        )
    }

    // Task: T05
    #[test]
    fn instruction_file_limit_is_64_kib() {
        assert!(compile_with(&resources_with("instructions/draft.md", 65_536)).is_ok());
        assert_eq!(
            rule_of(compile_with(&resources_with("instructions/draft.md", 65_537)).unwrap_err()),
            "7"
        );
    }

    // Task: T05
    #[test]
    fn resource_input_limit_is_32_mib() {
        let path = "resources/review-checklist.md";
        assert!(compile_with(&resources_with(path, 33_554_432)).is_ok());
        assert_eq!(
            rule_of(compile_with(&resources_with(path, 33_554_433)).unwrap_err()),
            "7"
        );
    }

    // ── M1 复核待修（合同 workbook.md §4 规则 5 新增一句，见 decisions.md M1 记录 B1） ─────

    fn optional_article() -> String {
        base().replace(
            "{ name = \"article\", path = \"article.md\", max_bytes = 262144 }",
            "{ name = \"article\", path = \"article.md\", max_bytes = 262144, required = false }",
        )
    }

    // Task: T05
    #[test]
    fn rejects_required_input_on_optional_output() {
        assert_eq!(rule_of(compile_text(&optional_article()).unwrap_err()), "5");
    }

    // Task: T05
    #[test]
    fn accepts_optional_input_on_optional_output() {
        let text = optional_article()
            .replace(
                "{ name = \"article\",   from = \"draft.article\" }",
                "{ name = \"article\",   from = \"draft.article\", required = false }",
            )
            .replace(
                "{ name = \"article\", from = \"draft.article\" }",
                "{ name = \"article\", from = \"draft.article\", required = false }",
            );
        assert!(compile_text(&text).is_ok());
    }

    // ── M1 复核 O1：图带 Workbook 全量 requires，按 manifest 声明顺序 ─────

    // Task: T05
    #[test]
    fn graph_carries_manifest_requires_in_declaration_order() {
        let def = parse_flow(base()).unwrap();
        let mut manifest = testkit::article_review_manifest();
        // 先 beta 后 alpha：若按名字或 kind:name 排序，顺序会反过来。
        manifest.requires.push(crate::workbook::HostRequire {
            kind: crate::workbook::RequireKind::Skill,
            name: "beta".into(),
            version: None,
            digest: None,
            source: None,
        });
        manifest.requires.push(crate::workbook::HostRequire {
            kind: crate::workbook::RequireKind::Mcp,
            name: "alpha".into(),
            version: Some("^1".into()),
            digest: None,
            source: None,
        });
        // base() 的节点一条 requires 都没引用，图里照样有全量声明。
        let g = compile(&def, &manifest, &testkit::article_review_resources()).unwrap();
        let got: Vec<(&str, Option<&str>)> = g
            .requires()
            .iter()
            .map(|r| (r.name.as_str(), r.version.as_deref()))
            .collect();
        assert_eq!(got, vec![("beta", None), ("alpha", Some("^1"))]);
    }
}
