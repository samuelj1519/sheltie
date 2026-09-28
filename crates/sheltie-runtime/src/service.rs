//! Work 服务：每个写操作走「无锁预检 → 管理根写锁 → 恢复 → 决定 → 一个事务 →
//! 发布效果」（架构 §4、存储合同 §2）。响应字段全部来自提交时快照（cli-result/v2）。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::Graph;
use sheltie_core::ids::{AttemptId, NodeId, WorkId, WorkName};
use sheltie_core::path::AbsPath;
use sheltie_core::work::{
    Command, Context, Decision, Effect, NextOp, ObservedFile, Reply, StatsJson, StatusCardJson,
    WorkState, WorkStatus, WorkbookRef, decide, legal_next, render_stats, render_stats_json,
    render_status_card, status_card_json,
};

use crate::effects::{
    CheckedEffects, EffectOp, RefJson, check_work_effects, decode_effects, encode_effects, execute,
};
use crate::error::{Error, Result};
use crate::fsx::ManagedRelPath;
use crate::home::Home;
use crate::observe::{now, observe_optional, principal};
use crate::request::{InputValue, RequestIntent};
use crate::store::{CommitInput, CommitOutcome, Store};
use crate::workbook_repo::WorkbookRepo;

/// `work start` 的参数。输入值此时还只是字面量或 `@file` 路径；内容在无锁预检的
/// 重放查重**之后**才读取（存储合同 §2.1）。
#[derive(Debug, Clone)]
pub struct StartArgs {
    pub workbook_id: String,
    pub version: Option<String>,
    pub flow: String,
    pub name: Option<String>,
    pub inputs: BTreeMap<String, InputValue>,
}

/// 提交时的完整响应快照（GF-15）。`data` 是协议的数据载荷；重放逐字段返回原值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub request_id: String,
    pub revision: u64,
    #[serde(default)]
    pub replayed: bool,
    pub reply: Reply,
    #[serde(default)]
    pub data: serde_json::Value,
    pub next: Vec<NextOp>,
}

/// `work list` 一行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkSummary {
    pub work_id: WorkId,
    pub name: String,
    pub status: WorkStatus,
    pub current: String,
    pub updated_at: String,
}

/// 服务句柄。不持长连接。
#[derive(Debug, Clone)]
pub struct WorkService {
    home: Home,
    store: Store,
}

/// 加载好的 Work：状态、revision、图。
struct Loaded {
    state: WorkState,
    revision: u64,
    graph: Graph,
    resource_files: BTreeMap<sheltie_core::path::RelPath, ObservedFile>,
    instructions: BTreeMap<sheltie_core::path::RelPath, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedResponse {
    request_id: String,
    revision: u64,
    replayed: bool,
    reply: Reply,
    data: serde_json::Value,
    next: Vec<NextOp>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingOwner {
    format: String,
    internal_id: String,
    request_id: String,
    op: String,
}

/// pending 侧车与暂存容器的创建（存储合同 §3.3）。start/add再建立payload；remove只
/// 建容器，提交后核验final身份才把它移入payload。
pub(crate) fn stage_pending(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
    request_id: &str,
    op: &str,
) -> Result<AbsPath> {
    let create_payload = match op {
        "start_work" | "add_workbook" => true,
        "remove_workbook" => false,
        _ => {
            return Err(Error::StoreCorrupt {
                detail: format!("未知pending操作 {op}"),
            });
        }
    };
    let pending_root = ManagedRelPath::new("pending")?;
    let container = ManagedRelPath::new(format!("pending/{internal_id}"))?;
    let sidecar = ManagedRelPath::new(format!("pending/{internal_id}.owner"))?;
    let owner = PendingOwner {
        format: "pending/v1".to_string(),
        internal_id: internal_id.to_string(),
        request_id: request_id.to_string(),
        op: op.to_string(),
    };
    let mut content = serde_json::to_vec(&owner).map_err(|error| Error::StoreCorrupt {
        detail: format!("pending owner序列化失败：{error}"),
    })?;
    content.push(b'\n');
    let fs = crate::fsx::ManagedFs::open_existing(home)?;
    fs.ensure_dir(lock, &pending_root)?;
    fs.write_new(lock, &sidecar, &content)?;
    fs.sync_dir_locked(lock, &pending_root)?;
    fs.ensure_dir(lock, &container)?;
    let payload = ManagedRelPath::new(format!("pending/{internal_id}/payload"))?;
    if create_payload {
        fs.ensure_dir(lock, &payload)?;
    }
    home.rel(payload.as_str())
}

pub(crate) fn verify_pending_owner(
    home: &Home,
    internal_id: &str,
    request_id: &str,
    op: &str,
) -> Result<()> {
    let path = ManagedRelPath::new(format!("pending/{internal_id}.owner"))?;
    let file = crate::fsx::ManagedFs::open_existing(home)?
        .open_regular(&path)
        .map_err(|error| Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner缺失或无效：{error}"),
        })?;
    let mut bytes = file
        .read_bounded(4096)
        .map_err(|error| Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner不可读：{error}"),
        })?;
    if bytes.pop() != Some(b'\n') {
        return Err(Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner缺少结尾换行"),
        });
    }
    let owner: PendingOwner =
        serde_json::from_slice(&bytes).map_err(|error| Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner JSON无效：{error}"),
        })?;
    if owner.format != "pending/v1"
        || owner.internal_id != internal_id
        || owner.request_id != request_id
        || owner.op != op
    {
        return Err(Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner与Store归属不一致"),
        });
    }
    Ok(())
}

impl WorkService {
    pub fn new(home: Home, store: Store) -> Self {
        Self { home, store }
    }

    fn rw(&self) -> Result<Store> {
        Store::open(&self.home.store_path(), crate::store::OpenMode::ReadWrite)
    }

    /// `work start`（协议 §3）。无副作用预检（GF-30，T02）之后进入写路径：
    /// 锁内 staging → 复制核验 → 序号 → decide → 事务 → 发布。
    pub fn start(&self, args: StartArgs, request_id: Option<String>) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        // 意图构造先于一切读取：名字规范化、输入值与选择器原样进意图。
        WorkName::normalize(args.name.as_deref().unwrap_or(&args.flow))?;
        let intent = RequestIntent::StartWork {
            workbook: args.workbook_id.clone(),
            version: args.version.clone(),
            flow: args.flow.clone(),
            name: args.name.clone(),
            inputs: args.inputs.clone(),
        };

