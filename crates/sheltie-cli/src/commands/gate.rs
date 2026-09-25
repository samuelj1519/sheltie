//! `gate approve`。

use serde_json::json;
use sheltie_core::ids::NodeId;
use sheltie_core::work::Reply;
use sheltie_runtime::store::OpenMode;

use crate::cli::GateCmd;
use crate::commands::Ctx;
use crate::commands::attempt::state_of;
use crate::commands::work::{next_lines, reply_mismatch, resolve, service};
use crate::output::Outcome;

/// `gate approve`：记录 `{ node, occurrence, by, at }` 后按协议决定 Work 状态。
pub fn run(ctx: &Ctx, cmd: GateCmd) -> Outcome {
    let GateCmd::Approve { work, node } = cmd;
    let node = match NodeId::new(&node) {
        Ok(n) => n,
        Err(e) => return crate::error_map::to_outcome(&sheltie_runtime::Error::Core(e)),
    };
    let svc = match service(ctx, OpenMode::ReadWrite) {
        Ok(s) => s,
        Err(out) => return out,
    };
    let wid = match resolve(&svc, &work) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.approve(&wid, &node, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let Reply::GateApproved { node, occurrence } = &resp.reply else {
        return reply_mismatch("GateApproved");
    };
    let state = match state_of(ctx, &wid) {
        Ok(s) => s,
        Err(out) => return out,
    };
    // 批准人以库里的记录为准（INV-6：身份是系统事实，不取自参数）。
    let approved_by = state.approvals.last().map(|a| a.by.0.clone());
    let data = json!({
        "node": node.as_str(),
        "occurrence": occurrence,
        "by": approved_by,
        "work_status": state.status,
        "replayed": resp.replayed,
    });
    let text = next_lines(
        format!("已批准 {node} 的门槛（第 {occurrence} 次到达）\n"),
        &resp,
        &wid,
    );
    crate::output::ok_work(
        text,
        Some(resp.request_id),
        Some(resp.revision),
        data,
        &resp.next,
        wid.as_str(),
    )
}
