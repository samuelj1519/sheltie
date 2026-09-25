//! `work start | list | status | cancel`。

use std::collections::BTreeMap;

use serde_json::json;
use sheltie_core::ids::WorkId;
use sheltie_core::work::{Reply, WorkState};
use sheltie_runtime::store::OpenMode;
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
pub(crate) fn service(ctx: &Ctx, mode: OpenMode) -> Result<WorkService, Outcome> {
    let store = ctx
        .store(mode)
        .map_err(|e| crate::error_map::to_outcome(&e))?;
    Ok(WorkService::new(ctx.home.clone(), store))
}

/// `<work>` 前缀解析；零个或多个匹配都是错误Outcome。
pub(crate) fn resolve(svc: &WorkService, work: &str) -> Result<WorkId, Outcome> {
    svc.resolve_work(work)
        .map_err(|e| crate::error_map::to_outcome(&e))
}

/// 只读加载某个 Work 的状态（`name`、`workbook`、`flow`、`status` 不在 reply 里）。
fn load_state(ctx: &Ctx, work: &WorkId) -> Result<WorkState, Outcome> {
    let store = ctx
        .store(OpenMode::ReadOnly)
        .map_err(|e| crate::error_map::to_outcome(&e))?;
    store
        .load_work(work)
        .map(|row| row.state)
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
    let svc = match service(ctx, OpenMode::ReadWrite) {
        Ok(s) => s,
        Err(out) => return out,
    };
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
    let Reply::Started {
        work_id,
        work_dir,
        requires,
    } = &resp.reply
    else {
        return reply_mismatch("Started");
    };
    let state = match load_state(ctx, work_id) {
        Ok(s) => s,
        Err(out) => return out,
    };
    let data = json!({
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
        "replayed": resp.replayed,
    });
    let text = next_lines(format!("Work {work_id} 已创建\n"), &resp, work_id);
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
    let svc = match service(ctx, OpenMode::ReadOnly) {
        Ok(s) => s,
        Err(out) => return out,
    };
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
    let svc = match service(ctx, OpenMode::ReadOnly) {
        Ok(s) => s,
        Err(out) => return out,
    };
    let wid = match resolve(&svc, work) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let (text, card) = match svc.status(&wid) {
        Ok(t) => t,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let next = card.next.clone();
    output::ok_work(text, None, None, json!(card), &next, wid.as_str())
}

/// `work stats`：事实视图。封装层的 `next` 与状态卡同源，再读一次状态卡取。
fn stats(ctx: &Ctx, work: &str) -> Outcome {
    let svc = match service(ctx, OpenMode::ReadOnly) {
        Ok(s) => s,
        Err(out) => return out,
    };
    let wid = match resolve(&svc, work) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let (text, stats) = match svc.stats(&wid) {
        Ok(t) => t,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let ops = match svc.status(&wid) {
        Ok((_, card)) => card.next,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    output::ok_work(text, None, None, json!(stats), &ops, wid.as_str())
}

/// `work cancel`。
fn cancel(ctx: &Ctx, work: &str) -> Outcome {
    let svc = match service(ctx, OpenMode::ReadWrite) {
        Ok(s) => s,
        Err(out) => return out,
    };
    let wid = match resolve(&svc, work) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.cancel(&wid, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let state = match load_state(ctx, &wid) {
        Ok(s) => s,
        Err(out) => return out,
    };
    let mut text = format!("已取消 {wid}\n");
    if resp.next.is_empty() {
        text.push_str("Work 已结束，没有下一步。\n");
    }
    let data = json!({
        "work_id": wid.as_str(),
        "work_status": state.status,
        "replayed": resp.replayed,
    });
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
pub(crate) fn reply_mismatch(expected: &str) -> Outcome {
    crate::output::err(
        sheltie_core::ErrorCode::StoreCorrupt,
        format!("响应与命令不匹配（期望 {expected}）"),
        None,
        Vec::new(),
    )
}
