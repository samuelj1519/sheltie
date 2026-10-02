//! Flow TOML 到 `FlowDef` 的解析。只做结构与取值范围，不做图语义（那是 `compile`）。

use serde::Deserialize;

use super::def::{
    EdgeDef, EdgeKind, Executor, FlowDef, InputDecl, InputSource, Instruction, NodeDef, OutputDecl,
    Tier,
};
use crate::error::{Error, Result};
use crate::ids::{FlowId, NodeId, validate_id};
use crate::path::RelPath;
use crate::workbook::RequireKind;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FlowDto {
    schema: String,
    id: String,
    entry: String,
    nodes: Vec<NodeDto>,
    #[serde(default)]
    edges: Vec<EdgeDto>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeDto {
    id: String,
    title: String,
    executor: String,
    #[serde(default)]
    tier: Option<String>,
    instruction: InstructionDto,
    #[serde(default)]
    inputs: Vec<InputDto>,
    #[serde(default)]
    outputs: Vec<OutputDto>,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    gate: Option<bool>,
    #[serde(default)]
    max_visits: Option<u32>,
    #[serde(default)]
    max_retries: Option<u32>,
}

/// `{ file = ... }` 或 `{ text = ... }`，恰一个。两个都给或都不给在 `convert` 里拒绝。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstructionDto {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputDto {
    name: String,
    from: String,
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    result: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputDto {
    name: String,
    path: String,
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    max_bytes: Option<u64>,
    #[serde(default)]
    result: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EdgeDto {
    from: String,
    to: String,
    kind: String,
}

/// 解析 Flow 文件。
///
/// TOML 语法错误与未知字段返回 `Error::FlowInvalid { rule: "parse", path: "toml", .. }`；
/// 字段级错误 `rule = "parse"`，`path` 用 `nodes[2].inputs[0].from` 这类写法。
pub fn parse_flow(toml_text: &str) -> Result<FlowDef> {
    let dto: FlowDto = toml::from_str(toml_text).map_err(|e| Error::FlowInvalid {
        rule: "parse",
        path: "toml".to_string(),
        reason: e.to_string(),
    })?;
    convert(dto)
}

/// 解析 `inputs[].from` 的四种写法。
///
/// - `start.<key>`：`key` 走 ID 字符规则。
/// - `resource.<path>`：第一个 `.` 之后全部是路径，可含 `/`。
/// - `engine.stats`：字面量，`engine.` 后只允许 `stats`。
/// - `<node>.<output>`：恰好一个 `.`，两边都非空；`node` 走 ID 规则，`output` 走 ID 规则。
///
/// 失败返回 `Error::FlowInvalid { rule: "parse", path, .. }`，`path` 由调用方传入。
pub fn parse_input_source(value: &str, path: &str) -> Result<InputSource> {
    let invalid = |reason: String| Error::FlowInvalid {
        rule: "parse",
        path: path.to_string(),
        reason,
    };
    let (head, rest) = value.split_once('.').ok_or_else(|| {
        invalid(
            "必须写成 start.<key>、resource.<path>、engine.stats 或 <node>.<output>".to_string(),
        )
    })?;
    match head {
        "start" => {
            if rest.contains('.') {
                return Err(invalid("start. 之后只允许一段 key".to_string()));
            }
            validate_id(rest, "key").map_err(|e| invalid(e.to_string()))?;
            Ok(InputSource::Start {
                key: rest.to_string(),
            })
        }
        "resource" => {
            // 第一个 `.` 之后整段是路径，可以再含 `/`。
            let p = RelPath::new(rest).map_err(|e| invalid(e.to_string()))?;
            Ok(InputSource::Resource { path: p })
        }
        "engine" => {
            if rest != "stats" {
                return Err(invalid("engine. 之后只允许 stats".to_string()));
            }
            Ok(InputSource::EngineStats)
        }
        _ => {
            if rest.is_empty() || rest.contains('.') {
                return Err(invalid(
                    "<node>.<output> 必须恰好一个点，两边都非空".to_string(),
                ));
            }
            let node = NodeId::new(head).map_err(|e| invalid(e.to_string()))?;
            validate_id(rest, "output").map_err(|e| invalid(e.to_string()))?;
            Ok(InputSource::Node {
                node,
                output: rest.to_string(),
            })
        }
    }
}

/// DTO 到 `FlowDef` 的逐字段转换与取值校验。这是 T04 要填的函数。
///
/// 要做的检查（都是单字段，不看图）：`schema == "flow/v1"`；各 ID 合规；
/// `executor` 二选一，`tier` 二选一或缺省；`instruction` 恰一个且 `text` 非空、≤ 8 KiB；
/// `inputs[].name` 唯一，`required` 默认 `true`；`outputs[].name` 唯一、`path` 走
/// workbook 合同 §3.2 的四条路径规则（可移植字符集、唯一、不互为祖先、ASCII 大小写
/// 折叠后仍不得相同或互为祖先）；`max_bytes` 在 1..=32 MiB；`requires[]` 形如
/// `kind:name`；`max_visits` 在 1..=32，`max_retries` 在 0..=8；`edges[].kind` 四选一。
fn convert(dto: FlowDto) -> Result<FlowDef> {
    let invalid = |path: String, reason: String| Error::FlowInvalid {
        rule: "parse",
        path,
        reason,
    };

    if dto.schema != "flow/v1" {
        return Err(invalid(
            "schema".to_string(),
            format!("必须是 flow/v1，实际 {:?}", dto.schema),
        ));
    }
    let id = FlowId::new(&dto.id).map_err(|e| invalid("id".to_string(), e.to_string()))?;
    let entry = NodeId::new(&dto.entry).map_err(|e| invalid("entry".to_string(), e.to_string()))?;

    let mut nodes = Vec::with_capacity(dto.nodes.len());
    for (i, n) in dto.nodes.into_iter().enumerate() {
        let np = |suffix: &str| format!("nodes[{i}].{suffix}");

        let node_id = NodeId::new(&n.id).map_err(|e| invalid(np("id"), e.to_string()))?;
        if n.title.len() > NodeDef::TITLE_MAX_BYTES {
            return Err(invalid(
                np("title"),
                format!("超过 {} 字节", NodeDef::TITLE_MAX_BYTES),
            ));
        }
        let executor = match n.executor.as_str() {
            "agent" => Executor::Agent,
            "human" => Executor::Human,
            other => {
                return Err(invalid(
                    np("executor"),
                    format!("{other:?} 不是 agent 或 human"),
                ));
            }
        };
        let tier_raw = match n.tier.as_deref() {
            None => None,
            Some("strong") => Some(Tier::Strong),
            Some("standard") => Some(Tier::Standard),
            Some(other) => {
                return Err(invalid(
                    np("tier"),
                    format!("{other:?} 不是 strong 或 standard"),
                ));
            }
        };
        // human 节点根本没有档位可选，写出来就是错的（合同 §3.2、规则 9）。
        let tier = match executor {
            Executor::Human => {
                if tier_raw.is_some() {
                    return Err(invalid(np("tier"), "human 节点不得声明 tier".to_string()));
                }
                None
            }
            Executor::Agent => Some(tier_raw.unwrap_or(Tier::Standard)),
        };

        let instruction = match (n.instruction.file, n.instruction.text) {
            (Some(_), Some(_)) => {
                return Err(invalid(
                    np("instruction"),
                    "file 与 text 只能给一个".to_string(),
                ));
            }
            (None, None) => {
                return Err(invalid(
                    np("instruction"),
                    "file 与 text 必须给一个".to_string(),
                ));
            }
            (Some(file), None) => {
                let p = RelPath::new(file)
                    .map_err(|e| invalid(np("instruction.file"), e.to_string()))?;
                Instruction::File(p)
            }
            (None, Some(text)) => {
                if text.is_empty() {
                    return Err(invalid(np("instruction.text"), "不能为空".to_string()));
                }
                if text.len() > NodeDef::TEXT_MAX_BYTES {
                    return Err(invalid(
                        np("instruction.text"),
                        format!("超过 {} 字节", NodeDef::TEXT_MAX_BYTES),
                    ));
                }
                Instruction::Text(text)
            }
        };

        let mut inputs = Vec::with_capacity(n.inputs.len());
        for (j, inp) in n.inputs.into_iter().enumerate() {
            let ip = |suffix: &str| format!("nodes[{i}].inputs[{j}].{suffix}");
            // 合同 §3.2 对 `inputs[].name` 只要求节点内唯一，不走 ID 字符规则（D-26）。
            if inputs.iter().any(|x: &InputDecl| x.name == inp.name) {
                return Err(invalid(ip("name"), "节点内输入名重复".to_string()));
            }
            let from = parse_input_source(&inp.from, &ip("from"))?;
            inputs.push(InputDecl {
                name: inp.name,
                from,
                required: inp.required.unwrap_or(true),
                result: inp.result,
            });
        }

        let mut outputs = Vec::with_capacity(n.outputs.len());
        for (j, out) in n.outputs.into_iter().enumerate() {
            let op = |suffix: &str| format!("nodes[{i}].outputs[{j}].{suffix}");
            validate_id(&out.name, "name").map_err(|e| invalid(op("name"), e.to_string()))?;
            if outputs.iter().any(|x: &OutputDecl| x.name == out.name) {
                return Err(invalid(op("name"), "节点内输出名重复".to_string()));
            }
            let path = RelPath::new(&out.path).map_err(|e| invalid(op("path"), e.to_string()))?;
            validate_output_portable(&path).map_err(|reason| invalid(op("path"), reason))?;
            if outputs
                .iter()
                .any(|x: &OutputDecl| output_paths_conflict(&x.path, &path))
            {
                return Err(invalid(
                    op("path"),
                    "节点内输出路径重复、互为祖先，或大小写折叠后相同或互为祖先".to_string(),
                ));
            }
            let max_bytes = out.max_bytes.unwrap_or(OutputDecl::DEFAULT_MAX_BYTES);
            if !(1..=OutputDecl::MAX_MAX_BYTES).contains(&max_bytes) {
                return Err(invalid(
                    op("max_bytes"),
                    format!("必须在 1..={} 之间", OutputDecl::MAX_MAX_BYTES),
                ));
            }
            outputs.push(OutputDecl {
                name: out.name,
                path,
                required: out.required.unwrap_or(true),
                max_bytes,
                result: out.result,
            });
        }

        let mut requires = Vec::with_capacity(n.requires.len());
        for (j, raw) in n.requires.into_iter().enumerate() {
            let rp = format!("nodes[{i}].requires[{j}]");
            let (kind_str, name) = raw
                .split_once(':')
                .ok_or_else(|| invalid(rp.clone(), "必须写成 <kind>:<name>".to_string()))?;
            let kind = RequireKind::parse(kind_str).ok_or_else(|| {
                invalid(rp.clone(), format!("{kind_str:?} 不是 skill、agent 或 mcp"))
            })?;
            validate_id(name, "name").map_err(|e| invalid(rp, e.to_string()))?;
            requires.push((kind, name.to_string()));
        }

        nodes.push(NodeDef {
            id: node_id,
            title: n.title,
            executor,
            tier,
            instruction,
            inputs,
            outputs,
            requires,
            gate: n.gate.unwrap_or(false),
            max_visits: n.max_visits.unwrap_or(NodeDef::DEFAULT_MAX_VISITS),
            max_retries: n.max_retries.unwrap_or(NodeDef::DEFAULT_MAX_RETRIES),
        });
    }

    // max_visits / max_retries 的取值范围按节点检查一次，免得默认值也走一遍。
    for (i, node) in nodes.iter().enumerate() {
        if !(1..=NodeDef::MAX_MAX_VISITS).contains(&node.max_visits) {
            return Err(invalid(
                format!("nodes[{i}].max_visits"),
                format!("必须在 1..={} 之间", NodeDef::MAX_MAX_VISITS),
            ));
        }
        if node.max_retries > NodeDef::MAX_MAX_RETRIES {
            return Err(invalid(
                format!("nodes[{i}].max_retries"),
                format!("必须在 0..={} 之间", NodeDef::MAX_MAX_RETRIES),
            ));
        }
    }

    let mut edges = Vec::with_capacity(dto.edges.len());
    for (i, e) in dto.edges.into_iter().enumerate() {
        let ep = |suffix: &str| format!("edges[{i}].{suffix}");
        let from = NodeId::new(&e.from).map_err(|err| invalid(ep("from"), err.to_string()))?;
        let to = NodeId::new(&e.to).map_err(|err| invalid(ep("to"), err.to_string()))?;
        let kind = match e.kind.as_str() {
            "main" => EdgeKind::Main,
            "back" => EdgeKind::Back,
            "branch" => EdgeKind::Branch,
            "re_review" => EdgeKind::ReReview,
            other => {
                return Err(invalid(
                    ep("kind"),
                    format!("{other:?} 不是 main、back、branch 或 re_review"),
                ));
            }
        };
        edges.push(EdgeDef { from, to, kind });
    }

    Ok(FlowDef {
        id,
        entry,
        nodes,
        edges,
    })
}

/// 输出路径的可移植字符集（workbook 合同 §3.2 第 2 条）：每段非空、≤ 128 字节、
/// 只含 `A-Z a-z 0-9 . _ -`。非 ASCII（含汉字、Unicode 变体）拒绝；这使 ASCII
/// 大小写折叠成为完整的别名判定，不需要 Unicode 归一化猜测。
fn validate_output_portable(path: &RelPath) -> std::result::Result<(), String> {
    const SEGMENT_MAX_BYTES: usize = 128;
    for segment in path.as_path().components() {
        let seg = segment.as_str();
        if seg.is_empty() {
            return Err("段不能为空".to_string());
        }
        if seg.len() > SEGMENT_MAX_BYTES {
            return Err(format!("段超过 {SEGMENT_MAX_BYTES} 字节"));
        }
        if !seg
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        {
            return Err("每段只能含 A-Z、a-z、0-9、.、_ 与 -".to_string());
        }
    }
    Ok(())
}

/// 两条节点内输出路径是否冲突：逐段先按 ASCII 小写折叠，再比较；完全相同或
/// 一方是另一方的祖先（段前缀）即冲突。`OUT.md` 与 `out.md`、`Out` 与 `out/x.md`
/// 都算（workbook 合同 §3.2 第 3、4 条）。
fn output_paths_conflict(a: &RelPath, b: &RelPath) -> bool {
    let fold = |p: &RelPath| -> Vec<String> {
        p.as_path()
            .components()
            .map(|c| c.as_str().to_ascii_lowercase())
            .collect()
    };
    let (fa, fb) = (fold(a), fold(b));
    let (short, long) = if fa.len() <= fb.len() {
        (&fa, &fb)
    } else {
        (&fb, &fa)
    };
    long.starts_with(short.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::def::{EdgeKind, Executor, Tier};

    const THREE_NODE: &str = include_str!("../../../../examples/article-review/flows/default.toml");

    fn minimal(node_extra: &str) -> String {
        format!(
            r#"
schema = "flow/v1"
id = "default"
entry = "a"

[[nodes]]
id = "a"
title = "A"
executor = "agent"
instruction = {{ text = "做 A" }}
{node_extra}
"#
        )
    }

    // Task: T04
    #[test]
    fn parses_three_node_flow() {
        let flow = parse_flow(THREE_NODE).unwrap();
        assert_eq!(flow.nodes.len(), 3);
        assert_eq!(flow.edges.len(), 3);
        assert_eq!(flow.entry.as_str(), "draft");
        let publish = flow
            .node(&"publish".to_string().try_into().unwrap())
            .unwrap();
        assert_eq!(publish.executor, Executor::Human);
        assert_eq!(publish.tier, None);
    }

    // Task: T04
    #[test]
    fn instruction_requires_exactly_one_of_file_or_text() {
        let both = minimal("").replace(
            "instruction = { text = \"做 A\" }",
            "instruction = { text = \"x\", file = \"i.md\" }",
        );
        assert!(matches!(
            parse_flow(&both),
            Err(Error::FlowInvalid { rule: "parse", .. })
        ));
        let none = minimal("").replace("instruction = { text = \"做 A\" }", "instruction = { }");
        assert!(parse_flow(&none).is_err());
    }

    // Task: T04
    #[test]
    fn input_from_parses_start_resource_and_node_forms() {
        assert_eq!(
            parse_input_source("start.topic", "p").unwrap(),
            InputSource::Start {
                key: "topic".into()
            }
        );
        assert!(matches!(
            parse_input_source("resource.resources/a.md", "p").unwrap(),
            InputSource::Resource { .. }
        ));
        assert!(matches!(
            parse_input_source("draft.article", "p").unwrap(),
            InputSource::Node { .. }
        ));
    }

    // Task: T04
    #[test]
    fn input_from_parses_engine_stats_only() {
        assert_eq!(
            parse_input_source("engine.stats", "p").unwrap(),
            InputSource::EngineStats
        );
        assert!(parse_input_source("engine.clock", "p").is_err());
        assert!(parse_input_source("engine", "p").is_err());
    }

    // Task: T04
    #[test]
    fn input_from_rejects_three_segments_for_start_and_node() {
        assert!(parse_input_source("start.a.b", "p").is_err());
        assert!(parse_input_source("draft.article.v2", "p").is_err());
        assert!(parse_input_source("draft", "p").is_err());
        assert!(parse_input_source("draft.", "p").is_err());
    }

    // Task: T04
    #[test]
    fn input_from_resource_keeps_slashes_in_path() {
        let src = parse_input_source("resource.resources/templates/spec.md", "p").unwrap();
        assert_eq!(
            src,
            InputSource::Resource {
                path: "resources/templates/spec.md"
                    .to_string()
                    .try_into()
                    .unwrap()
            }
        );
    }

    // Task: T04
    #[test]
    fn node_requires_parses_kind_colon_name() {
        let flow = parse_flow(&minimal("requires = [\"skill:company-api\", \"mcp:db\"]")).unwrap();
        assert_eq!(flow.nodes[0].requires.len(), 2);
        assert_eq!(flow.nodes[0].requires[0].1, "company-api");
    }

    // Task: T04
    #[test]
    fn node_requires_rejects_bad_kind() {
        assert!(parse_flow(&minimal("requires = [\"plugin:x\"]")).is_err());
        assert!(parse_flow(&minimal("requires = [\"skill\"]")).is_err());
    }

    // Task: T04
    #[test]
    fn input_required_defaults_true_and_parses_false() {
        let flow = parse_flow(&minimal("inputs = [{ name = \"a\", from = \"start.a\" }, { name = \"b\", from = \"x.y\", required = false }]")).unwrap();
        assert!(flow.nodes[0].inputs[0].required);
        assert!(!flow.nodes[0].inputs[1].required);
    }

    // Task: T04
    #[test]
    fn tier_defaults_standard_and_parses_strong() {
        assert_eq!(
            parse_flow(&minimal("")).unwrap().nodes[0].tier,
            Some(Tier::Standard)
        );
        assert_eq!(
            parse_flow(&minimal("tier = \"strong\"")).unwrap().nodes[0].tier,
            Some(Tier::Strong)
        );
        assert!(parse_flow(&minimal("tier = \"huge\"")).is_err());
    }

    // Task: T04
    #[test]
    fn defaults_gate_false_visits_1_retries_1() {
        let n = &parse_flow(&minimal("")).unwrap().nodes[0];
        assert!(!n.gate);
        assert_eq!(n.max_visits, 1);
        assert_eq!(n.max_retries, 1);
    }

    // Task: T04
    #[test]
    fn rejects_max_visits_zero_or_over_32() {
        assert!(parse_flow(&minimal("max_visits = 0")).is_err());
        assert!(parse_flow(&minimal("max_visits = 33")).is_err());
        assert!(parse_flow(&minimal("max_retries = 9")).is_err());
    }

    // 引擎文件与 worker 输出分目录后，声明 brief.md 或 outputs/brief.md 都不再与
    // Attempt 根的 brief.md 冲突（workbook 合同 §3.2 末段）。
    // Task: C002-T03
    #[test]
    fn accepts_output_paths_named_brief_or_stats() {
        for legal in [
            "outputs = [{ name = \"o\", path = \"brief.md\" }]",
            "outputs = [{ name = \"o\", path = \"outputs/brief.md\" }]",
            "outputs = [{ name = \"o\", path = \"stats.json\" }]",
            "outputs = [{ name = \"o\", path = \"outputs/stats.json\" }]",
            "outputs = [{ name = \"o\", path = \"out.md\" }]",
        ] {
            assert!(
                parse_flow(&minimal(legal)).is_ok(),
                "{legal} 应当合法（物理路径在 outputs/ 之下）"
            );
        }
    }

    // Task: C002-T03
    #[test]
    fn rejects_output_path_outside_portable_charset() {
        // 只改一个条件：合法路径的一个字符换成汉字。
        for bad in ["说明.md", "out／x.md", "a b.md", "a+b.md"] {
            let text = minimal(&format!("outputs = [{{ name = \"o\", path = \"{bad}\" }}]"));
            assert!(
                matches!(parse_flow(&text), Err(Error::FlowInvalid { rule: "parse", path, .. })
                    if path == "nodes[0].outputs[0].path"),
                "{bad:?} 应当拒绝"
            );
        }
        // 段恰好 128 字节接受，129 字节拒绝。
        let at = "a".repeat(128);
        let over = "a".repeat(129);
        assert!(
            parse_flow(&minimal(&format!(
                "outputs = [{{ name = \"o\", path = \"{at}\" }}]"
            )))
            .is_ok()
        );
        assert!(
            parse_flow(&minimal(&format!(
                "outputs = [{{ name = \"o\", path = \"{over}\" }}]"
            )))
            .is_err()
        );
    }

    // Task: C002-T03
    #[test]
    fn rejects_output_path_duplicate_ancestor_and_folded_alias() {
        let rejects = |a: &str, b: &str| {
            let text = minimal(&format!(
                "outputs = [{{ name = \"p\", path = \"{a}\" }}, {{ name = \"q\", path = \"{b}\" }}]"
            ));
            matches!(parse_flow(&text), Err(Error::FlowInvalid { path, .. }) if path == "nodes[0].outputs[1].path")
        };
        // 完全重复。
        assert!(rejects("out/x.md", "out/x.md"));
        // 互为祖先：out 与 out/sub。
        assert!(rejects("out", "out/sub"));
        assert!(rejects("out/sub", "out"));
        // ASCII 大小写折叠后相同或互为祖先（支持平台默认文件系统大小写不敏感）。
        assert!(rejects("OUT.md", "out.md"));
        assert!(rejects("Out", "out/x.md"));
        // 只改一个条件的正侧：不同名且不互为祖先、大小写不同的段名不同文件。
        assert!(!rejects("out/x.md", "out/y.md"));
        assert!(!rejects("notes/a.md", "notes/a/b.md"));
        assert!(!rejects("Draft.md", "review.md"));
    }

    // Task: T04
    #[test]
    fn rejects_unknown_edge_kind() {
        let text = format!(
            "{}\n[[edges]]\nfrom = \"a\"\nto = \"a\"\nkind = \"sideways\"\n",
            minimal("")
        );
        assert!(parse_flow(&text).is_err());
        let ok = format!(
            "{}\n[[nodes]]\nid = \"b\"\ntitle = \"B\"\nexecutor = \"agent\"\ninstruction = {{ text = \"b\" }}\n[[edges]]\nfrom = \"a\"\nto = \"b\"\nkind = \"re_review\"\n",
            minimal("")
        );
        assert_eq!(parse_flow(&ok).unwrap().edges[0].kind, EdgeKind::ReReview);
    }

    // ── M1 补测（上限：恰好上限接受，多一个字节拒绝） ─────────

    // Task: T04
    #[test]
    fn title_limit_is_128_bytes() {
        let title = |n: usize| {
            minimal("").replace("title = \"A\"", &format!("title = \"{}\"", "a".repeat(n)))
        };
        assert!(parse_flow(&title(128)).is_ok());
        assert!(matches!(
            parse_flow(&title(129)),
            Err(Error::FlowInvalid { path, .. }) if path == "nodes[0].title"
        ));
    }

    // Task: T04
    #[test]
    fn instruction_text_limit_is_8192_bytes() {
        let text = |n: usize| {
            minimal("").replace(
                "{ text = \"做 A\" }",
                &format!("{{ text = \"{}\" }}", "a".repeat(n)),
            )
        };
        assert!(parse_flow(&text(8192)).is_ok());
        assert!(matches!(
            parse_flow(&text(8193)),
            Err(Error::FlowInvalid { path, .. }) if path == "nodes[0].instruction.text"
        ));
    }

    // Task: T04
    #[test]
    fn max_retries_accepts_upper_bound_8() {
        let flow = parse_flow(&minimal("max_retries = 8")).unwrap();
        assert_eq!(flow.nodes[0].max_retries, 8);
    }

    // Task: C004-T01
    #[test]
    fn result_selection_defaults_false_and_accepts_explicit_true() {
        let flow = parse_flow(&minimal(
            "inputs = [{ name = \"task\", from = \"start.task\", result = true }]\noutputs = [{ name = \"chosen\", path = \"chosen.md\", result = true }, { name = \"ordinary\", path = \"ordinary.md\" }]",
        ))
        .unwrap();
        let json = serde_json::to_value(&flow).unwrap();
        assert_eq!(json["nodes"][0]["inputs"][0]["result"], true);
        assert_eq!(json["nodes"][0]["outputs"][0]["result"], true);
        assert_eq!(json["nodes"][0]["outputs"][1]["result"], false);
        let default = parse_flow(&minimal(
            "inputs = [{ name = \"task\", from = \"start.task\" }]",
        ))
        .unwrap();
        assert_eq!(
            serde_json::to_value(default).unwrap()["nodes"][0]["inputs"][0]["result"],
            false
        );
    }

    // Task: C004-T01
    #[test]
    fn result_selection_rejects_non_boolean_and_unknown_slot_fields() {
        for field in ["result = \"true\"", "result = 1", "results = true"] {
            let text = minimal(&format!(
                "outputs = [{{ name = \"chosen\", path = \"chosen.md\", {field} }}]"
            ));
            assert!(matches!(
                parse_flow(&text),
                Err(Error::FlowInvalid { rule: "parse", .. })
            ));
        }
    }
}
