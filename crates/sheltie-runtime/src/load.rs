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
                detail: format!("效果里的 work_id {work_id} 不合法：{error}"),
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
                "state_json.work_dir 是 {}，期望 {}",
                state.work_dir, expected
            ),
        ));
    }

    if !valid_workbook_version(&state.workbook.version) {
        return Err(corrupt(
            &state.work_id,
            "state_json.workbook.version不是合法单路径段".to_string(),
        ));
    }
    ManagedRelPath::new(format!(
        "workbooks/{}/{}",
        state.workbook.id, state.workbook.version
    ))
    .map_err(|error| corrupt(&state.work_id, format!("workbook身份路径无效：{error}")))?;
    let expected_frozen = expected.join_segment("workbook");
    let stored_frozen = state.workbook_dir();
    if stored_frozen != expected_frozen {
        return Err(corrupt(
            &state.work_id,
            format!("冻结副本路径 {stored_frozen} 与Work路径不一致"),
        ));
    }
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
                format!("state_json.attempts[{index}].id.node 不在冻结图中"),
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
                    format!("state_json.attempts[{index}].outputs.{name} 未在冻结图声明"),
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
                format!("attempts[{index}].entered_from 未继承被替换Attempt的进入来源",),
            ));
        }
        if attempt.inputs.len() != node.inputs().len() {
            return Err(corrupt(
                &state.work_id,
                format!("state_json.attempts[{index}].inputs 键集合与冻结图不一致"),
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
                            "state_json.attempts[{index}].inputs 缺少 {}",
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
                                "attempts[{index}].inputs.{} 未继承被替换Attempt的冻结引用/进入来源",
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
                                    "attempts[{index}].inputs.{} 的冻结资源 {path} 未被观察",
                                    declaration.name(),
                                ),
                            )
                        })?;
                        if reference.sha256 != observed.sha256 || reference.bytes != observed.bytes
                        {
                            return Err(corrupt(
                                &state.work_id,
                                format!(
                                    "attempts[{index}].inputs.{} 的摘要/字节数与冻结资源不一致",
                                    declaration.name(),
                                ),
                            ));
                        }
                    } else {
                        return Err(corrupt(
                            &state.work_id,
                            format!(
                                "attempts[{index}].inputs.{} 资源引用缺失",
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
                                "attempts[{index}].inputs.{} 缺少引擎stats",
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
                        corrupt(&state.work_id, format!("受管输入路径无效：{error}"))
                    })?)
                    .map_err(|error| corrupt(&state.work_id, error.to_string()))?;
                }
                _ => {
                    return Err(corrupt(
                        &state.work_id,
                        format!(
                            "state_json.attempts[{index}].inputs.{} 与冻结定义或历史产物不一致",
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
    let id = WorkbookId::new(&row.id).map_err(|error| Error::StoreCorrupt {
        detail: format!("workbooks行id不合法：{error}"),
    })?;
    if id.as_str() != row.id {
        return Err(Error::StoreCorrupt {
            detail: "workbooks行id不是规范表示".to_string(),
        });
    }
    if !valid_workbook_version(&row.version) {
        return Err(Error::StoreCorrupt {
            detail: format!("workbooks行version {:?} 不符合Workbook合同", row.version),
        });
    }
    ManagedRelPath::new(format!("workbooks/{}/{}", row.id, row.version)).map_err(|error| {
        Error::StoreCorrupt {
            detail: format!("workbooks行version不能用作受管路径：{error}"),
        }
    })?;
    Sha256Hex::new(row.digest.clone()).map_err(|error| Error::StoreCorrupt {
        detail: format!("workbooks行digest不合法：{error}"),
    })?;
    Timestamp::parse(&row.added_at).map_err(|error| Error::StoreCorrupt {
        detail: format!("workbooks行added_at不合法：{error}"),
    })?;
    let expected_rel = format!("workbooks/{}/{}", row.id, row.version);
    if row.dir != expected_rel {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "workbooks行dir {} 与登记身份 {expected_rel} 不一致",
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
                    detail: format!("冻结副本读不了：{other}"),
                },
            }
        })?;
    let manifest = &workbook.manifest;
    if manifest.id().as_str() != state.workbook.id.as_str()
        || manifest.version() != state.workbook.version
    {
        return Err(Error::StoreCorrupt {
            detail: "冻结副本manifest身份与WorkState不一致".to_string(),
        });
    }
    if workbook.flow(state.flow.as_str()).is_none() {
        return Err(Error::StoreCorrupt {
            detail: format!("冻结副本里没有Flow {}", state.flow),
        });
    }
    Ok(workbook)
}

fn require_path(work: &WorkId, field: &str, actual: &AbsPath, expected: &AbsPath) -> Result<()> {
    if actual != expected {
        return Err(corrupt(
            work,
            format!("state_json.{field} 路径 {actual} 与所属对象 {expected} 不一致"),
        ));
    }
    Ok(())
}

fn corrupt(work: &WorkId, detail: String) -> Error {
    Error::StoreCorrupt {
        detail: format!("Work {work} 的持久路径校验失败：{detail}"),
    }
}
