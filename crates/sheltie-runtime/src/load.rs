//! Trust boundaries for data loaded from the Store.

use std::collections::BTreeMap;

use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{Graph, InputSource};
use sheltie_core::ids::{WorkId, WorkbookId};
use sheltie_core::path::{AbsPath, RelPath};
use sheltie_core::work::{
    AttemptStatus, ObservedFile, Timestamp, WorkState, output_paths_for, validate_start_inputs,
};

use crate::error::{Error, Result};
use crate::fsx::ManagedRelPath;
use crate::home::Home;

/// Refresh cards from freshly loaded Work state after the effect batch has completed.
pub(crate) fn refresh_status_cards(
    home: &Home,
    effects: &crate::effects::CheckedEffects,
    lock: &crate::home::HomeLock,
    mut load_work: impl FnMut(&WorkId) -> Result<(WorkState, Graph)>,
) -> Result<()> {
    for op in effects.as_slice() {
        if let crate::effects::EffectOp::RefreshStatusCard { work_id } = op {
            let id = WorkId::parse(work_id).map_err(|error| Error::StoreCorrupt {
                detail: format!("Effect work_id {work_id} is invalid: {error}"),
            })?;
            let (state, graph) = load_work(&id)?;
            let card = sheltie_core::work::render_status_card(&state, &graph);
            crate::fsx::write_exclusive_atomic(
                home,
                lock,
                &state.status_card_path(),
                card.as_bytes(),
            )?;
        }
    }
    Ok(())
}

/// Check path-bearing Work fields against the Work identity before opening any referenced file.
pub(crate) fn validate_work_root(home: &Home, state: &WorkState) -> Result<AbsPath> {
    let expected = home.work_dir(&state.work_id);
    if state.work_dir != expected {
        return Err(corrupt(
            &state.work_id,
            format!(
                "state_json.work_dir is {}; expected {}",
                state.work_dir, expected
            ),
        ));
    }

    if !valid_workbook_version(&state.workbook.version) {
        return Err(corrupt(
            &state.work_id,
            "state_json.workbook.version is not a valid single path segment".to_string(),
        ));
    }
    ManagedRelPath::new(format!(
        "workbooks/{}/{}",
        state.workbook.id, state.workbook.version
    ))
    .map_err(|error| {
        corrupt(
            &state.work_id,
            format!("Invalid workbook identity path: {error}"),
        )
    })?;
    Ok(expected)
}

