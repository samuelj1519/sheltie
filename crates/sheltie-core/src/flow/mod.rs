//! Flow parsing (`parse`), compilation (`compile`), and graph representation (`graph`).
//! Formats and rules: `specs/contracts/workbook.md` §3 and §4.

pub mod compile;
pub mod def;
pub mod graph;
pub mod parse;

pub use compile::compile;
pub use def::{
    EdgeDef, EdgeKind, Executor, FlowDef, InputDecl, InputSource, Instruction, NodeDef, OutputDecl,
    Tier,
};
pub use graph::{Graph, ResourceIndex, ResourceMeta};
pub use parse::parse_flow;
