//! Flow domain types; fields and defaults are in `specs/contracts/workbook.md` §3.2 and §3.3.

use serde::{Deserialize, Serialize};

use crate::ids::{FlowId, NodeId};
use crate::path::RelPath;
use crate::workbook::RequireKind;

/// Executor identity; does not infer gates.
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

/// Model-selection label for the coordinator; the engine passes it through.
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

/// Instruction source: a Workbook file or inline text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Instruction {
    File(RelPath),
    Text(String),
}

/// The four input-source forms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    /// `start.<key>`
    Start { key: String },
    /// `resource.<path>`
    Resource { path: RelPath },
    /// `engine.stats`: bind a file containing the Work's fact view (`render_stats_json`) at begin time.
    EngineStats,
    /// `<node>.<output>`
    Node { node: NodeId, output: String },
}

/// Reserved node IDs: input-source prefixes.
pub const RESERVED_NODE_IDS: &[&str] = &["start", "resource", "engine"];

/// An input declaration; `required = false` applies only to `Node` sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputDecl {
    pub(crate) name: String,
    pub(crate) from: InputSource,
    pub(crate) required: bool,
    pub(crate) result: bool,
}

impl InputDecl {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &InputSource {
        &self.from
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn result(&self) -> bool {
        self.result
    }
}

/// An output declaration; `path` is relative to the Attempt output directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OutputDecl {
    pub(crate) name: String,
    pub(crate) path: RelPath,
    pub(crate) required: bool,
    pub(crate) max_bytes: u64,
    pub(crate) result: bool,
}

impl OutputDecl {
    pub fn result(&self) -> bool {
        self.result
    }

    /// `max_bytes` defaults to 1 MiB.
    pub const DEFAULT_MAX_BYTES: u64 = 1_048_576;
    /// `max_bytes` is capped at 32 MiB.
    pub const MAX_MAX_BYTES: u64 = 33_554_432;
}

/// Edge kind: a label; the engine applies the same legality rules to all four kinds.
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

/// An explicit edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EdgeDef {
    pub(crate) from: NodeId,
    pub(crate) to: NodeId,
    pub(crate) kind: EdgeKind,
}

impl EdgeDef {
    pub fn from(&self) -> &NodeId {
        &self.from
    }

    pub fn to(&self) -> &NodeId {
        &self.to
    }

    pub fn kind(&self) -> EdgeKind {
        self.kind
    }
}

/// A node; external callers may read but cannot modify parsed nodes.
///
/// ```compile_fail
/// use sheltie_core::flow::parse_flow;
/// let flow = parse_flow("schema = \"flow/v1\"\nid = \"f\"\nentry = \"a\"\n[[nodes]]\nid = \"a\"\ntitle = \"A\"\nexecutor = \"agent\"\ninstruction = { text = \"x\" }").unwrap();
/// let mut node = flow.nodes()[0].clone();
/// node.gate = true;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NodeDef {
    pub(crate) id: NodeId,
    pub(crate) title: String,
    pub(crate) executor: Executor,
    /// Human tier is `None` (rule 9 prohibits declaration); agent tier defaults to `Standard`.
    pub(crate) tier: Option<Tier>,
    pub(crate) instruction: Instruction,
    pub(crate) inputs: Vec<InputDecl>,
    pub(crate) outputs: Vec<OutputDecl>,
    /// `kind:name` must match a manifest `requires` declaration.
    pub(crate) requires: Vec<(RequireKind, String)>,
    pub(crate) gate: bool,
    pub(crate) max_visits: u32,
    pub(crate) max_retries: u32,
}

impl NodeDef {
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn executor(&self) -> Executor {
        self.executor
    }

    pub fn gate(&self) -> bool {
        self.gate
    }

    pub fn instruction(&self) -> &Instruction {
        &self.instruction
    }

    pub fn tier(&self) -> Option<Tier> {
        self.tier
    }

    pub fn inputs(&self) -> &[InputDecl] {
        &self.inputs
    }

    pub fn requires(&self) -> &[(RequireKind, String)] {
        &self.requires
    }

    pub const DEFAULT_MAX_VISITS: u32 = 1;
    pub const MAX_MAX_VISITS: u32 = 32;
    pub const DEFAULT_MAX_RETRIES: u32 = 1;
    pub const MAX_MAX_RETRIES: u32 = 8;
    /// `instruction.text` ≤ 8 KiB。
    pub const TEXT_MAX_BYTES: usize = 8192;
    /// `instruction.file` ≤ 64 KiB。
    pub const FILE_MAX_BYTES: u64 = 65_536;
    /// `title` is at most 128 bytes.
    pub const TITLE_MAX_BYTES: usize = 128;

    pub fn output(&self, name: &str) -> Option<&OutputDecl> {
        self.outputs.iter().find(|o| o.name == name)
    }
}

/// A parsed, uncompiled Flow; external callers cannot mutate nodes or edges after parsing.
///
/// ```compile_fail
/// use sheltie_core::flow::parse_flow;
/// let mut flow = parse_flow("schema = \"flow/v1\"\nid = \"f\"\nentry = \"a\"\n[[nodes]]\nid = \"a\"\ntitle = \"A\"\nexecutor = \"agent\"\ninstruction = { text = \"x\" }").unwrap();
/// flow.nodes.clear();
/// ```
///
/// ```compile_fail
/// use sheltie_core::flow::FlowDef;
/// let _: FlowDef = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FlowDef {
    pub(crate) id: FlowId,
    pub(crate) entry: NodeId,
    pub(crate) nodes: Vec<NodeDef>,
    pub(crate) edges: Vec<EdgeDef>,
}

impl FlowDef {
    pub fn id(&self) -> &FlowId {
        &self.id
    }

    pub fn entry(&self) -> &NodeId {
        &self.entry
    }

    pub fn nodes(&self) -> &[NodeDef] {
        &self.nodes
    }

    pub fn edges(&self) -> &[EdgeDef] {
        &self.edges
    }

    pub const MAX_NODES: usize = 64;
    pub const MAX_EDGES: usize = 256;

    pub fn node(&self, id: &NodeId) -> Option<&NodeDef> {
        self.nodes.iter().find(|n| &n.id == id)
    }
}