/// Verify all persisted references against the checked graph and their owning Attempt.
pub(crate) fn validate_work_paths(
    home: &Home,
    state: &WorkState,
    graph: &Graph,
    resource_files: &BTreeMap<RelPath, ObservedFile>,
) -> Result<()> {
    let work_dir = validate_work_root(home, state)?;
    state
        .validate_gate_facts(graph)
        .map_err(|detail| corrupt(&state.work_id, detail))?;
    validate_start_inputs(graph, state.inputs.keys())
        .map_err(|error| corrupt(&state.work_id, format!("state_json.inputs: {error}")))?;
    for (key, reference) in &state.inputs {
        let expected = work_dir.join_segment("start-inputs").join_segment(key);
        require_path(&state.work_id, "inputs", &reference.path, &expected)?;
    }

    for (index, attempt) in state.attempts.iter().enumerate() {
        let node = graph.node(&attempt.id.node).ok_or_else(|| {
            corrupt(
                &state.work_id,
                format!("state_json.attempts[{index}].id.node is absent from the frozen graph"),
            )
        })?;
        let attempt_dir = state.attempt_dir(&attempt.id);
        let declared_outputs = output_paths_for(state, graph, &attempt.id).map_err(|error| {
            corrupt(
                &state.work_id,
                format!("state_json.attempts[{index}].outputs: {error}"),
            )
        })?;
        for (name, reference) in &attempt.outputs {
            let expected = declared_outputs.get(name).ok_or_else(|| {
                corrupt(
                    &state.work_id,
                    format!("state_json.attempts[{index}].outputs.{name} is not declared in the frozen graph"),
                )
            })?;
            require_path(
                &state.work_id,
                &format!("attempts[{index}].outputs.{name}"),
                &reference.path,
                expected,
            )?;
        }

        let replaced_predecessor = state.attempts[..index]
            .iter()
            .rev()
            .find(|candidate| candidate.occurrence() == attempt.occurrence())
            .filter(|previous| previous.status == AttemptStatus::Superseded);
        if replaced_predecessor
            .is_some_and(|previous| attempt.entered_from != previous.entered_from)
        {
            return Err(corrupt(
                &state.work_id,
                format!(
                    "attempts[{index}].entered_from did not inherit the replaced Attempt's incoming source",
                ),
            ));
        }
        if attempt.inputs.len() != node.inputs().len() {
            return Err(corrupt(
                &state.work_id,
                format!("state_json.attempts[{index}].inputs keys differ from the frozen graph"),
            ));
        }
        for declaration in node.inputs() {
            let actual = attempt
                .inputs
                .get(declaration.name())
                .ok_or_else(|| {
                    corrupt(
                        &state.work_id,
                        format!(
                            "state_json.attempts[{index}].inputs is missing {}",
                            declaration.name()
                        ),
                    )
                })?
                .as_ref();
            if let Some(previous) = replaced_predecessor {
                if !matches!(declaration.source(), InputSource::EngineStats) {
                    if actual
                        != previous
                            .inputs
                            .get(declaration.name())
                            .and_then(Option::as_ref)
                    {
                        return Err(corrupt(
                            &state.work_id,
                            format!(
                                "attempts[{index}].inputs.{} did not inherit the replaced Attempt's frozen reference/incoming source",
                                declaration.name(),
                            ),
                        ));
                    }
                    continue;
                }
            }
            let expected = match declaration.source() {
                InputSource::Start { key } => state.inputs.get(key),
                InputSource::Resource { path } => {
                    let expected = state.workbook_dir().join(path);
                    if let Some(reference) = actual {
                        require_path(
                            &state.work_id,
                            &format!("attempts[{index}].inputs.{}", declaration.name()),
                            &reference.path,
                            &expected,
                        )?;
                        let observed = resource_files.get(path).ok_or_else(|| {
                            corrupt(
                                &state.work_id,
                                format!(
                                    "attempts[{index}].inputs.{} frozen resource {path} was not observed",
                                    declaration.name(),
                                ),
                            )
                        })?;
                        if reference.sha256 != observed.sha256 || reference.bytes != observed.bytes
                        {
                            return Err(corrupt(
                                &state.work_id,
                                format!(
                                    "attempts[{index}].inputs.{} digest/bytes differ from the frozen resource",
                                    declaration.name(),
                                ),
                            ));
                        }
                    } else {
                        return Err(corrupt(
                            &state.work_id,
                            format!(
                                "attempts[{index}].inputs.{} resource reference is missing",
                                declaration.name()
                            ),
                        ));
                    }
                    continue;
                }
                InputSource::EngineStats => {
                    let expected_path = sheltie_core::work::layout::engine_stats_path(&attempt_dir);
                    let reference = actual.ok_or_else(|| {
                        corrupt(
                            &state.work_id,
                            format!(
                                "attempts[{index}].inputs.{} lacks engine stats",
                                declaration.name()
                            ),
                        )
                    })?;
                    require_path(
                        &state.work_id,
                        &format!("attempts[{index}].inputs.{}", declaration.name()),
                        &reference.path,
                        &expected_path,
                    )?;
                    continue;
                }
                InputSource::Node {
                    node: source,
                    output,
                } => state.attempts[..index]
                    .iter()
                    .rev()
                    .find(|candidate| {
                        candidate.id.node == *source && candidate.status == AttemptStatus::Succeeded
                    })
                    .and_then(|candidate| candidate.outputs.get(output)),
            };
            match (actual, expected) {
                (None, None) if !declaration.required() => {}
                (Some(actual), Some(expected)) if actual == expected => {
                    ManagedRelPath::new(home.to_rel(&actual.path).map_err(|error| {
                        corrupt(
                            &state.work_id,
                            format!("Invalid managed input path: {error}"),
                        )
                    })?)
                    .map_err(|error| corrupt(&state.work_id, error.to_string()))?;
                }
                _ => {
                    return Err(corrupt(
                        &state.work_id,
                        format!(
                            "state_json.attempts[{index}].inputs.{} differs from frozen definitions or historical artifacts",
                            declaration.name()
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_workbook_row(
    home: &Home,
    row: &crate::store::WorkbookRow,
) -> Result<AbsPath> {
    WorkbookId::new(&row.id).map_err(|error| Error::StoreCorrupt {
        detail: format!("Invalid workbooks row ID: {error}"),
    })?;
    if !valid_workbook_version(&row.version) {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "workbooks row version {:?} violates the Workbook contract",
                row.version
            ),
        });
    }
    ManagedRelPath::new(format!("workbooks/{}/{}", row.id, row.version)).map_err(|error| {
        Error::StoreCorrupt {
            detail: format!("workbooks row version cannot form a managed path: {error}"),
        }
    })?;
    Sha256Hex::new(row.digest.clone()).map_err(|error| Error::StoreCorrupt {
        detail: format!("Invalid workbooks row digest: {error}"),
    })?;
    Timestamp::parse(&row.added_at).map_err(|error| Error::StoreCorrupt {
        detail: format!("Invalid workbooks row added_at: {error}"),
    })?;
    let expected_rel = format!("workbooks/{}/{}", row.id, row.version);
    if row.dir != expected_rel {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "workbooks row dir {} differs from registered identity {expected_rel}",
                row.dir
            ),
        });
    }
    Ok(home.workbook_dir(&row.id, &row.version))
}

pub(crate) fn valid_workbook_version(value: &str) -> bool {
    use sheltie_core::workbook::manifest::{RESERVED_VERSION, VERSION_MAX_BYTES};

    !value.is_empty()
        && value.len() <= VERSION_MAX_BYTES
        && !matches!(value, "." | ".." | RESERVED_VERSION)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'-'))
}

