//! `work start | list | status | result | stats | cancel`。

use std::collections::BTreeMap;

use serde_json::json;
use sheltie_core::ids::WorkId;
use sheltie_core::work::Reply;
use sheltie_runtime::{Response, WorkService};

use crate::cli::{WorkCmd, parse_input_arg, parse_workbook_spec};
use crate::commands::Ctx;
use crate::output::{self, Outcome};

pub(crate) fn result_artifact(ctx: &Ctx, work: &str, key: &str, revision: u64) -> i32 {
    let work = match sheltie_core::ids::WorkId::parse(work) {
        Ok(work) => work,
        Err(error) => return output::raw_error(&sheltie_runtime::Error::Core(error)),
    };
    let svc = service(ctx);
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    match svc.write_result_artifact(&work, key, revision, &mut writer) {
        Ok(()) => 0,
        Err(error) => output::raw_error(&error),
    }
}

/// `start` parses all `--input` values with `cli::parse_input_arg`, then calls `WorkService::start`.
/// `status`, `result`, `stats`, and `list` open read-only. Resolve `<work>` with `WorkService::resolve_work`.
pub fn run(ctx: &Ctx, cmd: WorkCmd) -> Outcome {
    match cmd {
        WorkCmd::Start(args) => start(ctx, args),
        WorkCmd::List => list(ctx),
        WorkCmd::Status { work } => status(ctx, &work),
        WorkCmd::Result { work, .. } => result(ctx, &work),
        WorkCmd::Stats { work } => stats(ctx, &work),
        WorkCmd::Cancel { work } => cancel(ctx, &work),
    }
}

/// Open storage and construct the service; the caller chooses read-only or read-write access.
pub(crate) fn service(ctx: &Ctx) -> WorkService {
    WorkService::new(ctx.home.clone())
}

/// Resolve the `<work>` prefix; zero or multiple matches produce an error Outcome.
pub(crate) fn resolve(
    svc: &WorkService,
    work: &str,
    request_id: Option<&str>,
) -> Result<WorkId, Outcome> {
    svc.resolve_work_for_request(work, request_id)
        .map_err(|e| crate::error_map::to_outcome(&e))
}

/// `work start` (protocol §3, step 8 response).
fn start(ctx: &Ctx, args: crate::cli::StartArgs) -> Outcome {
    // Parameter errors precede all storage access and return exit code 2.
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
    // Read-only preflight (GF-30): a new management root has no store.db or installed
    // Workbook. Return NOT_FOUND without creating storage; runtime reopens read-write for writes.
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
    // All response fields come from the commit-time snapshot (cli-result/v4); CLI does not reread Store (O04).
    let work_id = match &resp.reply {
        Reply::Started { work_id, .. } => work_id.clone(),
        other => return reply_mismatch("Started", other),
    };
    let text = next_lines(format!("Work {work_id} created\n"), &resp, &work_id);
    output::ok_response(text, resp, &work_id)
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

/// `work status`: print the status card in text mode.
fn status(ctx: &Ctx, work: &str) -> Outcome {
    let svc = service(ctx);
    let wid = match resolve(&svc, work, None) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let (text, view) = match svc.status_read(&wid) {
        Ok(t) => t,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    // Core assembles next in protocol form, from the same source and shape as data.next (O13).
    output::ok(text, None, None, json!(view), view.card.next.clone())
}

fn result(ctx: &Ctx, work: &str) -> Outcome {
    let svc = service(ctx);
    let wid = match resolve(&svc, work, None) {
        Ok(work) => work,
        Err(out) => return out,
    };
    let (view, operations) = match svc.result(&wid) {
        Ok(result) => result,
        Err(error) => return crate::error_map::to_outcome(&error),
    };
    let next = operations
        .iter()
        .map(|operation| sheltie_core::work::render::next_item_json(&view.work_id, operation))
        .collect();
    let text = sheltie_core::work::result::render_result(&view);
    output::ok(text, None, None, json!(view), next)
}

/// `work stats`: runtime returns facts and next from one load.
fn stats(ctx: &Ctx, work: &str) -> Outcome {
    let svc = service(ctx);
    let wid = match resolve(&svc, work, None) {
        Ok(w) => w,
        Err(out) => return out,
    };
    // stats and next share one loaded fact view (GF-29); next matches the status card's source and shape.
    let (text, stats, ops) = match svc.stats(&wid) {
        Ok(t) => t,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    output::ok(text, None, None, json!(stats), ops)
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
    let mut text = format!("Cancelled {wid}\n");
    if resp.next.is_empty() {
        text.push_str("Work has ended; there are no next actions.\n");
    }
    output::ok_response(text, resp, &wid)
}

// ── Rendering shared by attempt and gate commands ───────────────────────────────

/// The text-mode next-actions section ends every write response.
pub(crate) fn next_lines(head: String, resp: &Response, work: &WorkId) -> String {
    let mut text = head;
    if resp.next.is_empty() {
        text.push_str("No next actions (Work has ended).\n");
        return text;
    }
    text.push_str("Next actions:\n");
    for op in &resp.next {
        text.push_str(&format!("- {}\n", op.to_command_line(work)));
    }
    text
}

/// Runtime guarantees the reply matches the command; a mismatch indicates an interface inconsistency.
pub(crate) fn reply_mismatch(expected: &str, got: &Reply) -> Outcome {
    crate::output::err(
        sheltie_core::ErrorCode::StoreCorrupt,
        format!("Response does not match command (expected {expected}, got {got:?})"),
        None,
    )
}
