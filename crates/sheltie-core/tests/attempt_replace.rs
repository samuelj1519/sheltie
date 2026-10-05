#![allow(clippy::unwrap_used, clippy::expect_used)]

use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::InputSource;
use sheltie_core::ids::AttemptId;
use sheltie_core::path::AbsPath;
use sheltie_core::testkit::Fixture;
use sheltie_core::work::{
    Attempt, AttemptStatus, BlockedReason, Command, Effect, ObservedFile, Reply, WorkState,
    WorkStatus, decide, legal_next, render_stats_json, replacement_input_paths_for,
    reply_status_matches,
};

fn replacement(fixture: &Fixture, reason: &str) -> Command {
    let old = fixture.state().latest_attempt_of_current().unwrap();
    let node = fixture.graph.node(&old.id.node).unwrap();
    let observed_inputs = old
        .inputs
        .iter()
        .map(|(name, reference)| {
            let stats = node.inputs().iter().any(|declaration| {
                declaration.name() == name
                    && matches!(declaration.source(), InputSource::EngineStats)
            });
            let observation = if stats {
                None
            } else {
                reference.as_ref().map(|reference| {
                    ObservedFile::new(
                        reference.path.clone(),
                        reference.sha256.clone(),
                        reference.bytes,
                    )
                })
            };
            (name.clone(), observation)
        })
        .collect();
    Command::ReplaceAttempt {
        attempt: old.id.clone(),
        reason: reason.to_string(),
        observed_inputs,
        instruction_text: "Frozen instructions".to_string(),
    }
}

fn running() -> Fixture {
    let mut fixture = Fixture::article_review().started();
    fixture.begin("draft").unwrap();
    fixture
}

// Task: C005-T01
#[test]
fn replacement_atomically_ends_old_attempt_and_inherits_optional_frozen_inputs() {
    let fixture = running();
    let original = fixture.state().clone();
    let mut context = sheltie_core::testkit::ctx();
    context.now = sheltie_core::work::Timestamp::parse("2026-09-24T03:05:00Z").unwrap();
    let decision = decide(
        Some(&original),
        &fixture.graph,
        &replacement(&fixture, "Revoke the previous submission qualification"),
        &context,
    )
    .unwrap();
    assert_eq!(fixture.state(), &original);
    assert_eq!(decision.state.attempts.len(), 2);
    let old = &decision.state.attempts[0];
    assert_eq!(old.status, AttemptStatus::Superseded);
    assert_eq!(
        old.ended_at.as_ref().unwrap().as_str(),
        "2026-09-24T03:05:00Z"
    );
    assert_eq!(
        old.replacement_reason.as_ref().unwrap().as_str(),
        "Revoke the previous submission qualification"
    );
    assert!(old.summary.is_none() && old.fail_reason.is_none() && old.outputs.is_empty());
    let new = &decision.state.attempts[1];
    assert_eq!(new.id.to_string(), "draft#1.1");
    assert_eq!(new.status, AttemptStatus::Running);
    assert_eq!(new.entered_from, original.attempts[0].entered_from);
    assert_eq!(new.inputs, original.attempts[0].inputs);
    assert_eq!(new.inputs["review"], None);
    assert!(new.replacement_reason.is_none() && new.outputs.is_empty());
    assert_eq!(decision.state.current, original.current);
    assert_eq!(decision.state.visits, original.visits);
    assert_eq!(decision.state.blocked_count, original.blocked_count);
    assert_eq!(decision.state.status, WorkStatus::Active);
    let stats = render_stats_json(&decision.state, &fixture.graph);
    assert_eq!(stats.nodes[0].avg_seconds, 300);
    assert_eq!(stats.nodes[0].superseded, 1);
    assert_eq!(stats.nodes[0].failed, 0);
    decision.state.validate_persisted().unwrap();
    let Reply::AttemptReplaced {
        replaced_attempt,
        attempt,
        brief_path,
        output_dir,
        inputs,
        outputs,
        ..
    } = decision.reply
    else {
        panic!("replacement reply");
    };
    assert_eq!(replaced_attempt.to_string(), "draft#1.0");
    assert_eq!(attempt.to_string(), "draft#1.1");
    assert_eq!(
        brief_path.as_str(),
        "/tmp/sheltie-test/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-001/brief.md"
    );
    assert_eq!(
        output_dir.as_str(),
        "/tmp/sheltie-test/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-001/outputs"
    );
    assert_eq!(inputs["review"], None);
    assert!(
        outputs["article"]
            .as_str()
            .ends_with("attempt-001/outputs/article.md")
    );
    assert!(decision.effects.iter().any(|effect| matches!(effect, Effect::WriteBrief { content, .. } if content.contains("Frozen instructions"))));
}