        // ── 无锁预检：只读识别已有 Store，查可重放请求 ──
        let mut inputs: Option<BTreeMap<String, String>> = None;
        let mut replay_hit = false;
        if let Ok(ro) = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadOnly) {
            if let Some((hash, _reply_json)) = ro.lookup_request(&request_id)? {
                if hash != intent.hash().as_str() {
                    return Err(Error::RequestConflict {
                        request_id: request_id.clone(),
                    });
                }
                // 命中：不读取当前 Workbook / @file（O04），效果核对在写锁内做。
                replay_hit = true;
            }
        }

        if !replay_hit {
            // 预检继续：装入 Workbook（含登记摘要核对）与 Flow，校验起始输入键。
            let ro_store = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadOnly)
                .map(|s| WorkService::new(self.home.clone(), s))
                .map_err(|_| Error::NotFound {
                    what: format!("Workbook {}", args.workbook_id),
                })?;
            let wb = ro_store
                .repo()
                .load(&args.workbook_id, args.version.as_deref())?;
            let flow = wb.flow(&args.flow).ok_or_else(|| Error::NotFound {
                what: format!("Flow {}", args.flow),
            })?;
            // @file 内容在重放查重之后读取（协议 work start 第 3 步），写路径复用
            // 同一份观察，不读第二次。
            let values = materialize_inputs(&args.inputs)?;
            sheltie_core::work::validate_start_inputs(&flow.1, values.keys())?;
            inputs = Some(values);
        }

        // ── 写路径：锁 → 恢复 → 重核 → staging → 决定 → 事务 → 发布 ──
        let lock = self.home.acquire_lock()?;
        let store = self.rw()?;
        let svc = Self::new(self.home.clone(), store);
        svc.recover(&lock)?;
        // 锁内重核请求表（预检后可能有并发写者）。
        if let Some(row) = svc.store.inspect_request(&request_id)? {
            if row.intent_hash != intent.hash().as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return svc.replay(request_id, row.reply_json, &lock);
        }
        let wb = svc
            .repo()
            .load(&args.workbook_id, args.version.as_deref())?;
        let flow = wb.flow(&args.flow).ok_or_else(|| Error::NotFound {
            what: format!("Flow {}", args.flow),
        })?;
        // 锁内重核输入键（受并发影响的前置事实）。
        let inputs = match inputs {
            Some(values) => values,
            None => materialize_inputs(&args.inputs)?,
        };
        sheltie_core::work::validate_start_inputs(&flow.1, inputs.keys())?;

        let ctx = Context {
            now: now(),
            principal: principal(),
        };
        // works/ 的祖先软链在序号分配与任何物化之前核对（O01）：软链把根重定义到
        // 根外时，这里拒绝而不是等到发布效果。
        crate::fsx::ensure_dirs_under(&self.home, &lock, &self.home.works_dir())?;
        let internal_id = uuid::Uuid::now_v7().simple().to_string();
        let payload = stage_pending(&self.home, &lock, &internal_id, &request_id, "start_work")?;

        let day = ctx.now.day().to_string();
        let seq = svc.store.allocate_seq(&day)?;
        let name = WorkName::normalize(args.name.as_deref().unwrap_or(&args.flow))?;
        let work_id = WorkId::new(&day, seq, &name)?;
        let work_dir = self.home.work_dir(&work_id);

        // payload/workbook：本 Work 的冻结定义。复制后对最终副本重新 parse/compile
        // 并核对身份与摘要（GF-17；存储合同 §5.4）。
        let frozen = payload.join_segment("workbook");
        WorkbookRepo::copy_confined(&self.home, &lock, &wb.dir, &frozen)?;
        let copied = svc.repo().load_dir(&frozen)?;
        if copied.manifest.id() != wb.manifest.id()
            || copied.manifest.version() != wb.manifest.version()
        {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "冻结副本的 manifest 身份 {}@{} 与登记 {}@{} 不符",
                    copied.manifest.id(),
                    copied.manifest.version(),
                    wb.manifest.id(),
                    wb.manifest.version(),
                ),
            });
        }
        if copied.digest != wb.digest {
            return Err(Error::StoreCorrupt {
                detail: "冻结副本摘要与登记不符（复制期间源目录变化）".to_string(),
            });
        }
        let frozen_flow = copied
            .flow(flow.0.id().as_str())
            .ok_or_else(|| Error::NotFound {
                what: format!("Flow {}", args.flow),
            })?;
        crate::fsx::set_tree_readonly_confined(&self.home, &lock, &frozen)?;

        // payload/start-inputs/<key>：物化在 pending，**记录最终路径**（发布后生效）。
        let inputs_dir = payload.join_segment("start-inputs");
        crate::fsx::ensure_dirs_under(&self.home, &lock, &inputs_dir)?;
        let mut input_refs = BTreeMap::new();
        for (key, value) in &inputs {
            let staged = Home::confine(&inputs_dir, key)?;
            crate::fsx::write_new_file(&self.home, &lock, &staged, value.as_bytes())?;
            let final_path = sheltie_core::work::start_input_path(&work_dir, key);
            input_refs.insert(
                key.clone(),
                sheltie_core::work::ArtifactRef {
                    sha256: Sha256Hex::of_bytes(value.as_bytes()),
                    bytes: value.len() as u64,
                    path: final_path,
                },
            );
        }

        let cmd = Command::Start {
            work_id: work_id.clone(),
            name,
            workbook: WorkbookRef {
                id: wb.manifest.id().clone(),
                version: wb.manifest.version().to_string(),
                digest: copied.digest.clone(),
            },
            flow: flow.0.id().clone(),
            work_dir,
            inputs: input_refs,
        };
        let decision = decide(None, &frozen_flow.1, &cmd, &ctx)?;
        let effects = vec![
            EffectOp::PublishDir {
                pending: self.home.to_rel(&payload)?,
                final_path: format!("works/{work_id}"),
                owner: format!("work:{work_id}"),
                digest: copied.digest.as_str().to_string(),
                digest_root: "workbook".to_string(),
            },
            EffectOp::RefreshStatusCard {
                work_id: work_id.as_str().to_string(),
            },
        ];
        let resp = svc.commit(
            decision,
            &frozen_flow.1,
            &request_id,
            intent.hash().as_str().to_string(),
            None,
            effects,
            &ctx,
            &cmd,
            &lock,
        )?;
        // COMMIT 后发布；随后清理本操作的空容器与侧车。
        svc.finish_request(&request_id, &lock)?;
        Ok(resp)
    }

    pub fn begin(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::BeginAttempt {
            work: work.clone(),
            node: node.clone(),
        };
        self.run_command(work, &intent, request_id, &|loaded| {
            let paths = sheltie_core::work::input_paths_for(&loaded.state, &loaded.graph, node)?;
            let mut observed = BTreeMap::new();
            for (name, path) in paths {
                let obs = match path {
                    Some(p) => observe_loaded_input(&self.home, loaded, node, &name, &p)?,
                    None => None,
                };
                observed.insert(name, obs);
            }
            Ok(Command::BeginAttempt {
                node: node.clone(),
                observed_inputs: observed,
                instruction_text: instruction_text_of(loaded, node)?,
            })
        })
    }

    pub fn submit(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        summary: &InputValue,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::SubmitAttempt {
            work: work.clone(),
            attempt: attempt.clone(),
            summary: summary.clone(),
        };
        let summary_text = materialize_summary(summary)?;
        self.run_command(work, &intent, request_id, &|loaded| {
            let paths =
                sheltie_core::work::output_paths_for(&loaded.state, &loaded.graph, attempt)?;
            let mut observed = BTreeMap::new();
            for (name, path) in paths {
                let obs = observe_output(&self.home, &name, &path)?;
                observed.insert(name, obs);
            }
            Ok(Command::SubmitAttempt {
                attempt: attempt.clone(),
                summary: summary_text.clone(),
                observed_outputs: observed,
            })
        })
    }

    pub fn fail(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        reason: &InputValue,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::FailAttempt {
            work: work.clone(),
            attempt: attempt.clone(),
            reason: reason.clone(),
        };
        let reason_text = materialize_summary(reason)?;
        self.run_command(work, &intent, request_id, &|_| {
            Ok(Command::FailAttempt {
                attempt: attempt.clone(),
                reason: reason_text.clone(),
            })
        })
    }

    pub fn approve(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::ApproveGate {
            work: work.clone(),
            node: node.clone(),
        };
        self.run_command(work, &intent, request_id, &|_| {
            Ok(Command::ApproveGate { node: node.clone() })
        })
    }

    pub fn cancel(&self, work: &WorkId, request_id: Option<String>) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::CancelWork { work: work.clone() };
        self.run_command(work, &intent, request_id, &|_| Ok(Command::Cancel))
    }

    /// 只读：状态卡文本与结构化形式。
    pub fn status(&self, work: &WorkId) -> Result<(String, StatusCardJson)> {
        let loaded = self.load(work)?;
        Ok((
            render_status_card(&loaded.state, &loaded.graph),
            status_card_json(&loaded.state, &loaded.graph),
        ))
    }

    /// 只读：事实视图文本与结构化形式（`render_stats`、`render_stats_json`）。
    pub fn stats(&self, work: &WorkId) -> Result<(String, StatsJson)> {
        let loaded = self.load(work)?;
        Ok((
            render_stats(&loaded.state, &loaded.graph),
            render_stats_json(&loaded.state, &loaded.graph),
        ))
    }

    /// 只读：全部 Work 摘要。
    pub fn list(&self) -> Result<Vec<WorkSummary>> {
        Ok(self
            .store
            .list_works()?
            .into_iter()
            .map(|row| WorkSummary {
                work_id: row.state.work_id.clone(),
                name: row.state.name.to_string(),
                status: row.state.status,
                current: row.state.current.to_string(),
                updated_at: row.state.updated_at.as_str().to_string(),
            })
            .collect())
    }

    /// 只读：按完整 id 或唯一前缀解析。多个匹配报 `InvalidRequest` 并列出候选。
    pub fn resolve_work(&self, prefix: &str) -> Result<WorkId> {
        let matches = self.store.find_works_by_prefix(prefix)?;
        match matches.len() {
            0 => Err(Error::NotFound {
                what: format!("Work {prefix}"),
            }),
            1 => Ok(matches
                .into_iter()
                .next()
                .unwrap_or_else(|| unreachable!("长度为 1 的匹配向量必有元素"))),
            _ => Err(Error::InvalidRequest {
                reason: format!(
                    "前缀 {prefix} 匹配多个 Work：{}",
                    matches
                        .iter()
                        .map(|w| w.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }),
        }
    }

    fn repo(&self) -> WorkbookRepo {
        WorkbookRepo::new(self.home.clone(), self.store.clone())
    }

    /// 从冻结副本加载状态与图。副本缺失或摘要不符报 `StoreCorrupt`；未发布的 Work
    /// 从受 Store 保护的 pending 原件读取（存储合同 §3.3）。
    fn load(&self, work: &WorkId) -> Result<Loaded> {
        let row = self.store.load_work(work)?;
        let work_dir = crate::load::validate_work_root(&self.home, &row.state)?;
        let frozen = work_dir.join_segment("workbook");
        let frozen = if frozen.as_path().exists() {
            frozen
        } else {
            self.pending_workbook(work, &row.state)?.unwrap_or(frozen)
        };
        if !frozen.as_path().exists() {
            return Err(Error::StoreCorrupt {
                detail: format!("Work {work} 的冻结副本 {} 缺失", frozen),
            });
        }
        let workbook = crate::load::compile_frozen_workbook(&self.home, &frozen, &row.state)?;
        if workbook.digest != row.state.workbook.digest {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "冻结副本 {} 的摘要 {} 与记录不符",
                    frozen,
                    workbook.digest.as_str()
                ),
            });
        }
        let graph = workbook
            .flow(row.state.flow.as_str())
            .map(|(_, graph)| graph.clone())
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("冻结副本里没有Flow {}", row.state.flow),
            })?;
        crate::load::validate_work_paths(&self.home, &row.state, &graph)?;
        Ok(Loaded {
            state: row.state,
            revision: row.revision,
            graph,
            resource_files: workbook.resource_files,
            instructions: workbook.instructions,
        })
    }

    /// 未发布 Work 的冻结副本位置：本 Work 的 `publish_dir` 效果指向的 pending 原件。
    fn pending_workbook(&self, work: &WorkId, state: &WorkState) -> Result<Option<AbsPath>> {
        for (request_id, effects_json) in self.store.unpublished_requests()? {
            let ops = decode_effects(&effects_json)?;
            let publish = ops.iter().find_map(|op| match op {
                EffectOp::PublishDir {
                    pending,
                    final_path,
                    owner,
                    digest,
                    digest_root,
                } if owner == &format!("work:{work}") => {
                    Some((pending, final_path, owner, digest, digest_root))
                }
                _ => None,
            });
            let Some((pending, final_path, owner, digest, digest_root)) = publish else {
                continue;
            };
            if ops.len() != 2
                || ops.iter().filter(|op| matches!(op, EffectOp::PublishDir { .. })).count() != 1
                || ops.iter().filter(|op| matches!(op, EffectOp::RefreshStatusCard { work_id } if work_id == work.as_str())).count() != 1
                || final_path != &format!("works/{work}")
                || owner != &format!("work:{work}")
                || digest != state.workbook.digest.as_str()
                || digest_root != "workbook"
            {
                return Err(Error::StoreCorrupt {
                    detail: format!("Work {work} 的未发布Start效果归属不一致"),
                });
            }
            let segments = pending.split('/').collect::<Vec<_>>();
            if segments.len() != 3
                || segments[0] != "pending"
                || segments[2] != "payload"
                || uuid::Uuid::parse_str(segments[1]).is_err()
            {
                return Err(Error::StoreCorrupt {
                    detail: format!("Work {work} 的pending路径无效"),
                });
            }
            let request =
                self.store
                    .inspect_request(&request_id)?
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("未发布Start {request_id} 缺少requests行"),
                    })?;
            if request.work_id.as_deref() != Some(work.as_str()) {
                return Err(Error::StoreCorrupt {
                    detail: format!("未发布Start {request_id} 的Work归属不一致"),
                });
            }
            Sha256Hex::new(request.intent_hash.clone()).map_err(|error| Error::StoreCorrupt {
                detail: format!("未发布Start {request_id} 的intent_hash无效：{error}"),
            })?;
            let audits = self.store.audit_rows(&request_id)?;
            let [audit] = audits.as_slice() else {
                return Err(Error::StoreCorrupt {
                    detail: format!("未发布Start {request_id} 应有且仅有一条audit"),
                });
            };
            let response: PersistedResponse =
                serde_json::from_str(&request.reply_json).map_err(|error| Error::StoreCorrupt {
                    detail: format!("未发布Start {request_id} 的snapshot解不开：{error}"),
                })?;
            let command: Command =
                serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                    detail: format!("未发布Start {request_id} 的audit命令解不开：{error}"),
                })?;
            let valid_start = match (&command, &response.reply) {
                (
                    Command::Start {
                        work_id: command_work,
                        name,
                        workbook,
                        flow,
                        work_dir,
                        inputs,
                    },
                    Reply::Started {
                        work_id: reply_work,
                        work_dir: reply_dir,
                        requires,
                    },
                ) => {
                    command_work == work
                        && reply_work == work
                        && name == &state.name
                        && workbook == &state.workbook
                        && flow == &state.flow
                        && inputs == &state.inputs
                        && work_dir == &state.work_dir
                        && reply_dir == &state.work_dir
                        && response.request_id == request_id
                        && response.revision == 1
                        && !response.replayed
                        && audit.work_id == work.as_str()
                        && audit.revision == 1
                        && audit.at == request.at
                        && response
                            .data
                            .get("work_id")
                            .and_then(serde_json::Value::as_str)
                            == Some(work.as_str())
                        && response
                            .data
                            .get("work_dir")
                            .and_then(serde_json::Value::as_str)
                            == Some(state.work_dir.as_str())
                        && response
                            .data
                            .get("name")
                            .and_then(serde_json::Value::as_str)
                            == Some(state.name.as_str())
                        && response
                            .data
                            .get("flow")
                            .and_then(serde_json::Value::as_str)
                            == Some(state.flow.as_str())
                        && response.data.get("workbook")
                            == Some(&serde_json::json!({
                                "id": state.workbook.id.as_str(),
                                "version": state.workbook.version.as_str(),
                                "digest": state.workbook.digest.as_str(),
                            }))
                        && response.data.get("requires")
                            == serde_json::to_value(requires).ok().as_ref()
                }
                _ => false,
            };
            if !valid_start {
                return Err(Error::StoreCorrupt {
                    detail: format!("未发布Start {request_id} 的Command、snapshot与Work不一致"),
                });
            }
            verify_pending_owner(&self.home, segments[1], &request_id, "start_work")?;
            return Ok(Some(self.home.rel(pending)?.join_segment("workbook")));
        }
        Ok(None)
    }

    /// Work 写操作的公共流程：无锁预检查重 → 锁 → 恢复 → 锁内重核 → load → 观察 →
    /// decide → 事务 → 发布。重放命中先于 load/observe（协议 §2）。
    fn run_command(
        &self,
        work: &WorkId,
        intent: &RequestIntent,
        request_id: String,
        build: &dyn Fn(&Loaded) -> Result<Command>,
    ) -> Result<Response> {
        // ── 无锁预检：只读查重。命中的历史请求绑定完整 WorkId（§2.1），不重解析
        // 前缀；效果未完成的请求不在这里恢复，交给写路径。
        {
            let ro = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadOnly)?;
            if let Some((hash, _reply_json)) = ro.lookup_request(&request_id)? {
                if hash != intent.hash().as_str() {
                    return Err(Error::RequestConflict {
                        request_id: request_id.clone(),
                    });
                }
                // 效果核对（含 write_file 的缺失补齐）在写锁内做，不在这里早退。
            }
        }

        // ── 写路径：锁 → 恢复 → 锁内重核 → load → 观察 → decide → 事务 → 发布 ──
        let lock = self.home.acquire_lock()?;
        let store = self.rw()?;
        let svc = Self::new(self.home.clone(), store);
        svc.recover(&lock)?;
        if let Some(row) = svc.store.inspect_request(&request_id)? {
            if row.intent_hash != intent.hash().as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return svc.replay(request_id, row.reply_json, &lock);
        }

        let ctx = Context {
            now: now(),
            principal: principal(),
        };
        let mut last_conflict = Error::RevisionConflict {
            expected: 0,
            actual: 0,
        };
        for _ in 0..3 {
            let loaded = svc.load(work)?;
            let cmd = build(&loaded)?;
            let decision = decide(Some(&loaded.state), &loaded.graph, &cmd, &ctx)?;
            let effects = core_effects_to_ops(&svc.home, &decision, &cmd)?;
            match svc.commit(
                decision,
                &loaded.graph,
                &request_id,
                intent.hash().as_str().to_string(),
                Some(loaded.revision),
                effects,
                &ctx,
                &cmd,
                &lock,
            ) {
                Ok(resp) => {
                    svc.finish_request(&request_id, &lock)?;
                    return Ok(resp);
                }
                Err(Error::RevisionConflict { expected, actual }) => {
                    last_conflict = Error::RevisionConflict { expected, actual };
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_conflict)
    }

    /// 恢复未完成效果（锁内，先于一切新命令）：按提交先后逐个执行；失败即阻断新
    /// 请求并返回 `EFFECT_PENDING`（committed=false，指向旧请求）。
    pub(crate) fn recover(&self, lock: &crate::home::HomeLock) -> Result<()> {
        for (request_id, _) in self.store.unpublished_requests()? {
            let row =
                self.store
                    .inspect_request(&request_id)?
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("未发布请求 {request_id} 缺少requests行"),
                    })?;
            let effects = if row.work_id.is_some() {
                self.load_checked_request(&request_id)?.2
            } else {
                self.repo().checked_effects_for(&request_id)?.1
            };
            if let Err(e) = execute(&self.home, lock, &effects, true) {
                return Err(Error::EffectPending {
                    committed: false,
                    request_id: String::new(),
                    pending_request_id: Some(request_id),
                    detail: e.to_string(),
                    original: None,
                });
            }
            self.refresh_cards_of(&effects, lock)?;
            self.store.mark_published(&request_id)?;
        }
        Ok(())
    }

    /// 效果涉及的状态卡从**最新**状态重生成（不回退历史版本，§6）。
    fn refresh_cards_of(&self, ops: &CheckedEffects, lock: &crate::home::HomeLock) -> Result<()> {
        for op in ops.as_slice() {
            if let EffectOp::RefreshStatusCard { work_id } = op {
                let id = WorkId::parse(work_id).map_err(|e| Error::StoreCorrupt {
                    detail: format!("效果里的 work_id {work_id} 不合法：{e}"),
                })?;
                let loaded = self.load(&id)?;
                let card = render_status_card(&loaded.state, &loaded.graph);
                crate::fsx::write_exclusive_atomic(
                    &self.home,
                    lock,
                    &loaded.state.status_card_path(),
                    card.as_bytes(),
                )?;
            }
        }
        Ok(())
    }

    /// COMMIT 与效果发布之间被杀的请求由 `recover` 兜底；正常路径在这里发布并标记。
    fn finish_request(&self, request_id: &str, lock: &crate::home::HomeLock) -> Result<()> {
        if self.store.inspect_request(request_id)?.is_some() {
            let (row, _, ops) = self.load_checked_request(request_id)?;
            if !row.published {
                if let Err(e) = execute(&self.home, lock, &ops, true) {
                    return Err(Error::EffectPending {
                        committed: true,
                        request_id: request_id.to_string(),
                        pending_request_id: None,
                        detail: e.to_string(),
                        original: None,
                    });
                }
                self.refresh_cards_of(&ops, lock)?;
                self.store.mark_published(request_id)?;
            } else {
                // 已完成的请求显式重放：只核对历史文件（write_file），不重做发布。
                execute(&self.home, lock, &ops, false)?;
            }
        }
        Ok(())
    }

    fn load_checked_request(
        &self,
        request_id: &str,
    ) -> Result<(crate::store::read::RequestRow, Response, CheckedEffects)> {
        let row = self
            .store
            .inspect_request(request_id)?
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("缺少请求 {request_id} 的持久记录"),
            })?;
        Sha256Hex::new(row.intent_hash.clone()).map_err(|error| Error::StoreCorrupt {
            detail: format!("请求 {request_id} 的intent_hash不合法：{error}"),
        })?;
        let audits = self.store.audit_rows(request_id)?;
        let [audit] = audits.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "请求 {request_id} 应有且仅有一条audit，实际 {} 条",
                    audits.len()
                ),
            });
        };
        let work = row
            .work_id
            .as_deref()
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Work请求 {request_id} 缺少requests.work_id"),
            })
            .and_then(|value| {
                WorkId::parse(value).map_err(|error| Error::StoreCorrupt {
                    detail: format!("请求 {request_id} 的work_id不合法：{error}"),
                })
            })?;
        if audit.work_id != work.as_str() || row.at != audit.at {
            return Err(Error::StoreCorrupt {
                detail: format!("请求 {request_id} 的audit归属/时间与requests行不一致"),
            });
        }
        let audit_revision = u64::try_from(audit.revision)
            .ok()
            .filter(|revision| *revision > 0)
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("请求 {request_id} 的audit.revision无效"),
            })?;
        let persisted: PersistedResponse =
            serde_json::from_str(&row.reply_json).map_err(|error| Error::StoreCorrupt {
                detail: format!("请求 {request_id} 的reply_json解不开：{error}"),
            })?;
        if persisted.request_id != request_id
            || persisted.revision != audit_revision
            || persisted.replayed
        {
            return Err(Error::StoreCorrupt {
                detail: format!("请求 {request_id} 的snapshot身份/revision与audit不一致"),
            });
        }
        let snapshot = Response {
            request_id: persisted.request_id,
            revision: persisted.revision,
            replayed: persisted.replayed,
            reply: persisted.reply,
            data: persisted.data,
            next: persisted.next,
        };
        let state = self.load(&work)?;
        if state.revision != audit_revision {
            // A Work's later revisions are valid; audit ties to the original request, while
            // the current state remains the checked owner for path validation.
            if state.revision < audit_revision {
                return Err(Error::StoreCorrupt {
                    detail: format!("请求 {request_id} 的audit revision超出Work当前revision"),
                });
            }
        }
        let command: Command =
            serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                detail: format!("请求 {request_id} 的audit.command_json解不开：{error}"),
            })?;
        validate_command_owner(
            request_id,
            &work,
            &state.state,
            &state.graph,
            &command,
            &snapshot,
        )?;
        let ops = decode_effects(&row.effects_json)?;
        let checked = check_work_effects(
            &self.home,
            &state.state,
            &state.graph,
            &command,
            &snapshot.reply,
            ops,
        )?;
        for effect in checked.as_slice() {
            if !row.published {
                if let EffectOp::PublishDir { pending, .. } = effect {
                    let internal_id =
                        pending
                            .split('/')
                            .nth(1)
                            .ok_or_else(|| Error::StoreCorrupt {
                                detail: format!("请求 {request_id} 的pending owner路径无效"),
                            })?;
                    verify_pending_owner(&self.home, internal_id, request_id, "start_work")?;
                }
            }
        }
        Ok((row, snapshot, checked))
    }

    /// `decide` + 单事务提交 + 快照。`data` 在提交前组装，重放原样返回（GF-15）。
    #[allow(clippy::too_many_arguments)]
    fn commit(
        &self,
        decision: Decision,
        graph: &Graph,
        request_id: &str,
        intent_hash: String,
        expected_revision: Option<u64>,
        effects: Vec<EffectOp>,
        ctx: &Context,
        cmd: &Command,
        lock: &crate::home::HomeLock,
    ) -> Result<Response> {
        let revision = expected_revision.map_or(1, |r| r + 1);
        let next = legal_next(&decision.state, graph);
        let data = snapshot_data(&decision, &next);
        let snapshot = Response {
            request_id: request_id.to_string(),
            revision,
            replayed: false,
            reply: decision.reply.clone(),
            data: data.clone(),
            next: next.clone(),
        };
        let reply_json = serde_json::to_string(&snapshot).map_err(|e| Error::StoreCorrupt {
            detail: format!("序列化响应失败：{e}"),
        })?;
        let input = CommitInput {
            work_id: Some(decision.state.work_id.clone()),
            workbook_insert: None,
            workbook_delete: None,
            workbook_in_use_check: None,
            expected_revision,
            state: Some(decision.state.clone()),
            request_id: request_id.to_string(),
            intent_hash,
            reply_json,
            effects_json: encode_effects(&effects),
            principal: ctx.principal.clone(),
            command_json: audit_json(cmd)?,
            at: ctx.now.clone(),
        };
        match self.store.commit(input)? {
            CommitOutcome::Committed { revision: rev } => {
                // 崩溃窗口：COMMIT 之后、效果发布之前（存储合同 §3 第四行）。
                crate::failpoint::maybe_exit("after_commit_before_effects");
                Ok(Response {
                    request_id: request_id.to_string(),
                    revision: rev,
                    replayed: false,
                    reply: decision.reply,
                    data,
                    next,
                })
            }
            CommitOutcome::Replayed { reply_json, .. } => {
                self.replay(request_id.to_string(), reply_json, lock)
            }
        }
    }

    /// 重放：返回原快照（`replayed = true`），先完成未发布效果；历史 `next` 是历史
    /// 事实，续接一律查当前状态。
    fn replay(
        &self,
        request_id: String,
        reply_json: String,
        lock: &crate::home::HomeLock,
    ) -> Result<Response> {
        let (row, mut resp, ops) = self.load_checked_request(&request_id)?;
        if row.reply_json != reply_json {
            return Err(Error::StoreCorrupt {
                detail: format!("请求 {request_id} 的历史响应与requests记录不一致"),
            });
        }
        resp.request_id = request_id.clone();
        resp.replayed = true;
        // 完成未发布效果（若有）；已完成的只核对历史文件。
        if !row.published {
            if let Err(e) = execute(&self.home, lock, &ops, true) {
                return Err(Error::EffectPending {
                    committed: true,
                    request_id,
                    pending_request_id: None,
                    detail: e.to_string(),
                    original: Some(reply_json),
                });
            }
            self.refresh_cards_of(&ops, lock)?;
            self.store.mark_published(&request_id)?;
        } else {
            execute(&self.home, lock, &ops, false)?;
        }
        Ok(resp)
    }
}

