//! `gate approve`。

use sheltie_core::ids::NodeId;
use sheltie_core::work::Reply;

use crate::cli::GateCmd;
use crate::commands::Ctx;
use crate::commands::work::{next_lines, reply_mismatch, resolve, service};
use crate::output::{self, Outcome};

/// `gate approve`: record `{ node, occurrence, by, at }`, then determine Work status under the protocol.
/// Approver and time come from the commit-time snapshot (INV-6: system facts, not arguments).
pub fn run(ctx: &Ctx, cmd: GateCmd) -> Outcome {
    let GateCmd::Approve { work, node } = cmd;
    let node = match NodeId::new(&node) {
        Ok(n) => n,
        Err(e) => return crate::error_map::to_outcome(&sheltie_runtime::Error::Core(e)),
    };
    let svc = service(ctx);
    let wid = match resolve(&svc, &work, ctx.request_id.as_deref()) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.approve(&wid, &node, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let (node, occurrence) = match &resp.reply {
        Reply::GateApproved { node, occurrence } => (node.clone(), *occurrence),
        other => return reply_mismatch("GateApproved", other),
    };
    // Data, including by/at and work_status, comes from the commit-time snapshot without rereading Store (cli-result/v2).
    let text = next_lines(
        format!("Approved gate for {node} (occurrence {occurrence})\n"),
        &resp,
        &wid,
    );
    output::ok_response(text, resp, &wid)
}
