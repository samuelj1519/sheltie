//! `gate approve`。

use serde_json::json;
use sheltie_core::ids::NodeId;
use sheltie_core::work::Reply;
use sheltie_runtime::store::OpenMode;

use crate::cli::GateCmd;
use crate::commands::Ctx;
use crate::commands::work::{next_lines, reply_mismatch, resolve, service};
use crate::output::Outcome;

/// `gate approve`：记录 `{ node, occurrence, by, at }` 后按协议决定 Work 状态。
/// 批准人与时间来自提交时快照（INV-6：身份与时间是系统事实，不取自参数）。
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
    let (node, occurrence) = match &resp.reply {
        Reply::GateApproved { node, occurrence } => (node.clone(), *occurrence),
        other => return reply_mismatch("GateApproved", other),
    };
    // 数据（含 by/at 与 work_status）来自提交时快照，不回读 Store（cli-result/v2）。
    let mut data = resp.data.clone();
    if let serde_json::Value::Object(map) = &mut data {
        map.insert("replayed".to_string(), json!(resp.replayed));
    }
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
