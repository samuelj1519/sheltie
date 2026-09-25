//! T11：三份样例与 spec-dev 都能按合同编译。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use sheltie_core::flow::{Executor, Tier, compile, parse_flow};
use sheltie_core::ids::NodeId;
use sheltie_core::testkit::{example_files, resources_of};
use sheltie_core::workbook::parse_manifest;

fn compile_example(name: &str) -> sheltie_core::flow::Graph {
    let files = example_files(name);
    let manifest = files
        .iter()
        .find(|(p, _)| *p == "workbook.toml")
        .map(|(_, c)| *c)
        .unwrap();
    let flow = files
        .iter()
        .find(|(p, _)| *p == "flows/default.toml")
        .map(|(_, c)| *c)
        .unwrap();
    let m = parse_manifest(manifest).unwrap();
    let f = parse_flow(flow).unwrap();
    compile(&f, &m, &resources_of(name)).unwrap()
}

fn id(s: &str) -> NodeId {
    NodeId::new(s).unwrap()
}

// Task: T11
#[test]
fn all_examples_compile() {
    for name in ["two-step", "article-review", "gated-release", "spec-dev"] {
        compile_example(name);
    }
}

// Task: T11
#[test]
fn two_step_has_one_main_edge_and_no_gate() {
    let g = compile_example("two-step");
    assert_eq!(g.node_count(), 2);
    assert_eq!(g.edge_count(), 1);
    assert!(g.nodes().all(|n| !n.gate));
}

// Task: T11
#[test]
fn article_review_has_back_edge_human_publish_and_resource_input() {
    let g = compile_example("article-review");
    assert!(
        g.out_edges(&id("review"))
            .iter()
            .any(|e| e.kind == sheltie_core::flow::EdgeKind::Back)
    );
    assert_eq!(g.node(&id("publish")).unwrap().executor, Executor::Human);
    assert!(
        g.node(&id("review"))
            .unwrap()
            .inputs
            .iter()
            .any(|i| matches!(i.from, sheltie_core::flow::InputSource::Resource { .. }))
    );
}

// Task: T11
#[test]
fn gated_release_first_node_is_gate() {
    let g = compile_example("gated-release");
    assert!(g.node(g.entry()).unwrap().gate);
}

// Task: T11
#[test]
fn no_example_declares_requires() {
    for name in ["two-step", "article-review", "gated-release", "spec-dev"] {
        let g = compile_example(name);
        assert!(g.nodes().all(|n| n.requires.is_empty()), "{name}");
    }
}

// Task: T11
#[test]
fn spec_dev_compiles_with_eleven_nodes_twenty_four_edges() {
    let g = compile_example("spec-dev");
    assert_eq!(g.node_count(), 11);
    assert_eq!(g.edge_count(), 24);
}

// Task: T11
#[test]
fn spec_dev_retro_reads_engine_stats() {
    let g = compile_example("spec-dev");
    let retro = g.node(&id("retro")).unwrap();
    assert!(
        retro
            .inputs
            .iter()
            .any(|i| i.from == sheltie_core::flow::InputSource::EngineStats)
    );
    assert!(g.is_terminal(&id("retro")));
}

// Task: T11
#[test]
fn spec_dev_optional_inputs_all_point_to_reachable_upstream() {
    // 编译规则 5 已经保证；这里再确认每个可选输入都是 Node 来源。
    let g = compile_example("spec-dev");
    for n in g.nodes() {
        for i in &n.inputs {
            if !i.required {
                assert!(
                    matches!(i.from, sheltie_core::flow::InputSource::Node { .. }),
                    "{}.{}",
                    n.id,
                    i.name
                );
            }
        }
    }
}

// Task: T11
#[test]
fn spec_dev_only_retro_is_gated_and_human_nodes_are_plan_review_and_escalate() {
    let g = compile_example("spec-dev");
    let gated: Vec<_> = g
        .nodes()
        .filter(|n| n.gate)
        .map(|n| n.id.as_str().to_string())
        .collect();
    assert_eq!(gated, vec!["retro"]);
    let humans: Vec<_> = g
        .nodes()
        .filter(|n| n.executor == Executor::Human)
        .map(|n| n.id.as_str().to_string())
        .collect();
    assert_eq!(humans, vec!["plan-review", "escalate"]);
}

// Task: T11
#[test]
fn spec_dev_strong_tier_nodes_are_spec_plan_scaffold_review() {
    let g = compile_example("spec-dev");
    let strong: Vec<_> = g
        .nodes()
        .filter(|n| n.tier == Some(Tier::Strong))
        .map(|n| n.id.as_str().to_string())
        .collect();
    assert_eq!(strong, vec!["spec", "plan", "scaffold", "review"]);
}

// Task: T11
#[test]
fn mutated_two_step_manifest_with_extra_field_is_rejected() {
    let text = format!(
        "{}\nauthor = \"x\"\n",
        sheltie_core::testkit::TWO_STEP_MANIFEST
    );
    assert!(parse_manifest(&text).is_err());
}
