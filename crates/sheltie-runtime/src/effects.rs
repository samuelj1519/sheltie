//! 效果登记与恢复（存储合同 §3.2、GF-31）。
//!
//! `requests.effects_json` 是效果对象数组：I/O 完成情况，不参与业务选边，不构成
//! 第二套 Work 状态。全部路径相对管理根；`write_file` 携带精确字节，历史任务书与
//! `engine/stats.json` 按提交时字节恢复，不从最新状态重算。恢复顺序按 `audit.seq`
//! 递增；同请求先 `publish_dir` / `prepare_attempt`，再历史文件、封存或删除，最后
//! `refresh_status_card`。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{Graph, InputSource};
use sheltie_core::ids::AttemptId;
use sheltie_core::path::AbsPath;
use sheltie_core::work::{Command, Reply, WorkState, output_paths_for};

use crate::error::{Error, Result};
use crate::fsx::{self, ManagedRelPath};
use crate::home::Home;

/// Effects whose complete batch has been checked against its owning Work and frozen graph.
#[derive(Debug)]
pub(crate) struct CheckedEffects {
    ops: Vec<EffectOp>,
    start_inputs: Option<BTreeMap<String, RefJson>>,
    request_id: Option<String>,
}

impl CheckedEffects {
    pub(crate) fn as_slice(&self) -> &[EffectOp] {
        &self.ops
    }
}

/// `effects_json` 的一个效果对象。字段与存储合同 §3.2 的表逐项对应，未知字段拒绝。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectOp {
    /// `pending/<id>/payload/` → 最终目录的原子发布。
    PublishDir {
        pending: String,
        #[serde(rename = "final")]
        final_path: String,
        /// `work:<work_id>` 或 `workbook:<id>@<version>`。
        owner: String,
        /// `workbook-digest/v2`（Workbook 与 Work 冻结副本同口径）。
        digest: String,
        /// 摘要核算的子路径（相对 payload，空串为整棵）。Work 的 payload 含
        /// `workbook/` 与 `start-inputs/`，摘要只核 `workbook/`（存储合同 §3.2）。
        digest_root: String,
    },
    /// 在已提交 Attempt 下安全建立目录骨架。
    PrepareAttempt {
        work_id: String,
        attempt_id: String,
        /// 按父先于子排序的目录路径（相对管理根）。
        dirs: Vec<String>,
    },
    /// 精确字节的历史文件（任务书、`engine/stats.json`）。
    WriteFile {
        path: String,
        sha256: String,
        content: String,
    },
    /// 对原产物引用核对后置只读。
    SealOutputs { refs: Vec<RefJson> },
    /// 已核归属目录的移入与删除。
    DeleteDir {
        pending: String,
        #[serde(rename = "final")]
        final_path: String,
        owner: String,
        digest: String,
    },
    /// 从最新状态重新生成状态卡（当前投影，不是历史文件）。
    RefreshStatusCard { work_id: String },
}

/// 完整产物引用（协议 §6 形状；`seal_outputs` 用它核对原对象）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefJson {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeletedMarker {
    pub(crate) format: String,
    pub(crate) internal_id: String,
}

/// 把效果数组从 JSON 解出；结构不符报 `STORE_CORRUPT`，不猜默认值。
pub fn decode_effects(json: &str) -> Result<Vec<EffectOp>> {
    serde_json::from_str(json).map_err(|e| Error::StoreCorrupt {
        detail: format!("effects_json 解不开：{e}"),
    })
}

