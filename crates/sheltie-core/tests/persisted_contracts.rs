#![allow(clippy::unwrap_used)]

use sheltie_core::flow::{ResourceIndex, compile, parse_flow};
use sheltie_core::ids::{AttemptId, NodeId, WorkName};
use sheltie_core::testkit::Fixture;
use sheltie_core::text::Summary;
use sheltie_core::work::{Approval, BlockedReason, Principal, Timestamp, WorkState, WorkStatus};
use sheltie_core::workbook::{HostRequire, RequireKind, parse_manifest};

const MANIFEST: &str = r#"schema = "workbook/v1"
id = "metadata"
version = "1.2.3"
name = "Metadata workbook"
description = "Declared metadata"
flows = ["flows/default.toml"]
[[requires]]
kind = "skill"
name = "writer"
"#;

// Task: C002-T31
#[test]
fn validated_metadata_preserves_declared_values_for_public_consumers() {
    let manifest = parse_manifest(MANIFEST).unwrap();
    assert_eq!(manifest.version(), "1.2.3");
    assert_eq!(manifest.name(), "Metadata workbook");
    assert_eq!(manifest.description(), Some("Declared metadata"));
    assert_eq!(manifest.flows().len(), 1);
    assert_eq!(manifest.flows()[0].as_str(), "flows/default.toml");
    assert_eq!(manifest.requires().len(), 1);
    assert_eq!(manifest.requires()[0].kind(), RequireKind::Skill);
    assert_eq!(manifest.requires()[0].name(), "writer");
    let flow = parse_flow(
        r#"schema = "flow/v1"
id = "default"
entry = "first"
[[nodes]]
id = "first"
title = "First node"
executor = "agent"
instruction = { text = "first" }
[[nodes]]
id = "last"
title = "Last node"
executor = "agent"
instruction = { text = "last" }
[[edges]]
from = "first"
to = "last"
kind = "main"
"#,
    )
    .unwrap();
    assert_eq!(flow.nodes()[0].title(), "First node");
    assert_eq!(flow.nodes()[1].title(), "Last node");
    assert_eq!(flow.edges().len(), 1);
    let graph = compile(&flow, &manifest, &ResourceIndex::default()).unwrap();
    assert_eq!(graph.nodes().count(), 2);
}

// Task: C002-T31
#[test]
fn host_requirement_snapshots_accept_exact_byte_limits_and_reject_one_extra_byte() {
    let expected = serde_json::json!({
        "kind": "skill", "name": "writer", "version": "v".repeat(32),
        "source": "https://example.com/".to_owned() + &"x".repeat(492), "digest": null
    });
    let require: HostRequire = serde_json::from_value(expected.clone()).unwrap();
    assert_eq!(serde_json::to_value(require).unwrap(), expected);
    for (field, value) in [
        ("version", "v".repeat(33)),
        (
            "source",
            expected["source"].as_str().unwrap().to_owned() + "x",
        ),
    ] {
        let mut bad = expected.clone();
        bad[field] = value.into();
        assert!(
            serde_json::from_value::<HostRequire>(bad).is_err(),
            "{field}"
        );
    }
}

fn rejected(base: &WorkState, label: &str, change: impl FnOnce(&mut WorkState)) {
    base.validate_persisted().unwrap();
    let mut bad = base.clone();
    change(&mut bad);
    assert!(bad.validate_persisted().is_err(), "{label}");
}

fn gated_fixture(flow: &str) -> Fixture {
    let manifest = parse_manifest(MANIFEST).unwrap();
    let flow = parse_flow(flow).unwrap();
    let graph = compile(&flow, &manifest, &ResourceIndex::default()).unwrap();
    let mut fixture = Fixture::two_step();
    fixture.instructions = graph
        .nodes()
        .map(|node| (node.id().clone(), "do".into()))
        .collect();
    fixture.graph = graph;
    fixture.manifest = manifest;
    fixture.started_with(&[])
}

