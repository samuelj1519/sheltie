//! Work 服务：每个写操作走「加载 → 观察 → 决定 → 提交 → 效果」（架构 §4）。

use std::collections::BTreeMap;

use serde::Serialize;
use sheltie_core::flow::Graph;
use sheltie_core::ids::{AttemptId, NodeId, WorkId};
use sheltie_core::work::{
    Command, NextOp, Reply, StatsJson, StatusCardJson, WorkState, WorkStatus,
};

use crate::error::Result;
use crate::home::Home;
use crate::store::Store;

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

/// 写操作的统一响应，对应协议 §5 的包络。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

    /// `work start`。比其他写操作多三步前置：查 `requests` 表重放；`allocate_seq` 拼 `work_id`；
    /// 复制冻结副本（存储合同 §5.1）并写起始输入文件。之后走 `run_command`。
    #[allow(unused_variables)]
    pub fn start(&self, args: StartArgs, request_id: Option<String>) -> Result<Response> {
        todo!("T16")
    }

    #[allow(unused_variables)]
    pub fn begin(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        todo!("T16")
    }

    #[allow(unused_variables)]
    pub fn submit(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        summary: &str,
        request_id: Option<String>,
    ) -> Result<Response> {
        todo!("T16")
    }

    #[allow(unused_variables)]
    pub fn fail(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        reason: &str,
        request_id: Option<String>,
    ) -> Result<Response> {
        todo!("T16")
    }

    #[allow(unused_variables)]
    pub fn approve(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        todo!("T16")
    }

    #[allow(unused_variables)]
    pub fn cancel(&self, work: &WorkId, request_id: Option<String>) -> Result<Response> {
        todo!("T16")
    }

    /// 只读：状态卡文本与结构化形式。
    #[allow(unused_variables)]
    pub fn status(&self, work: &WorkId) -> Result<(String, StatusCardJson)> {
        todo!("T16")
    }

    /// 只读：事实视图文本与结构化形式（`render_stats`、`render_stats_json`）。
    #[allow(unused_variables)]
    pub fn stats(&self, work: &WorkId) -> Result<(String, StatsJson)> {
        todo!("T16")
    }

    /// 只读：全部 Work 摘要。
    pub fn list(&self) -> Result<Vec<WorkSummary>> {
        todo!("T16")
    }

    /// 只读：按完整 id 或唯一前缀解析。多个匹配报 `InvalidRequest` 并列出候选。
    #[allow(unused_variables)]
    pub fn resolve_work(&self, prefix: &str) -> Result<WorkId> {
        todo!("T16")
    }

    /// 从冻结副本加载状态与图。副本缺失或摘要不符报 `StoreCorrupt`。
    #[allow(unused_variables)]
    fn load(&self, work: &WorkId) -> Result<Loaded> {
        todo!("T16")
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
    #[allow(unused_variables)]
    fn run_command(
        &self,
        work: &WorkId,
        build: &dyn Fn(&Loaded) -> Result<Command>,
        request_id: Option<String>,
    ) -> Result<Response> {
        crate::failpoint::maybe_exit("after_commit_before_effects");
        todo!("T16")
    }
}
