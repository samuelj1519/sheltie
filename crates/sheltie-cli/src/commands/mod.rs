//! Command dispatch. Each command group has a file; function signatures are fixed by the skeleton.

pub mod attempt;
pub mod gate;
pub mod self_cmd;
pub mod work;
pub mod workbook;

use sheltie_runtime::Home;

use crate::cli::{Cli, Group, SelfCmd, WorkCmd, WorkbookCmd};
use crate::output::{self, Outcome};

/// Context shared by a single invocation.
pub struct Ctx {
    pub home: Home,
    pub json: bool,
    pub request_id: Option<String>,
}

/// Resolve the management root, dispatch, print, and return the exit code.
pub fn dispatch(cli: Cli) -> i32 {
    let raw = matches!(
        &cli.group,
        Group::Work(WorkCmd::Result { artifact, revision, .. })
            if artifact.is_some() || revision.is_some()
    );
    if raw && (cli.json || cli.request_id.is_some()) {
        return output::raw_param_error("Raw-byte mode does not support --json or --request-id");
    }
    if raw
        && !matches!(
            &cli.group,
            Group::Work(WorkCmd::Result { artifact: Some(_), revision: Some(revision), .. }) if *revision > 0
        )
    {
        return output::raw_param_error(
            "--artifact and a positive --revision must be supplied together",
        );
    }
    let home = match Home::resolve(cli.home.as_deref()) {
        Ok(h) => h,
        Err(e) => {
            if raw {
                return output::raw_error(&e);
            }
            let out = crate::error_map::to_outcome(&e);
            output::print(&out, cli.json);
            return out.exit_code;
        }
    };
    // Read-only and self commands reject --request-id as a parameter error (protocol §1).
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
            "--request-id is only for Work and Workbook writes; read-only and self commands do not support it".to_string(),
        );
        output::print(&out, cli.json);
        return out.exit_code;
    }
    let ctx = Ctx {
        home,
        json: cli.json,
        request_id: cli.request_id,
    };
    if let Group::Work(WorkCmd::Result {
        work,
        artifact: Some(key),
        revision: Some(revision),
    }) = &cli.group
    {
        return work::result_artifact(&ctx, work, key, *revision);
    }
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
            eprintln!("warning: tmp maintenance incomplete: {error}");
        }
    }
    if outcome.exit_code == 0 && cleanup_after_success {
        match sheltie_runtime::WorkbookRepo::new(ctx.home.clone()).cleanup_pending() {
            Ok(warnings) => {
                for warning in warnings {
                    eprintln!("warning: pending maintenance: {warning}");
                }
            }
            Err(error) => eprintln!("warning: pending maintenance incomplete: {error}"),
        }
    }
    outcome.exit_code
}
