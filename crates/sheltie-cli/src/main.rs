//! `sheltie` CLI. `cli` parses arguments; command groups have modules; `output` and `error_map` handle output and exit codes.
// Tests may unwrap; library code may not (workspace lints).
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