pub(crate) fn compile_frozen_workbook(
    home: &Home,
    frozen: &AbsPath,
    state: &WorkState,
) -> Result<crate::workbook_repo::LoadedWorkbook> {
    let workbook =
        crate::workbook_repo::WorkbookRepo::load_managed_dir(home, frozen).map_err(|error| {
            match error {
                Error::NotFound { .. } | Error::Io { .. } => error,
                other => Error::StoreCorrupt {
                    detail: format!("Cannot read frozen copy: {other}"),
                },
            }
        })?;
    let manifest = &workbook.manifest;
    if manifest.id().as_str() != state.workbook.id.as_str()
        || manifest.version() != state.workbook.version
    {
        return Err(Error::StoreCorrupt {
            detail: "Frozen-copy manifest identity differs from WorkState".to_string(),
        });
    }
    if workbook.flow(state.flow.as_str()).is_none() {
        return Err(Error::StoreCorrupt {
            detail: format!("Frozen copy has no Flow {}", state.flow),
        });
    }
    Ok(workbook)
}

fn require_path(work: &WorkId, field: &str, actual: &AbsPath, expected: &AbsPath) -> Result<()> {
    if actual != expected {
        return Err(corrupt(
            work,
            format!("state_json.{field} path {actual} differs from owning object {expected}"),
        ));
    }
    Ok(())
}

fn corrupt(work: &WorkId, detail: String) -> Error {
    Error::StoreCorrupt {
        detail: format!("Work {work} persistent path validation failed: {detail}"),
    }
}

#[cfg(test)]
mod binding_contract_tests {
    use super::*;
    use crate::request::InputValue;
    use sheltie_core::ids::{AttemptId, NodeId};

    // Task: C002-T48
    #[test]
    fn genuine_attempts_keep_namespace_and_required_binding_checks_independent() {
        let directory = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(directory.path().to_str().unwrap())).unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/two-step")
            .canonicalize()
            .unwrap();
        let repo = crate::WorkbookRepo::new(home.clone());
        repo.add(&AbsPath::new(source.to_str().unwrap()).unwrap(), None)
            .unwrap();
        let service = crate::WorkService::new(home.clone());
        let started = service
            .start(
                crate::StartArgs {
                    workbook_id: "two-step".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: BTreeMap::from([(
                        "topic".into(),
                        InputValue::Literal {
                            text: "real topic".into(),
                        },
                    )]),
                },
                Some("binding-start".into()),
            )
            .unwrap();
        let work = WorkId::parse(started.data["work_id"].as_str().unwrap()).unwrap();
        let begun = service
            .begin(
                &work,
                &NodeId::new("outline").unwrap(),
                Some("binding-outline".into()),
            )
            .unwrap();
        let output = begun.data["outputs"]["outline"].as_str().unwrap();
        std::fs::write(output, b"actual outline\n").unwrap();
        service
            .submit(
                &work,
                &AttemptId::parse("outline#1.0").unwrap(),
                &InputValue::Literal {
                    text: "ready".into(),
                },
                Some("binding-submit".into()),
            )
            .unwrap();
        service
            .begin(
                &work,
                &NodeId::new("summary").unwrap(),
                Some("binding-summary".into()),
            )
            .unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let raw: String = connection
            .query_row("SELECT state_json FROM works", [], |row| row.get(0))
            .unwrap();
        let state: WorkState = serde_json::from_str(&raw).unwrap();
        let loaded = repo.load("two-step", None).unwrap();
        let graph = &loaded.flow("default").unwrap().1;
        validate_work_paths(&home, &state, graph, &BTreeMap::new()).unwrap();

        let mut wrong_namespace = state.clone();
        let alias = home.root().join_segment("other-topic");
        wrong_namespace.inputs.get_mut("topic").unwrap().path = alias.clone();
        wrong_namespace.attempts[0]
            .inputs
            .get_mut("topic")
            .unwrap()
            .as_mut()
            .unwrap()
            .path = alias;
        assert!(matches!(
            validate_work_paths(&home, &wrong_namespace, graph, &BTreeMap::new()),
            Err(Error::StoreCorrupt { .. })
        ));

        let mut missing_required = state;
        missing_required.attempts[0].outputs.clear();
        missing_required.attempts[1]
            .inputs
            .insert("outline".into(), None);
        assert!(matches!(
            validate_work_paths(&home, &missing_required, graph, &BTreeMap::new()),
            Err(Error::StoreCorrupt { .. })
        ));
        let after: String = connection
            .query_row("SELECT state_json FROM works", [], |row| row.get(0))
            .unwrap();
        assert_eq!(after, raw);
    }
}
