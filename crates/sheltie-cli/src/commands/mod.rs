//! 命令分派。每个命令组一个文件，函数签名由骨架定死。

pub mod attempt;
pub mod gate;
pub mod self_cmd;
pub mod work;
pub mod workbook;

use sheltie_runtime::store::OpenMode;
use sheltie_runtime::{Home, Store};

use crate::cli::{Cli, Group};
use crate::output::{self, Outcome};

/// 一次调用共享的东西。
pub struct Ctx {
    pub home: Home,
    pub json: bool,
    pub request_id: Option<String>,
}

impl Ctx {
    /// 按只读或读写打开存储。
    pub fn store(&self, mode: OpenMode) -> sheltie_runtime::Result<Store> {
        Store::open(&self.home.store_path(), mode)
    }
}

/// 解析管理根、分派到命令组、打印、返回退出码。
pub fn dispatch(cli: Cli) -> i32 {
    let home = match Home::resolve(cli.home.as_deref()) {
        Ok(h) => h,
        Err(e) => {
            let out = crate::error_map::to_outcome(&e);
            output::print(&out, cli.json);
            return out.exit_code;
        }
    };
    let ctx = Ctx {
        home,
        json: cli.json,
        request_id: cli.request_id,
    };
    let outcome: Outcome = match cli.group {
        Group::SelfCmd(cmd) => self_cmd::run(&ctx, cmd),
        Group::Workbook(cmd) => workbook::run(&ctx, cmd),
        Group::Work(cmd) => work::run(&ctx, cmd),
        Group::Attempt(cmd) => attempt::run(&ctx, cmd),
        Group::Gate(cmd) => gate::run(&ctx, cmd),
    };
    output::print(&outcome, ctx.json);
    outcome.exit_code
}
