//! `self install | update | rollback | uninstall | version`. Only `install` opens `store.db` to initialize it.
//!
//! Never change host configuration (`INV-3`): PATH is advice only. Install and update messages must
//! state the current Store schema and preservation of old data; rollback restores the binary without downgrading Store.

use serde_json::json;
use sheltie_runtime::selfmgmt;

use crate::cli::SelfCmd;
use crate::commands::Ctx;
use crate::output::{self, Outcome};

fn schema_note(ctx: &Ctx) -> String {
    format!(
        "Store schema is {}: old data remains in its old management root; old binaries reject the current format.\
rollback restores only the old binary, without downgrading Store; inspect old records with the old binary and old management root.",
        selfmgmt::version_info(&ctx.home).schema_version
    )
}

/// `uninstall --purge` without `--yes` reads stdin and requires `yes` in text mode; JSON mode requires `--yes`.
pub fn run(ctx: &Ctx, cmd: SelfCmd) -> Outcome {
    match cmd {
        SelfCmd::Install => install(ctx),
        SelfCmd::Update { version } => update(ctx, version.as_deref()),
        SelfCmd::Rollback => rollback(ctx),
        SelfCmd::Uninstall { purge, yes } => uninstall(ctx, purge, yes),
        SelfCmd::Version => version(ctx),
    }
}

/// `self install`。
fn install(ctx: &Ctx) -> Outcome {
    match selfmgmt::install(&ctx.home) {
        Ok(out) => {
            let data = json!({
                "installed_to": out.installed_to.as_str(),
                "already_installed": out.already_installed,
                "path_hint": out.path_hint,
            });
            let verb = if out.already_installed {
                "Already installed at"
            } else {
                "Installed at"
            };
            let text = format!(
                "{verb} {}\n{}\n{}\n",
                out.installed_to,
                out.path_hint,
                schema_note(ctx)
            );
            output::ok(text, ctx.request_id.clone(), None, data, Vec::new())
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `self update`。
fn update(ctx: &Ctx, version: Option<&str>) -> Outcome {
    let source = selfmgmt::ReleaseSource::from_env();
    match selfmgmt::update(&ctx.home, &source, version) {
        Ok(out) => {
            let data = json!({
                "from": out.from,
                "to": out.to,
                "up_to_date": out.up_to_date,
            });
            let head = if out.up_to_date {
                format!("Already up to date ({})\n", out.to)
            } else {
                format!("Updated from {} to {}\n", out.from, out.to)
            };
            let text = format!("{head}{}\n", schema_note(ctx));
            output::ok(text, ctx.request_id.clone(), None, data, Vec::new())
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `self rollback`。
fn rollback(ctx: &Ctx) -> Outcome {
    match selfmgmt::rollback(&ctx.home) {
        Ok(()) => output::ok(
            "Restored the previous version\n".to_string(),
            ctx.request_id.clone(),
            None,
            json!({ "rolled_back": true }),
            Vec::new(),
        ),
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `self uninstall [--purge]`。
fn uninstall(ctx: &Ctx, purge: bool, yes: bool) -> Outcome {
    // Only purge needs confirmation; text mode reads stdin, and JSON requires --yes (protocol §3 self uninstall).
    let confirmed = if !purge || yes {
        true
    } else if ctx.json {
        false
    } else {
        println!(
            "Clear Workbooks, Works, pending data, temporary files, binary, and database; retain the root and lock file: {}",
            ctx.home.root().as_str()
        );
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => line.trim() == "yes",
            Err(_) => false,
        }
    };
    match selfmgmt::uninstall(&ctx.home, purge, confirmed) {
        Ok(kept) => {
            if purge {
                return output::ok(
                    format!(
                        "Cleared managed data and binary. Retained:\n  {}\n  {}\n",
                        ctx.home.root(),
                        ctx.home.lock_path()
                    ),
                    ctx.request_id.clone(),
                    None,
                    json!({ "kept": kept.iter().map(|path| path.as_str()).collect::<Vec<_>>() }),
                    Vec::new(),
                );
            }
            let mut text = String::from("Removed bin/. Retained:\n");
            let mut paths = Vec::with_capacity(kept.len());
            for p in &kept {
                text.push_str(&format!("  {}\n", p.as_str()));
                paths.push(p.as_str());
            }
            output::ok(
                text,
                ctx.request_id.clone(),
                None,
                json!({ "kept": paths }),
                Vec::new(),
            )
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `self version`: read-only, without creating the management root.
fn version(ctx: &Ctx) -> Outcome {
    let info = selfmgmt::version_info(&ctx.home);
    let data = json!({
        "version": info.version,
        "platform": info.platform,
        "home": info.home.as_str(),
        "schema_version": info.schema_version,
    });
    let text = format!(
        "sheltie {}\nplatform: {}\nhome: {}\nSCHEMA_VERSION: {}\n",
        info.version,
        info.platform,
        info.home.as_str(),
        info.schema_version
    );
    output::ok(text, None, None, data, Vec::new())
}