pub fn encode_effects(ops: &[EffectOp]) -> String {
    serde_json::to_string(ops).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn check_work_effects(
    home: &Home,
    request_id: &str,
    state: &WorkState,
    graph: &Graph,
    command: &Command,
    reply: &Reply,
    ops: Vec<EffectOp>,
) -> Result<CheckedEffects> {
    let work = state.work_id.as_str();
    let mut seen_prepare = false;
    let command_attempt = match command {
        Command::BeginAttempt { .. } => match reply {
            Reply::AttemptBegun { attempt, .. } => Some(attempt.clone()),
            _ => None,
        },
        Command::SubmitAttempt { attempt, .. } | Command::FailAttempt { attempt, .. } => {
            Some(attempt.clone())
        }
        _ => None,
    };
    validate_effect_shape(home, state, graph, command, reply, &ops)?;
    for (index, op) in ops.iter().enumerate() {
        let invalid = |field: &str, reason: &str| Error::StoreCorrupt {
            detail: format!("Work {work} requests.effects[{index}].{field} {reason}"),
        };
        match op {
            EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                digest_root,
            } => {
                let Command::Start {
                    work_id, workbook, ..
                } = command
                else {
                    return Err(invalid("kind", "不允许非start请求发布Work目录"));
                };
                let parts = pending.split('/').collect::<Vec<_>>();
                if parts.len() != 3
                    || parts[0] != "pending"
                    || parts[2] != "payload"
                    || uuid::Uuid::parse_str(parts[1]).is_err()
                {
                    return Err(invalid("pending", "不是本引擎的pending/<id>/payload路径"));
                }
                if work_id.as_str() != work
                    || final_path != &format!("works/{work}")
                    || owner != &format!("work:{work}")
                    || digest != workbook.digest.as_str()
                    || digest_root != "workbook"
                {
                    return Err(invalid("owner", "与Start命令或已提交Work身份不一致"));
                }
                ManagedRelPath::new(pending.clone())
                    .map_err(|error| invalid("pending", &format!("无效：{error}")))?;
                ManagedRelPath::new(final_path.clone())
                    .map_err(|error| invalid("final", &format!("无效：{error}")))?;
            }
            EffectOp::PrepareAttempt {
                work_id,
                attempt_id,
                dirs,
            } => {
                if seen_prepare || work_id != work {
                    return Err(invalid("work_id", "不匹配或重复准备Attempt目录"));
                }
                seen_prepare = true;
                let attempt_id = AttemptId::parse(attempt_id)
                    .map_err(|error| invalid("attempt_id", &format!("不合法：{error}")))?;
                if command_attempt.as_ref() != Some(&attempt_id)
                    || state.attempt(&attempt_id).is_none()
                {
                    return Err(invalid("attempt_id", "不属于该请求的已提交Attempt"));
                }
                let expected = expected_attempt_dirs(home, state, graph, &attempt_id)?;
                if dirs != &expected {
                    return Err(invalid(
                        "dirs",
                        &format!(
                            "与冻结图声明的Attempt目录不一致：actual={dirs:?}, expected={expected:?}"
                        ),
                    ));
                }
            }
            EffectOp::WriteFile {
                path,
                sha256,
                content,
            } => {
                let attempt_id = command_attempt
                    .as_ref()
                    .ok_or_else(|| invalid("path", "没有所属Attempt"))?;
                let attempt = state
                    .attempt(attempt_id)
                    .ok_or_else(|| invalid("path", "Attempt不在已校验Work状态中"))?;
                if !matches!(command, Command::BeginAttempt { .. }) {
                    return Err(invalid("path", "历史文件只可由begin请求登记"));
                }
                let expected_brief =
                    home.to_rel(&state.attempt_dir(attempt_id).join_segment("brief.md"))?;
                let expected_stats = home.to_rel(
                    &sheltie_core::work::layout::engine_stats_path(&state.attempt_dir(attempt_id)),
                )?;
                if path != &expected_brief && path != &expected_stats {
                    return Err(invalid("path", "不等于此Attempt的brief或engine.stats路径"));
                }
                ManagedRelPath::new(path.clone())
                    .map_err(|error| invalid("path", &format!("无效：{error}")))?;
                if Sha256Hex::new(sha256.clone()).is_err()
                    || Sha256Hex::of_bytes(content.as_bytes()).as_str() != sha256
                {
                    return Err(invalid("sha256", "与历史文件精确字节不匹配"));
                }
                if path == &expected_stats {
                    let stats_path = state
                        .attempt_dir(attempt_id)
                        .join_segment("engine")
                        .join_segment("stats.json");
                    let references = attempt
                        .inputs
                        .values()
                        .filter_map(Option::as_ref)
                        .filter(|reference| reference.path == stats_path)
                        .collect::<Vec<_>>();
                    let bytes = content.len() as u64;
                    if references.is_empty()
                        || references.iter().any(|reference| {
                            reference.sha256.as_str() != sha256 || reference.bytes != bytes
                        })
                    {
                        return Err(invalid(
                            "content",
                            "engine.stats字节与同次begin绑定的ArtifactRef不一致",
                        ));
                    }
                }
            }
            EffectOp::SealOutputs { refs } => {
                let Command::SubmitAttempt { attempt, .. } = command else {
                    return Err(invalid("refs", "封存只能由submit请求登记"));
                };
                let persisted = state
                    .attempt(attempt)
                    .ok_or_else(|| invalid("refs", "Attempt不在已校验Work状态中"))?;
                let expected = persisted
                    .outputs
                    .iter()
                    .map(|(name, reference)| {
                        let path = home
                            .to_rel(&reference.path)
                            .map_err(|error| invalid(name, &format!("输出路径不受管：{error}")))?;
                        Ok((
                            name,
                            RefJson {
                                path,
                                sha256: reference.sha256.as_str().to_string(),
                                bytes: reference.bytes,
                            },
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                let expected_by_path = expected
                    .values()
                    .map(|reference| (reference.path.as_str(), reference))
                    .collect::<BTreeMap<_, _>>();
                let actual = refs
                    .iter()
                    .map(|reference| (reference.path.as_str(), reference))
                    .collect::<BTreeMap<_, _>>();
                if actual.len() != refs.len()
                    || actual.len() != expected_by_path.len()
                    || actual.iter().any(|(path, actual)| {
                        expected_by_path
                            .get(path)
                            .is_none_or(|expected| *actual != *expected)
                    })
                {
                    return Err(invalid("refs", "与该Attempt已提交的产物引用不一致"));
                }
            }
            EffectOp::RefreshStatusCard { work_id } if work_id == work => {}
            EffectOp::RefreshStatusCard { .. } => {
                return Err(invalid("work_id", "状态卡归属不是本Work"));
            }
            EffectOp::DeleteDir { .. } => {
                return Err(invalid("kind", "Work请求不得删除Workbook目录"));
            }
        }
    }
    let start_inputs = matches!(command, Command::Start { .. })
        .then(|| {
            state
                .inputs
                .iter()
                .map(|(key, reference)| {
                    let path = home.to_rel(&reference.path)?;
                    Ok((
                        key.clone(),
                        RefJson {
                            path,
                            sha256: reference.sha256.as_str().to_string(),
                            bytes: reference.bytes,
                        },
                    ))
                })
                .collect::<Result<BTreeMap<_, _>>>()
        })
        .transpose()?;
    Ok(CheckedEffects {
        ops,
        start_inputs,
        request_id: Some(request_id.to_string()),
    })
}

fn validate_effect_shape(
    home: &Home,
    state: &WorkState,
    graph: &Graph,
    command: &Command,
    reply: &Reply,
    ops: &[EffectOp],
) -> Result<()> {
    let work = state.work_id.as_str();
    let valid = match (command, reply) {
        (Command::Start { .. }, Reply::Started { .. }) => {
            matches!(ops, [EffectOp::PublishDir { .. }, EffectOp::RefreshStatusCard { work_id }] if work_id == work)
        }
        (Command::BeginAttempt { .. }, Reply::AttemptBegun { attempt, .. }) => {
            let Some(node) = graph.node(&attempt.node) else {
                return Err(Error::StoreCorrupt {
                    detail: format!("Work {work} 的begin Attempt节点不在冻结图中"),
                });
            };
            let wants_stats = node
                .inputs()
                .iter()
                .any(|input| matches!(input.source(), InputSource::EngineStats));
            let stats_path = home.to_rel(&sheltie_core::work::layout::engine_stats_path(
                &state.attempt_dir(attempt),
            ))?;
            let brief_path = home.to_rel(&state.attempt_dir(attempt).join_segment("brief.md"))?;
            let prepare_matches = matches!(ops.first(), Some(EffectOp::PrepareAttempt {
                work_id,
                attempt_id,
                ..
            }) if work_id == state.work_id.as_str() && attempt_id == &attempt.to_string());
            let card_matches = matches!(ops.last(), Some(EffectOp::RefreshStatusCard { work_id }) if work_id == state.work_id.as_str());
            let expected_writes = usize::from(wants_stats) + 1;
            let writes_match = ops.len() == expected_writes + 2
                && ops[1..1 + usize::from(wants_stats)].iter().all(
                    |op| matches!(op, EffectOp::WriteFile { path, .. } if path == &stats_path),
                )
                && matches!(&ops[ops.len() - 2], EffectOp::WriteFile { path, .. } if path == &brief_path);
            prepare_matches && writes_match && card_matches
        }
        (
            Command::SubmitAttempt { attempt, .. },
            Reply::AttemptSubmitted {
                attempt: reply_attempt,
                ..
            },
        ) => {
            attempt == reply_attempt
                && matches!(ops, [EffectOp::SealOutputs { .. }, EffectOp::RefreshStatusCard { work_id }] if work_id == work)
        }
        (Command::FailAttempt { .. }, Reply::AttemptFailed { .. })
        | (Command::ApproveGate { .. }, Reply::GateApproved { .. })
        | (Command::Cancel, Reply::Cancelled) => {
            matches!(ops, [EffectOp::RefreshStatusCard { work_id }] if work_id == work)
        }
        _ => false,
    };
    if !valid {
        return Err(Error::StoreCorrupt {
            detail: format!("Work {work} requests.effects的数量、种类或顺序与Command不一致"),
        });
    }
    Ok(())
}

pub(crate) enum WorkbookEffectIdentity {
    Add {
        id: String,
        version: String,
        digest: String,
    },
    Remove {
        id: String,
        version: String,
    },
}

pub(crate) fn check_workbook_effects(
    request_id: &str,
    audit_work_id: &str,
    audit_revision: i64,
    audit_at: &str,
    identity: WorkbookEffectIdentity,
    ops: Vec<EffectOp>,
    registered: Option<&crate::store::WorkbookRow>,
) -> Result<CheckedEffects> {
    if !audit_work_id.is_empty() || audit_revision != 0 || ops.len() != 1 {
        return Err(Error::StoreCorrupt {
            detail: "Workbook请求的audit归属或效果数量无效".to_string(),
        });
    }
    match (&ops[0], identity) {
        (
            EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                digest_root,
            },
            WorkbookEffectIdentity::Add {
                id,
                version,
                digest: expected_digest,
            },
        ) => {
            validate_workbook_identity(&id, &version)?;
            let expected_final = format!("workbooks/{id}/{version}");
            ManagedRelPath::new(expected_final.clone()).map_err(|error| Error::StoreCorrupt {
                detail: format!("add snapshot的Workbook身份路径不合法：{error}"),
            })?;
            if registered.is_some_and(|registered| {
                registered.id != id
                    || registered.version != version
                    || registered.digest != expected_digest
                    || registered.dir != expected_final
                    || registered.added_at != audit_at
            }) || final_path != &expected_final
                || owner != &format!("workbook:{id}@{version}")
                || digest != &expected_digest
                || !digest_root.is_empty()
                || Sha256Hex::new(digest.clone()).is_err()
            {
                return Err(Error::StoreCorrupt {
                    detail: "add请求的audit、snapshot、Store行与PublishDir归属不一致".to_string(),
                });
            }
            validate_pending_payload(pending)?;
        }
        (
            EffectOp::DeleteDir {
                pending,
                final_path,
                owner,
                digest,
            },
            WorkbookEffectIdentity::Remove { id, version },
        ) => {
            validate_workbook_identity(&id, &version)?;
            let expected_final = format!("workbooks/{id}/{version}");
            ManagedRelPath::new(expected_final.clone()).map_err(|error| Error::StoreCorrupt {
                detail: format!("remove snapshot的Workbook身份路径不合法：{error}"),
            })?;
            if final_path != &expected_final
                || owner != &format!("workbook:{id}@{version}")
                || Sha256Hex::new(digest.clone()).is_err()
            {
                return Err(Error::StoreCorrupt {
                    detail: "remove请求的audit、snapshot与DeleteDir归属不一致".to_string(),
                });
            }
            validate_pending_payload(pending)?;
        }
        _ => {
            return Err(Error::StoreCorrupt {
                detail: "Workbook请求的intent与效果kind不匹配".to_string(),
            });
        }
    }
    Ok(CheckedEffects {
        ops,
        start_inputs: None,
        request_id: Some(request_id.to_string()),
    })
}

fn validate_pending_payload(path: &str) -> Result<()> {
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() != 3
        || segments[0] != "pending"
        || segments[2] != "payload"
        || uuid::Uuid::parse_str(segments[1]).is_err()
    {
        return Err(Error::StoreCorrupt {
            detail: format!("pending原件路径 {path} 不合法"),
        });
    }
    ManagedRelPath::new(path.to_string()).map_err(|error| Error::StoreCorrupt {
        detail: format!("pending原件路径 {path} 无效：{error}"),
    })?;
    Ok(())
}

fn validate_workbook_identity(id: &str, version: &str) -> Result<()> {
    sheltie_core::ids::WorkbookId::new(id).map_err(|error| Error::StoreCorrupt {
        detail: format!("Workbook effect的id不合法：{error}"),
    })?;
    if !crate::load::valid_workbook_version(version) {
        return Err(Error::StoreCorrupt {
            detail: format!("Workbook effect的version {version:?} 不符合合同"),
        });
    }
    Ok(())
}

fn expected_attempt_dirs(
    home: &Home,
    state: &WorkState,
    graph: &Graph,
    attempt_id: &AttemptId,
) -> Result<Vec<String>> {
    let mut directories = BTreeMap::<String, ()>::new();
    let attempt_dir = state.attempt_dir(attempt_id);
    let output_dir = sheltie_core::work::layout::outputs_dir(&attempt_dir);
    add_directory_chain(home, &output_dir, &mut directories)?;
    let outputs =
        output_paths_for(state, graph, attempt_id).map_err(|error| Error::StoreCorrupt {
            detail: format!("Attempt {attempt_id} 的冻结输出定义无效：{error}"),
        })?;
    for output in outputs.values() {
        let parent = output
            .as_path()
            .parent()
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Attempt {attempt_id} 的输出没有父目录"),
            })?;
        add_directory_chain(
            home,
            &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
            &mut directories,
        )?;
    }
    let uses_engine_stats = graph.node(&attempt_id.node).is_some_and(|node| {
        node.inputs()
            .iter()
            .any(|input| matches!(input.source(), InputSource::EngineStats))
    });
    if uses_engine_stats {
        let engine_stats = sheltie_core::work::layout::engine_stats_path(&attempt_dir);
        add_directory_chain(
            home,
            &AbsPath::new(
                engine_stats
                    .as_path()
                    .parent()
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("Attempt {attempt_id} 的engine.stats没有父目录"),
                    })?
                    .to_string(),
            )
            .map_err(Error::Core)?,
            &mut directories,
        )?;
    }
    Ok(directories.into_keys().collect())
}

