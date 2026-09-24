//! Flow TOML 到 `FlowDef` 的解析。只做形状与取值范围，不做图语义（那是 `compile`）。

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
/// `inputs[].name` 唯一，`required` 默认 `true`；`outputs[].name` 与 `path` 唯一，
/// `path` 不得是 `brief.md`，`max_bytes` 在 1..=32 MiB；`requires[]` 形如 `kind:name`；
/// `max_visits` 在 1..=32，`max_retries` 在 0..=8；`edges[].kind` 四选一。
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
            if path.as_str() == "brief.md" {
                return Err(invalid(op("path"), "不得与 brief.md 相同".to_string()));
            }
            if outputs.iter().any(|x: &OutputDecl| x.path == path) {
                return Err(invalid(op("path"), "节点内输出路径重复".to_string()));
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

    #[test]
    fn t04_parses_three_node_flow() {
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

    #[test]
    fn t04_instruction_requires_exactly_one_of_file_or_text() {
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

    #[test]
    fn t04_input_from_parses_start_resource_and_node_forms() {
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

    #[test]
    fn t04_input_from_parses_engine_stats_only() {
        assert_eq!(
            parse_input_source("engine.stats", "p").unwrap(),
            InputSource::EngineStats
        );
        assert!(parse_input_source("engine.clock", "p").is_err());
        assert!(parse_input_source("engine", "p").is_err());
    }

    #[test]
    fn t04_input_from_rejects_three_segments_for_start_and_node() {
        assert!(parse_input_source("start.a.b", "p").is_err());
        assert!(parse_input_source("draft.article.v2", "p").is_err());
        assert!(parse_input_source("draft", "p").is_err());
        assert!(parse_input_source("draft.", "p").is_err());
    }

    #[test]
    fn t04_input_from_resource_keeps_slashes_in_path() {
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

    #[test]
    fn t04_node_requires_parses_kind_colon_name() {
        let flow = parse_flow(&minimal("requires = [\"skill:company-api\", \"mcp:db\"]")).unwrap();
        assert_eq!(flow.nodes[0].requires.len(), 2);
        assert_eq!(flow.nodes[0].requires[0].1, "company-api");
    }

    #[test]
    fn t04_node_requires_rejects_bad_kind() {
        assert!(parse_flow(&minimal("requires = [\"plugin:x\"]")).is_err());
        assert!(parse_flow(&minimal("requires = [\"skill\"]")).is_err());
    }

    #[test]
    fn t04_input_required_defaults_true_and_parses_false() {
        let flow = parse_flow(&minimal("inputs = [{ name = \"a\", from = \"start.a\" }, { name = \"b\", from = \"x.y\", required = false }]")).unwrap();
        assert!(flow.nodes[0].inputs[0].required);
        assert!(!flow.nodes[0].inputs[1].required);
    }

    #[test]
    fn t04_tier_defaults_standard_and_parses_strong() {
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

    #[test]
    fn t04_defaults_gate_false_visits_1_retries_1() {
        let n = &parse_flow(&minimal("")).unwrap().nodes[0];
        assert!(!n.gate);
        assert_eq!(n.max_visits, 1);
        assert_eq!(n.max_retries, 1);
    }

    #[test]
    fn t04_rejects_max_visits_zero_or_over_32() {
        assert!(parse_flow(&minimal("max_visits = 0")).is_err());
        assert!(parse_flow(&minimal("max_visits = 33")).is_err());
        assert!(parse_flow(&minimal("max_retries = 9")).is_err());
    }

    #[test]
    fn t04_rejects_output_path_brief_md() {
        assert!(
            parse_flow(&minimal(
                "outputs = [{ name = \"o\", path = \"brief.md\" }]"
            ))
            .is_err()
        );
        assert!(parse_flow(&minimal("outputs = [{ name = \"o\", path = \"out.md\" }]")).is_ok());
    }

    #[test]
    fn t04_rejects_unknown_edge_kind() {
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

    #[test]
    fn t04_title_limit_is_128_bytes() {
        let title = |n: usize| {
            minimal("").replace("title = \"A\"", &format!("title = \"{}\"", "a".repeat(n)))
        };
        assert!(parse_flow(&title(128)).is_ok());
        assert!(matches!(
            parse_flow(&title(129)),
            Err(Error::FlowInvalid { path, .. }) if path == "nodes[0].title"
        ));
    }

    #[test]
    fn t04_instruction_text_limit_is_8192_bytes() {
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

    #[test]
    fn t04_max_retries_accepts_upper_bound_8() {
        let flow = parse_flow(&minimal("max_retries = 8")).unwrap();
        assert_eq!(flow.nodes[0].max_retries, 8);
    }
}
