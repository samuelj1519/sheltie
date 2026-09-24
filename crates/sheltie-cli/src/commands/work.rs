//! `work start | list | status | cancel`。

use crate::cli::WorkCmd;
use crate::commands::Ctx;
use crate::output::Outcome;

/// `start` 先用 `cli::parse_input_arg` 解析全部 `--input`，再调 `WorkService::start`。
/// `status` 与 `list` 只读打开。`<work>` 先经 `WorkService::resolve_work`。
#[allow(unused_variables)]
pub fn run(ctx: &Ctx, cmd: WorkCmd) -> Outcome {
    todo!("T18")
}
