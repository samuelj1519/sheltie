//! `self install | update | rollback | uninstall | version`。除 `install` 建库外不打开 `store.db`。
//!
//! 不写任何宿主配置（`INV-3`）：`install` 的 PATH 只是提示文本。安装与更新的提示都要
//! 说清 Store schema 2 与旧数据保留——rollback 只换回旧二进制，不降级 Store。

use serde_json::json;
use sheltie_runtime::selfmgmt;

use crate::cli::SelfCmd;
use crate::commands::Ctx;
use crate::output::{self, Outcome};

/// Store schema 与降级口径的固定提示（存储合同 §9、C002 design：只换二进制不等于降级）。
const SCHEMA_NOTE: &str = "Store schema 是 2：旧数据保留在旧管理根，旧二进制拒绝 schema 2 的库。\
rollback 只换回旧二进制，不降级 Store；查旧记录要旧二进制配旧管理根。";

/// `uninstall --purge` 在文本模式下没有 `--yes` 时读一行 stdin，必须是 `yes`；JSON 模式下必须给 `--yes`。
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
                "已装在"
            } else {
                "已装到"
            };
            let text = format!(
                "{verb} {}\n{}\n{}\n",
                out.installed_to, out.path_hint, SCHEMA_NOTE
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
                format!("已是最新（{}）\n", out.to)
            } else {
                format!("已从 {} 升到 {}\n", out.from, out.to)
            };
            let text = format!("{head}{SCHEMA_NOTE}\n");
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
        println!(
            "将清理管理根中的 Workbook、Work、pending、临时文件、binary 与数据库；保留根目录和锁文件：{}",
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
                        "已清理管理数据与 binary。保留：\n  {}\n  {}\n",
                        ctx.home.root(),
                        ctx.home.lock_path()
                    ),
                    ctx.request_id.clone(),
                    None,
                    json!({ "kept": kept.iter().map(|path| path.as_str()).collect::<Vec<_>>() }),
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
