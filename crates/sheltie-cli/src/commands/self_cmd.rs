//! `self install | update | rollback | uninstall | version`。不打开 `store.db`。

use crate::cli::SelfCmd;
use crate::commands::Ctx;
use crate::output::Outcome;

/// `uninstall --purge` 在文本模式下没有 `--yes` 时读一行 stdin，必须是 `yes`；JSON 模式下必须给 `--yes`。
#[allow(unused_variables)]
pub fn run(ctx: &Ctx, cmd: SelfCmd) -> Outcome {
    todo!("T20")
}