// Task: C005-T01
#[test]
fn second_replacement_rejects_without_removing_current_submit_or_fail_eligibility() {
    let mut fixture = running();
    fixture.replace("draft#1.0", "First replacement").unwrap();
    let before = fixture.state().clone();
    assert!(matches!(
        fixture.replace("draft#1.1", "Second replacement"),
        Err(sheltie_core::Error::ReplacementsExhausted { .. })
    ));
    assert_eq!(fixture.state(), &before);
    assert!(legal_next(fixture.state(), &fixture.graph).iter().any(|operation| matches!(operation, sheltie_core::work::NextOp::SubmitAttempt { attempt } if attempt.to_string() == "draft#1.1")));
    fixture.submit_ok("draft#1.1", "Completed").unwrap();
}

// Task: C005-T01
#[test]
fn replacement_does_not_consume_failure_budget_and_historical_fail_uses_original_prefix() {
    let mut fixture = running();
    fixture.replace("draft#1.0", "Revoked").unwrap();
    let first = fixture.fail("draft#1.1", "First actual failure").unwrap();
    assert_eq!(first.state.status, WorkStatus::Active);
    fixture.begin("draft").unwrap();
    assert_eq!(
        fixture
            .state()
            .latest_attempt_of_current()
            .unwrap()
            .id
            .to_string(),
        "draft#1.2"
    );
    let second = fixture.fail("draft#1.2", "Second actual failure").unwrap();
    assert_eq!(
        second.state.status,
        WorkStatus::Blocked(BlockedReason::RetriesExhausted)
    );
    assert!(reply_status_matches(
        &first.reply,
        WorkStatus::Active,
        fixture.state(),
        &fixture.graph
    ));
    assert!(!reply_status_matches(
        &first.reply,
        WorkStatus::Blocked(BlockedReason::RetriesExhausted),
        fixture.state(),
        &fixture.graph
    ));
    assert!(reply_status_matches(
        &second.reply,
        second.state.status,
        fixture.state(),
        &fixture.graph
    ));
    for attempt in ["draft#1.0", "draft#1.3"] {
        assert!(!reply_status_matches(
            &Reply::AttemptFailed {
                attempt: AttemptId::parse(attempt).unwrap()
            },
            WorkStatus::Active,
            fixture.state(),
            &fixture.graph
        ));
    }
    fixture.state().validate_persisted().unwrap();
}

// Task: C005-T01
#[test]
fn zero_retries_still_allows_replacement_but_first_real_failure_blocks() {
    let files = sheltie_core::testkit::example_files("article-review");
    let flow = sheltie_core::testkit::ARTICLE_REVIEW_FLOW
        .replace("max_visits = 3", "max_visits = 3\nmax_retries = 0");
    let mut fixture = Fixture::from_texts(
        sheltie_core::testkit::ARTICLE_REVIEW_MANIFEST,
        &flow,
        &files,
    )
    .started();
    fixture.begin("draft").unwrap();
    fixture.replace("draft#1.0", "Revoked").unwrap();
    fixture.fail("draft#1.1", "Failed").unwrap();
    assert_eq!(
        fixture.state().status,
        WorkStatus::Blocked(BlockedReason::RetriesExhausted)
    );
    assert_eq!(fixture.state().blocked_count, 1);
}

// Task: C005-T01
#[test]
fn replacement_quota_resets_for_new_occurrence_and_keeps_entered_from() {
    let mut fixture = running();
    fixture.replace("draft#1.0", "Revoked").unwrap();
    fixture.submit_ok("draft#1.1", "Completed").unwrap();
    fixture.begin("review").unwrap();
    fixture.submit_ok("review#1.0", "Rework").unwrap();
    fixture.begin("draft").unwrap();
    let source = fixture
        .state()
        .latest_attempt_of_current()
        .unwrap()
        .entered_from
        .clone();
    fixture
        .replace("draft#2.0", "Revocation in a new Occurrence")
        .unwrap();
    let new = fixture.state().latest_attempt_of_current().unwrap();
    assert_eq!(new.id.to_string(), "draft#2.1");
    assert_eq!(new.entered_from, source);
    assert_eq!(source.unwrap().0.to_string(), "review#1");
    fixture.state().validate_persisted().unwrap();
}

// Task: C005-T01
#[test]
fn superseded_submit_and_fail_reject_and_terminal_guard_takes_precedence() {
    let mut fixture = running();
    fixture.replace("draft#1.0", "Revoked").unwrap();
    assert!(matches!(
        fixture.fail("draft#1.0", "Late"),
        Err(sheltie_core::Error::AttemptNotRunning { .. })
    ));
    assert!(matches!(
        fixture.submit_ok("draft#1.0", "Late"),
        Err(sheltie_core::Error::AttemptNotRunning { .. })
    ));
    fixture.cancel().unwrap();
    assert!(matches!(
        fixture.fail("draft#1.0", "Late"),
        Err(sheltie_core::Error::WorkTerminal { .. })
    ));
}

