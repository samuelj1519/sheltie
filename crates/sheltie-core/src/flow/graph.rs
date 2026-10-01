//! 编译通过后的图，以及编译需要的资源索引。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::def::{EdgeDef, NodeDef};
use crate::ids::NodeId;
use crate::path::RelPath;
use crate::workbook::HostRequire;

/// runtime 观察到的 Workbook 内文件元数据。core 用它做规则 7 的存在性、大小、编码检查。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceMeta {
    pub bytes: u64,
    pub is_utf8: bool,
}

/// Workbook 目录里全部普通文件的索引，键是相对路径。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceIndex {
    pub files: BTreeMap<RelPath, ResourceMeta>,
}

impl ResourceIndex {
    pub fn get(&self, path: &RelPath) -> Option<&ResourceMeta> {
        self.files.get(path)
    }

    /// 测试夹具登记 UTF-8 小文件，不暴露为生产 API。
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

/// 校验通过的图。只能由 `compile` 构造，不派生 `Deserialize`，读出来也绕不过校验。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Graph {
    entry: NodeId,
    /// 声明顺序。状态卡的 `pending` 与 `visits` 按它排。
    order: Vec<NodeId>,
    nodes: BTreeMap<NodeId, NodeDef>,
    out_edges: BTreeMap<NodeId, Vec<EdgeDef>>,
    /// Workbook 声明的**全部** `requires`，按 manifest 声明顺序，不止本图用到的（work start 用它）。
    requires: Vec<HostRequire>,
}

impl Graph {
    /// 仅供 `compile` 在全部规则通过后调用。
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

    /// 按声明顺序。
    pub fn nodes(&self) -> impl Iterator<Item = &NodeDef> {
        self.order.iter().filter_map(|id| self.nodes.get(id))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 从某节点出发的全部边，按声明顺序。
    pub fn out_edges(&self, id: &NodeId) -> &[EdgeDef] {
        self.out_edges.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn edge_count(&self) -> usize {
        self.out_edges.values().map(Vec::len).sum()
    }

    /// 没有出边的节点是终点。
    pub fn is_terminal(&self, id: &NodeId) -> bool {
        self.out_edges(id).is_empty()
    }

    /// 本节点引用的完整宿主声明，保留节点引用顺序。
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

    /// Workbook 声明的全部宿主资源，按 manifest 声明顺序。
    pub fn requires(&self) -> &[HostRequire] {
        &self.requires
    }
}
