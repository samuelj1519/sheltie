//! Work 服务：每个写操作走「加载 → 观察 → 决定 → 提交 → 效果」（架构 §4）。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{FlowDef, Graph};
use sheltie_core::ids::{AttemptId, NodeId, WorkId, WorkName};
use sheltie_core::path::AbsPath;
use sheltie_core::work::{
    Command, Context, Decision, Effect, NextOp, Reply, StatsJson, StatusCardJson, WorkState,
    WorkStatus, WorkbookRef, decide, legal_next, render_brief, render_stats, render_stats_json,
    render_status_card, status_card_json,
};
use sheltie_core::workbook::parse_manifest;

use crate::error::{Error, Result};
use crate::home::Home;
use crate::observe::{build_resource_index, now, observe_optional, principal};
use crate::store::{CommitInput, CommitOutcome, Store};
use crate::workbook_repo::{LoadedWorkbook, WorkbookRepo, set_tree_readonly};

/// `work start` 的参数。
#[derive(Debug, Clone)]
pub struct StartArgs {
    pub workbook_id: String,
    pub version: Option<String>,
    pub flow: String,
    pub name: Option<String>,
    /// 起始输入的键与值（`@file` 已由 CLI 读成内容）。
    pub inputs: BTreeMap<String, String>,
}

/// 写操作的统一响应，对应协议 §5 的响应封装。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub request_id: String,
    pub revision: u64,
    pub replayed: bool,
    pub reply: Reply,
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
}

impl WorkService {
    pub fn new(home: Home, store: Store) -> Self {
        Self { home, store }
    }