// Task: C005-T01
#[test]
fn replacement_checks_target_and_qualification_before_reason_or_inputs() {
    let mut fixture = running();
    let missing = AttemptId::parse("draft#1.8").unwrap();
    assert!(matches!(
        replacement_input_paths_for(fixture.state(), &fixture.graph, &missing),
        Err(sheltie_core::Error::AttemptNotFound { .. })
    ));
    fixture.fail("draft#1.0", "Failed").unwrap();
    assert!(matches!(
        replacement_input_paths_for(
            fixture.state(),
            &fixture.graph,
            &AttemptId::parse("draft#1.0").unwrap()
        ),
        Err(sheltie_core::Error::AttemptNotRunning { .. })
    ));
    fixture.begin("draft").unwrap();
    let mut displaced = fixture.state().clone();
    displaced.current.n = 2;
    assert!(matches!(
        replacement_input_paths_for(
            &displaced,
            &fixture.graph,
            &AttemptId::parse("draft#1.1").unwrap()
        ),
        Err(sheltie_core::Error::IllegalNext { .. })
    ));
}

// Task: C005-T01
#[test]
fn replacement_rejects_any_changed_frozen_observation_and_optional_rebinding() {
    let fixture = running();
    let original = fixture.state().clone();
    for change in 0..5 {
        let mut command = replacement(&fixture, "Revoked");
        let Command::ReplaceAttempt {
            observed_inputs, ..
        } = &mut command
        else {
            panic!();
        };
        match change {
            0 => {
                observed_inputs
                    .get_mut("topic")
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .bytes += 1
            }
            1 => {
                observed_inputs
                    .get_mut("topic")
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .sha256 = Sha256Hex::new("a".repeat(64)).unwrap()
            }
            2 => {
                observed_inputs
                    .get_mut("topic")
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .path = AbsPath::new("/wrong/topic").unwrap()
            }
            3 => {
                observed_inputs.insert("topic".to_string(), None);
            }
            4 => {
                observed_inputs.insert(
                    "review".to_string(),
                    Some(ObservedFile::new(
                        AbsPath::new("/wrong/review").unwrap(),
                        Sha256Hex::new("a".repeat(64)).unwrap(),
                        1,
                    )),
                );
            }
            _ => unreachable!(),
        }
        assert!(
            decide(
                Some(&original),
                &fixture.graph,
                &command,
                &sheltie_core::testkit::ctx()
            )
            .is_err()
        );
        assert_eq!(fixture.state(), &original);
    }
    let mut command = replacement(&fixture, "Revoked");
    let Command::ReplaceAttempt {
        observed_inputs, ..
    } = &mut command
    else {
        panic!();
    };
    observed_inputs.insert("extra".to_string(), None);
    assert!(
        decide(
            Some(&original),
            &fixture.graph,
            &command,
            &sheltie_core::testkit::ctx()
        )
        .is_err()
    );
}

// Task: C005-T01
#[test]
fn replacement_reason_accepts_exact_limit_and_rejects_one_more_byte_without_mutation() {
    let fixture = running();
    let original = fixture.state().clone();
    let accepted = decide(
        Some(&original),
        &fixture.graph,
        &replacement(&fixture, &"x".repeat(4096)),
        &sheltie_core::testkit::ctx(),
    )
    .unwrap();
    assert_eq!(
        accepted.state.attempts[0]
            .replacement_reason
            .as_ref()
            .unwrap()
            .as_str()
            .len(),
        4096
    );
    assert!(matches!(
        decide(
            Some(&original),
            &fixture.graph,
            &replacement(&fixture, &"x".repeat(4097)),
            &sheltie_core::testkit::ctx()
        ),
        Err(sheltie_core::Error::SummaryTooLong { actual: 4097, .. })
    ));
    assert_eq!(fixture.state(), &original);
}

// Task: C005-T01
#[test]
fn replacement_stats_are_fresh_post_state_bytes_and_old_binding_is_preserved() {
    let mut fixture = Fixture::with_engine_stats_input();
    fixture.begin("only").unwrap();
    let old = fixture.state().attempts[0].inputs["stats"].clone();
    let decision = fixture.replace("only#1.0", "Revoked").unwrap();
    let new = decision.state.attempts[1].inputs["stats"].as_ref().unwrap();
    assert_eq!(decision.state.attempts[0].inputs["stats"], old);
    assert_ne!(Some(new), old.as_ref());
    assert!(new.path.as_str().ends_with("attempt-001/engine/stats.json"));
    let contents = decision
        .effects
        .iter()
        .find_map(|effect| match effect {
            Effect::WriteFile { path, content } if path == &new.path => Some(content),
            _ => None,
        })
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(contents).unwrap();
    assert_eq!(json["nodes"][0]["attempts"], 2);
    assert_eq!(json["nodes"][0]["failed"], 0);
    assert_eq!(json["nodes"][0]["superseded"], 1);
    assert_eq!(new.sha256, Sha256Hex::of_bytes(contents.as_bytes()));
    assert_eq!(new.bytes, contents.len() as u64);
    let stats = render_stats_json(&decision.state, &fixture.graph);
    assert_eq!(stats.nodes[0].superseded, 1);
    decision.state.validate_persisted().unwrap();
}

