//! Flow：解析（`parse`）、编译成图（`compile`）、图本身（`graph`）。
//! 格式与规则见 `specs/contracts/workbook.md` §3、§4。

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