fn validate_command_owner(
    request_id: &str,
    work_id: &WorkId,
    state: &WorkState,
    graph: &Graph,
    command: &Command,
    snapshot: &Response,
) -> Result<()> {
    let valid = match (command, &snapshot.reply) {
        (
            Command::Start {
                work_id: command_work,
                name,
                workbook,
                flow,
                work_dir,
                inputs,
            },
            Reply::Started {
                work_id: reply_work,
                work_dir: reply_dir,
                requires,
            },
        ) => {
            snapshot_data_has_exact_fields(
                &snapshot.data,
                &[
                    "work_id", "name", "workbook", "flow", "work_dir", "requires",
                ],
            ) && command_work == work_id
                && reply_work == work_id
                && name == &state.name
                && workbook == &state.workbook
                && flow == &state.flow
                && work_dir == &state.work_dir
                && inputs == &state.inputs
                && reply_dir == &state.work_dir
                && snapshot
                    .data
                    .get("work_id")
                    .and_then(serde_json::Value::as_str)
                    == Some(work_id.as_str())
                && snapshot
                    .data
                    .get("work_dir")
                    .and_then(serde_json::Value::as_str)
                    == Some(state.work_dir.as_str())
                && snapshot
                    .data
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    == Some(state.name.as_str())
                && snapshot
                    .data
                    .get("flow")
                    .and_then(serde_json::Value::as_str)
                    == Some(state.flow.as_str())
                && snapshot.data.get("workbook")
                    == Some(&serde_json::json!({
                        "id": state.workbook.id.as_str(),
                        "version": state.workbook.version.as_str(),
                        "digest": state.workbook.digest.as_str(),
                    }))
                && snapshot.data.get("requires") == serde_json::to_value(requires).ok().as_ref()
        }
        (
            Command::BeginAttempt { node, .. },
            Reply::AttemptBegun {
                attempt,
                brief_path,
                output_dir,
                inputs,
                outputs,
                requires,
            },
        ) => {
            let Some(record) = state.attempt(attempt) else {
                return Err(Error::StoreCorrupt {
                    detail: format!("请求 {request_id} 的snapshot引用未知Attempt {attempt}"),
                });
            };
            let expected_outputs = sheltie_core::work::output_paths_for(state, graph, attempt)
                .map_err(|error| Error::StoreCorrupt {
                    detail: format!("Work {work_id} 的冻结输出定义无效：{error}"),
                })?;
            let expected_inputs = record
                .inputs
                .iter()
                .map(|(name, reference)| {
                    (
                        name.clone(),
                        reference.as_ref().map(|item| item.path.clone()),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let expected_inputs_data = serde_json::to_value(inputs).ok();
            let expected_outputs_data = serde_json::to_value(outputs).ok();
            let expected_requires_data = serde_json::to_value(requires).ok();
            let attempt_text = attempt.to_string();
            snapshot_data_has_exact_fields(
                &snapshot.data,
                &[
                    "attempt",
                    "node",
                    "occurrence",
                    "retry",
                    "brief_path",
                    "output_dir",
                    "inputs",
                    "outputs",
                    "requires",
                ],
            ) && record.id.node == *node
                && brief_path == &state.attempt_dir(attempt).join_segment("brief.md")
                && output_dir
                    == &sheltie_core::work::layout::outputs_dir(&state.attempt_dir(attempt))
                && inputs == &expected_inputs
                && outputs == &expected_outputs
                && snapshot
                    .data
                    .get("attempt")
                    .and_then(serde_json::Value::as_str)
                    == Some(attempt_text.as_str())
                && snapshot
                    .data
                    .get("node")
                    .and_then(serde_json::Value::as_str)
                    == Some(node.as_str())
                && snapshot
                    .data
                    .get("occurrence")
                    .and_then(serde_json::Value::as_u64)
                    == Some(u64::from(attempt.occurrence))
                && snapshot
                    .data
                    .get("retry")
                    .and_then(serde_json::Value::as_u64)
                    == Some(u64::from(attempt.retry))
                && snapshot
                    .data
                    .get("brief_path")
                    .and_then(serde_json::Value::as_str)
                    == Some(brief_path.as_str())
                && snapshot
                    .data
                    .get("output_dir")
                    .and_then(serde_json::Value::as_str)
                    == Some(output_dir.as_str())
                && snapshot.data.get("inputs") == expected_inputs_data.as_ref()
                && snapshot.data.get("outputs") == expected_outputs_data.as_ref()
                && snapshot.data.get("requires") == expected_requires_data.as_ref()
        }
        (
            Command::SubmitAttempt { attempt, .. },
            Reply::AttemptSubmitted {
                attempt: reply_attempt,
                outputs,
            },
        ) => {
            let Some(record) = state.attempt(attempt) else {
                return Err(Error::StoreCorrupt {
                    detail: format!("请求 {request_id} 的snapshot引用未知Attempt {attempt}"),
                });
            };
            let attempt_text = attempt.to_string();
            let output_data = serde_json::to_value(outputs).ok();
            snapshot_data_has_exact_fields(&snapshot.data, &["attempt", "outputs", "work_status"])
                && reply_attempt == attempt
                && record.status == sheltie_core::work::AttemptStatus::Succeeded
                && outputs == &record.outputs
                && snapshot
                    .data
                    .get("attempt")
                    .and_then(serde_json::Value::as_str)
                    == Some(attempt_text.as_str())
                && snapshot.data.get("outputs") == output_data.as_ref()
                && snapshot
                    .data
                    .get("work_status")
                    .cloned()
                    .is_some_and(|value| serde_json::from_value::<WorkStatus>(value).is_ok())
        }
        (
            Command::FailAttempt { attempt, .. },
            Reply::AttemptFailed {
                attempt: reply_attempt,
            },
        ) => state.attempt(attempt).is_some_and(|record| {
            let attempt_text = attempt.to_string();
            snapshot_data_has_exact_fields(&snapshot.data, &["attempt", "work_status"])
                && reply_attempt == attempt
                && record.status == sheltie_core::work::AttemptStatus::Failed
                && snapshot
                    .data
                    .get("attempt")
                    .and_then(serde_json::Value::as_str)
                    == Some(attempt_text.as_str())
                && snapshot
                    .data
                    .get("work_status")
                    .cloned()
                    .is_some_and(|value| serde_json::from_value::<WorkStatus>(value).is_ok())
        }),
        (
            Command::ApproveGate { node },
            Reply::GateApproved {
                node: reply_node,
                occurrence,
            },
        ) => state
            .approvals
            .iter()
            .find(|approval| approval.node == *node && approval.occurrence == *occurrence)
            .is_some_and(|approval| {
                snapshot_data_has_exact_fields(
                    &snapshot.data,
                    &["node", "occurrence", "by", "at", "work_status"],
                ) && reply_node == node
                    && snapshot
                        .data
                        .get("node")
                        .and_then(serde_json::Value::as_str)
                        == Some(node.as_str())
                    && snapshot
                        .data
                        .get("occurrence")
                        .and_then(serde_json::Value::as_u64)
                        == Some(u64::from(*occurrence))
                    && snapshot.data.get("by").and_then(serde_json::Value::as_str)
                        == Some(approval.by.0.as_str())
                    && snapshot.data.get("at").and_then(serde_json::Value::as_str)
                        == Some(approval.at.as_str())
                    && snapshot
                        .data
                        .get("work_status")
                        .cloned()
                        .is_some_and(|value| serde_json::from_value::<WorkStatus>(value).is_ok())
            }),
        (Command::Cancel, Reply::Cancelled) => {
            snapshot_data_has_exact_fields(&snapshot.data, &["work_id", "work_status"])
                && state.status == WorkStatus::Cancelled
                && snapshot
                    .data
                    .get("work_id")
                    .and_then(serde_json::Value::as_str)
                    == Some(work_id.as_str())
                && snapshot
                    .data
                    .get("work_status")
                    .cloned()
                    .is_some_and(|value| serde_json::from_value::<WorkStatus>(value).is_ok())
        }
        _ => false,
    };
    if !valid {
        return Err(Error::StoreCorrupt {
            detail: format!("请求 {request_id} 的audit命令与Work {work_id}状态不一致"),
        });
    }
    Ok(())
}

fn snapshot_data_has_exact_fields(data: &serde_json::Value, expected: &[&str]) -> bool {
    let Some(object) = data.as_object() else {
        return false;
    };
    object.len() == expected.len() && expected.iter().all(|name| object.contains_key(*name))
}

/// `@file` 的内容读取。意图只记路径；内容是首次执行时的观察结果。
fn materialize_inputs(inputs: &BTreeMap<String, InputValue>) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (key, value) in inputs {
        out.insert(key.clone(), materialize_summary(value)?);
    }
    Ok(out)
}

fn materialize_summary(value: &InputValue) -> Result<String> {
    match value {
        InputValue::Literal { text } => Ok(text.clone()),
        InputValue::AtFile { path } => {
            let f = crate::fsx::ExternalReadFile::open_regular(
                &AbsPath::new(path.clone()).map_err(Error::Core)?,
            )?;
            let bytes = f.read_bounded(crate::fsx::MAX_FILE_BYTES)?;
            String::from_utf8(bytes).map_err(|_| Error::InvalidRequest {
                reason: format!("读不了 {path}：不是 UTF-8"),
            })
        }
    }
}

/// core 的效果转换成持久效果登记（路径相对管理根；write_file 携带精确字节）。
fn core_effects_to_ops(home: &Home, decision: &Decision, _cmd: &Command) -> Result<Vec<EffectOp>> {
    let state = &decision.state;
    let mut ops = Vec::new();
    // begin 的目录骨架：Attempt 目录、engine/、outputs/ 与声明输出的父目录，父先于子。
    if let Reply::AttemptBegun {
        attempt,
        brief_path,
        output_dir,
        outputs,
        ..
    } = &decision.reply
    {
        let _ = brief_path;
        let mut dirs: Vec<String> = Vec::new();
        let push_chain = |abs: &AbsPath, dirs: &mut Vec<String>| -> Result<()> {
            let rel = home.to_rel(abs)?;
            let mut acc = String::new();
            for seg in rel.split('/') {
                if seg.is_empty() {
                    continue;
                }
                acc = if acc.is_empty() {
                    seg.to_string()
                } else {
                    format!("{acc}/{seg}")
                };
                dirs.push(acc.clone());
            }
            Ok(())
        };
        push_chain(output_dir, &mut dirs)?;
        for p in outputs.values() {
            if let Some(parent) = p.as_path().parent() {
                push_chain(
                    &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                    &mut dirs,
                )?;
            }
        }
        // 引擎生成文件（brief.md、engine/stats.json）的父目录也在骨架里。
        for effect in &decision.effects {
            if let Effect::WriteBrief { path, .. } | Effect::WriteFile { path, .. } = effect {
                if let Some(parent) = path.as_path().parent() {
                    push_chain(
                        &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                        &mut dirs,
                    )?;
                }
            }
        }
        dirs.sort();
        dirs.dedup();
        ops.push(EffectOp::PrepareAttempt {
            work_id: state.work_id.as_str().to_string(),
            attempt_id: attempt.to_string(),
            dirs,
        });
    }
    for effect in &decision.effects {
        match effect {
            Effect::WriteBrief { path, content } | Effect::WriteFile { path, content } => {
                ops.push(EffectOp::WriteFile {
                    path: home.to_rel(path)?,
                    sha256: Sha256Hex::of_bytes(content.as_bytes()).as_str().to_string(),
                    content: content.clone(),
                });
            }
            Effect::SealOutputs { .. } => {
                // refs 从提交后的状态取（含精确 ArtifactRef）。
                let attempt = state.attempts.last().ok_or_else(|| Error::StoreCorrupt {
                    detail: "封存效果没有对应 Attempt".to_string(),
                })?;
                let mut refs = Vec::new();
                for r in attempt.outputs.values() {
                    refs.push(RefJson {
                        path: home.to_rel(&r.path)?,
                        sha256: r.sha256.as_str().to_string(),
                        bytes: r.bytes,
                    });
                }
                ops.push(EffectOp::SealOutputs { refs });
            }
            Effect::RefreshStatusCard => {
                ops.push(EffectOp::RefreshStatusCard {
                    work_id: state.work_id.as_str().to_string(),
                });
            }
        }
    }
    Ok(ops)
}

/// 提交时快照的数据载荷（协议 §3 各操作返回；重放逐字段原样）。
fn snapshot_data(decision: &Decision, next: &[NextOp]) -> serde_json::Value {
    let state = &decision.state;
    let _ = next;
    match &decision.reply {
        Reply::Started {
            work_id,
            work_dir,
            requires,
        } => serde_json::json!({
            "work_id": work_id.as_str(),
            "name": state.name.as_str(),
            "workbook": {
                "id": state.workbook.id.as_str(),
                "version": state.workbook.version,
                "digest": state.workbook.digest.as_str(),
            },
            "flow": state.flow.as_str(),
            "work_dir": work_dir.as_str(),
            "requires": requires,
        }),
        Reply::AttemptBegun {
            attempt,
            brief_path,
            output_dir,
            inputs,
            outputs,
            requires,
        } => {
            let inputs: serde_json::Map<String, serde_json::Value> = inputs
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_ref()
                            .map_or(serde_json::Value::Null, |p| serde_json::json!(p.as_str())),
                    )
                })
                .collect();
            let outputs: serde_json::Map<String, serde_json::Value> = outputs
                .iter()
                .map(|(k, p)| (k.clone(), serde_json::json!(p.as_str())))
                .collect();
            serde_json::json!({
                "attempt": attempt.to_string(),
                "node": attempt.node.as_str(),
                "occurrence": attempt.occurrence,
                "retry": attempt.retry,
                "brief_path": brief_path.as_str(),
                "output_dir": output_dir.as_str(),
                "inputs": inputs,
                "outputs": outputs,
                "requires": requires,
            })
        }
        Reply::AttemptSubmitted { attempt, outputs } => {
            let outs: serde_json::Map<String, serde_json::Value> = outputs
                .iter()
                .map(|(k, r)| {
                    (
                        k.clone(),
                        serde_json::json!({
                            "path": r.path.as_str(),
                            "sha256": r.sha256.as_str(),
                            "bytes": r.bytes,
                        }),
                    )
                })
                .collect();
            serde_json::json!({
                "attempt": attempt.to_string(),
                "outputs": outs,
                "work_status": state.status,
            })
        }
        Reply::AttemptFailed { attempt } => serde_json::json!({
            "attempt": attempt.to_string(),
            "work_status": state.status,
        }),
        Reply::GateApproved { node, occurrence } => {
            let approval = state
                .approvals
                .last()
                .cloned()
                .unwrap_or_else(|| unreachable!("批准回复必有批准记录"));
            serde_json::json!({
                "node": node.as_str(),
                "occurrence": occurrence,
                "by": approval.by.0,
                "at": approval.at.as_str(),
                "work_status": state.status,
            })
        }
        Reply::Cancelled => serde_json::json!({
            "work_id": state.work_id.as_str(),
            "work_status": state.status,
        }),
    }
}

