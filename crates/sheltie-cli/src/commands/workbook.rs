//! `workbook add | list | show | remove | verify`。

use serde_json::json;
use sheltie_core::path::AbsPath;
use sheltie_runtime::Error;
use sheltie_runtime::WorkbookRepo;
use sheltie_runtime::store::OpenMode;

use crate::cli::{WorkbookCmd, parse_workbook_spec};
use crate::commands::Ctx;
use crate::output::{self, Outcome};

/// 分派到五个子命令。每个子命令：打开存储、建 `WorkbookRepo`、调用、渲染文本、包成 `Outcome`。
/// 错误经 `error_map::to_outcome`。
pub fn run(ctx: &Ctx, cmd: WorkbookCmd) -> Outcome {
    match cmd {
        WorkbookCmd::Add { dir } => add(ctx, &dir),
        WorkbookCmd::List => list(ctx),
        WorkbookCmd::Show { spec } => show(ctx, &spec),
        WorkbookCmd::Remove { spec } => remove(ctx, &spec),
        WorkbookCmd::Verify { spec } => verify(ctx, spec.as_deref()),
    }
}

/// 相对目录按当前工作目录转成绝对路径；`AbsPath` 只收绝对的。
fn abs_arg(value: &str) -> Result<AbsPath, Error> {
    let joined = if std::path::Path::new(value).is_absolute() {
        value.to_string()
    } else {
        let cwd = std::env::current_dir().map_err(|e| Error::io(".", e))?;
        cwd.join(value).to_string_lossy().into_owned()
    };
    AbsPath::new(joined).map_err(Error::from)
}

