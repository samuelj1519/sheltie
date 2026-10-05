//! Compiled graph and the resource index required for compilation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::def::{EdgeDef, NodeDef};
use crate::ids::NodeId;
use crate::path::RelPath;
use crate::workbook::HostRequire;

/// Runtime-observed Workbook file metadata for core rule 7 existence, size, and encoding checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceMeta {
    pub bytes: u64,
    pub is_utf8: bool,
}

/// Index of every regular Workbook file, keyed by relative path.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceIndex {
    pub files: BTreeMap<RelPath, ResourceMeta>,
}

impl ResourceIndex {
    pub fn get(&self, path: &RelPath) -> Option<&ResourceMeta> {
        self.files.get(path)
    }

    /// Register small UTF-8 fixture files without exposing a production API.
    #[cfg(any(test, feature = "testkit"))]
    pub(crate) fn with_utf8(mut self, path: &str, bytes: u64) -> Self {
        if let Ok(p) = RelPath::new(path) {
            self.files.insert(
                p,
                ResourceMeta {
                    bytes,
                    is_utf8: true,
                },
            );
        }
        self
    }
}

/// Validated graph, constructed only by compile; no Deserialize bypass for validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Graph {
    entry: NodeId,
    /// Declaration order used by status-card pending and visits.
    order: Vec<NodeId>,
    nodes: BTreeMap<NodeId, NodeDef>,
    out_edges: BTreeMap<NodeId, Vec<EdgeDef>>,
    /// All Workbook requires in manifest declaration order, including unused declarations, for work start.
    requires: Vec<HostRequire>,
}

impl Graph {
    /// Called only by compile after every rule passes.
    pub(crate) fn from_checked(
        entry: NodeId,
        nodes: Vec<NodeDef>,
        edges: Vec<EdgeDef>,
        requires: Vec<HostRequire>,
    ) -> Self {
        let mut out_edges: BTreeMap<NodeId, Vec<EdgeDef>> =
            nodes.iter().map(|n| (n.id.clone(), Vec::new())).collect();
        for e in edges {
            out_edges.entry(e.from.clone()).or_default().push(e);
        }
        Self {
            entry,
            order: nodes.iter().map(|n| n.id.clone()).collect(),
            nodes: nodes.into_iter().map(|n| (n.id.clone(), n)).collect(),
            out_edges,
            requires,
        }
    }

    pub fn entry(&self) -> &NodeId {
        &self.entry
    }

    pub fn node(&self, id: &NodeId) -> Option<&NodeDef> {
        self.nodes.get(id)
    }

    /// In declaration order.
    pub fn nodes(&self) -> impl Iterator<Item = &NodeDef> {
        self.order.iter().filter_map(|id| self.nodes.get(id))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// All outgoing edges from a node, in declaration order.
    pub fn out_edges(&self, id: &NodeId) -> &[EdgeDef] {
        self.out_edges.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn edge_count(&self) -> usize {
        self.out_edges.values().map(Vec::len).sum()
    }

    /// Nodes without outgoing edges are terminal.
    pub fn is_terminal(&self, id: &NodeId) -> bool {
        self.out_edges(id).is_empty()
    }

    /// Complete host declarations referenced by this node, preserving reference order.
    pub fn node_requires(&self, node: &NodeId) -> Option<Vec<HostRequire>> {
        Some(
            self.node(node)?
                .requires()
                .iter()
                .filter_map(|(kind, name)| {
                    self.requires
                        .iter()
                        .find(|require| require.kind == *kind && require.name == *name)
                        .cloned()
                })
                .collect(),
        )
    }

    /// All declared Workbook host resources in manifest declaration order.
    pub fn requires(&self) -> &[HostRequire] {
        &self.requires
    }
}
