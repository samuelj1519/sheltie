//! `workbook add | list | show | remove | verify`。

use serde_json::json;
use sheltie_core::path::AbsPath;
use sheltie_runtime::Error;
use sheltie_runtime::WorkbookRepo;

use crate::cli::{WorkbookCmd, parse_workbook_spec};
use crate::commands::Ctx;
use crate::output::{self, Outcome};

/// Dispatch to five subcommands: open storage, construct `WorkbookRepo`, invoke, render text, and wrap in `Outcome`.
/// Map errors through `error_map::to_outcome`.
pub fn run(ctx: &Ctx, cmd: WorkbookCmd) -> Outcome {
    match cmd {
        WorkbookCmd::Add { dir } => add(ctx, &dir),
        WorkbookCmd::List => list(ctx),
        WorkbookCmd::Show { spec } => show(ctx, &spec),
        WorkbookCmd::Remove { spec } => remove(ctx, &spec),
        WorkbookCmd::Verify { spec } => verify(ctx, spec.as_deref()),
    }
}

/// Resolve relative directories against the working directory; `AbsPath` accepts only absolute paths.
fn abs_arg(value: &str) -> Result<AbsPath, Error> {
    AbsPath::new(sheltie_runtime::request::lexical_abs(value)?).map_err(Error::from)
}

/// `workbook add <dir>` (protocol §3: success returns `{ id, version, digest, flows, requires }`).
fn add(ctx: &Ctx, dir: &str) -> Outcome {
    let dir = match abs_arg(dir) {
        Ok(p) => p,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let repo = WorkbookRepo::new(ctx.home.clone());
    match repo.add(&dir, ctx.request_id.clone()) {
        Ok(snapshot) => {
            // All response fields come from the commit-time snapshot (cli-result/v2); replay includes replayed.
            let data = output::replayed_data(snapshot.data, snapshot.replayed);
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
            let text = format!("Installed {id}@{version}\ndigest: {digest}\nflows: {flows}\n");
            output::ok(text, Some(snapshot.request_id), None, data, Vec::new())
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `workbook list`: mark the highest lexicographic version of each ID as `latest`.
fn list(ctx: &Ctx) -> Outcome {
    let repo = WorkbookRepo::new(ctx.home.clone());
    let rows = match repo.list() {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let mut data = Vec::with_capacity(rows.len());
    let mut text = String::new();
    for (i, row) in rows.iter().enumerate() {
        // rows are ordered by (id, version); the last row for each ID is its highest version.
        let latest = rows.get(i + 1).is_none_or(|next| next.id != row.id);
        data.push(json!({
            "id": row.id,
            "version": row.version,
            "digest": row.digest,
            "latest": latest,
            "pending_publish": row.pending_publish,
        }));
        text.push_str(&format!(
            "{}  {}  {}{}\n",
            row.id,
            row.version,
            if latest { "latest" } else { "" },
            if row.pending_publish {
                "  publication pending"
            } else {
                ""
            },
        ));
    }
    output::ok(text, None, None, data, Vec::new())
}

/// `workbook show <id>[@<version>]`: manifest, host resource declarations, and Flow nodes and edges.
fn show(ctx: &Ctx, spec: &str) -> Outcome {
    let (id, version) = match parse_workbook_spec(spec) {
        Ok(v) => v,
        Err(m) => return output::param_error(m),
    };
    let repo = WorkbookRepo::new(ctx.home.clone());
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
            "none".to_string()
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
    if loaded.pending_publish {
        text.push_str("Publication status: pending (reading the committed frozen copy)\n");
    }
    for (def, graph) in &loaded.flows {
        // Ordered start input keys (GF-30) share start_requirements with runtime preflight,
        // so the coordinator obtains every required key without probing a failed start.
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
        text.push_str(&format!("\nflow {} (entry {})\n", def.id(), def.entry()));
        text.push_str(&format!(
            "  Start inputs: {}\n",
            if start_inputs.is_empty() {
                "none".to_string()
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
        "pending_publish": loaded.pending_publish,
        "requires": loaded.manifest.requires(),
        "flows": flows,
    });
    output::ok(text, None, None, data, Vec::new())
}

/// `workbook remove <id>@<version>`. Require an explicit version to prevent unintended deletion.
fn remove(ctx: &Ctx, spec: &str) -> Outcome {
    let (id, version) = match parse_workbook_spec(spec) {
        Ok(v) => v,
        Err(m) => return output::param_error(m),
    };
    let Some(version) = version else {
        return output::param_error(format!(
            "remove requires an explicit version, {id}@<version>; no highest-version default is accepted"
        ));
    };
    let repo = WorkbookRepo::new(ctx.home.clone());
    match repo.remove(&id, &version, ctx.request_id.clone()) {
        Ok(snapshot) => {
            let data = output::replayed_data(snapshot.data, snapshot.replayed);
            let text = format!("Removed {id}@{version}\n");
            output::ok(text, Some(snapshot.request_id), None, data, Vec::new())
        }
        Err(e) => crate::error_map::to_outcome(&e),
    }
}

/// `workbook verify [<id>@<version>]`. Verify all when omitted; any non-`ok` result yields `WORKBOOK_TAMPERED`.
fn verify(ctx: &Ctx, spec: Option<&str>) -> Outcome {
    let filter = match spec {
        None => None,
        Some(s) => match parse_workbook_spec(s) {
            Ok((id, Some(version))) => Some((id, version)),
            Ok((_, None)) => {
                return output::param_error(format!(
                    "verify requires either no argument for all versions or the full {s}@<version>"
                ));
            }
            Err(m) => return output::param_error(m),
        },
    };
    let repo = WorkbookRepo::new(ctx.home.clone());
    let rows = match repo.verify(filter.as_ref().map(|(i, v)| (i.as_str(), v.as_str()))) {
        Ok(r) => r,
        Err(e) => return crate::error_map::to_outcome(&e),
    };
    let mut text = String::new();
    for row in &rows {
        text.push_str(&format!(
            "{}@{}  {}{}\n",
            row.id,
            row.version,
            match row.status {
                sheltie_runtime::VerifyStatus::Ok => "ok",
                sheltie_runtime::VerifyStatus::Tampered => "tampered",
                sheltie_runtime::VerifyStatus::Missing => "missing",
            },
            if row.pending_publish {
                "  publication pending"
            } else {
                ""
            },
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
            "Installed Workbook does not match its stored record".to_string(),
            Some(json!({ "results": rows })),
        );
        out.text = text;
        out
    }
}