// Task: C002-T31
#[test]
fn graph_checks_reject_a_gate_block_on_a_non_gate_and_accept_unapproved_cancellation() {
    let mut fixture = gated_fixture(
        r#"schema = "flow/v1"
id = "default"
entry = "first"
[[nodes]]
id = "first"
title = "First"
executor = "agent"
instruction = { text = "first" }
gate = true
max_visits = 1
[[nodes]]
id = "last"
title = "Last"
executor = "agent"
instruction = { text = "last" }
[[nodes]]
id = "done"
title = "Done"
executor = "agent"
instruction = { text = "done" }
[[edges]]
from = "first"
to = "last"
kind = "main"
[[edges]]
from = "last"
to = "first"
kind = "back"
[[edges]]
from = "first"
to = "done"
kind = "branch"
"#,
    );
    fixture.begin("first").unwrap();
    fixture.submit_ok("first#1.0", "first").unwrap();
    let mut cancelled = Fixture::single_gated_terminal();
    cancelled.begin("only").unwrap();
    cancelled.submit_ok("only#1.0", "cancel instead").unwrap();
    cancelled.cancel().unwrap();
    cancelled
        .state()
        .validate_gate_facts(&cancelled.graph)
        .unwrap();
    fixture.approve("first").unwrap();
    fixture.begin("last").unwrap();
    fixture.submit_ok("last#1.0", "last").unwrap();
    fixture.state().validate_persisted().unwrap();
    fixture.state().validate_gate_facts(&fixture.graph).unwrap();
    let mut bad = fixture.state().clone();
    bad.status = WorkStatus::Blocked(BlockedReason::Gate);
    bad.validate_persisted().unwrap();
    assert!(bad.validate_gate_facts(&fixture.graph).is_err());
}

