//! Flow 的领域类型。字段与默认值见 `specs/contracts/workbook.md` §3.2、§3.3。

use serde::{Deserialize, Serialize};

use crate::ids::{FlowId, NodeId};
use crate::path::RelPath;
use crate::workbook::RequireKind;

/// 谁干活。只表示执行者，不推断门槛。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Executor {
    Agent,
    Human,
}

impl Executor {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Human => "human",
        }
    }
}

/// 给协调者选模型的标签。引擎只透传。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Strong,
    #[default]
    Standard,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strong => "strong",
            Self::Standard => "standard",
        }
    }
}

/// 说明书来源：Workbook 内文件，或内联文本。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Instruction {
    File(RelPath),
    Text(String),
}

/// 输入来源的四种写法。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    /// `start.<key>`
    Start { key: String },
    /// `resource.<path>`
    Resource { path: RelPath },
    /// `engine.stats`：引擎在开工时把本 Work 的事实视图（`render_stats_json`）写成文件绑进来。
    EngineStats,
    /// `<node>.<output>`
    Node { node: NodeId, output: String },
}

/// 节点 id 的保留字：输入来源的前缀。
pub const RESERVED_NODE_IDS: &[&str] = &["start", "resource", "engine"];

/// 一条输入声明。`required = false` 只对 `Node` 来源有意义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputDecl {
    pub name: String,
    pub from: InputSource,
    pub required: bool,
}

/// 一条输出声明。`path` 相对 Attempt 目录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDecl {
    pub name: String,
    pub path: RelPath,
    pub required: bool,
    pub max_bytes: u64,
}

impl OutputDecl {
    /// `max_bytes` 默认 1 MiB。
    pub const DEFAULT_MAX_BYTES: u64 = 1_048_576;
    /// `max_bytes` 上限 32 MiB。
    pub const MAX_MAX_BYTES: u64 = 33_554_432;
}

/// 边类型。对引擎只是标签，四种边的合法性判断相同。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Main,
    Back,
    Branch,
    ReReview,
}

impl EdgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Back => "back",
            Self::Branch => "branch",
            Self::ReReview => "re_review",
        }
    }
}

/// 一条显式边。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeDef {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

/// 一个节点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDef {
    pub id: NodeId,
    pub title: String,
    pub executor: Executor,
    /// `human` 节点为 `None`（规则 9 禁止声明）；`agent` 节点默认 `Standard`。
    pub tier: Option<Tier>,
    pub instruction: Instruction,
    pub inputs: Vec<InputDecl>,
    pub outputs: Vec<OutputDecl>,
    /// `kind:name`，必须对应 manifest 的一条 `requires`。
    pub requires: Vec<(RequireKind, String)>,
    pub gate: bool,
    pub max_visits: u32,
    pub max_retries: u32,
}

impl NodeDef {
    pub const DEFAULT_MAX_VISITS: u32 = 1;
    pub const MAX_MAX_VISITS: u32 = 32;
    pub const DEFAULT_MAX_RETRIES: u32 = 1;
    pub const MAX_MAX_RETRIES: u32 = 8;
    /// `instruction.text` ≤ 8 KiB。
    pub const TEXT_MAX_BYTES: usize = 8192;
    /// `instruction.file` ≤ 64 KiB。
    pub const FILE_MAX_BYTES: u64 = 65_536;
    /// `title` ≤ 128 字节。
    pub const TITLE_MAX_BYTES: usize = 128;

    pub fn output(&self, name: &str) -> Option<&OutputDecl> {
        self.outputs.iter().find(|o| o.name == name)
    }
}

/// 解析后、编译前的 Flow。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowDef {
    pub id: FlowId,
    pub entry: NodeId,
    pub nodes: Vec<NodeDef>,
    pub edges: Vec<EdgeDef>,
}

impl FlowDef {
    pub const MAX_NODES: usize = 64;
    pub const MAX_EDGES: usize = 256;

    pub fn node(&self, id: &NodeId) -> Option<&NodeDef> {
        self.nodes.iter().find(|n| &n.id == id)
    }
}
