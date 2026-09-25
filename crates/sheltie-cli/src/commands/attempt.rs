//! `attempt begin | submit | fail`。

use serde_json::json;
use sheltie_core::ids::NodeId;
use sheltie_core::work::Reply;
use sheltie_runtime::store::OpenMode;

use crate::cli::AttemptCmd;
use crate::commands::Ctx;
use crate::commands::work::{next_lines, reply_mismatch, resolve, service};
use crate::output::{self, Outcome};

/// `--summary` 与 `--reason` 经 `cli::read_text_arg`；`--attempt` 经 `AttemptId::parse`。
/// `begin` 已随 T18 一并实现（`work cancel` 的测试要走它验证 `WORK_TERMINAL`，见 plan.md T18 任务卡）；
/// `submit` 与 `fail` 归 T19。
pub fn run(ctx: &Ctx, cmd: AttemptCmd) -> Outcome {
    match cmd {
        AttemptCmd::Begin { work, node } => begin(ctx, &work, &node),
        AttemptCmd::Submit { .. } => todo!("T19"),
        AttemptCmd::Fail { .. } => todo!("T19"),
    }
}

/// `attempt begin`（协议 §3 第 5 步的返回）。
fn begin(ctx: &Ctx, work: &str, node: &str) -> Outcome {
    let node = match NodeId::new(node) {
        Ok(n) => n,
        Err(e) => return crate::error_map::to_outcome(&sheltie_runtime::Error::Core(e)),
    };
    let svc = match service(ctx, OpenMode::ReadWrite) {
        Ok(s) => s,
        Err(out) => return out,
    };
    let wid = match resolve(&svc, work) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.begin(&wid, &node, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let Reply::AttemptBegun {
        attempt,
        brief_path,
        output_dir,
        inputs,
        outputs,
        requires,
    } = &resp.reply
    else {
        return reply_mismatch("AttemptBegun");
    };
    let data = json!({
        "attempt": attempt.to_string(),
        "node": attempt.node.as_str(),
        "occurrence": attempt.occurrence,
        "retry": attempt.retry,
        "brief_path": brief_path.as_str(),
        "output_dir": output_dir.as_str(),
        "inputs": inputs,
        "outputs": outputs,
        "requires": requires,
        "replayed": resp.replayed,
    });
    let text = next_lines(
        format!("已开始 {attempt}\n任务书：{brief_path}\n输出目录：{output_dir}\n"),
        &resp,
        &wid,
    );
    output::ok_work(
        text,
        Some(resp.request_id),
        Some(resp.revision),
        data,
        &resp.next,
        wid.as_str(),
    )
}
