//! Trust boundaries for data loaded from the Store.

use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{Graph, InputSource};
use sheltie_core::ids::{WorkId, WorkbookId};
use sheltie_core::path::AbsPath;
use sheltie_core::work::{
    AttemptStatus, Timestamp, WorkState, output_paths_for, validate_start_inputs,
};

use crate::error::{Error, Result};
use crate::fsx::ManagedRelPath;
use crate::home::Home;
use crate::observe::build_resource_index;
use sheltie_core::workbook::parse_manifest;

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
pub(crate) fn validate_work_paths(home: &Home, state: &WorkState, graph: &Graph) -> Result<()> {
    let work_dir = validate_work_root(home, state)?;
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
) -> Result<Graph> {
    let manifest_text =
        crate::fsx::open_managed_regular(home, &frozen.join_segment("workbook.toml"))?
            .read_bounded(crate::fsx::MAX_FILE_BYTES)?;
    let manifest_text = String::from_utf8(manifest_text).map_err(|_| Error::StoreCorrupt {
        detail: "冻结副本的 workbook.toml 不是 UTF-8".to_string(),
    })?;
    let manifest = parse_manifest(&manifest_text).map_err(|error| Error::StoreCorrupt {
        detail: format!("冻结副本的 workbook.toml 解不开：{error}"),
    })?;
    if manifest.id().as_str() != state.workbook.id.as_str()
        || manifest.version() != state.workbook.version
    {
        return Err(Error::StoreCorrupt {
            detail: "冻结副本manifest身份与WorkState不一致".to_string(),
        });
    }
    let resources = build_resource_index(frozen).map_err(|error| Error::StoreCorrupt {
        detail: format!("冻结副本读不了：{error}"),
    })?;
    for path in manifest.flows() {
        let bytes = crate::fsx::open_managed_regular(home, &frozen.join(path))?
            .read_bounded(crate::fsx::MAX_FILE_BYTES)?;
        let text = String::from_utf8(bytes).map_err(|_| Error::StoreCorrupt {
            detail: format!("冻结副本的 {path} 不是 UTF-8"),
        })?;
        let definition =
            sheltie_core::flow::parse_flow(&text).map_err(|error| Error::StoreCorrupt {
                detail: format!("冻结副本的 {path} 解不开：{error}"),
            })?;
        if definition.id() == &state.flow {
            return sheltie_core::flow::compile(&definition, &manifest, &resources).map_err(
                |error| Error::StoreCorrupt {
                    detail: format!("冻结副本的图编不过：{error}"),
                },
            );
        }
    }
    Err(Error::StoreCorrupt {
        detail: format!("冻结副本里没有Flow {}", state.flow),
    })
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
