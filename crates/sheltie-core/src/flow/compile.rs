//! Compile `FlowDef` into `Graph`; validation rules are in `specs/contracts/workbook.md` §4.
//!
//! Call one private function per rule in order; any failure rejects the whole Flow.
//! All errors use `Error::FlowInvalid { rule: "<number>", path, reason }`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::def::{
    EdgeDef, Executor, FlowDef, InputSource, Instruction, NodeDef, RESERVED_NODE_IDS,
};
use super::graph::{Graph, ResourceIndex};
use crate::error::{Error, Result};
use crate::ids::NodeId;
use crate::workbook::Manifest;

/// Files bound through `resource.<path>` have a 32 MiB limit and unrestricted encoding.
const RESOURCE_MAX_BYTES: u64 = 32 * 1024 * 1024;

fn invalid(rule: &'static str, path: impl Into<String>, reason: impl Into<String>) -> Error {
    Error::FlowInvalid {
        rule,
        path: path.into(),
        reason: reason.into(),
    }
}

/// Compilation entry point; return `Graph` only after every rule passes.
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
    check_rule_10(def, &out_edges)?;
    Ok(Graph::from_checked(
        def.entry.clone(),
        def.nodes.clone(),
        def.edges.clone(),
        manifest.requires.clone(),
    ))
}

/// Adjacency list: `from -> [to]`. Before rule 2, collect duplicate/unknown edges without validating them.
fn build_out_edges(edges: &[EdgeDef]) -> BTreeMap<NodeId, Vec<NodeId>> {
    let mut map: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
    for e in edges {
        map.entry(e.from.clone()).or_default().push(e.to.clone());
    }
    map
}

/// BFS over nodes reachable from `start` through outgoing edges, including itself.
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

/// Rule 1: valid unique IDs; no reserved node IDs (`def::RESERVED_NODE_IDS`: `start`, `resource`, `engine`); entry exists.
/// Require 1..=64 nodes and at most 256 edges.
fn check_rule_1(def: &FlowDef) -> Result<()> {
    if def.nodes.is_empty() || def.nodes.len() > FlowDef::MAX_NODES {
        return Err(invalid(
            "1",
            "nodes",
            format!("Node count must be in 1..={}", FlowDef::MAX_NODES),
        ));
    }
    if def.edges.len() > FlowDef::MAX_EDGES {
        return Err(invalid(
            "1",
            "edges",
            format!("Edge count must not exceed {}", FlowDef::MAX_EDGES),
        ));
    }
    let mut seen = BTreeSet::new();
    for (i, node) in def.nodes.iter().enumerate() {
        let path = format!("nodes[{i}].id");
        if RESERVED_NODE_IDS.contains(&node.id.as_str()) {
            return Err(invalid(
                "1",
                path,
                format!("Node ID must not be reserved: {}", node.id),
            ));
        }
        if !seen.insert(node.id.clone()) {
            return Err(invalid("1", path, "Duplicate node ID"));
        }
    }
    if def.node(&def.entry).is_none() {
        return Err(invalid(
            "1",
            "entry",
            format!("Entry {} is not a node ID", def.entry),
        ));
    }
    Ok(())
}

/// Rule 2: both edge endpoints exist; no self-loops or duplicate `(from, to)` pairs.
fn check_rule_2(def: &FlowDef) -> Result<()> {
    let mut seen = BTreeSet::new();
    for (i, e) in def.edges.iter().enumerate() {
        let path = format!("edges[{i}]");
        if e.from == e.to {
            return Err(invalid("2", path.clone(), "Self-loops are not allowed"));
        }
        if def.node(&e.from).is_none() {
            return Err(invalid(
                "2",
                path.clone(),
                format!("from {} is not a node ID", e.from),
            ));
        }
        if def.node(&e.to).is_none() {
            return Err(invalid(
                "2",
                path.clone(),
                format!("to {} is not a node ID", e.to),
            ));
        }
        if !seen.insert((e.from.clone(), e.to.clone())) {
            return Err(invalid("2", path, "Duplicate (from, to) edge"));
        }
    }
    Ok(())
}