fn add_directory_chain(
    home: &Home,
    directory: &AbsPath,
    directories: &mut BTreeMap<String, ()>,
) -> Result<()> {
    let relative = home.to_rel(directory)?;
    let mut prefix = String::new();
    for segment in relative.split('/') {
        prefix = if prefix.is_empty() {
            segment.to_string()
        } else {
            format!("{prefix}/{segment}")
        };
        directories.insert(prefix.clone(), ());
    }
    Ok(())
}

/// 执行（或恢复）一批效果。幂等：每个动作先核对现状再动手；同对象视为已完成，
/// 不同对象报错，不覆盖、不删原件。全部成功返回 `Ok(())`；失败携带定位信息。
///
/// `publish` 是发布动作的开关：已 `published = 1` 的请求重放只核对 `write_file`，
/// 不重做发布/封存/删除（存储合同 §3.2 末段）。
pub(crate) fn execute(
    home: &Home,
    lock: &crate::home::HomeLock,
    ops: &CheckedEffects,
    publish: bool,
) -> Result<()> {
    execute_with_observed_outputs(home, lock, ops, publish, None)
}

pub(crate) fn execute_with_observed_outputs(
    home: &Home,
    lock: &crate::home::HomeLock,
    ops: &CheckedEffects,
    publish: bool,
    observed: Option<&BTreeMap<String, fsx::SafeFile>>,
) -> Result<()> {
    for op in ops.as_slice() {
        match op {
            EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                digest_root,
            } => {
                if !publish {
                    continue;
                }
                publish_dir(
                    home,
                    lock,
                    PublishSpec {
                        pending,
                        final_path,
                        owner,
                        digest,
                        digest_root,
                        start_inputs: ops.start_inputs.as_ref(),
                        request_id: ops.request_id.as_deref(),
                    },
                )?;
            }
            EffectOp::PrepareAttempt { dirs, .. } => {
                if !publish {
                    continue;
                }
                for rel in dirs {
                    let target = home.rel(rel)?;
                    fsx::ensure_dirs_under(home, lock, &target).map_err(|error| {
                        classify_committed_path_error("Attempt目录", &target, error)
                    })?;
                    fsx::sync_managed_directory_entry(home, lock, rel).map_err(|error| {
                        classify_committed_path_error("Attempt目录", &target, error)
                    })?;
                }
            }
            EffectOp::WriteFile {
                path,
                sha256,
                content,
            } => {
                let target = home.rel(path)?;
                if let Some(f) = fsx::open_managed_optional(home, &target)
                    .map_err(|error| classify_committed_path_error("历史文件", &target, error))?
                {
                    // No-follow inspection distinguishes a missing leaf from an abnormal object.
                    // A matching existing original is verified and left unchanged.
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES).map_err(|error| {
                        classify_committed_path_error("历史文件", &target, error)
                    })?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 的摘要与登记不符（被修改）"),
                        });
                    }
                    // A completed request can still have an interrupted explicit history repair.
                    fsx::sync_managed_regular_file_handle(home, lock, path, &f).map_err(
                        |error| classify_committed_path_error("历史文件", &target, error),
                    )?;
                } else {
                    // 父目录缺失或不可信时不臆造。
                    let parent = target
                        .as_path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .ok_or_else(|| Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 没有父目录"),
                        })?;
                    if !parent.exists() {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 的父目录缺失，不能恢复"),
                        });
                    }
                    fsx::write_new_atomic_file(home, lock, &target, content.as_bytes()).map_err(
                        |error| classify_committed_path_error("历史文件", &target, error),
                    )?;
                    let f = fsx::open_managed_regular(home, &target).map_err(|error| {
                        classify_committed_path_error("历史文件", &target, error)
                    })?;
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES).map_err(|error| {
                        classify_committed_path_error("历史文件", &target, error)
                    })?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 恢复后摘要与登记不符"),
                        });
                    }
                }
                crate::failpoint::maybe_exit("after_first_history_file_write");
            }
            EffectOp::SealOutputs { refs } => {
                if !publish {
                    continue;
                }
                crate::failpoint::maybe_exit("submit_before_seal");
                for r in refs {
                    let target = home.rel(&r.path)?;
                    if let Some(observed) = observed {
                        let file =
                            observed
                                .get(target.as_str())
                                .ok_or_else(|| Error::StoreCorrupt {
                                    detail: format!("正常submit缺少输出 {} 的观察句柄", r.path),
                                })?;
                        if file.path() != &target {
                            return Err(Error::StoreCorrupt {
                                detail: format!("封存句柄 {} 与效果路径不一致", r.path),
                            });
                        }
                        seal_output(home, lock, &target, file, r)?;
                    } else {
                        let file = fsx::open_managed_regular(home, &target)?;
                        seal_output(home, lock, &target, &file, r)?;
                    }
                }
            }
            EffectOp::DeleteDir {
                pending,
                final_path,
                owner,
                digest,
            } => {
                if !publish {
                    continue;
                }
                let request_id = ops
                    .request_id
                    .as_deref()
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: "DeleteDir效果缺少已校验请求身份".to_string(),
                    })?;
                delete_dir(home, lock, request_id, pending, final_path, owner, digest)?;
            }
            EffectOp::RefreshStatusCard { .. } => {
                // 状态卡是当前投影：由调用方（持有 Store）从最新 state_json 生成，
                // 不在本执行器里读库，也不保存历史卡字节。
            }
        }
    }
    Ok(())
}