// Task: C002-T31
#[test]
fn persisted_facts_reject_single_field_contradictions_and_accept_real_lifecycle_states() {
    let mut running = Fixture::two_step().started_with(&[("topic", "contract")]);
    let started = running.state().clone();
    running.begin("outline").unwrap();
    let active = running.state().clone();
    running.cancel().unwrap();
    let cancelled = running.state().clone();
    let mut failed = Fixture::two_step().started_with(&[("topic", "contract")]);
    failed.begin("outline").unwrap();
    failed.fail("outline#1.0", "failure").unwrap();
    let failure = failed.state().clone();
    failed.begin("outline").unwrap();
    failed.fail("outline#1.1", "exhausted").unwrap();
    let exhausted = failed.state().clone();
    let mut gated = Fixture::single_gated_terminal();
    gated.begin("only").unwrap();
    gated.submit_ok("only#1.0", "success").unwrap();
    let awaiting = gated.state().clone();
    gated.approve("only").unwrap();
    let succeeded = gated.state().clone();
    for state in [
        &started, &active, &cancelled, &failure, &exhausted, &awaiting, &succeeded,
    ] {
        state.validate_persisted().unwrap();
    }
    rejected(&started, "name does not match work id", |s| {
        s.name = WorkName::normalize("different").unwrap();
    });
    rejected(&started, "date does not match work id", |s| {
        s.created_at = Timestamp::parse("2026-09-25T03:00:00Z").unwrap();
    });
    rejected(&started, "current exceeds recorded visit", |s| {
        s.current.n = 2
    });
    rejected(&started, "zero occurrence", |s| s.current.n = 0);
    rejected(&started, "zero historical visit", |s| {
        s.visits.insert(NodeId::new("other").unwrap(), 0);
    });
    rejected(&failure, "attempt beyond recorded visits", |s| {
        s.attempts[0].id.occurrence = 2
    });
    rejected(&failure, "zero attempt occurrence", |s| {
        s.attempts[0].id.occurrence = 0
    });
    rejected(&failure, "duplicate attempt", |s| {
        s.attempts.push(s.attempts[0].clone())
    });
    rejected(&active, "two running attempts", |s| {
        let mut other = s.attempts[0].clone();
        other.id = AttemptId::parse("outline#1.1").unwrap();
        s.attempts.push(other);
    });
    rejected(&active, "running has end time", |s| {
        s.attempts[0].ended_at = Some(s.updated_at.clone())
    });
    rejected(&active, "running has summary", |s| {
        s.attempts[0].summary = Some(Summary::new("unexpected", "summary").unwrap());
    });
    rejected(&active, "running has failure", |s| {
        s.attempts[0].fail_reason = Some(Summary::new("unexpected", "reason").unwrap());
    });
    rejected(&active, "running has outputs", |s| {
        s.attempts[0]
            .outputs
            .insert("unexpected".into(), s.inputs["topic"].clone());
    });
    rejected(&failure, "failure has no end time", |s| {
        s.attempts[0].ended_at = None
    });
    rejected(&failure, "failure has summary", |s| {
        s.attempts[0].summary = Some(Summary::new("unexpected", "summary").unwrap());
    });
    rejected(&failure, "failure has no reason", |s| {
        s.attempts[0].fail_reason = None
    });
    rejected(&failure, "failure has outputs", |s| {
        s.attempts[0]
            .outputs
            .insert("unexpected".into(), s.inputs["topic"].clone());
    });
    rejected(&succeeded, "success has no end time", |s| {
        s.attempts[0].ended_at = None
    });
    rejected(&succeeded, "success has no summary", |s| {
        s.attempts[0].summary = None
    });
    rejected(&succeeded, "success has failure", |s| {
        s.attempts[0].fail_reason = Some(Summary::new("unexpected", "reason").unwrap());
    });
    rejected(&succeeded, "approval has wrong occurrence", |s| {
        s.approvals[0].occurrence = 2
    });
    rejected(&succeeded, "approval has no matching node", |s| {
        s.approvals[0].node = NodeId::new("other").unwrap();
    });
    rejected(
        &awaiting,
        "blocked count exceeds its historical upper bound",
        |s| s.blocked_count = u32::MAX,
    );
    rejected(&awaiting, "blocked state has zero accumulated count", |s| {
        s.blocked_count = 0
    });
    rejected(
        &succeeded,
        "gate approval lost its earlier blocked fact",
        |s| s.blocked_count = 0,
    );
    rejected(&awaiting, "approved current gate remains blocked", |s| {
        s.approvals.push(Approval {
            node: s.current.node.clone(),
            occurrence: s.current.n,
            by: Principal("reviewer".into()),
            at: s.updated_at.clone(),
        });
    });
    let mut moved = Fixture::two_step().started_with(&[("topic", "contract")]);
    moved.begin("outline").unwrap();
    moved.submit_ok("outline#1.0", "done").unwrap();
    moved.begin("summary").unwrap();
    rejected(moved.state(), "running belongs to old occurrence", |s| {
        s.attempts.last_mut().unwrap().id = AttemptId::parse("outline#1.1").unwrap();
    });

    // A previous approval with the same occurrence number belongs to another node.
    let mut two_gates = gated_fixture(
        r#"schema = "flow/v1"
id = "default"
entry = "first"
[[nodes]]
id = "first"
title = "First"
executor = "agent"
instruction = { text = "first" }
gate = true
[[nodes]]
id = "last"
title = "Last"
executor = "agent"
instruction = { text = "last" }
gate = true
[[edges]]
from = "first"
to = "last"
kind = "main"
"#,
    );
    two_gates.begin("first").unwrap();
    two_gates.submit_ok("first#1.0", "first").unwrap();
    two_gates.approve("first").unwrap();
    two_gates.begin("last").unwrap();
    two_gates.submit_ok("last#1.0", "last").unwrap();
    two_gates.state().validate_persisted().unwrap();
    rejected(
        two_gates.state(),
        "current gate already approved within valid count bounds",
        |s| {
            s.approvals[0].node = NodeId::new("last").unwrap();
        },
    );

    // An approval of an earlier visit does not approve this visit of the same node.
    let mut revisited = gated_fixture(
        r#"schema = "flow/v1"
id = "default"
entry = "first"
[[nodes]]
id = "first"
title = "First"
executor = "agent"
instruction = { text = "first" }
gate = true
max_visits = 2
[[nodes]]
id = "middle"
title = "Middle"
executor = "agent"
instruction = { text = "middle" }
[[nodes]]
id = "done"
title = "Done"
executor = "agent"
instruction = { text = "done" }
[[edges]]
from = "first"
to = "middle"
kind = "main"
[[edges]]
from = "middle"
to = "first"
kind = "back"
[[edges]]
from = "middle"
to = "done"
kind = "branch"
"#,
    );
    revisited.begin("first").unwrap();
    revisited.submit_ok("first#1.0", "first").unwrap();
    revisited.approve("first").unwrap();
    revisited.begin("middle").unwrap();
    revisited.submit_ok("middle#1.0", "middle").unwrap();
    revisited.begin("first").unwrap();
    revisited.submit_ok("first#2.0", "again").unwrap();
    revisited.state().validate_persisted().unwrap();
}

