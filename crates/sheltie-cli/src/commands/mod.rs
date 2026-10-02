//! 命令分派。每个命令组一个文件，函数签名由骨架定死。

pub mod attempt;
pub mod gate;
pub mod self_cmd;
pub mod work;
pub mod workbook;

use sheltie_runtime::Home;

use crate::cli::{Cli, Group, SelfCmd, WorkCmd, WorkbookCmd};
use crate::output::{self, Outcome};

/// 一次调用共享的东西。
pub struct Ctx {
    pub home: Home,
    pub json: bool,
    pub request_id: Option<String>,
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
    // 只读操作与整个 self 组不支持 --request-id：给出即参数错误（协议 §1）。
    let read_only = matches!(
        cli.group,
        Group::SelfCmd(_)
            | Group::Workbook(
                WorkbookCmd::List | WorkbookCmd::Show { .. } | WorkbookCmd::Verify { .. }
            )
            | Group::Work(
                WorkCmd::List
                    | WorkCmd::Status { .. }
                    | WorkCmd::Result { .. }
                    | WorkCmd::Stats { .. }
            )
    );
    let cleanup_after_success = matches!(
        &cli.group,
        Group::Workbook(WorkbookCmd::Add { .. } | WorkbookCmd::Remove { .. })
            | Group::Work(WorkCmd::Start(_) | WorkCmd::Cancel { .. })
            | Group::Attempt(_)
            | Group::Gate(_)
    );
    let cleanup_tmp_after_success = match &cli.group {
        Group::SelfCmd(command) => !matches!(command, SelfCmd::Version),
        _ => cleanup_after_success,
    };
    if cli.request_id.is_some() && read_only {
        let out = crate::output::param_error(
            "--request-id 只用于 Work 与 Workbook 写操作；只读与 self 命令不支持".to_string(),
        );
        output::print(&out, cli.json);
        return out.exit_code;
    }
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
    if outcome.exit_code == 0 && cleanup_tmp_after_success {
        if let Err(error) = ctx.home.cleanup_tmp() {
            eprintln!("warning: tmp维护未完成：{error}");
        }
    }
    if outcome.exit_code == 0 && cleanup_after_success {
        match sheltie_runtime::WorkbookRepo::new(ctx.home.clone()).cleanup_pending() {
            Ok(warnings) => {
                for warning in warnings {
                    eprintln!("warning: pending维护：{warning}");
                }
            }
            Err(error) => eprintln!("warning: pending维护未完成：{error}"),
        }
    }
    outcome.exit_code
}
