//! `attempt begin | submit | fail`。

use crate::cli::AttemptCmd;
use crate::commands::Ctx;
use crate::output::Outcome;

/// `--summary` 与 `--reason` 经 `cli::read_text_arg`；`--attempt` 经 `AttemptId::parse`。
#[allow(unused_variables)]
pub fn run(ctx: &Ctx, cmd: AttemptCmd) -> Outcome {
    todo!("T19")
}