// Task: C002-T31
#[test]
fn each_blocking_command_rejects_counter_overflow_without_mutating_its_input() {
    use sheltie_core::work::{Command, ObservedFile, decide};
    let mut submission = Fixture::single_gated_terminal();
    submission.begin("only").unwrap();
    let running = submission.state().clone();
    let submitted = submission.submit_ok("only#1.0", "done").unwrap();
    let observed = submitted.state.attempts[0]
        .outputs
        .iter()
        .map(|(name, output)| {
            (
                name.clone(),
                Some(ObservedFile::new(
                    output.path.clone(),
                    output.sha256.clone(),
                    output.bytes,
                )),
            )
        })
        .collect();
    let mut failure = Fixture::two_step().started_with(&[("topic", "counter")]);
    failure.begin("outline").unwrap();
    failure.fail("outline#1.0", "retry").unwrap();
    failure.begin("outline").unwrap();
    let mut approval = gated_fixture(
        r#"schema = "flow/v1"
id = "default"
entry = "draft"
[[nodes]]
id = "draft"
title = "Draft"
executor = "agent"
max_visits = 1
instruction = { text = "draft" }
[[nodes]]
id = "review"
title = "Review"
executor = "agent"
gate = true
instruction = { text = "review" }
[[nodes]]
id = "done"
title = "Done"
executor = "agent"
instruction = { text = "done" }
[[edges]]
from = "draft"
to = "review"
kind = "main"
[[edges]]
from = "draft"
to = "done"
kind = "branch"
[[edges]]
from = "review"
to = "draft"
kind = "back"
"#,
    );
    approval.begin("draft").unwrap();
    approval.submit_ok("draft#1.0", "draft").unwrap();
    approval.begin("review").unwrap();
    approval.submit_ok("review#1.0", "review").unwrap();
    for (state, graph, command, expected) in [
        (
            running,
            &submission.graph,
            Command::SubmitAttempt {
                attempt: AttemptId::parse("only#1.0").unwrap(),
                summary: "done".into(),
                observed_outputs: observed,
            },
            1,
        ),
        (
            failure.state().clone(),
            &failure.graph,
            Command::FailAttempt {
                attempt: AttemptId::parse("outline#1.1").unwrap(),
                reason: "exhausted".into(),
            },
            1,
        ),
        (
            approval.state().clone(),
            &approval.graph,
            Command::ApproveGate {
                node: NodeId::new("review").unwrap(),
            },
            2,
        ),
    ] {
        let control = decide(Some(&state), graph, &command, &sheltie_core::testkit::ctx()).unwrap();
        assert_eq!(control.state.blocked_count, expected);
        let mut bad = state.clone();
        bad.blocked_count = u32::MAX;
        let before = bad.clone();
        let error = decide(Some(&bad), graph, &command, &sheltie_core::testkit::ctx()).unwrap_err();
        assert!(matches!(error, sheltie_core::Error::InvalidRequest { .. }));
        assert_eq!(bad, before);
    }
}