/// 说明书原文：`File` 从冻结副本读，`Text` 直接取值。
fn instruction_text_of(loaded: &Loaded, node: &NodeId) -> Result<String> {
    let def = loaded.graph.node(node).ok_or_else(|| Error::NotFound {
        what: format!("节点 {node}"),
    })?;
    match def.instruction() {
        sheltie_core::flow::Instruction::Text(t) => Ok(t.clone()),
        sheltie_core::flow::Instruction::File(rel) => loaded
            .instructions
            .get(rel)
            .cloned()
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("冻结副本装入时缺少节点 {node} 的说明书 {rel}"),
            }),
    }
}

fn observe_loaded_input(
    home: &Home,
    loaded: &Loaded,
    node: &NodeId,
    name: &str,
    path: &AbsPath,
) -> Result<Option<ObservedFile>> {
    let declaration = loaded
        .graph
        .node(node)
        .and_then(|definition| {
            definition
                .inputs()
                .iter()
                .find(|input| input.name() == name)
        })
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("冻结Flow缺少节点 {node} 的输入声明 {name}"),
        })?;
    if let sheltie_core::flow::InputSource::Resource { path: relative } = declaration.source() {
        if path != &loaded.state.workbook_dir().join(relative) {
            return Err(Error::StoreCorrupt {
                detail: format!("资源输入 {name} 的路径与冻结Workbook不一致"),
            });
        }
        return loaded
            .resource_files
            .get(relative)
            .cloned()
            .map(Some)
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("冻结副本装入时缺少资源 {relative}"),
            });
    }
    if path.as_path().starts_with(home.root().as_path()) {
        let Some(file) = crate::fsx::open_managed_optional(home, path)? else {
            return Ok(None);
        };
        let (sha256, bytes) = file.sha256_bounded(crate::fsx::MAX_FILE_BYTES)?;
        return Ok(Some(ObservedFile::new(path.clone(), sha256, bytes)));
    }
    observe_optional(path)
}

