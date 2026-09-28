//! `work start | list | status | cancel`。

use std::collections::BTreeMap;

use serde_json::json;
use sheltie_core::ids::WorkId;
use sheltie_core::work::Reply;
use sheltie_runtime::{Response, WorkService};

use crate::cli::{WorkCmd, parse_input_arg, parse_workbook_spec};
use crate::commands::Ctx;
use crate::output::{self, Outcome};

/// `start` 先用 `cli::parse_input_arg` 解析全部 `--input`，再调 `WorkService::start`。
/// `status`、`stats` 与 `list` 只读打开。`<work>` 先经 `WorkService::resolve_work`。
pub fn run(ctx: &Ctx, cmd: WorkCmd) -> Outcome {
    match cmd {
        WorkCmd::Start(args) => start(ctx, args),
        WorkCmd::List => list(ctx),
        WorkCmd::Status { work } => status(ctx, &work),
        WorkCmd::Stats { work } => stats(ctx, &work),
        WorkCmd::Cancel { work } => cancel(ctx, &work),
    }
}

/// 打开存储并建服务。只读或读写由调用方定。
pub(crate) fn service(ctx: &Ctx) -> WorkService {
    WorkService::new(ctx.home.clone())
}

/// `<work>` 前缀解析；零个或多个匹配都是错误Outcome。
pub(crate) fn resolve(
    svc: &WorkService,
    work: &str,
    request_id: Option<&str>,
) -> Result<WorkId, Outcome> {
    svc.resolve_work_for_request(work, request_id)
        .map_err(|e| crate::error_map::to_outcome(&e))
}

/// `work start`（协议 §3 第 8 步的返回）。
fn start(ctx: &Ctx, args: crate::cli::StartArgs) -> Outcome {
    // 参数解析错误先于任何存储访问，退出码 2。
    let mut inputs = BTreeMap::new();
    for raw in &args.inputs {
        match parse_input_arg(raw) {
            Ok((k, v)) => {
                inputs.insert(k, v);
            }
            Err(m) => return output::param_error(m),
        }
    }
    let (workbook_id, version) = match parse_workbook_spec(&args.workbook) {
        Ok(v) => v,
        Err(m) => return output::param_error(m),
    };
    // 只读打开做预检（GF-30）：新管理根连 store.db 都没有，说明没有任何已装
    // Workbook，按 NOT_FOUND 拒绝，不为失败的 start 建库；写路径由 runtime 重开读写库。
    let svc = WorkService::new(ctx.home.clone());
    let rt_args = sheltie_runtime::StartArgs {
        workbook_id,
        version,
        flow: args.flow,
        name: args.name,
        inputs,
    };
    let resp = match svc.start(rt_args, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    // 响应字段全部来自提交时快照（cli-result/v2）：CLI 不再回读 Store 拼数据（O04）。
    let work_id = match &resp.reply {
        Reply::Started { work_id, .. } => work_id.clone(),
        other => return reply_mismatch("Started", other),
    };
    let mut data = resp.data.clone();
    if let serde_json::Value::Object(map) = &mut data {
        map.insert("replayed".to_string(), json!(resp.replayed));
    }
    let text = next_lines(format!("Work {work_id} 已创建\n"), &resp, &work_id);
    output::ok_work(
        text,
        Some(resp.request_id),
        Some(resp.revision),
        data,
        &resp.next,
        work_id.as_str(),
    )
}

/// `work list`。
fn list(ctx: &Ctx) -> Outcome {
    let svc = service(ctx);
    let rows = match svc.list() {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let mut text = String::new();
    let mut data = Vec::with_capacity(rows.len());
    for row in &rows {
        text.push_str(&format!(
            "{}  {}  {}  {}\n",
            row.work_id, row.name, row.status, row.current
        ));
        data.push(json!({
            "work_id": row.work_id.as_str(),
            "name": row.name,
            "status": row.status,
            "current": row.current,
            "updated_at": row.updated_at,
        }));
    }
    output::ok(text, None, None, data, Vec::new())
}

/// `work status`：文本模式直接打状态卡。
fn status(ctx: &Ctx, work: &str) -> Outcome {
    let svc = service(ctx);
    let wid = match resolve(&svc, work, None) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let (text, card) = match svc.status(&wid) {
        Ok(t) => t,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    // next 已由 core 装配成协议形状，与 data.next 同源同形（O13）。
    output::ok_work_next(text, None, None, json!(card), card.next.clone())
}

/// `work stats`：事实视图。封装层的 `next` 与状态卡同源，再读一次状态卡取。
fn stats(ctx: &Ctx, work: &str) -> Outcome {
    let svc = service(ctx);
    let wid = match resolve(&svc, work, None) {
        Ok(w) => w,
        Err(out) => return out,
    };
    // stats 与 next 用同一次加载的事实视图（GF-29）；next 与状态卡同源同形。
    let (text, stats) = match svc.stats(&wid) {
        Ok(t) => t,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let ops = match svc.status(&wid) {
        Ok((_, card)) => card.next,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    output::ok_work_next(text, None, None, json!(stats), ops)
}

/// `work cancel`。
fn cancel(ctx: &Ctx, work: &str) -> Outcome {
    let svc = service(ctx);
    let wid = match resolve(&svc, work, ctx.request_id.as_deref()) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.cancel(&wid, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let mut text = format!("已取消 {wid}\n");
    if resp.next.is_empty() {
        text.push_str("Work 已结束，没有下一步。\n");
    }
    let mut data = resp.data.clone();
    if let serde_json::Value::Object(map) = &mut data {
        map.insert("replayed".to_string(), json!(resp.replayed));
    }
    output::ok_work(
        text,
        Some(resp.request_id),
        Some(resp.revision),
        data,
        &resp.next,
        wid.as_str(),
    )
}

// ── attempt 与 gate 组共用的渲染 ───────────────────────────────

/// 文本模式的「下一步」段。写操作的文本都以它收尾。
pub(crate) fn next_lines(head: String, resp: &Response, work: &WorkId) -> String {
    let mut text = head;
    if resp.next.is_empty() {
        text.push_str("没有下一步（Work 已结束）。\n");
        return text;
    }
    text.push_str("下一步：\n");
    for op in &resp.next {
        text.push_str(&format!("- {}\n", op.to_command_line(work)));
    }
    text
}

/// runtime 保证 reply 与命令对应；对不上说明两端不一致。
pub(crate) fn reply_mismatch(expected: &str, got: &Reply) -> Outcome {
    crate::output::err(
        sheltie_core::ErrorCode::StoreCorrupt,
        format!("响应与命令不匹配（期望 {expected}，实际 {got:?}）"),
        None,
        Vec::new(),
    )
}