fn seal_output(
    home: &Home,
    lock: &crate::home::HomeLock,
    target: &AbsPath,
    file: &fsx::SafeFile,
    reference: &RefJson,
) -> Result<()> {
    file.verify_seal_reference(&reference.sha256, reference.bytes)
        .map_err(|error| classify_committed_path_error("封存", target, error))?;
    fsx::ManagedFs::open_existing(home)?
        .set_readonly(lock, file)
        .map_err(|error| classify_committed_path_error("封存", target, error))
}

fn classify_committed_path_error(operation: &str, target: &AbsPath, error: Error) -> Error {
    match error {
        Error::Io { source, .. } if source.kind() == std::io::ErrorKind::AlreadyExists => {
            Error::StoreCorrupt {
                detail: format!("{operation}路径 {target} 在恢复落位前出现已有对象：{source}"),
            }
        }
        Error::InvalidRequest { reason } => Error::StoreCorrupt {
            detail: format!("{operation}路径 {target} 的对象身份或类型不符：{reason}"),
        },
        Error::NotFound { what } => Error::StoreCorrupt {
            detail: format!("{operation}路径 {target} 在完成前消失：{what}"),
        },
        other => other,
    }
}

/// `publish_dir`：`final` 不存在且 `pending` 在 → 核原件归属与摘要后 rename 并置
/// 只读；仅 `final` 在 → 核归属与摘要后视为完成；两者都在或内容不符 → 停止。
struct PublishSpec<'a> {
    pending: &'a str,
    final_path: &'a str,
    owner: &'a str,
    digest: &'a str,
    digest_root: &'a str,
    start_inputs: Option<&'a BTreeMap<String, RefJson>>,
    request_id: Option<&'a str>,
}