/// 观察一个声明输出：句柄核对身份后在句柄上算摘要。超过任何输出合同都不可能满足的
/// 32 MiB 硬上限（workbook 合同 §3.2 `max_bytes` 上限）时，在读取全部内容**之前**按
/// `OUTPUT_TOO_LARGE` 拒绝；对声明上限的精确比较仍由 core 在 decide 里做。
fn observe_output(home: &Home, name: &str, path: &AbsPath) -> Result<Option<ObservedFile>> {
    let Some(f) = crate::fsx::open_managed_optional(home, path)? else {
        return Ok(None);
    };
    let len = f.metadata().len();
    if len > crate::fsx::MAX_FILE_BYTES {
        return Err(Error::Core(sheltie_core::Error::OutputTooLarge {
            output: name.to_string(),
            max_bytes: crate::fsx::MAX_FILE_BYTES,
            actual: len,
        }));
    }
    let (sha256, bytes) = f.sha256_bounded(crate::fsx::MAX_FILE_BYTES)?;
    Ok(Some(ObservedFile::new(path.clone(), sha256, bytes)))
}

/// 审计用的 Command JSON：把 `instruction_text` 这类大字段换成长度（CommitInput 的约定）。
fn audit_json(cmd: &Command) -> Result<String> {
    let mut value = serde_json::to_value(cmd).map_err(|e| Error::StoreCorrupt {
        detail: format!("序列化命令失败：{e}"),
    })?;
    if let serde_json::Value::Object(map) = &mut value {
        if let Some(text) = map.get("instruction_text").and_then(|v| v.as_str()) {
            map.insert(
                "instruction_text".to_string(),
                serde_json::Value::String(format!("<{} 字节>", text.len())),
            );
        }
        if let Some(text) = map.get("summary").and_then(|v| v.as_str()) {
            map.insert(
                "summary".to_string(),
                serde_json::Value::String(format!("<{} 字节>", text.len())),
            );
        }
        if let Some(text) = map.get("reason").and_then(|v| v.as_str()) {
            map.insert(
                "reason".to_string(),
                serde_json::Value::String(format!("<{} 字节>", text.len())),
            );
        }
    }
    serde_json::to_string(&value).map_err(|e| Error::StoreCorrupt {
        detail: format!("序列化命令失败：{e}"),
    })
}

