//! 从具体终点 Attempt 的冻结槽选择最终成果，不读取产物内容。

use serde::Serialize;

use crate::digest::Sha256Hex;
use crate::flow::{Graph, InputDecl, InputSource, OutputDecl};
use crate::ids::{FlowId, WorkId};
use crate::path::AbsPath;

use super::{ArtifactRef, Attempt, AttemptStatus, WorkState, WorkStatus, WorkbookRef, layout};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultView {
    pub format: &'static str,
    pub work_id: WorkId,
    pub revision: u64,
    pub workbook: WorkbookRef,
    pub flow: FlowId,
    pub status: WorkStatus,
    pub effects_pending: bool,
    pub r#final: bool,
    pub artifacts: Vec<ResultArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultArtifact {
    pub key: String,
    pub path: AbsPath,
    pub sha256: Sha256Hex,
    pub bytes: u64,
    pub source: ResultSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultSource {
    pub attempt: String,
    pub kind: ResultSlotKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultSlotKind {
    Input,
    Output,
}

pub fn result_view(
    state: &WorkState,
    graph: &Graph,
    revision: u64,
    effects_pending: bool,
) -> std::result::Result<ResultView, String> {
    state.validate_persisted()?;
    state.validate_gate_facts(graph)?;
    let mut view = ResultView {
        format: "work-result/v1",
        work_id: state.work_id.clone(),
        revision,
        workbook: state.workbook.clone(),
        flow: state.flow.clone(),
        status: state.status,
        effects_pending,
        r#final: false,
        artifacts: Vec::new(),
    };
    if state.status != WorkStatus::Succeeded {
        return Ok(view);
    }
    let node = graph
        .node(&state.current.node)
        .ok_or_else(|| "成功 Work 的当前节点不在冻结图中".to_string())?;
    if !graph.is_terminal(&state.current.node) {
        return Err("成功 Work 的当前节点不是终点".to_string());
    }
    let (terminal_index, terminal) = state
        .attempts
        .iter()
        .enumerate()
        .rev()
        .find(|(_, attempt)| attempt.occurrence() == state.current)
        .ok_or_else(|| "成功 Work 缺少对应终点 Attempt".to_string())?;
    if terminal.status != AttemptStatus::Succeeded {
        return Err("成功 Work 的终点 Attempt 未成功".to_string());
    }

    let mut artifacts = Vec::new();
    for input in node.inputs().iter().filter(|input| input.result()) {
        let reference = terminal
            .inputs
            .get(input.name())
            .and_then(Option::as_ref)
            .ok_or_else(|| format!("最终输入 {} 缺少冻结引用", input.name()))?;
        validate_input(state, graph, terminal, terminal_index, input, reference)?;
        artifacts.push(selected(
            terminal,
            input.name(),
            ResultSlotKind::Input,
            reference,
        ));
    }
    for output in node.outputs.iter().filter(|output| output.result()) {
        let reference = terminal
            .outputs
            .get(&output.name)
            .ok_or_else(|| format!("最终输出 {} 缺少封存引用", output.name))?;
        validate_output(state, terminal, output, reference)?;
        artifacts.push(selected(
            terminal,
            &output.name,
            ResultSlotKind::Output,
            reference,
        ));
    }
    artifacts.sort_by(|left, right| left.key.cmp(&right.key));
    if !effects_pending {
        view.r#final = true;
        view.artifacts = artifacts;
    }
    Ok(view)
}

fn selected(
    attempt: &Attempt,
    name: &str,
    kind: ResultSlotKind,
    reference: &ArtifactRef,
) -> ResultArtifact {
    ResultArtifact {
        key: name.to_string(),
        path: reference.path.clone(),
        sha256: reference.sha256.clone(),
        bytes: reference.bytes,
        source: ResultSource {
            attempt: attempt.id.to_string(),
            kind,
            name: name.to_string(),
        },
    }
}

fn validate_output(
    state: &WorkState,
    attempt: &Attempt,
    output: &OutputDecl,
    reference: &ArtifactRef,
) -> std::result::Result<(), String> {
    if reference.path != layout::output_path(&state.attempt_dir(&attempt.id), &output.path)
        || reference.bytes > output.max_bytes
    {
        return Err(format!(
            "最终成果引用 {} 与输出归属或大小合同不一致",
            output.name
        ));
    }
    Ok(())
}

fn validate_input(
    state: &WorkState,
    graph: &Graph,
    terminal: &Attempt,
    terminal_index: usize,
    input: &InputDecl,
    reference: &ArtifactRef,
) -> std::result::Result<(), String> {
    let invalid = || format!("最终输入 {} 与冻结来源不一致", input.name());
    match input.source() {
        InputSource::Start { key } => {
            if state.inputs.get(key) != Some(reference)
                || reference.path != layout::start_input_path(&state.work_dir, key)
            {
                return Err(invalid());
            }
        }
        InputSource::Resource { path } => {
            if reference.path != state.workbook_dir().join(path) {
                return Err(invalid());
            }
        }
        InputSource::EngineStats => {
            if reference.path != layout::engine_stats_path(&state.attempt_dir(&terminal.id)) {
                return Err(invalid());
            }
        }
        InputSource::Node { node, output } => {
            let producer = state.attempts[..terminal_index]
                .iter()
                .rev()
                .find(|attempt| {
                    &attempt.id.node == node && attempt.status == AttemptStatus::Succeeded
                })
                .ok_or_else(invalid)?;
            if producer.outputs.get(output) != Some(reference) {
                return Err(invalid());
            }
            let declaration = graph
                .node(node)
                .and_then(|node| node.output(output))
                .ok_or_else(invalid)?;
            validate_output(state, producer, declaration, reference)?;
        }
    }
    Ok(())
}

pub fn render_result(view: &ResultView) -> String {
    let mut text = format!(
        "# Work {} 的成果\n\nworkbook: {}@{}   flow: {}   status: {}\nrevision: {}   final: {}   effects_pending: {}\n",
        view.work_id,
        view.workbook.id,
        view.workbook.version,
        view.flow,
        view.status,
        view.revision,
        view.r#final,
        view.effects_pending,
    );
    if !view.r#final {
        text.push_str("\n最终成果尚未就绪。\n");
    } else if view.artifacts.is_empty() {
        text.push_str("\n未声明最终成果。\n");
    } else {
        text.push_str("\n## 最终成果\n\n");
        for artifact in &view.artifacts {
            let kind = match artifact.source.kind {
                ResultSlotKind::Input => "input",
                ResultSlotKind::Output => "output",
            };
            text.push_str(&format!(
                "{} → {} (sha256 {}, {} B)\n  source: {} {} {}\n",
                artifact.key,
                artifact.path,
                artifact.sha256,
                artifact.bytes,
                artifact.source.attempt,
                kind,
                artifact.source.name,
            ));
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::AttemptId;
    use crate::testkit::Fixture;

    const MANIFEST: &str = "schema = \"workbook/v1\"\nid = \"results\"\nversion = \"1.0.0\"\nname = \"Results\"\nflows = [\"flows/default.toml\"]\n";
    const FLOW: &str = r#"
schema = "flow/v1"
id = "default"
entry = "produce"
[[nodes]]
id = "produce"
title = "Produce"
executor = "agent"
instruction = { text = "Produce" }
outputs = [{ name = "out", path = "old.md", max_bytes = 32 }]
max_visits = 2
[[nodes]]
id = "finish"
title = "Finish"
executor = "agent"
instruction = { text = "Finish" }
inputs = [{ name = "z-input", from = "produce.out", result = true }]
outputs = [{ name = "a-output", path = "final.md", max_bytes = 32, result = true }, { name = "ordinary", path = "ordinary.md" }]
[[edges]]
from = "produce"
to = "finish"
kind = "main"
"#;

    fn completed() -> Fixture {
        let mut fixture = Fixture::from_texts(MANIFEST, FLOW, &[]).started_with(&[]);
        fixture.begin("produce").unwrap();
        fixture.submit_ok("produce#1.0", "Produced").unwrap();
        fixture.begin("finish").unwrap();
        fixture.submit_ok("finish#1.0", "Finished").unwrap();
        fixture
    }

    // Task: C004-T01
    #[test]
    fn result_uses_sorted_terminal_slots_and_preserves_complete_references() {
        let fixture = completed();
        let view = result_view(fixture.state(), &fixture.graph, 7, false).unwrap();
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["format"], "work-result/v1");
        assert_eq!(json["work_id"], "2026-09-24-001-t");
        assert_eq!(json["revision"], 7);
        assert_eq!(json["workbook"]["id"], "results");
        assert_eq!(json["flow"], "default");
        assert_eq!(json["status"], serde_json::json!({"kind":"succeeded"}));
        assert_eq!(json["final"], true);
        assert_eq!(json["effects_pending"], false);
        assert_eq!(view.artifacts.len(), 2);
        assert_eq!(
            json["artifacts"][0],
            serde_json::json!({
                "key":"a-output",
                "path":"/tmp/sheltie-test/works/2026-09-24-001-t/attempts/finish/occurrence-001/attempt-000/outputs/final.md",
                "sha256":"fc11d6f28e59d3cc33c0b14ceb644bf0902ebd63d61218dffe9e7dac7c254542",
                "bytes":10,
                "source":{"attempt":"finish#1.0","kind":"output","name":"a-output"}
            })
        );
        assert_eq!(
            json["artifacts"][1],
            serde_json::json!({
                "key":"z-input",
                "path":"/tmp/sheltie-test/works/2026-09-24-001-t/attempts/produce/occurrence-001/attempt-000/outputs/old.md",
                "sha256":"fc11d6f28e59d3cc33c0b14ceb644bf0902ebd63d61218dffe9e7dac7c254542",
                "bytes":10,
                "source":{"attempt":"finish#1.0","kind":"input","name":"z-input"}
            })
        );
        let text = render_result(&view);
        assert!(text.contains("revision: 7   final: true   effects_pending: false"));
        for item in &view.artifacts {
            assert!(text.contains(item.path.as_str()));
            assert!(text.contains(item.sha256.as_str()));
            assert!(text.contains("10 B"));
        }
        assert!(text.contains("source: finish#1.0 input z-input"));
    }

    // Task: C004-T01
    #[test]
    fn pending_effects_withhold_final_artifacts_without_changing_flow_status() {
        let fixture = completed();
        let view = result_view(fixture.state(), &fixture.graph, 8, true).unwrap();
        assert_eq!(view.status, WorkStatus::Succeeded);
        assert!(!view.r#final);
        assert!(view.effects_pending);
        assert!(view.artifacts.is_empty());
        assert!(render_result(&view).contains("最终成果尚未就绪"));
    }

    // Task: C004-T01
    #[test]
    fn uncompleted_and_cancelled_work_have_no_final_artifacts() {
        let mut fixture = Fixture::from_texts(MANIFEST, FLOW, &[]).started_with(&[]);
        for state in [
            fixture.state().clone(),
            {
                fixture.begin("produce").unwrap();
                fixture.state().clone()
            },
            {
                fixture.fail("produce#1.0", "Interrupted").unwrap();
                fixture.state().clone()
            },
            {
                fixture.cancel().unwrap();
                fixture.state().clone()
            },
        ] {
            let view = result_view(&state, &fixture.graph, 4, false).unwrap();
            assert!(!view.r#final);
            assert!(view.artifacts.is_empty());
            assert_eq!(view.status, state.status);
        }
    }

    // Task: C004-T01
    #[test]
    fn final_work_without_selections_reports_empty_explicit_result() {
        let mut fixture = Fixture::with_optional_output();
        fixture.begin("only").unwrap();
        fixture.submit_ok("only#1.0", "Finished").unwrap();
        let view = result_view(fixture.state(), &fixture.graph, 3, false).unwrap();
        assert!(view.r#final);
        assert!(view.artifacts.is_empty());
        assert!(render_result(&view).contains("未声明最终成果"));
    }

    // Task: C004-T01
    #[test]
    fn gate_blocks_final_result_and_missing_approval_is_corrupt() {
        let mut fixture = Fixture::single_gated_terminal();
        fixture.begin("only").unwrap();
        fixture.submit_ok("only#1.0", "Finished").unwrap();
        assert!(
            !result_view(fixture.state(), &fixture.graph, 3, false)
                .unwrap()
                .r#final
        );
        let mut forged = fixture.state().clone();
        forged.status = WorkStatus::Succeeded;
        assert!(result_view(&forged, &fixture.graph, 3, false).is_err());
        fixture.approve("only").unwrap();
        assert!(
            result_view(fixture.state(), &fixture.graph, 4, false)
                .unwrap()
                .r#final
        );
    }

    // Task: C004-T01
    #[test]
    fn missing_selected_reference_is_corrupt_even_when_effects_are_pending() {
        let fixture = completed();
        for pending in [false, true] {
            let mut state = fixture.state().clone();
            state.attempts[1].outputs.remove("a-output");
            assert!(result_view(&state, &fixture.graph, 7, pending).is_err());
            let mut state = fixture.state().clone();
            state.attempts[1].inputs.insert("z-input".to_string(), None);
            assert!(result_view(&state, &fixture.graph, 7, pending).is_err());
        }
    }

    // Task: C004-T01
    #[test]
    fn selected_input_must_match_frozen_source_path_digest_and_size() {
        let fixture = completed();
        for changed in 0..3 {
            let mut state = fixture.state().clone();
            let input = state.attempts[1]
                .inputs
                .get_mut("z-input")
                .unwrap()
                .as_mut()
                .unwrap();
            match changed {
                0 => input.path = AbsPath::new("/elsewhere/old.md").unwrap(),
                1 => input.sha256 = Sha256Hex::new("a".repeat(64)).unwrap(),
                2 => input.bytes += 1,
                _ => unreachable!(),
            }
            assert!(result_view(&state, &fixture.graph, 7, false).is_err());
        }
    }

    // Task: C004-T01
    #[test]
    fn selected_output_must_belong_to_terminal_attempt_and_fit_declared_limit() {
        let fixture = completed();
        let mut boundary = fixture.state().clone();
        boundary.attempts[1]
            .outputs
            .get_mut("a-output")
            .unwrap()
            .bytes = 32;
        assert_eq!(
            result_view(&boundary, &fixture.graph, 7, false)
                .unwrap()
                .artifacts[0]
                .bytes,
            32
        );
        let mut over = boundary.clone();
        over.attempts[1].outputs.get_mut("a-output").unwrap().bytes = 33;
        assert!(result_view(&over, &fixture.graph, 7, false).is_err());
        boundary.attempts[1]
            .outputs
            .get_mut("a-output")
            .unwrap()
            .path = AbsPath::new("/elsewhere/final.md").unwrap();
        assert!(result_view(&boundary, &fixture.graph, 7, false).is_err());
    }

    // Task: C004-T01
    #[test]
    fn succeeded_work_requires_successful_terminal_attempt() {
        let mut fixture = Fixture::from_texts(MANIFEST, FLOW, &[]).started_with(&[]);
        fixture.begin("produce").unwrap();
        fixture.submit_ok("produce#1.0", "Produced").unwrap();
        let mut state = fixture.state().clone();
        state.status = WorkStatus::Succeeded;
        assert!(result_view(&state, &fixture.graph, 2, false).is_err());
        let fixture = completed();
        let mut state = fixture.state().clone();
        state.attempts.pop();
        assert!(result_view(&state, &fixture.graph, 7, false).is_err());
    }

    // Task: C004-T01
    #[test]
    fn result_uses_specific_frozen_version_after_real_rework() {
        let flow = FLOW.replace(
            "[[edges]]\nfrom = \"produce\"\nto = \"finish\"\nkind = \"main\"",
            "[[nodes]]\nid = \"inspect\"\ntitle = \"Inspect\"\nexecutor = \"agent\"\ninstruction = { text = \"Inspect\" }\nmax_visits = 2\n[[edges]]\nfrom = \"produce\"\nto = \"inspect\"\nkind = \"main\"\n[[edges]]\nfrom = \"inspect\"\nto = \"produce\"\nkind = \"back\"\n[[edges]]\nfrom = \"inspect\"\nto = \"finish\"\nkind = \"main\"",
        );
        let mut fixture = Fixture::from_texts(MANIFEST, &flow, &[]).started_with(&[]);
        fixture.begin("produce").unwrap();
        fixture
            .submit_with(
                &AttemptId::parse("produce#1.0").unwrap(),
                "First",
                &[("out", 3)],
            )
            .unwrap();
        fixture.begin("inspect").unwrap();
        fixture.submit_ok("inspect#1.0", "Revise").unwrap();
        fixture.begin("produce").unwrap();
        fixture.submit_ok("produce#2.0", "Second").unwrap();
        fixture.begin("inspect").unwrap();
        fixture.submit_ok("inspect#2.0", "Finished").unwrap();
        fixture.begin("finish").unwrap();
        fixture.submit_ok("finish#1.0", "Delivered").unwrap();
        let view = result_view(fixture.state(), &fixture.graph, 11, false).unwrap();
        assert_eq!(
            view.artifacts[1].path.as_str(),
            "/tmp/sheltie-test/works/2026-09-24-001-t/attempts/produce/occurrence-002/attempt-000/outputs/old.md"
        );
        assert_eq!(
            view.artifacts[1].sha256.as_str(),
            "fc11d6f28e59d3cc33c0b14ceb644bf0902ebd63d61218dffe9e7dac7c254542"
        );
    }

    // Task: C004-T01
    #[test]
    fn result_supports_start_resource_and_engine_stats_input_slots() {
        let flow = "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n[[nodes]]\nid = \"only\"\ntitle = \"Only\"\nexecutor = \"agent\"\ninstruction = { text = \"x\" }\ninputs = [{ name = \"task\", from = \"start.task\", result = true }, { name = \"resource\", from = \"resource.resources/info.txt\", result = true }, { name = \"stats\", from = \"engine.stats\", result = true }]\n";
        let mut fixture = Fixture::from_texts(MANIFEST, flow, &[("resources/info.txt", "info")])
            .started_with(&[("task", "task")]);
        fixture.begin("only").unwrap();
        fixture.submit_ok("only#1.0", "Finished").unwrap();
        let view = result_view(fixture.state(), &fixture.graph, 3, false).unwrap();
        assert_eq!(
            view.artifacts
                .iter()
                .map(|item| item.key.as_str())
                .collect::<Vec<_>>(),
            ["resource", "stats", "task"]
        );
        assert_eq!(
            view.artifacts[0].path.as_str(),
            "/tmp/sheltie-test/works/2026-09-24-001-t/workbook/resources/info.txt"
        );
        assert_eq!(
            view.artifacts[1].path.as_str(),
            "/tmp/sheltie-test/works/2026-09-24-001-t/attempts/only/occurrence-001/attempt-000/engine/stats.json"
        );
        assert_eq!(
            view.artifacts[2].path.as_str(),
            "/tmp/sheltie-test/works/2026-09-24-001-t/start-inputs/task"
        );
        for key in ["resource", "stats", "task"] {
            let mut state = fixture.state().clone();
            state.attempts[0]
                .inputs
                .get_mut(key)
                .unwrap()
                .as_mut()
                .unwrap()
                .path = AbsPath::new("/elsewhere/wrong").unwrap();
            assert!(result_view(&state, &fixture.graph, 3, false).is_err());
        }
    }
}