fn publish_dir(home: &Home, lock: &crate::home::HomeLock, spec: PublishSpec<'_>) -> Result<()> {
    let PublishSpec {
        pending,
        final_path,
        owner,
        digest,
        digest_root,
        start_inputs,
        request_id,
    } = spec;
    // 摘要核算的是原件的某个子路径（Work 的 payload 含 workbook/ 与 start-inputs/，
    // 摘要只核 workbook/，存储合同 §3.2）；发布动作移动的是**整个 payload**。
    let payload = home.rel(pending)?;
    let verify_at = if digest_root.is_empty() {
        payload.clone()
    } else {
        payload.join_segment(digest_root)
    };
    let dst = home.rel(final_path)?;
    let pending_exists = fsx::managed_directory_exists(home, lock, pending)?;
    let final_exists = fsx::managed_directory_exists(home, lock, final_path)?;
    match (pending_exists, final_exists) {
        (true, false) => {
            let tree = fsx::open_managed_tree(home, lock, pending)?;
            fsx::verify_managed_tree_at(home, &tree, pending).map_err(|error| {
                integrity_error(format!("发布原件 {pending} 身份复核失败"), error)
            })?;
            verify_publish_object(home, &verify_at, owner, digest)?;
            verify_start_input_references(home, pending, final_path, owner, start_inputs)?;
            fsx::sync_managed_tree(home, lock, &tree)
                .map_err(|error| integrity_error(format!("发布原件 {pending} 同步失败"), error))?;
            if let Some(request_id) = request_id {
                crate::failpoint::rendezvous("publish_after_tree_sync", request_id)
                    .map_err(|error| Error::io(pending, error))?;
            }
            fsx::verify_managed_tree_at(home, &tree, pending).map_err(|error| {
                integrity_error(format!("发布同步后原件 {pending} 身份复核失败"), error)
            })?;
            if let Some(parent) = dst.as_path().parent() {
                fsx::ensure_dirs_under(
                    home,
                    lock,
                    &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                )?;
            }
            fsx::rename_managed_tree_new(home, lock, &tree, final_path)?;
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(format!("发布后原件 {final_path} 身份复核失败"), error)
            })?;
            let final_verify = if digest_root.is_empty() {
                dst.clone()
            } else {
                dst.join_segment(digest_root)
            };
            verify_publish_object(home, &final_verify, owner, digest)?;
            verify_start_input_references(home, final_path, final_path, owner, start_inputs)?;
            fsx::sync_publish_final_root(home, lock, final_path)?;
            // 只读化只属于 Workbook 目录（合同 §5.2：目录含根 0555、文件 0444）。
            // Work 目录要保持可写——状态卡与 Attempt 目录随后还要写入；冻结副本
            // `workbook/` 子树已在提交前置只读（§5.4）。
            if owner.starts_with("workbook:") {
                fsx::set_tree_readonly_confined(home, lock, &dst)?;
            } else {
                let workbook = dst.join_segment("workbook");
                fsx::set_tree_readonly_confined(home, lock, &workbook)?;
            }
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(format!("readonly后原件 {final_path} 身份复核失败"), error)
            })?;
            verify_publish_object(home, &final_verify, owner, digest)?;
            verify_start_input_references(home, final_path, final_path, owner, start_inputs)?;
            verify_publish_final_state(home, lock, pending, final_path, &tree)?;
            Ok(())
        }
        (false, true) => {
            // 仅最终对象在：核最终对象的归属与摘要后视为完成（恢复语义 §3.1）；
            // Work 的摘要只核 `digest_root`（workbook/）子路径。
            let final_verify = if digest_root.is_empty() {
                dst.clone()
            } else {
                dst.join_segment(digest_root)
            };
            let tree = fsx::open_managed_tree(home, lock, final_path)?;
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(format!("已发布原件 {final_path} 身份复核失败"), error)
            })?;
            verify_publish_object(home, &final_verify, owner, digest)?;
            verify_start_input_references(home, final_path, final_path, owner, start_inputs)?;
            fsx::sync_managed_tree(home, lock, &tree).map_err(|error| {
                integrity_error(format!("已发布原件 {final_path} 同步失败"), error)
            })?;
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(format!("同步后原件 {final_path} 身份复核失败"), error)
            })?;
            fsx::sync_publish_parents(home, lock, pending, final_path)?;
            fsx::sync_publish_final_root(home, lock, final_path)?;
            if owner.starts_with("workbook:") {
                fsx::set_tree_readonly_confined(home, lock, &dst)?;
            } else {
                let workbook = dst.join_segment("workbook");
                fsx::set_tree_readonly_confined(home, lock, &workbook)?;
            }
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(format!("已发布原件 {final_path} 身份复核失败"), error)
            })?;
            verify_publish_object(home, &final_verify, owner, digest)?;
            verify_start_input_references(home, final_path, final_path, owner, start_inputs)?;
            verify_publish_final_state(home, lock, pending, final_path, &tree)?;
            Ok(())
        }
        (false, false) => Err(Error::StoreCorrupt {
            detail: format!("发布对象 {final_path} 与原件 {pending} 都不存在"),
        }),
        (true, true) => Err(Error::StoreCorrupt {
            detail: format!("发布对象 {final_path} 已存在且原件 {pending} 仍在，不能覆盖"),
        }),
    }
}

