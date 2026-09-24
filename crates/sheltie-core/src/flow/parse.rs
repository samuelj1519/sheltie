//! Flow TOML 到 `FlowDef` 的解析。只做形状与取值范围，不做图语义（那是 `compile`）。

use serde::Deserialize;

use super::def::{FlowDef, InputSource};
use crate::error::{Error, Result};

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

/// 解析 `inputs[].from` 的三种写法。
///
/// - `start.<key>`：`key` 走 ID 字符规则。
/// - `resource.<path>`：第一个 `.` 之后全部是路径，可含 `/`。
/// - `<node>.<output>`：恰好一个 `.`，两边都非空；`node` 走 ID 规则，`output` 走 ID 规则。
///
/// 失败返回 `Error::FlowInvalid { rule: "parse", path, .. }`，`path` 由调用方传入。
#[allow(unused_variables)]
pub fn parse_input_source(value: &str, path: &str) -> Result<InputSource> {
    todo!("T04")
}

/// DTO 到 `FlowDef` 的逐字段转换与取值校验。这是 T04 要填的函数。
///
/// 要做的检查（都是单字段，不看图）：`schema == "flow/v1"`；各 ID 合规；
/// `executor` 二选一，`tier` 二选一或缺省；`instruction` 恰一个且 `text` 非空、≤ 8 KiB；
/// `inputs[].name` 唯一，`required` 默认 `true`；`outputs[].name` 与 `path` 唯一，
/// `path` 不得是 `brief.md`，`max_bytes` 在 1..=32 MiB；`requires[]` 形如 `kind:name`；
/// `max_visits` 在 1..=32，`max_retries` 在 0..=8；`edges[].kind` 四选一。
#[allow(unused_variables)]
fn convert(dto: FlowDto) -> Result<FlowDef> {
    todo!("T04")
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
    #[ignore = "T04"]
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
    #[ignore = "T04"]
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
    #[ignore = "T04"]
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
    #[ignore = "T04"]
    fn t04_input_from_rejects_three_segments_for_start_and_node() {
        assert!(parse_input_source("start.a.b", "p").is_err());
        assert!(parse_input_source("draft.article.v2", "p").is_err());
        assert!(parse_input_source("draft", "p").is_err());
        assert!(parse_input_source("draft.", "p").is_err());
    }

    #[test]
    #[ignore = "T04"]
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
    #[ignore = "T04"]
    fn t04_node_requires_parses_kind_colon_name() {
        let flow = parse_flow(&minimal("requires = [\"skill:company-api\", \"mcp:db\"]")).unwrap();
        assert_eq!(flow.nodes[0].requires.len(), 2);
        assert_eq!(flow.nodes[0].requires[0].1, "company-api");
    }

    #[test]
    #[ignore = "T04"]
    fn t04_node_requires_rejects_bad_kind() {
        assert!(parse_flow(&minimal("requires = [\"plugin:x\"]")).is_err());
        assert!(parse_flow(&minimal("requires = [\"skill\"]")).is_err());
    }

    #[test]
    #[ignore = "T04"]
    fn t04_input_required_defaults_true_and_parses_false() {
        let flow = parse_flow(&minimal("inputs = [{ name = \"a\", from = \"start.a\" }, { name = \"b\", from = \"x.y\", required = false }]")).unwrap();
        assert!(flow.nodes[0].inputs[0].required);
        assert!(!flow.nodes[0].inputs[1].required);
    }

    #[test]
    #[ignore = "T04"]
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
    #[ignore = "T04"]
    fn t04_defaults_gate_false_visits_1_retries_1() {
        let n = &parse_flow(&minimal("")).unwrap().nodes[0];
        assert!(!n.gate);
        assert_eq!(n.max_visits, 1);
        assert_eq!(n.max_retries, 1);
    }

    #[test]
    #[ignore = "T04"]
    fn t04_rejects_max_visits_zero_or_over_32() {
        assert!(parse_flow(&minimal("max_visits = 0")).is_err());
        assert!(parse_flow(&minimal("max_visits = 33")).is_err());
        assert!(parse_flow(&minimal("max_retries = 9")).is_err());
    }

    #[test]
    #[ignore = "T04"]
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
    #[ignore = "T04"]
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
}