/// `workbook add <dir>`（协议 §3：成功返回 `{ id, version, digest, flows, requires }`）。
fn add(ctx: &Ctx, dir: &str) -> Outcome {
    let dir = match abs_arg(dir) {
        Ok(p) => p,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let store = match ctx.store(OpenMode::ReadWrite) {
        Ok(s) => s,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let repo = WorkbookRepo::new(ctx.home.clone(), store);
    match repo.add(&dir, ctx.request_id.clone()) {
        Ok(snapshot) => {
            // 响应字段全部来自提交时快照（cli-result/v2）；重放带 replayed。
            let mut data = snapshot.data.clone();
            if let serde_json::Value::Object(map) = &mut data {
                map.insert("replayed".to_string(), serde_json::json!(snapshot.replayed));
            }
            let id = data["id"].as_str().unwrap_or_default().to_string();
            let version = data["version"].as_str().unwrap_or_default().to_string();
            let digest = data["digest"].as_str().unwrap_or_default().to_string();
            let flows = data["flows"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let text = format!("已装 {id}@{version}\ndigest: {digest}\nflows: {flows}\n");
            output::ok(text, Some(snapshot.request_id), None, data, Vec::new())
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `workbook list`：每个 id 的最高版本（字面排序）标 `latest`。
fn list(ctx: &Ctx) -> Outcome {
    let store = match ctx.store(OpenMode::ReadOnly) {
        Ok(s) => s,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let repo = WorkbookRepo::new(ctx.home.clone(), store);
    let rows = match repo.list() {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let mut data = Vec::with_capacity(rows.len());
    let mut text = String::new();
    for (i, row) in rows.iter().enumerate() {
        // rows 按 (id, version) 升序，同 id 的最后一行就是最高版本。
        let latest = rows.get(i + 1).is_none_or(|next| next.id != row.id);
        data.push(json!({
            "id": row.id,
            "version": row.version,
            "digest": row.digest,
            "latest": latest,
        }));
        text.push_str(&format!(
            "{}  {}  {}\n",
            row.id,
            row.version,
            if latest { "latest" } else { "" }
        ));
    }
    output::ok(text, None, None, data, Vec::new())
}

/// `workbook show <id>[@<version>]`：manifest、宿主资源声明与每个 Flow 的节点、边。
fn show(ctx: &Ctx, spec: &str) -> Outcome {
    let (id, version) = match parse_workbook_spec(spec) {
        Ok(v) => v,
        Err(m) => return output::param_error(m),
    };
    let store = match ctx.store(OpenMode::ReadOnly) {
        Ok(s) => s,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let repo = WorkbookRepo::new(ctx.home.clone(), store);
    let loaded = match repo.load(&id, version.as_deref()) {
        Ok(l) => l,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let mut flows = Vec::with_capacity(loaded.flows.len());
    let mut text = format!(
        "workbook: {}@{}（{}）\ndigest: {}\nrequires: {}\n",
        loaded.manifest.id(),
        loaded.manifest.version(),
        loaded.manifest.name(),
        loaded.digest.as_str(),
        if loaded.manifest.requires().is_empty() {
            "无".to_string()
        } else {
            loaded
                .manifest
                .requires()
                .iter()
                .map(|r| format!("{}:{}", r.kind().as_str(), r.name()))
                .collect::<Vec<_>>()
                .join(", ")
        },
    );
    for (def, graph) in &loaded.flows {
        // 有序起始输入键（GF-30）：与 runtime preflight 同一份 start_requirements，
        // 协调者第一次调用就能拿全开一个 Work 需要的键，不用失败 start 探测。
        let start_inputs = sheltie_core::work::start_requirements(graph);
        flows.push(json!({
            "id": def.id().as_str(),
            "entry": def.entry().as_str(),
            "start_inputs": start_inputs,
            "nodes": def.nodes().iter().map(|n| json!({
                "id": n.id().as_str(),
                "title": n.title(),
                "executor": n.executor().as_str(),
                "gate": n.gate(),
            })).collect::<Vec<_>>(),
            "edges": def.edges().iter().map(|e| json!({
                "from": e.from().as_str(),
                "to": e.to().as_str(),
                "kind": e.kind().as_str(),
            })).collect::<Vec<_>>(),
        }));
        text.push_str(&format!("\nflow {}（入口 {}）\n", def.id(), def.entry()));
        text.push_str(&format!(
            "  起始输入: {}\n",
            if start_inputs.is_empty() {
                "无".to_string()
            } else {
                start_inputs.join(", ")
            }
        ));
        for n in def.nodes() {
            text.push_str(&format!(
                "  {}  {}  {}{}\n",
                n.id(),
                n.title(),
                n.executor().as_str(),
                if n.gate() { "  [gate]" } else { "" }
            ));
        }
        for e in def.edges() {
            text.push_str(&format!(
                "  {} -> {}（{}）\n",
                e.from(),
                e.to(),
                e.kind().as_str()
            ));
        }
    }
    let data = json!({
        "id": loaded.manifest.id().as_str(),
        "version": loaded.manifest.version(),
        "name": loaded.manifest.name(),
        "description": loaded.manifest.description(),
        "digest": loaded.digest.as_str(),
        "requires": loaded.manifest.requires(),
        "flows": flows,
    });
    output::ok(text, None, None, data, Vec::new())
}

/// `workbook remove <id>@<version>`。版本必须给全，防误删。
fn remove(ctx: &Ctx, spec: &str) -> Outcome {
    let (id, version) = match parse_workbook_spec(spec) {
        Ok(v) => v,
        Err(m) => return output::param_error(m),
    };
    let Some(version) = version else {
        return output::param_error(format!(
            "remove 必须给全版本，写 {id}@<version>；不接受「最高版本」默认"
        ));
    };
    let store = match ctx.store(OpenMode::ReadWrite) {
        Ok(s) => s,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let repo = WorkbookRepo::new(ctx.home.clone(), store);
    match repo.remove(&id, &version, ctx.request_id.clone()) {
        Ok(snapshot) => {
            let mut data = snapshot.data.clone();
            if let serde_json::Value::Object(map) = &mut data {
                map.insert("replayed".to_string(), serde_json::json!(snapshot.replayed));
            }
            let text = format!("已删除 {id}@{version}\n");
            output::ok(text, Some(snapshot.request_id), None, data, Vec::new())
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `workbook verify [<id>@<version>]`。省略参数核对全部。任一非 `ok` 则 `WORKBOOK_TAMPERED`。
fn verify(ctx: &Ctx, spec: Option<&str>) -> Outcome {
    let filter = match spec {
        None => None,
        Some(s) => match parse_workbook_spec(s) {
            Ok((id, Some(version))) => Some((id, version)),
            Ok((_, None)) => {
                return output::param_error(format!(
                    "verify 要么不带参数核对全部，要么写全 {s}@<version>"
                ));
            }
            Err(m) => return output::param_error(m),
        },
    };
    let store = match ctx.store(OpenMode::ReadOnly) {
        Ok(s) => s,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let repo = WorkbookRepo::new(ctx.home.clone(), store);
    let rows = match repo.verify(filter.as_ref().map(|(i, v)| (i.as_str(), v.as_str()))) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let mut text = String::new();
    for row in &rows {
        text.push_str(&format!(
            "{}@{}  {}\n",
            row.id,
            row.version,
            match row.status {
                sheltie_runtime::VerifyStatus::Ok => "ok",
                sheltie_runtime::VerifyStatus::Tampered => "tampered",
                sheltie_runtime::VerifyStatus::Missing => "missing",
            }
        ));
    }
    let all_ok = rows
        .iter()
        .all(|r| r.status == sheltie_runtime::VerifyStatus::Ok);
    if all_ok {
        output::ok(text, None, None, json!({ "results": rows }), Vec::new())
    } else {
        let mut out = crate::output::err(
            sheltie_core::ErrorCode::WorkbookTampered,
            "已装 Workbook 与记录不符".to_string(),
            Some(json!({ "results": rows })),
            Vec::new(),
        );
        out.text = text;
        out
    }
}