    /// `work start`。先做无副作用预检（GF-30）：名字规范化、Workbook/Flow 存在性、
    /// 起始输入键集合的确定性拒绝都发生在当日序号分配与任何目录物化之前；失败后补齐
    /// 条件用同一参数重试即成功，不烧号。重放查重在装入 Workbook 之前（协议 work start
    /// 第 2 步）。预检只读；通过后才以读写库进入写路径——合法 start 要求 Workbook 已装，
    /// store.db 已由 `workbook add` 建好，这里不会为新请求建库。
    pub fn start(&self, args: StartArgs, request_id: Option<String>) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        // 名字规范化属于意图构造（协议 work start 第 1 步），先于一切读取；
        // 规范化值幂等，写路径按同一参数重算。
        WorkName::normalize(args.name.as_deref().unwrap_or(&args.flow))?;
        // 载荷指纹：同一请求重放必须逐字节相同（BTreeMap 保证键序稳定）。
        let payload = serde_json::json!({
            "workbook": args.workbook_id,
            "version": args.version,
            "flow": args.flow,
            "name": args.name,
            "inputs": args.inputs,
        });
        let payload_hash = hash_json(&payload)?;
        // 重放预检在装入 Workbook 与分配序号之前（存储合同 §7.1），重放不烧号。
        if let Some((hash, reply_json)) = self.store.lookup_request(&request_id)? {
            if hash != payload_hash {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return self.replay(None, request_id, reply_json);
        }
        let wb = self
            .repo()
            .load(&args.workbook_id, args.version.as_deref())?;
        let flow = wb.flow(&args.flow).ok_or_else(|| Error::NotFound {
            what: format!("Flow {}", args.flow),
        })?;
        // 起始输入键与 `workbook show` 同源（GF-30）；缺/多都在烧号之前拒绝。
        sheltie_core::work::validate_start_inputs(&flow.1, args.inputs.keys())?;
        let ctx = Context {
            now: now(),
            principal: principal(),
        };
        // 写路径：预检全部通过才以读写重新打开（调用方给 start 的句柄是只读的）。
        let store = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadWrite)?;
        WorkService::new(self.home.clone(), store).start_write(
            args,
            &wb,
            flow,
            ctx,
            request_id,
            payload_hash,
        )
    }

    /// 预检通过后的写路径：分配序号、物化冻结副本与起始输入、提交。
    fn start_write(
        &self,
        args: StartArgs,
        wb: &LoadedWorkbook,
        flow: &(FlowDef, Graph),
        ctx: Context,
        request_id: String,
        payload_hash: String,
    ) -> Result<Response> {
        let name = WorkName::normalize(args.name.as_deref().unwrap_or(&args.flow))?;
        let day = ctx.now.day().to_string();
        let seq = self.store.allocate_seq(&day)?;
        let work_id = WorkId::new(&day, seq, &name)?;
        let work_dir = self.home.work_dir(&work_id);
        // 冻结副本：本 Work 之后只读它（存储合同 §5.1）。
        std::fs::create_dir_all(work_dir.as_path()).map_err(|e| Error::io(work_dir.as_str(), e))?;
        let frozen = work_dir.join_segment("workbook");
        WorkbookRepo::copy_confined(&wb.dir, &frozen)?;
        set_tree_readonly(&frozen)?;
        // 起始输入物化成文件并记 ArtifactRef。
        let inputs_dir = work_dir.join_segment("inputs");
        std::fs::create_dir_all(inputs_dir.as_path())
            .map_err(|e| Error::io(inputs_dir.as_str(), e))?;
        let mut inputs = BTreeMap::new();
        for (key, value) in &args.inputs {
            // 键来自外部输入，先经 confine 限制在 inputs/ 之下。
            let path = Home::confine(&inputs_dir, key)?;
            std::fs::write(path.as_path(), value.as_bytes())
                .map_err(|e| Error::io(path.as_str(), e))?;
            inputs.insert(
                key.clone(),
                sheltie_core::work::ArtifactRef {
                    sha256: Sha256Hex::of_bytes(value.as_bytes()),
                    bytes: value.len() as u64,
                    path,
                },
            );
        }
        let cmd = Command::Start {
            work_id: work_id.clone(),
            name,
            workbook: WorkbookRef {
                id: wb.manifest.id().clone(),
                version: wb.manifest.version().to_string(),
                digest: wb.digest.clone(),
            },
            flow: flow.0.id().clone(),
            work_dir,
            inputs,
        };
        self.commit_one(None, &flow.1, &cmd, &ctx, request_id, payload_hash)
    }

    pub fn begin(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        self.run_command(
            work,
            &|loaded| {
                let paths =
                    sheltie_core::work::input_paths_for(&loaded.state, &loaded.graph, node)?;
                let mut observed = BTreeMap::new();
                for (name, path) in paths {
                    // 缺文件记 None，由 core 按合同的 required 决定拒绝还是留空。
                    let obs = match path {
                        Some(p) => observe_optional(&p)?,
                        None => None,
                    };
                    observed.insert(name, obs);
                }
                Ok(Command::BeginAttempt {
                    node: node.clone(),
                    observed_inputs: observed,
                    instruction_text: instruction_text_of(&loaded.state, &loaded.graph, node)?,
                })
            },
            request_id,
        )
    }

    pub fn submit(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        summary: &str,
        request_id: Option<String>,
    ) -> Result<Response> {
        self.run_command(
            work,
            &|loaded| {
                let paths =
                    sheltie_core::work::output_paths_for(&loaded.state, &loaded.graph, attempt)?;
                let mut observed = BTreeMap::new();
                for (name, path) in paths {
                    // 缺文件记 None（OUTPUT_MISSING 由 core 报）；软链等观察错误直接拒绝。
                    observed.insert(name, observe_optional(&path)?);
                }
                Ok(Command::SubmitAttempt {
                    attempt: attempt.clone(),
                    summary: summary.to_string(),
                    observed_outputs: observed,
                })
            },
            request_id,
        )
    }

    pub fn fail(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        reason: &str,
        request_id: Option<String>,
    ) -> Result<Response> {
        self.run_command(
            work,
            &|_| {
                Ok(Command::FailAttempt {
                    attempt: attempt.clone(),
                    reason: reason.to_string(),
                })
            },
            request_id,
        )
    }

    pub fn approve(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        self.run_command(
            work,
            &|_| Ok(Command::ApproveGate { node: node.clone() }),
            request_id,
        )
    }

    pub fn cancel(&self, work: &WorkId, request_id: Option<String>) -> Result<Response> {
        self.run_command(work, &|_| Ok(Command::Cancel), request_id)
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

    /// 从冻结副本加载状态与图。副本缺失或摘要不符报 `StoreCorrupt`。
    fn load(&self, work: &WorkId) -> Result<Loaded> {
        let row = self.store.load_work(work)?;
        let frozen = row.state.workbook_dir();
        if !frozen.as_path().exists() {
            return Err(Error::StoreCorrupt {
                detail: format!("Work {work} 的冻结副本 {} 缺失", frozen),
            });
        }
        let digest = WorkbookRepo::digest_dir(&frozen).map_err(|e| Error::StoreCorrupt {
            detail: format!("冻结副本 {} 读不了：{e}", frozen),
        })?;
        if digest != row.state.workbook.digest {
            return Err(Error::StoreCorrupt {
                detail: format!("冻结副本 {} 的摘要 {} 与记录不符", frozen, digest.as_str()),
            });
        }
        let graph = self.compile_frozen(&frozen, &row.state)?;
        Ok(Loaded {
            state: row.state,
            revision: row.revision,
            graph,
        })
    }

    /// 从冻结副本解析 manifest 与 Flow，取出本 Work 的图。
    fn compile_frozen(&self, frozen: &AbsPath, state: &WorkState) -> Result<Graph> {
        let manifest_text = std::fs::read_to_string(frozen.join_segment("workbook.toml").as_path())
            .map_err(|e| Error::StoreCorrupt {
                detail: format!("冻结副本的 workbook.toml 读不了：{e}"),
            })?;
        let manifest = parse_manifest(&manifest_text).map_err(|e| Error::StoreCorrupt {
            detail: format!("冻结副本的 workbook.toml 解不开：{e}"),
        })?;
        let res = build_resource_index(frozen).map_err(|e| Error::StoreCorrupt {
            detail: format!("冻结副本读不了：{e}"),
        })?;
        for path in manifest.flows() {
            let text = std::fs::read_to_string(frozen.join(path).as_path()).map_err(|e| {
                Error::StoreCorrupt {
                    detail: format!("冻结副本的 {path} 读不了：{e}"),
                }
            })?;
            let def = sheltie_core::flow::parse_flow(&text).map_err(|e| Error::StoreCorrupt {
                detail: format!("冻结副本的 {path} 解不开：{e}"),
            })?;
            if def.id() == &state.flow {
                return sheltie_core::flow::compile(&def, &manifest, &res).map_err(|e| {
                    Error::StoreCorrupt {
                        detail: format!("冻结副本的图编不过：{e}"),
                    }
                });
            }
        }
        Err(Error::StoreCorrupt {
            detail: format!("冻结副本里没有 Flow {}", state.flow),
        })
    }

    /// 所有写操作的公共流程：
    ///
    /// 1. `load`。
    /// 2. 观察：`BeginAttempt` 用 `input_paths_for` 观察输入并读说明书；`SubmitAttempt` 用 `output_paths_for` 观察输出。
    /// 3. `sheltie_core::work::decide`。
    /// 4. `store.commit`（`REVISION_CONFLICT` 时从第 1 步重来，最多 3 次）。
    /// 5. 效果：写任务书与引擎生成的输入文件（临时文件再 rename）、置只读、重写状态卡。全部幂等。
    ///
    /// 重放（`Replayed`）时不做第 3 步以后，但仍补做效果，保证崩溃后任务书与状态卡齐全。
    fn run_command(
        &self,
        work: &WorkId,
        build: &dyn Fn(&Loaded) -> Result<Command>,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        // 重放预检：decide 在状态已推进后会拒绝同一命令，必须先于 decide 查 requests。
        {
            let loaded = self.load(work)?;
            let cmd = build(&loaded)?;
            let payload_hash = hash_command(&cmd)?;
            if let Some((hash, reply_json)) = self.store.lookup_request(&request_id)? {
                if hash != payload_hash {
                    return Err(Error::RequestConflict {
                        request_id: request_id.clone(),
                    });
                }
                return self.replay(Some(work), request_id, reply_json);
            }
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
            let loaded = self.load(work)?;
            let cmd = build(&loaded)?;
            match self.commit_one(
                Some((&loaded.state, loaded.revision)),
                &loaded.graph,
                &cmd,
                &ctx,
                request_id.clone(),
                hash_command(&cmd)?,
            ) {
                Ok(resp) => return Ok(resp),
                Err(Error::RevisionConflict { expected, actual }) => {
                    last_conflict = Error::RevisionConflict { expected, actual };
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_conflict)
    }

    /// `decide` + `commit` + 效果。`pre` 是「提交前的状态与 revision」；`None` 只出现在
    /// `start`（新 Work，插入 revision 1 的行）。
    fn commit_one(
        &self,
        pre: Option<(&WorkState, u64)>,
        graph: &Graph,
        cmd: &Command,
        ctx: &Context,
        request_id: String,
        payload_hash: String,
    ) -> Result<Response> {
        let decision: Decision = decide(pre.map(|(s, _)| s), graph, cmd, ctx)?;
        let next = legal_next(&decision.state, graph);
        let reply_json = serde_json::to_string(&Response {
            request_id: request_id.clone(),
            revision: pre.map_or(1, |(_, r)| r + 1),
            replayed: false,
            reply: decision.reply.clone(),
            next: next.clone(),
        })
        .map_err(|e| Error::StoreCorrupt {
            detail: format!("序列化响应失败：{e}"),
        })?;
        let input = CommitInput {
            work_id: Some(decision.state.work_id.clone()),
            expected_revision: pre.map(|(_, r)| r),
            state: Some(decision.state.clone()),
            request_id: request_id.clone(),
            payload_hash,
            reply_json,
            principal: ctx.principal.clone(),
            command_json: audit_json(cmd)?,
            at: ctx.now.clone(),
        };
        match self.store.commit(input)? {
            CommitOutcome::Committed { revision } => {
                // 崩溃窗口：COMMIT 之后、效果之前（存储合同 §3 第二行）。
                crate::failpoint::maybe_exit("after_commit_before_effects");
                self.apply_effects(&decision.state, graph, &decision.effects)?;
                Ok(Response {
                    request_id,
                    revision,
                    replayed: false,
                    reply: decision.reply,
                    next,
                })
            }
            CommitOutcome::Replayed { reply_json } => {
                self.replay(Some(&decision.state.work_id), request_id, reply_json)
            }
        }
    }

    /// 重放：返回原响应（`replayed = true`）并补做效果。崩溃可能发生在 `COMMIT` 之后、
    /// 效果之前，任务书与状态卡靠这里补齐（存储合同 §3）。`work_hint` 是调用方已知的
    /// work_id；start 的重放发生在 work_id 分配之前，从 `Reply::Started` 里取。
    fn replay(
        &self,
        work_hint: Option<&WorkId>,
        request_id: String,
        reply_json: String,
    ) -> Result<Response> {
        let mut resp: Response =
            serde_json::from_str(&reply_json).map_err(|e| Error::StoreCorrupt {
                detail: format!("requests 表里的响应解不开：{e}"),
            })?;
        resp.request_id = request_id;
        resp.replayed = true;
        let work = match work_hint.cloned().or_else(|| match &resp.reply {
            Reply::Started { work_id, .. } => Some(work_id.clone()),
            _ => None,
        }) {
            Some(w) => w,
            None => {
                return Err(Error::StoreCorrupt {
                    detail: "重放响应里没有 work_id".to_string(),
                });
            }
        };
        // 能走到重放说明原事务已提交，works 行一定在。
        let loaded = self.load(&work)?;
        self.replay_effects(&loaded, &resp.reply)?;
        Ok(resp)
    }

    /// 效果执行。幂等：文件用「写临时再 rename」，置只读可重复，状态卡整份重写。
    /// 效果失败不回滚状态（存储合同 §3）；这里把 IO 错误往上抛给调用者记录。
    fn apply_effects(&self, state: &WorkState, graph: &Graph, effects: &[Effect]) -> Result<()> {
        for effect in effects {
            match effect {
                Effect::WriteBrief { path, content } | Effect::WriteFile { path, content } => {
                    write_atomic(path, content)?;
                }
                Effect::SealOutputs { paths } => {
                    for p in paths {
                        use std::os::unix::fs::PermissionsExt;
                        // 尽力而为；封存以记录的 sha256 为准，不是只读位。
                        let _ = std::fs::set_permissions(
                            p.as_path(),
                            std::fs::Permissions::from_mode(0o444),
                        );
                    }
                }
                Effect::RefreshStatusCard => {
                    let card = render_status_card(state, graph);
                    write_atomic(&state.status_card_path(), &card)?;
                }
            }
        }
        Ok(())
    }

    /// 重放时的效果补齐：状态已在库里，按回复重建任务书与状态卡。
    fn replay_effects(&self, loaded: &Loaded, reply: &Reply) -> Result<()> {
        if let Reply::AttemptBegun { attempt, .. } = reply {
            let Some(at) = loaded.state.attempt(attempt) else {
                return Ok(());
            };
            let brief = render_brief(
                &loaded.state,
                &loaded.graph,
                at,
                &instruction_text_of(&loaded.state, &loaded.graph, &attempt.node)?,
            );
            write_atomic(
                &loaded.state.attempt_dir(attempt).join_segment("brief.md"),
                &brief,
            )?;
            // engine.stats 的 stats.json：只在文件缺失时重算。口径含本次 Attempt（D-29），
            // 库里的状态就是提交时的状态，重算与提交时逐字节一致。
            let stats_path = loaded.state.attempt_dir(attempt).join_segment("stats.json");
            if at
                .inputs
                .values()
                .any(|r| r.as_ref().is_some_and(|a| a.path == stats_path))
                && !stats_path.as_path().exists()
            {
                let content = serde_json::to_string(&sheltie_core::work::render_stats_json(
                    &loaded.state,
                    &loaded.graph,
                ))
                .map_err(|e| Error::StoreCorrupt {
                    detail: format!("engine.stats 序列化失败：{e}"),
                })?;
                write_atomic(&stats_path, &content)?;
            }
        }
        let card = render_status_card(&loaded.state, &loaded.graph);
        write_atomic(&loaded.state.status_card_path(), &card)
    }
}

/// 说明书原文：`File` 从冻结副本读，`Text` 直接取值。
fn instruction_text_of(state: &WorkState, graph: &Graph, node: &NodeId) -> Result<String> {
    let def = graph.node(node).ok_or_else(|| Error::NotFound {
        what: format!("节点 {node}"),
    })?;
    match def.instruction() {
        sheltie_core::flow::Instruction::Text(t) => Ok(t.clone()),
        sheltie_core::flow::Instruction::File(rel) => {
            let path = state.workbook_dir().join(rel);
            std::fs::read_to_string(path.as_path()).map_err(|e| Error::io(path.as_str(), e))
        }
    }
}

/// 写临时文件再 rename；父目录不存在就先建（Attempt 目录由这里首次创建）。
fn write_atomic(path: &AbsPath, content: &str) -> Result<()> {
    if let Some(parent) = path.as_path().parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(path.as_str(), e))?;
    }
    let tmp = path.as_path().with_extension("tmp-pending");
    std::fs::write(&tmp, content).map_err(|e| Error::io(path.as_str(), e))?;
    std::fs::rename(&tmp, path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
    Ok(())
}

// ── 自由函数与共用小件 ─────────────────────────────────────────

/// start 的载荷指纹。
fn hash_json(value: &serde_json::Value) -> Result<String> {
    serde_json::to_string(value)
        .map(|s| Sha256Hex::of_bytes(s.as_bytes()).as_str().to_string())
        .map_err(|e| Error::StoreCorrupt {
            detail: format!("序列化载荷失败：{e}"),
        })
}

/// 命令的载荷指纹：同一 request_id 换载荷必须在这里露出差别。
fn hash_command(cmd: &Command) -> Result<String> {
    serde_json::to_string(cmd)
        .map(|s| Sha256Hex::of_bytes(s.as_bytes()).as_str().to_string())
        .map_err(|e| Error::StoreCorrupt {
            detail: format!("序列化命令失败：{e}"),
        })
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
    }
    serde_json::to_string(&value).map_err(|e| Error::StoreCorrupt {
        detail: format!("序列化命令失败：{e}"),
    })
}
