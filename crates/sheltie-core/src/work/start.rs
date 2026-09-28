//! `work start` 的共享事实源（GF-30）：收集与校验起始输入键。
//!
//! `decide`、runtime preflight 与 `workbook show` 用同一份结果，不各自重算
//! （架构 §3；缺/多键的确定性拒绝发生在序号分配与目录物化之前，协议 `work start`）。

use std::collections::BTreeSet;

use crate::error::{Error, Result};
use crate::flow::{Graph, InputSource};

/// 图里全部 `start.<key>` 引用，按节点声明顺序首次出现（重复引用只记一次）。
/// `workbook show` 的 `start_inputs` 与它同源。
pub fn start_requirements(graph: &Graph) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for node in graph.nodes() {
        for decl in node.inputs() {
            if let InputSource::Start { key } = decl.source() {
                if seen.insert(key.clone()) {
                    out.push(key.clone());
                }
            }
        }
    }
    out
}

/// 校验给出的键集合与 [`start_requirements`] 恰好相等：缺与多都是 `Error::InputMissing`。
/// `missing` 与 `extra` 按字典序，保证同一输入的错误信息稳定。
pub fn validate_start_inputs(
    graph: &Graph,
    provided: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<()> {
    let wanted: BTreeSet<String> = start_requirements(graph).into_iter().collect();
    let provided: BTreeSet<String> = provided
        .into_iter()
        .map(|key| key.as_ref().to_string())
        .collect();
    let missing: Vec<String> = wanted.difference(&provided).cloned().collect();
    let extra: Vec<String> = provided.difference(&wanted).cloned().collect();
    if !missing.is_empty() || !extra.is_empty() {
        return Err(Error::InputMissing { missing, extra });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::{ResourceIndex, compile, parse_flow};
    use crate::workbook::parse_manifest;

    /// 手写 TOML 构图，期望值不经过被测函数。`first` 引用 `start.b`，`second` 引用
    /// `start.a` 与 `start.b`（重复引用）：声明顺序是 b、a。
    fn graph_with_start_keys() -> Graph {
        let manifest = parse_manifest(
            "schema = \"workbook/v1\"\nid = \"keys\"\nversion = \"1.0.0\"\nname = \"键\"\nflows = [\"flows/default.toml\"]\n",
        )
        .unwrap();
        let def = parse_flow(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"first\"\n\n\
             [[nodes]]\nid = \"first\"\ntitle = \"一\"\nexecutor = \"agent\"\ninstruction = { text = \"做一。\" }\n\
             inputs = [{ name = \"in_b\", from = \"start.b\" }]\n\n\
             [[nodes]]\nid = \"second\"\ntitle = \"二\"\nexecutor = \"agent\"\ninstruction = { text = \"做二。\" }\n\
             inputs = [{ name = \"in_a\", from = \"start.a\" }, { name = \"in_b2\", from = \"start.b\" }]\n\n\
             [[edges]]\nfrom = \"first\"\nto = \"second\"\nkind = \"main\"",
        )
        .unwrap();
        compile(&def, &manifest, &ResourceIndex::default()).unwrap()
    }

    fn graph_without_start_keys() -> Graph {
        let manifest = parse_manifest(
            "schema = \"workbook/v1\"\nid = \"none\"\nversion = \"1.0.0\"\nname = \"无键\"\nflows = [\"flows/default.toml\"]\n",
        )
        .unwrap();
        let def = parse_flow(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n\
             [[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = { text = \"做。\" }",
        )
        .unwrap();
        compile(&def, &manifest, &ResourceIndex::default()).unwrap()
    }

    // Task: C002-T02
    #[test]
    fn start_requirements_lists_keys_in_declaration_order_deduplicated() {
        // first 声明 start.b，second 声明 start.a 再引用 start.b；首次出现顺序是 b、a。
        assert_eq!(
            start_requirements(&graph_with_start_keys()),
            vec!["b".to_string(), "a".to_string()]
        );
        assert_eq!(
            start_requirements(&graph_without_start_keys()),
            Vec::<String>::new()
        );
    }

    // Task: C002-T02
    #[test]
    fn validate_accepts_exact_key_set_in_any_order() {
        let graph = graph_with_start_keys();
        assert!(validate_start_inputs(&graph, ["b", "a"]).is_ok());
        assert!(validate_start_inputs(&graph, ["a", "b"]).is_ok());
    }

    // Task: C002-T02
    #[test]
    fn validate_rejects_missing_extra_and_both_sorted() {
        let graph = graph_with_start_keys();
        // 只改一个条件：少给 b。
        let err = validate_start_inputs(&graph, ["a"]).unwrap_err();
        assert!(
            matches!(&err, Error::InputMissing { missing, extra }
                if missing == &["b".to_string()] && extra.is_empty()),
            "{err:?}"
        );
        // 只改一个条件：多给 c。
        let err = validate_start_inputs(&graph, ["a", "b", "c"]).unwrap_err();
        assert!(
            matches!(&err, Error::InputMissing { missing, extra }
                if missing.is_empty() && extra == &["c".to_string()]),
            "{err:?}"
        );
        // 全换：缺 a、b，多 c；两个列表都按字典序。
        let err = validate_start_inputs(&graph, ["c"]).unwrap_err();
        assert!(
            matches!(&err, Error::InputMissing { missing, extra }
                if missing == &["a".to_string(), "b".to_string()] && extra == &["c".to_string()]),
            "{err:?}"
        );
    }

    // Task: C002-T02
    #[test]
    fn validate_on_keyless_flow_accepts_empty_and_rejects_any_key() {
        let graph = graph_without_start_keys();
        assert!(validate_start_inputs(&graph, Vec::<&str>::new()).is_ok());
        let err = validate_start_inputs(&graph, ["x"]).unwrap_err();
        assert!(
            matches!(&err, Error::InputMissing { missing, extra } if missing.is_empty() && extra == &["x".to_string()]),
            "{err:?}"
        );
    }
}
