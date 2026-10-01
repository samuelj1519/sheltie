//! Work 状态机。类型见 `specs/architecture.md` §2，转换见 §3，操作细则见 `specs/contracts/protocol.md` §3。
//!
//! 唯一入口是 [`decide`]：给定当前状态、图、一个命令与上下文，返回新状态、效果与回复。
//! [`legal_next`] 算当前合法下一步。[`render`] 把状态渲染成任务书与状态卡。

pub mod command;
pub mod decide;
pub mod layout;
pub mod next;
pub mod render;
pub mod start;
pub mod state;

pub use command::{Command, Context, Decision, Effect, ObservedFile, Reply};
pub use decide::{decide, input_paths_for, output_paths_for, reply_status_matches};
pub use layout::{
    attempt_dir, brief_path, engine_stats_path, output_path, outputs_dir, start_input_path,
    start_inputs_dir, status_card_path, workbook_copy_dir,
};
pub use next::{NextOp, legal_next};
pub use render::{
    NodeStatsJson, StatsJson, StatusCardJson, render_brief, render_stats, render_stats_json,
    render_status_card, status_card_json,
};
pub use start::{start_requirements, validate_start_inputs};
pub use state::{
    Approval, ArtifactRef, Attempt, AttemptStatus, BlockedReason, Occurrence, Principal, Timestamp,
    WorkState, WorkStatus, WorkbookRef,
};
