//! `sheltie` 命令行。参数解析在 `cli`，每个命令组一个模块，输出与退出码在 `output` 与 `error_map`。
// 测试代码允许 unwrap；库代码不允许（workspace lints）。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod cli;
mod commands;
mod error_map;
mod output;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();
    let code = commands::dispatch(args);
    std::process::exit(code);
}