// Task: C005-T01
#[test]
fn replacement_reason_field_is_required_nullable_and_unknown_fields_are_rejected() {
    let mut fixture = running();
    let original = &fixture.state().attempts[0];
    let value = serde_json::to_value(original).unwrap();
    assert_eq!(value["replacement_reason"], serde_json::Value::Null);
    assert!(serde_json::from_value::<Attempt>(value.clone()).is_ok());
    let mut missing = value.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("replacement_reason");
    assert!(serde_json::from_value::<Attempt>(missing).is_err());
    let mut extra = value;
    extra["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Attempt>(extra).is_err());
    fixture.replace("draft#1.0", "Revoked").unwrap();
    let state: WorkState =
        serde_json::from_str(&serde_json::to_string(fixture.state()).unwrap()).unwrap();
    state.validate_persisted().unwrap();
}

// Task: C005-T01
#[test]
fn persisted_replacement_rejects_reason_combinations_number_gaps_and_second_superseded() {
    let mut fixture = running();
    fixture.replace("draft#1.0", "Revoked").unwrap();
    let base = fixture.state().clone();
    for changed in 0..10 {
        let mut state = base.clone();
        match changed {
            0 => state.attempts[0].replacement_reason = None,
            1 => state.attempts[0].ended_at = None,
            2 => state.attempts[0].fail_reason = state.attempts[0].replacement_reason.clone(),
            3 => {
                state.attempts[1].replacement_reason = state.attempts[0].replacement_reason.clone()
            }
            4 => state.attempts[1].id.number = 2,
            5 => {
                state.attempts.pop();
            }
            6 => {
                let mut second = state.attempts[0].clone();
                second.id.number = 1;
                state.attempts[1] = second;
            }
            7 => state.attempts[0].summary = state.attempts[0].replacement_reason.clone(),
            8 => {
                let reference = state.attempts[0].inputs["topic"].as_ref().unwrap().clone();
                state.attempts[0]
                    .outputs
                    .insert("article".to_string(), reference);
            }
            9 => {
                let mut later = state.attempts[1].clone();
                later.id = AttemptId::parse("review#1.0").unwrap();
                later.status = AttemptStatus::Succeeded;
                later.summary = state.attempts[0].replacement_reason.clone();
                later.ended_at = state.attempts[0].ended_at.clone();
                state
                    .visits
                    .insert(sheltie_core::ids::NodeId::new("review").unwrap(), 1);
                state.attempts.push(later);
            }
            _ => unreachable!(),
        }
        assert!(state.validate_persisted().is_err(), "change {changed}");
    }
}

// Task: C005-T02
#[test]
fn public_next_offers_current_replacement_once_and_restores_quota_on_new_occurrence() {
    let mut fixture = running();
    let json = serde_json::to_value(sheltie_core::work::status_card_json(
        fixture.state(),
        &fixture.graph,
    ))
    .unwrap();
    let selected = json["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["op"] == "attempt replace")
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0],
        &serde_json::json!({"op":"attempt replace","args":{"work":"2026-09-24-001-t","attempt":"draft#1.0"}})
    );
    fixture.replace("draft#1.0", "Revoked").unwrap();
    assert!(
        !legal_next(fixture.state(), &fixture.graph)
            .iter()
            .any(|operation| matches!(
                operation,
                sheltie_core::work::NextOp::ReplaceAttempt { .. }
            ))
    );
    fixture.fail("draft#1.1", "Failed").unwrap();
    fixture.begin("draft").unwrap();
    assert!(
        !legal_next(fixture.state(), &fixture.graph)
            .iter()
            .any(|operation| matches!(
                operation,
                sheltie_core::work::NextOp::ReplaceAttempt { .. }
            ))
    );
    fixture.submit_ok("draft#1.2", "Completed").unwrap();
    fixture.begin("review").unwrap();
    fixture.submit_ok("review#1.0", "Rework").unwrap();
    fixture.begin("draft").unwrap();
    assert!(legal_next(fixture.state(), &fixture.graph).iter().any(|operation| matches!(operation, sheltie_core::work::NextOp::ReplaceAttempt { attempt } if attempt.to_string() == "draft#2.0")));
    fixture.cancel().unwrap();
    assert!(legal_next(fixture.state(), &fixture.graph).is_empty());
}