fn verify_publish_final_state(
    home: &Home,
    lock: &crate::home::HomeLock,
    pending: &str,
    final_path: &str,
    tree: &fsx::ManagedTree,
) -> Result<()> {
    let pending_exists = fsx::managed_directory_exists(home, lock, pending)?;
    let final_exists = fsx::managed_directory_exists(home, lock, final_path)?;
    if pending_exists || !final_exists {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "发布完成前四格状态改变：pending={pending_exists}, final={final_exists}"
            ),
        });
    }
    fsx::verify_managed_tree_at(home, tree, final_path)
        .map_err(|error| integrity_error(format!("发布终态 {final_path} 不再绑定原件"), error))?;
    Ok(())
}

fn integrity_error(context: String, error: Error) -> Error {
    match error {
        Error::Io { .. } | Error::RecoveryRequired { .. } | Error::StoreCorrupt { .. } => error,
        other => Error::StoreCorrupt {
            detail: format!("{context}：{other}"),
        },
    }
}

fn verify_publish_object(home: &Home, dir: &AbsPath, owner: &str, digest: &str) -> Result<()> {
    let Some(identity) = owner.strip_prefix("workbook:") else {
        return verify_owned_digest(home, dir, owner, digest);
    };
    let (expected_id, expected_version) =
        identity
            .split_once('@')
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Workbook发布owner {owner:?} 格式无效"),
            })?;
    let loaded =
        crate::workbook_repo::WorkbookRepo::load_managed_dir(home, dir).map_err(|error| {
            integrity_error(format!("Workbook发布目录 {dir} manifest校验失败"), error)
        })?;
    if loaded.manifest.id().as_str() != expected_id
        || loaded.manifest.version() != expected_version
        || loaded.digest.as_str() != digest
    {
        return Err(Error::StoreCorrupt {
            detail: format!("Workbook发布目录 {dir} 的manifest身份或摘要与请求owner不符"),
        });
    }
    Ok(())
}

fn verify_start_input_references(
    home: &Home,
    source_root: &str,
    final_root: &str,
    owner: &str,
    start_inputs: Option<&BTreeMap<String, RefJson>>,
) -> Result<()> {
    if !owner.starts_with("work:") {
        if start_inputs.is_some() {
            return Err(Error::StoreCorrupt {
                detail: "Workbook发布效果意外携带Work起始输入".to_string(),
            });
        }
        return Ok(());
    }
    let inputs = start_inputs.ok_or_else(|| Error::StoreCorrupt {
        detail: "Work发布缺少已校验的起始输入引用".to_string(),
    })?;
    for (key, reference) in inputs {
        let expected_final = format!("{final_root}/start-inputs/{key}");
        if reference.path != expected_final || Sha256Hex::new(reference.sha256.clone()).is_err() {
            return Err(Error::StoreCorrupt {
                detail: format!("Work发布起始输入 {key} 的路径或摘要归属无效"),
            });
        }
        let source_path = format!("{source_root}/start-inputs/{key}");
        let path = home.rel(&source_path)?;
        let file = fsx::open_managed_regular(home, &path).map_err(|error| {
            integrity_error(
                format!("Work发布起始输入 {key} 缺失或不是受管普通文件"),
                error,
            )
        })?;
        let (digest, bytes) = file
            .sha256_bounded(fsx::MAX_FILE_BYTES)
            .map_err(|error| integrity_error(format!("Work发布起始输入 {key} 读取失败"), error))?;
        if digest.as_str() != reference.sha256 || bytes != reference.bytes {
            return Err(Error::StoreCorrupt {
                detail: format!("Work发布起始输入 {key} 的实际字节与提交引用不一致"),
            });
        }
    }
    Ok(())
}

/// 核对目录摘要与归属标记（侧车）；不符报 `STORE_CORRUPT`。
fn verify_owned_digest(home: &Home, dir: &AbsPath, owner: &str, digest: &str) -> Result<()> {
    let got = crate::workbook_digest::digest_managed_dir_v2(home, dir)
        .map_err(|error| integrity_error(format!("发布原件 {dir} 摘要读取失败"), error))?;
    if got.as_str() != digest {
        return Err(Error::StoreCorrupt {
            detail: format!("发布原件 {dir} 的摘要与登记不符"),
        });
    }
    // 归属由 pending 侧车与 Store 引用共同承担；owner 串只做存在性核对。
    if owner.is_empty() {
        return Err(Error::StoreCorrupt {
            detail: "发布效果缺归属".to_string(),
        });
    }
    Ok(())
}

