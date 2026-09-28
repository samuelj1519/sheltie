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
pub(crate) struct CheckedEffects(Vec<EffectOp>);

impl CheckedEffects {
    pub(crate) fn as_slice(&self) -> &[EffectOp] {
        &self.0
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
    Ok(CheckedEffects(ops))
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
            let Some(registered) = registered else {
                return Err(Error::StoreCorrupt {
                    detail: "add请求缺少对应workbooks登记行".to_string(),
                });
            };
            if registered.id != id
                || registered.version != version
                || registered.digest != expected_digest
                || registered.dir != expected_final
                || registered.added_at != audit_at
                || final_path != &expected_final
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
    Ok(CheckedEffects(ops))
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
                publish_dir(home, lock, pending, final_path, owner, digest, digest_root)?;
            }
            EffectOp::PrepareAttempt { dirs, .. } => {
                if !publish {
                    continue;
                }
                for rel in dirs {
                    fsx::ensure_dirs_under(home, lock, &home.rel(rel)?)?;
                }
            }
            EffectOp::WriteFile {
                path,
                sha256,
                content,
            } => {
                let target = home.rel(path)?;
                if target.as_path().exists() {
                    // 存在且摘要相同不写；不同是完整性错误，不掩盖修改。
                    let f = fsx::open_managed_regular(home, &target)?;
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES)?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 的摘要与登记不符（被修改）"),
                        });
                    }
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
                    fsx::write_exclusive_atomic(home, lock, &target, content.as_bytes())?;
                    let f = fsx::open_managed_regular(home, &target)?;
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES)?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 恢复后摘要与登记不符"),
                        });
                    }
                }
            }
            EffectOp::SealOutputs { refs } => {
                if !publish {
                    continue;
                }
                for r in refs {
                    let target = home.rel(&r.path)?;
                    let f = fsx::open_managed_regular(home, &target)?;
                    let (got, bytes) = f.sha256_bounded(fsx::MAX_FILE_BYTES)?;
                    if got.as_str() != r.sha256 || bytes != r.bytes {
                        return Err(Error::StoreCorrupt {
                            detail: format!(
                                "封存目标 {} 与提交时引用不符（不存在或被改变）",
                                r.path
                            ),
                        });
                    }
                    fsx::ManagedFs::open_existing(home)?.set_readonly(lock, &f)?;
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
                let _ = owner;
                delete_dir(home, lock, pending, final_path, digest)?;
            }
            EffectOp::RefreshStatusCard { .. } => {
                // 状态卡是当前投影：由调用方（持有 Store）从最新 state_json 生成，
                // 不在本执行器里读库，也不保存历史卡字节。
            }
        }
    }
    Ok(())
}

/// `publish_dir`：`final` 不存在且 `pending` 在 → 核原件归属与摘要后 rename 并置
/// 只读；仅 `final` 在 → 核归属与摘要后视为完成；两者都在或内容不符 → 停止。
fn publish_dir(
    home: &Home,
    lock: &crate::home::HomeLock,
    pending: &str,
    final_path: &str,
    owner: &str,
    digest: &str,
    digest_root: &str,
) -> Result<()> {
    // 摘要核算的是原件的某个子路径（Work 的 payload 含 workbook/ 与 start-inputs/，
    // 摘要只核 workbook/，存储合同 §3.2）；发布动作移动的是**整个 payload**。
    let payload = home.rel(pending)?;
    let verify_at = if digest_root.is_empty() {
        payload.clone()
    } else {
        payload.join_segment(digest_root)
    };
    let dst = home.rel(final_path)?;
    match (payload.as_path().exists(), dst.as_path().exists()) {
        (true, false) => {
            verify_owned_digest(home, &verify_at, owner, digest)?;
            if let Some(parent) = dst.as_path().parent() {
                fsx::ensure_dirs_under(
                    home,
                    lock,
                    &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                )?;
            }
            fsx::rename_managed_new(home, lock, pending, final_path)?;
            fsx::fsync_dir(home, lock, &dst)?;
            // 只读化只属于 Workbook 目录（合同 §5.2：目录含根 0555、文件 0444）。
            // Work 目录要保持可写——状态卡与 Attempt 目录随后还要写入；冻结副本
            // `workbook/` 子树已在提交前置只读（§5.4）。
            if owner.starts_with("workbook:") {
                fsx::set_tree_readonly_confined(home, lock, &dst)?;
            }
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
            verify_owned_digest(home, &final_verify, owner, digest)?;
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

/// 核对目录摘要与归属标记（侧车）；不符报 `STORE_CORRUPT`。
fn verify_owned_digest(home: &Home, dir: &AbsPath, owner: &str, digest: &str) -> Result<()> {
    let got = crate::workbook_digest::digest_managed_dir_v2(home, dir).map_err(|e| {
        Error::StoreCorrupt {
            detail: format!("发布原件 {dir} 读不了：{e}"),
        }
    })?;
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

/// `delete_dir`（存储合同 §3.2/§3.3）：最终目录仍在时核身份与摘要——**只有登记的
/// 那个对象**才移入本操作 pending 并删除；摘要不符说明那是别人的新生命周期对象
///（同版本重新 add），本操作的删除视为已完成，不碰它。移入后只删同一对象；完成
/// 后写 `.deleted` 持久标记，两处都缺时只有合法标记才能证明完成。
fn delete_dir(
    home: &Home,
    lock: &crate::home::HomeLock,
    pending: &str,
    final_path: &str,
    digest: &str,
) -> Result<()> {
    let fin = home.rel(final_path)?;
    let pen = home.rel(pending)?;
    let internal_id = pen
        .as_path()
        .file_name()
        .map(|n| n.to_string())
        .unwrap_or_default();
    if fin.as_path().exists() && !digest.is_empty() {
        let got = crate::workbook_digest::digest_managed_dir_v2(home, &fin).map_err(|e| {
            Error::StoreCorrupt {
                detail: format!("删除对象 {fin} 读不了：{e}"),
            }
        })?;
        if got.as_str() != digest {
            // 不是登记要删的对象（新生命周期或外部替换）：不删、不覆盖。
            write_deleted_marker(home, lock, &internal_id)?;
            return Ok(());
        }
    }
    if fin.as_path().exists() {
        fsx::make_tree_writable(home, lock, &fin)?;
        if let Some(parent) = pen.as_path().parent() {
            fsx::ensure_dirs_under(
                home,
                lock,
                &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
            )?;
        }
        fsx::rename_managed_new(home, lock, final_path, pending)?;
    }
    if pen.as_path().exists() {
        fsx::remove_tree_no_follow(home, lock, &pen)?;
    }
    // 两处都缺时本函数的执行本身就是完成证明：写持久标记（§3.3）。
    write_deleted_marker(home, lock, &internal_id)?;
    Ok(())
}

/// 独占创建并 fsync `pending/<internal_id>.deleted` 完成标记（§3.3）。
fn write_deleted_marker(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
) -> Result<()> {
    if internal_id.is_empty() {
        return Ok(());
    }
    let marker = home
        .pending_dir()
        .join_segment(&format!("{internal_id}.deleted"));
    if marker.as_path().exists() {
        return Ok(());
    }
    let content =
        format!("{{\"format\":\"delete-complete/v1\",\"internal_id\":\"{internal_id}\"}}\n");
    fsx::write_new_file(home, lock, &marker, content.as_bytes())?;
    fsx::fsync_dir(home, lock, &home.pending_dir())?;
    Ok(())
}

/// 独立校验一个 sha256 串。
pub fn valid_digest(s: &str) -> bool {
    Sha256Hex::new(s).is_ok()
}
