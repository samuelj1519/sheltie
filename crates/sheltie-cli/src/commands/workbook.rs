//! `workbook add | list | show | remove | verify`。

use crate::cli::WorkbookCmd;
use crate::commands::Ctx;
use crate::output::Outcome;

/// 分派到五个子命令。每个子命令：打开存储、建 `WorkbookRepo`、调用、渲染文本、包成 `Outcome`。
/// 错误经 `error_map::to_outcome`。
#[allow(unused_variables)]
pub fn run(ctx: &Ctx, cmd: WorkbookCmd) -> Outcome {
    todo!("T17")
}