/// `delete_dir`（存储合同 §3.2/§3.3）：final或pending中只处理摘要与owner均匹配的
/// 登记对象；两处都缺时仅本请求的合法`.deleted` marker能证明完成。摘要不符、两端点
/// 同时存在或两端点与marker同时缺失都表示归属/结果不明，必须停止并保留现场。
fn delete_dir(
    home: &Home,
    lock: &crate::home::HomeLock,
    request_id: &str,
    pending: &str,
    final_path: &str,
    owner: &str,
    digest: &str,
) -> Result<()> {
    let internal_id = pending_internal_id(pending)?;
    if !owner.starts_with("workbook:") {
        return Err(Error::StoreCorrupt {
            detail: format!("remove请求 {request_id} 的owner不是Workbook"),
        });
    }
    crate::service::verify_pending_owner(home, internal_id, request_id, "remove_workbook")?;
    let marker_file = read_deleted_marker(home, lock, internal_id)?;
    let pending_exists = fsx::managed_directory_exists(home, lock, pending)?;
    let final_exists = fsx::managed_directory_exists(home, lock, final_path)?;
    if let Some(marker_file) = marker_file {
        if pending_exists || final_exists {
            return Err(Error::StoreCorrupt {
                detail: format!("删除请求 {request_id} 的完成marker与目录对象同时存在"),
            });
        }
        sync_deleted_marker(home, lock, internal_id, request_id, &marker_file)?;
        if fsx::managed_directory_exists(home, lock, pending)?
            || fsx::managed_directory_exists(home, lock, final_path)?
        {
            return Err(Error::StoreCorrupt {
                detail: format!("marker同步期间删除请求 {request_id} 出现目录对象"),
            });
        }
        return Ok(());
    }
    if pending_exists && final_exists {
        return Err(Error::StoreCorrupt {
            detail: format!("删除请求 {request_id} 的final与pending同时存在"),
        });
    }
    if !pending_exists && !final_exists {
        return Err(Error::StoreCorrupt {
            detail: format!("删除请求 {request_id} 两端点均缺失且没有合法完成marker，结果不明"),
        });
    }

    if final_exists {
        let tree = fsx::open_managed_tree(home, lock, final_path)?;
        fsx::verify_managed_tree_at(home, &tree, final_path)?;
        let final_dir = home.rel(final_path)?;
        verify_publish_object(home, &final_dir, owner, digest)?;
        fsx::make_managed_tree_writable(home, lock, &tree, final_path)?;
        fsx::verify_managed_tree_at(home, &tree, final_path)?;
        fsx::rename_managed_tree_new(home, lock, &tree, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        let pending_dir = home.rel(pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        fsx::make_managed_tree_writable(home, lock, &tree, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        crate::failpoint::maybe_exit("delete_payload_verified_before_remove");
        fsx::remove_managed_tree(home, lock, &tree, pending, request_id)?;
    } else {
        let tree = fsx::open_managed_tree(home, lock, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        // A crash may have moved the original before either rename parent was synced.
        fsx::sync_publish_parents(home, lock, final_path, pending)?;
        let pending_dir = home.rel(pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        fsx::make_managed_tree_writable(home, lock, &tree, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        crate::failpoint::maybe_exit("delete_payload_verified_before_remove");
        fsx::remove_managed_tree(home, lock, &tree, pending, request_id)?;
    }
    crate::failpoint::maybe_exit("delete_after_tree_removed_before_marker");
    if fsx::managed_directory_exists(home, lock, pending)?
        || fsx::managed_directory_exists(home, lock, final_path)?
    {
        return Err(Error::StoreCorrupt {
            detail: format!("删除请求 {request_id} 后仍有final或pending对象"),
        });
    }
    write_deleted_marker(home, lock, internal_id, request_id)?;
    crate::failpoint::maybe_exit("delete_marker_synced_before_mark");
    if fsx::managed_directory_exists(home, lock, pending)?
        || fsx::managed_directory_exists(home, lock, final_path)?
    {
        return Err(Error::StoreCorrupt {
            detail: format!("删除请求 {request_id} 写marker期间出现目录对象"),
        });
    }
    Ok(())
}

pub(crate) fn pending_internal_id(pending: &str) -> Result<&str> {
    let segments = pending.split('/').collect::<Vec<_>>();
    if segments.len() != 3
        || segments[0] != "pending"
        || segments[2] != "payload"
        || uuid::Uuid::parse_str(segments[1]).is_err()
    {
        return Err(Error::StoreCorrupt {
            detail: format!("删除效果的pending路径 {pending:?} 无效"),
        });
    }
    Ok(segments[1])
}

/// 独占创建并 fsync `pending/<internal_id>.deleted` 完成标记（§3.3）。
fn write_deleted_marker(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
    request_id: &str,
) -> Result<()> {
    let _ = uuid::Uuid::parse_str(internal_id).map_err(|error| Error::StoreCorrupt {
        detail: format!("删除marker内部id无效：{error}"),
    })?;
    let relative = format!("pending/{internal_id}.deleted");
    if let Some(file) = read_deleted_marker(home, lock, internal_id)? {
        return sync_deleted_marker(home, lock, internal_id, request_id, &file);
    }
    let marker = DeletedMarker {
        format: "delete-complete/v1".to_string(),
        internal_id: internal_id.to_string(),
    };
    let mut content = serde_json::to_vec(&marker).map_err(|error| Error::StoreCorrupt {
        detail: format!("删除marker序列化失败：{error}"),
    })?;
    content.push(b'\n');
    let path = home.rel(&relative)?;
    let file = fsx::write_new_file_observed(home, lock, &path, &content)?;
    sync_deleted_marker(home, lock, internal_id, request_id, &file)?;
    Ok(())
}

pub(crate) fn read_deleted_marker(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
) -> Result<Option<fsx::SafeFile>> {
    let marker_path = home
        .pending_dir()
        .join_segment(&format!("{internal_id}.deleted"));
    let Some(file) =
        fsx::open_managed_optional_locked(home, lock, &marker_path).map_err(|error| {
            integrity_error(format!("删除marker {marker_path} 身份校验失败"), error)
        })?
    else {
        return Ok(None);
    };
    validate_deleted_marker_file(&file, &marker_path, internal_id)?;
    Ok(Some(file))
}

fn validate_deleted_marker_file(
    file: &fsx::SafeFile,
    marker_path: &AbsPath,
    internal_id: &str,
) -> Result<()> {
    let mut bytes = file
        .read_bounded(4096)
        .map_err(|error| integrity_error(format!("删除marker {marker_path} 读取失败"), error))?;
    if bytes.pop() != Some(b'\n') {
        return Err(Error::StoreCorrupt {
            detail: format!("删除marker {marker_path} 缺少结尾换行"),
        });
    }
    let marker: DeletedMarker =
        serde_json::from_slice(&bytes).map_err(|error| Error::StoreCorrupt {
            detail: format!("删除marker {marker_path} JSON无效：{error}"),
        })?;
    if marker.format != "delete-complete/v1" || marker.internal_id != internal_id {
        return Err(Error::StoreCorrupt {
            detail: format!("删除marker {marker_path} 与本请求internal_id不一致"),
        });
    }
    let mut expected = serde_json::to_vec(&DeletedMarker {
        format: "delete-complete/v1".to_string(),
        internal_id: internal_id.to_string(),
    })
    .map_err(|error| Error::StoreCorrupt {
        detail: format!("删除marker期望值序列化失败：{error}"),
    })?;
    expected.push(b'\n');
    let mut actual = bytes;
    actual.push(b'\n');
    if actual != expected {
        return Err(Error::StoreCorrupt {
            detail: format!("删除marker {marker_path} 字节不是合同格式"),
        });
    }
    Ok(())
}

fn sync_deleted_marker(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
    request_id: &str,
    file: &fsx::SafeFile,
) -> Result<()> {
    let relative = format!("pending/{internal_id}.deleted");
    let path = home.rel(&relative)?;
    validate_deleted_marker_file(file, &path, internal_id)?;
    crate::failpoint::rendezvous("delete_marker_after_validation_before_sync", request_id)
        .map_err(|error| Error::io(path.as_str(), error))?;
    fsx::sync_managed_regular_file_handle(home, lock, &relative, file).map_err(|error| {
        integrity_error(format!("删除marker {internal_id}.deleted sync失败"), error)
    })?;
    validate_deleted_marker_file(file, &path, internal_id)?;
    fsx::verify_managed_file_bound(home, lock, &path, file).map_err(|error| {
        integrity_error(
            format!("删除marker {internal_id}.deleted 同步后路径绑定失败"),
            error,
        )
    })
}

/// 独立校验一个 sha256 串。
pub fn valid_digest(s: &str) -> bool {
    Sha256Hex::new(s).is_ok()
}

#[cfg(test)]
mod seal_output_tests {
    use super::*;
    use crate::fsx::{ManagedFs, SafeFile};
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    fn checked_submit(path: &ManagedRelPath, sha256: &str, bytes: u64) -> CheckedEffects {
        CheckedEffects {
            ops: vec![
                EffectOp::SealOutputs {
                    refs: vec![RefJson {
                        path: path.as_str().to_string(),
                        sha256: sha256.to_string(),
                        bytes,
                    }],
                },
                EffectOp::RefreshStatusCard {
                    work_id: "w20260929-001".to_string(),
                },
            ],
            start_inputs: None,
            request_id: None,
        }
    }

    fn observed_file() -> (
        tempfile::TempDir,
        Home,
        crate::home::HomeLock,
        ManagedFs,
        ManagedRelPath,
        SafeFile,
        Sha256Hex,
        u64,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::at(AbsPath::new(dir.path().to_str().unwrap()).unwrap());
        let lock = home.acquire_lock().unwrap();
        let fs = ManagedFs::open_existing(&home).unwrap();
        let path = ManagedRelPath::new(
            "works/w20260929-001/attempts/n/occurrence-001/attempt-000/outputs/out.md",
        )
        .unwrap();
        fs.ensure_dir(
            &lock,
            &ManagedRelPath::new(
                "works/w20260929-001/attempts/n/occurrence-001/attempt-000/outputs",
            )
            .unwrap(),
        )
        .unwrap();
        fs.write_new(&lock, &path, b"committed bytes").unwrap();
        let file = fs.open_regular(&path).unwrap();
        let (digest, bytes) = file.sha256_bounded(fsx::MAX_FILE_BYTES).unwrap();
        (dir, home, lock, fs, path, file, digest, bytes)
    }

    fn file_set(path: &AbsPath, file: SafeFile) -> BTreeMap<String, SafeFile> {
        BTreeMap::from([(path.as_str().to_string(), file)])
    }

    // Task: C002-T22
    #[test]
    fn same_inode_byte_change_stops_before_permission_change() {
        let (_dir, home, lock, _fs, relative, held, expected, bytes) = observed_file();
        let absolute = home.rel(relative.as_str()).unwrap();
        std::fs::write(absolute.as_path(), b"tampered bytes!").unwrap();
        let metadata = std::fs::metadata(absolute.as_path()).unwrap();
        assert_eq!(metadata.ino(), held.metadata().ino());
        assert_eq!(metadata.len(), bytes);
        let effects = checked_submit(&relative, expected.as_str(), bytes);

        assert!(
            execute_with_observed_outputs(
                &home,
                &lock,
                &effects,
                true,
                Some(&file_set(&absolute, held)),
            )
            .is_err()
        );
        assert_ne!(
            std::fs::metadata(absolute.as_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o222,
            0,
            "摘要变化时必须在chmod前停止"
        );
    }

    // Task: C002-T22
    #[test]
    fn path_replacement_seals_observed_object_and_leaves_new_target_untouched() {
        let (_dir, home, lock, _fs, relative, held, expected, bytes) = observed_file();
        let target = home.rel(relative.as_str()).unwrap();
        let outside = tempfile::tempdir().unwrap();
        let replacement = outside.path().join("sentinel");
        let moved_original = outside.path().join("observed-original");
        std::fs::rename(target.as_path(), &moved_original).unwrap();
        std::fs::write(&replacement, b"outside sentinel").unwrap();
        let replacement_mode = replacement.metadata().unwrap().permissions().mode();
        std::os::unix::fs::symlink(&replacement, target.as_path()).unwrap();
        let effects = checked_submit(&relative, expected.as_str(), bytes);

        assert!(
            execute_with_observed_outputs(
                &home,
                &lock,
                &effects,
                true,
                Some(&file_set(&target, held)),
            )
            .is_err()
        );
        assert_eq!(
            moved_original.metadata().unwrap().permissions().mode() & 0o777,
            0o444
        );
        assert_eq!(std::fs::read(&replacement).unwrap(), b"outside sentinel");
        assert_eq!(
            replacement.metadata().unwrap().permissions().mode(),
            replacement_mode
        );
    }

    // Task: C002-T22
    #[test]
    fn normal_submit_requires_every_committed_ref_to_have_its_observed_handle() {
        let (_dir, home, lock, _fs, relative, _held, expected, bytes) = observed_file();
        let effects = checked_submit(&relative, expected.as_str(), bytes);

        assert!(matches!(
            execute_with_observed_outputs(&home, &lock, &effects, true, Some(&BTreeMap::new())),
            Err(Error::StoreCorrupt { detail }) if detail.contains("缺少输出")
        ));
        assert_ne!(
            std::fs::metadata(home.rel(relative.as_str()).unwrap().as_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o222,
            0
        );
    }
}
