//! Work 状态机。类型见 `specs/architecture.md` §2，转换见 §3，操作细则见 `specs/contracts/protocol.md` §3。
//!
//! 唯一入口是 [`decide`]：给定当前状态、图、一个命令与上下文，返回新状态、效果与回复。
//! [`legal_next`] 算当前合法下一步。[`render`] 把状态投影成任务书与状态卡。

pub mod command;
pub mod decide;
pub mod next;
pub mod render;
pub mod state;

pub use command::{Command, Context, Decision, Effect, ObservedFile, Reply};
pub use decide::{decide, input_paths_for, output_paths_for};
pub use next::{NextOp, legal_next};
pub use render::{StatusCardJson, render_brief, render_status_card, status_card_json};
pub use state::{
    Approval, ArtifactRef, Attempt, AttemptStatus, BlockedReason, Occurrence, Principal, Timestamp,
    WorkState, WorkStatus, WorkbookRef,
};
