//! `attempt begin | submit | fail | replace`。

use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_core::work::Reply;

use crate::cli::AttemptCmd;
use crate::commands::Ctx;
use crate::commands::work::{next_lines, reply_mismatch, resolve, service};
use crate::output::{self, Outcome};

/// Parse `--summary` and `--reason` with `cli::parse_text_arg`; parse `--attempt` with `AttemptId::parse`.
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
        AttemptCmd::Replace {
            work,
            attempt,
            reason,
        } => replace(ctx, &work, &attempt, &reason),
    }
}

/// `attempt begin` (protocol §3, step 5 response).
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
    // Data comes from the commit-time snapshot (cli-result/v4), without rereading Store (O04).
    let (attempt, brief_path, output_dir) = match &resp.reply {
        Reply::AttemptBegun {
            attempt,
            brief_path,
            output_dir,
            ..
        } => (attempt, brief_path, output_dir),
        other => return reply_mismatch("AttemptBegun", other),
    };
    let text = next_lines(
        format!("Started {attempt}\nBrief: {brief_path}\nOutput directory: {output_dir}\n"),
        &resp,
        &wid,
    );
    output::ok_response(text, resp, &wid)
}

/// `attempt submit` (protocol §3, step 6 response).
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
        Reply::AttemptSubmitted { attempt, outputs } => (attempt, outputs),
        other => return reply_mismatch("AttemptSubmitted", other),
    };
    let mut text = format!("Submitted {attempt}\n");
    for (name, r) in outputs {
        text.push_str(&format!(
            "  {name} → {} (sha256 {})\n",
            r.path,
            r.sha256.as_str()
        ));
    }
    let text = next_lines(text, &resp, &wid);
    output::ok_response(text, resp, &wid)
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
        Reply::AttemptFailed { attempt } => attempt,
        other => return reply_mismatch("AttemptFailed", other),
    };
    let text = next_lines(format!("Marked {attempt} as failed\n"), &resp, &wid);
    output::ok_response(text, resp, &wid)
}

fn replace(ctx: &Ctx, work: &str, attempt: &str, reason: &str) -> Outcome {
    let reason = match crate::cli::parse_text_arg(reason) {
        Ok(value) => value,
        Err(message) => return output::param_error(message),
    };
    let attempt = match AttemptId::parse(attempt) {
        Ok(value) => value,
        Err(error) => return crate::error_map::to_outcome(&sheltie_runtime::Error::Core(error)),
    };
    let svc = service(ctx);
    let wid = match resolve(&svc, work, ctx.request_id.as_deref()) {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let resp = match svc.replace(&wid, &attempt, &reason, ctx.request_id.clone()) {
        Ok(value) => value,
        Err(error) => return crate::error_map::to_outcome(&error),
    };
    let Reply::AttemptReplaced {
        replaced_attempt,
        attempt,
        brief_path,
        output_dir,
        ..
    } = &resp.reply
    else {
        return reply_mismatch("AttemptReplaced", &resp.reply);
    };
    let text = next_lines(
        format!(
            "Replaced {replaced_attempt}; new Attempt: {attempt}\nBrief: {brief_path}\nOutput directory: {output_dir}\n"
        ),
        &resp,
        &wid,
    );
    output::ok_response(text, resp, &wid)
}
