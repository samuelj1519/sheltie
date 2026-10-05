//! Shared work start facts (GF-30): collect and validate start input keys.
//!
//! decide, runtime preflight, and workbook show share one result rather than recomputing it
//! (architecture §3); missing/extra keys reject deterministically before allocating sequence or materializing directories.

use std::collections::BTreeSet;

use crate::error::{Error, Result};
use crate::flow::{Graph, InputSource};

/// All start.<key> references in first-observed node declaration order, deduplicated.
/// Shares the source for workbook show's start_inputs.
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

/// Require exactly [`start_requirements`]; missing or extra keys yield Error::InputMissing.
/// Sort missing and extra lexicographically for stable diagnostics.
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

    /// Handwritten TOML graph and independent expectations: first references start.b, second references
    /// start.a and start.b again; declaration order is b, a.
    fn graph_with_start_keys() -> Graph {
        let manifest = parse_manifest(
            "schema = \"workbook/v1\"\nid = \"keys\"\nversion = \"1.0.0\"\nname = \"Keys\"\nflows = [\"flows/default.toml\"]\n",
        )
        .unwrap();
        let def = parse_flow(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"first\"\n\n\
             [[nodes]]\nid = \"first\"\ntitle = \"First\"\nexecutor = \"agent\"\ninstruction = { text = \"Do the first task.\" }\n\
             inputs = [{ name = \"in_b\", from = \"start.b\" }]\n\n\
             [[nodes]]\nid = \"second\"\ntitle = \"Second\"\nexecutor = \"agent\"\ninstruction = { text = \"Do the second task.\" }\n\
             inputs = [{ name = \"in_a\", from = \"start.a\" }, { name = \"in_b2\", from = \"start.b\" }]\n\n\
             [[edges]]\nfrom = \"first\"\nto = \"second\"\nkind = \"main\"",
        )
        .unwrap();
        compile(&def, &manifest, &ResourceIndex::default()).unwrap()
    }

    fn graph_without_start_keys() -> Graph {
        let manifest = parse_manifest(
            "schema = \"workbook/v1\"\nid = \"none\"\nversion = \"1.0.0\"\nname = \"No keys\"\nflows = [\"flows/default.toml\"]\n",
        )
        .unwrap();
        let def = parse_flow(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n\
             [[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = { text = \"Do the task.\" }",
        )
        .unwrap();
        compile(&def, &manifest, &ResourceIndex::default()).unwrap()
    }

    // Task: C002-T02
    #[test]
    fn start_requirements_lists_keys_in_declaration_order_deduplicated() {
        // first declares start.b; second declares start.a then repeats start.b; first-observed order is b, a.
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
        // One changed condition: omit b.
        let err = validate_start_inputs(&graph, ["a"]).unwrap_err();
        assert!(
            matches!(&err, Error::InputMissing { missing, extra }
                if missing == &["b".to_string()] && extra.is_empty()),
            "{err:?}"
        );
        // One changed condition: add c.
        let err = validate_start_inputs(&graph, ["a", "b", "c"]).unwrap_err();
        assert!(
            matches!(&err, Error::InputMissing { missing, extra }
                if missing.is_empty() && extra == &["c".to_string()]),
            "{err:?}"
        );
        // Replace all: missing a/b, extra c; both lists are lexicographically ordered.
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