/// Rule 3: every node is reachable from `entry`.
fn check_rule_3(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    let reachable = reachable_from(out_edges, &def.entry);
    for (i, node) in def.nodes.iter().enumerate() {
        if !reachable.contains(&node.id) {
            return Err(invalid(
                "3",
                format!("nodes[{i}].id"),
                format!("Node {1} is unreachable from entry {0}", def.entry, node.id),
            ));
        }
    }
    Ok(())
}

/// Rule 4: at least one node has no outgoing edges.
fn check_rule_4(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    let has_terminal = def
        .nodes
        .iter()
        .any(|n| out_edges.get(&n.id).is_none_or(Vec::is_empty));
    if !has_terminal {
        return Err(invalid(
            "4",
            "edges",
            "No terminal node (every node has outgoing edges)",
        ));
    }
    Ok(())
}

/// Rule 5: a `Node` input source exists, is not self, declares the output, and can reach this node;
/// if the source output is optional, the input must also be optional.
/// Reject `required = false` for `Start`, `Resource`, and `EngineStats` sources.
fn check_rule_5(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        for (j, input) in node.inputs.iter().enumerate() {
            let path = format!("nodes[{i}].inputs[{j}].from");
            match &input.from {
                InputSource::Start { .. }
                | InputSource::Resource { .. }
                | InputSource::EngineStats => {
                    // These three source kinds always exist; required = false has no meaning.
                    if !input.required {
                        return Err(invalid(
                            "5",
                            path,
                            "start, resource, and engine.stats sources must not declare required = false",
                        ));
                    }
                }
                InputSource::Node { node: src, output } => {
                    if src == &node.id {
                        return Err(invalid("5", path.clone(), "Source node must not be self"));
                    }
                    let src_def = def.node(src).ok_or_else(|| {
                        invalid(
                            "5",
                            path.clone(),
                            format!("Source node {src} does not exist"),
                        )
                    })?;
                    let src_out = src_def.output(output).ok_or_else(|| {
                        invalid(
                            "5",
                            path.clone(),
                            format!("Node {src} does not declare output {output}"),
                        )
                    })?;
                    // Downstream inputs must not require optional outputs (contract §3.2 and §4, rule 5).
                    if !src_out.required && input.required {
                        return Err(invalid(
                            "5",
                            path,
                            format!(
                                "Source output {src}.{output} is optional; this input must not set required = true"
                            ),
                        ));
                    }
                    if !reachable_from(out_edges, src).contains(&node.id) {
                        return Err(invalid(
                            "5",
                            path,
                            format!("Node {} is unreachable from source {src}", node.id),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Rule 6: `gate = true` nodes require nonblank instructions.
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
                    "Gate node instructions must not be blank",
                ));
            }
        }
    }
    Ok(())
}

/// Rule 7: instruction files exist, are UTF-8, and at most 64 KiB; resource inputs exist and are at most 32 MiB, with unrestricted encoding.
fn check_rule_7(def: &FlowDef, res: &ResourceIndex) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        if let Instruction::File(path) = &node.instruction {
            let at = format!("nodes[{i}].instruction.file");
            let meta = res
                .get(path)
                .ok_or_else(|| invalid("7", at.clone(), format!("File {path} does not exist")))?;
            if meta.bytes > NodeDef::FILE_MAX_BYTES {
                return Err(invalid(
                    "7",
                    at.clone(),
                    format!("File exceeds {} bytes", NodeDef::FILE_MAX_BYTES),
                ));
            }
            if !meta.is_utf8 {
                return Err(invalid("7", at, "Instruction file must be UTF-8"));
            }
        }
        for (j, input) in node.inputs.iter().enumerate() {
            if let InputSource::Resource { path } = &input.from {
                let at = format!("nodes[{i}].inputs[{j}].from");
                let meta = res.get(path).ok_or_else(|| {
                    invalid("7", at.clone(), format!("File {path} does not exist"))
                })?;
                if meta.bytes > RESOURCE_MAX_BYTES {
                    return Err(invalid(
                        "7",
                        at,
                        format!("File exceeds {RESOURCE_MAX_BYTES} bytes"),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Rule 8: each node `requires[]` item is declared in the manifest and unique within the node.
fn check_rule_8(def: &FlowDef, manifest: &Manifest) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        let mut seen = BTreeSet::new();
        for (j, (kind, name)) in node.requires.iter().enumerate() {
            let path = format!("nodes[{i}].requires[{j}]");
            if !seen.insert((*kind, name.clone())) {
                return Err(invalid(
                    "8",
                    path.clone(),
                    "Duplicate requires within a node",
                ));
            }
            if manifest.find_require(*kind, name).is_none() {
                return Err(invalid(
                    "8",
                    path,
                    format!("workbook.toml does not declare {}:{}", kind.as_str(), name),
                ));
            }
        }
    }
    Ok(())
}

/// Rule 9: `executor = human` nodes have no tier.
fn check_rule_9(def: &FlowDef) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        if node.executor == Executor::Human && node.tier.is_some() {
            return Err(invalid(
                "9",
                format!("nodes[{i}].tier"),
                "Human nodes must not declare tier",
            ));
        }
    }
    Ok(())
}

fn check_rule_10(def: &FlowDef, out_edges: &BTreeMap<NodeId, Vec<NodeId>>) -> Result<()> {
    for (i, node) in def.nodes.iter().enumerate() {
        let terminal = out_edges.get(&node.id).is_none_or(Vec::is_empty);
        let mut selected = BTreeSet::new();
        let slots = node
            .inputs
            .iter()
            .enumerate()
            .map(|(j, input)| {
                (
                    format!("nodes[{i}].inputs[{j}].result"),
                    input.name.as_str(),
                    input.required,
                    input.result,
                )
            })
            .chain(node.outputs.iter().enumerate().map(|(j, output)| {
                (
                    format!("nodes[{i}].outputs[{j}].result"),
                    output.name.as_str(),
                    output.required,
                    output.result,
                )
            }));
        for (path, name, required, result) in slots {
            if !result {
                continue;
            }
            if !terminal {
                return Err(invalid(
                    "10",
                    path,
                    "Only terminal nodes may declare final results",
                ));
            }
            if !required {
                return Err(invalid(
                    "10",
                    path,
                    "Final results must be required inputs or outputs",
                ));
            }
            if !selected.insert(name) {
                return Err(invalid(
                    "10",
                    path,
                    "Final result input and output logical names must not overlap",
                ));
            }
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
            other => panic!("Expected FlowInvalid, got {other:?}"),
        }
    }

    /// Use article-review as the valid baseline; change one condition for each rejected case.
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

    fn result_flow(first: &str, last: &str) -> String {
        format!(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"first\"\n[[nodes]]\nid = \"first\"\ntitle = \"First\"\nexecutor = \"agent\"\ninstruction = {{ text = \"x\" }}\n{first}\n[[nodes]]\nid = \"last\"\ntitle = \"Last\"\nexecutor = \"agent\"\ninstruction = {{ text = \"x\" }}\n{last}\n[[edges]]\nfrom = \"first\"\nto = \"last\"\nkind = \"main\"\n"
        )
    }

    // Task: C004-T01
    #[test]
    fn terminal_required_input_and_output_can_be_selected_results() {
        let graph = compile_text(&result_flow(
            "outputs = [{ name = \"out\", path = \"out.md\", result = false }]",
            "inputs = [{ name = \"upstream\", from = \"first.out\", result = true }]\noutputs = [{ name = \"delivery\", path = \"delivery.md\", result = true }]",
        ))
        .unwrap();
        let node = graph.node(&NodeId::new("last").unwrap()).unwrap();
        assert!(node.inputs()[0].result());
        assert!(node.output("delivery").unwrap().result());
    }

    // Task: C004-T01
    #[test]
    fn selected_results_on_nonterminal_input_or_output_are_rejected() {
        for first in [
            "inputs = [{ name = \"task\", from = \"start.task\", result = true }]",
            "outputs = [{ name = \"out\", path = \"out.md\", result = true }]",
        ] {
            let error = compile_text(&result_flow(first, "")).unwrap_err();
            assert!(
                matches!(error, Error::FlowInvalid { rule: "10", path, .. } if path.ends_with(".result"))
            );
        }
    }

    // Task: C004-T01
    #[test]
    fn selected_optional_input_or_output_is_rejected() {
        let first = "outputs = [{ name = \"out\", path = \"out.md\", required = false }]";
        for last in [
            "inputs = [{ name = \"upstream\", from = \"first.out\", required = false, result = true }]",
            "outputs = [{ name = \"delivery\", path = \"delivery.md\", required = false, result = true }]",
        ] {
            assert_eq!(
                rule_of(compile_text(&result_flow(first, last)).unwrap_err()),
                "10"
            );
        }
    }

    // Task: C004-T01
    #[test]
    fn selected_input_and_output_keys_must_be_unique() {
        let first = "outputs = [{ name = \"out\", path = \"out.md\" }]";
        let duplicate = "inputs = [{ name = \"same\", from = \"first.out\", result = true }]\noutputs = [{ name = \"same\", path = \"same.md\", result = true }]";
        assert!(
            matches!(compile_text(&result_flow(first, duplicate)), Err(Error::FlowInvalid { rule: "10", path, .. }) if path == "nodes[1].outputs[0].result")
        );
        let ordinary = duplicate.replace(
            "path = \"same.md\", result = true",
            "path = \"same.md\", result = false",
        );
        assert!(compile_text(&result_flow(first, &ordinary)).is_ok());
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
        // draft references publish output; publish is terminal and cannot reach draft.
        // Match fixture bytes to the example's multiline draft inputs; replace the topic input.
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
            "instruction = { text = \"Read the accepted article, confirm publication, and copy the final version to final.md.\" }",
            "gate = true\ninstruction = { text = \"   \" }",
        );
        // Blank publish instructions and add gate; parse's blank-text rejection is a prerequisite to rule 6.
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
        // The §3.2 field constraint rejects in parse with rule "parse", before rule 9.
        assert!(matches!(
            compile_text(&text),
            Err(Error::FlowInvalid { rule: "parse", .. })
        ));
        // Modify FlowDef directly to bypass parse and isolate compile rule 9.
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
                Err(other) => panic!("Unexpected error: {other:?}"),
            }
        }
    }

    // ── M1 additional rule 1/7 count/size limits and terminal-node rejection coverage ─────

    /// An `n`-node chain `n0 -> n1 -> ...`, adding forward (`i < j`) nonduplicate, noncyclic edges to reach `edges`.
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

    // ── M1 review repair: added workbook.md §4 rule 5 clause; see decisions.md M1 B1 ─────

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

    // ── M1 review O1: graph retains all Workbook requires in manifest declaration order ─────

    // Task: T05
    #[test]
    fn graph_carries_manifest_requires_in_declaration_order() {
        let def = parse_flow(base()).unwrap();
        let mut manifest = testkit::article_review_manifest();
        // beta precedes alpha; sorting by name or kind:name would reverse them.
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
        // base() nodes reference no requires, but the graph retains every declaration.
        let g = compile(&def, &manifest, &testkit::article_review_resources()).unwrap();
        let got: Vec<(&str, Option<&str>)> = g
            .requires()
            .iter()
            .map(|r| (r.name.as_str(), r.version.as_deref()))
            .collect();
        assert_eq!(got, vec![("beta", None), ("alpha", Some("^1"))]);
    }
}
