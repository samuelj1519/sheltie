//! `attempt begin | submit | fail`。

use serde_json::json;
use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_core::work::Reply;

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
        AttemptCmd::Submit {
            work,
            attempt,
            summary,
        } => submit(ctx, &work, &attempt, &summary),
        AttemptCmd::Fail {
            work,
            attempt,
            reason,
        } => fail(ctx, &work, &attempt, &reason),
    }
}

/// `attempt begin`（协议 §3 第 5 步的返回）。
fn begin(ctx: &Ctx, work: &str, node: &str) -> Outcome {
    let node = match NodeId::new(node) {
        Ok(n) => n,
        Err(e) => return crate::error_map::to_outcome(&sheltie_runtime::Error::Core(e)),
    };
    let svc = service(ctx);
    let wid = match resolve(&svc, work, ctx.request_id.as_deref()) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.begin(&wid, &node, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    // 数据来自提交时快照（cli-result/v2），不回读 Store（O04）。
    let (attempt, brief_path, output_dir) = match &resp.reply {
        Reply::AttemptBegun {
            attempt,
            brief_path,
            output_dir,
            ..
        } => (attempt.clone(), brief_path.clone(), output_dir.clone()),
        other => return reply_mismatch("AttemptBegun", other),
    };
    let mut data = resp.data.clone();
    if let serde_json::Value::Object(map) = &mut data {
        map.insert("replayed".to_string(), json!(resp.replayed));
    }
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

/// `attempt submit`（协议 §3 第 6 步的返回）。
fn submit(ctx: &Ctx, work: &str, attempt: &str, summary: &str) -> Outcome {
    let summary = match crate::cli::parse_text_arg(summary) {
        Ok(s) => s,
        Err(m) => return output::param_error(m),
    };
    let attempt = match AttemptId::parse(attempt) {
        Ok(a) => a,
        Err(e) => return crate::error_map::to_outcome(&sheltie_runtime::Error::Core(e)),
    };
    let svc = service(ctx);
    let wid = match resolve(&svc, work, ctx.request_id.as_deref()) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.submit(&wid, &attempt, &summary, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let (attempt, outputs) = match &resp.reply {
        Reply::AttemptSubmitted { attempt, outputs } => (attempt.clone(), outputs.clone()),
        other => return reply_mismatch("AttemptSubmitted", other),
    };
    let mut data = resp.data.clone();
    if let serde_json::Value::Object(map) = &mut data {
        map.insert("replayed".to_string(), json!(resp.replayed));
    }
    let mut text = format!("已提交 {attempt}\n");
    for (name, r) in &outputs {
        text.push_str(&format!(
            "  {name} → {} (sha256 {})\n",
            r.path,
            r.sha256.as_str()
        ));
    }
    let text = next_lines(text, &resp, &wid);
    output::ok_work(
        text,
        Some(resp.request_id),
        Some(resp.revision),
        data,
        &resp.next,
        wid.as_str(),
    )
}

/// `attempt fail`。
fn fail(ctx: &Ctx, work: &str, attempt: &str, reason: &str) -> Outcome {
    let reason = match crate::cli::parse_text_arg(reason) {
        Ok(s) => s,
        Err(m) => return output::param_error(m),
    };
    let attempt = match AttemptId::parse(attempt) {
        Ok(a) => a,
        Err(e) => return crate::error_map::to_outcome(&sheltie_runtime::Error::Core(e)),
    };
    let svc = service(ctx);
    let wid = match resolve(&svc, work, ctx.request_id.as_deref()) {
        Ok(w) => w,
        Err(out) => return out,
    };
    let resp = match svc.fail(&wid, &attempt, &reason, ctx.request_id.clone()) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let attempt = match &resp.reply {
        Reply::AttemptFailed { attempt } => attempt.clone(),
        other => return reply_mismatch("AttemptFailed", other),
    };
    let mut data = resp.data.clone();
    if let serde_json::Value::Object(map) = &mut data {
        map.insert("replayed".to_string(), json!(resp.replayed));
    }
    let text = next_lines(format!("已标记 {attempt} 失败\n"), &resp, &wid);
    output::ok_work(
        text,
        Some(resp.request_id),
        Some(resp.revision),
        data,
        &resp.next,
        wid.as_str(),
    )
}