#[cfg(test)]
mod pending_stage_tests {
    use super::stage_pending;
    use crate::home::Home;
    use sheltie_core::path::AbsPath;

    // Task: C002-T19
    #[test]
    fn remove_stage_persists_owner_and_container_without_payload() {
        let temp = tempfile::tempdir().unwrap();
        let home = Home::at(AbsPath::new(temp.path().to_string_lossy().into_owned()).unwrap());
        let lock = home.acquire_lock().unwrap();
        let payload =
            stage_pending(&home, &lock, "internal-1", "request-1", "remove_workbook").unwrap();
        let container = temp.path().join("pending/internal-1");
        let owner = temp.path().join("pending/internal-1.owner");

        assert!(container.is_dir());
        assert!(!container.join("payload").exists());
        assert_eq!(
            std::fs::read_to_string(owner).unwrap(),
            "{\"format\":\"pending/v1\",\"internal_id\":\"internal-1\",\"request_id\":\"request-1\",\"op\":\"remove_workbook\"}\n"
        );
        assert_eq!(
            payload,
            home.root()
                .join_segment("pending")
                .join_segment("internal-1")
                .join_segment("payload")
        );
    }

    // Task: C002-T19
    #[test]
    fn publish_stage_persists_owner_container_and_payload() {
        let temp = tempfile::tempdir().unwrap();
        let home = Home::at(AbsPath::new(temp.path().to_string_lossy().into_owned()).unwrap());
        let lock = home.acquire_lock().unwrap();
        let payload = stage_pending(&home, &lock, "internal-2", "request-2", "start_work").unwrap();
        assert!(std::path::Path::new(payload.as_str()).is_dir());
    }
}
