//! `self install | update | rollback | uninstall | version`。不打开 `store.db`。

use serde_json::json;
use sheltie_runtime::selfmgmt;

use crate::cli::SelfCmd;
use crate::commands::Ctx;
use crate::output::{self, Outcome};

/// `uninstall --purge` 在文本模式下没有 `--yes` 时读一行 stdin，必须是 `yes`；JSON 模式下必须给 `--yes`。
pub fn run(ctx: &Ctx, cmd: SelfCmd) -> Outcome {
    match cmd {
        SelfCmd::Install { modify_path } => install(ctx, modify_path),
        SelfCmd::Update { version } => update(ctx, version.as_deref()),
        SelfCmd::Rollback => rollback(ctx),
        SelfCmd::Uninstall { purge, yes } => uninstall(ctx, purge, yes),
        SelfCmd::Version => version(ctx),
    }
}

/// `self install`。
fn install(ctx: &Ctx, modify_path: bool) -> Outcome {
    match selfmgmt::install(&ctx.home, modify_path) {
        Ok(out) => {
            let data = json!({
                "installed_to": out.installed_to.as_str(),
                "already_installed": out.already_installed,
                "path_hint": out.path_hint,
            });
            let verb = if out.already_installed {
                "已装在"
            } else {
                "已装到"
            };
            let text = format!("{verb} {}\n{}\n", out.installed_to, out.path_hint);
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
            let text = if out.up_to_date {
                format!("已是最新（{}）\n", out.to)
            } else {
                format!("已从 {} 升到 {}\n", out.from, out.to)
            };
            output::ok(text, ctx.request_id.clone(), None, data, Vec::new())
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `self rollback`。
fn rollback(ctx: &Ctx) -> Outcome {
    match selfmgmt::rollback(&ctx.home) {
        Ok(()) => output::ok(
            "已换回上一版本\n".to_string(),
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
    // 只有 purge 要确认；文本模式读一行 stdin，JSON 模式必须给 --yes（协议 §3 self uninstall）。
    let confirmed = if !purge || yes {
        true
    } else if ctx.json {
        false
    } else {
        println!("将删除整个管理根 {}", ctx.home.root().as_str());
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
                    format!("已删除 {}\n", ctx.home.root().as_str()),
                    ctx.request_id.clone(),
                    None,
                    json!({ "kept": [] }),
                    Vec::new(),
                );
            }
            let mut text = String::from("已删 bin/。保留：\n");
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

/// `self version`。只读，不建管理根。
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
