//! Work state machine: architecture §2 types, §3 transitions, and protocol §3 operation rules.
//!
//! [`decide`] is the sole entry: current state, graph, command, and context yield new state, effects, and reply.
//! [`legal_next`] computes legal actions; [`render`] produces briefs and status cards.

pub mod command;
pub mod decide;
pub mod layout;
pub mod next;
pub mod render;
pub mod result;
pub mod start;
pub mod state;

pub use command::{Command, Context, Decision, Effect, ObservedFile, Reply};
pub use decide::{
    decide, input_paths_for, output_paths_for, replacement_input_paths_for, reply_status_matches,
};
pub use layout::{
    attempt_dir, brief_path, engine_stats_path, output_path, outputs_dir, start_input_path,
    start_inputs_dir, status_card_path, workbook_copy_dir,
};
pub use next::{NextOp, legal_next};
pub use render::{
    NodeStatsJson, StatsJson, StatusCardJson, render_brief, render_stats, render_stats_json,
    render_status_card, status_card_json,
};
pub use result::{
    ResultArtifact, ResultSlotKind, ResultSource, ResultView, render_result, result_view,
};
pub use start::{start_requirements, validate_start_inputs};
pub use state::{
    Approval, ArtifactRef, Attempt, AttemptStatus, BlockedReason, Occurrence, Principal, Timestamp,
    WorkState, WorkStatus, WorkbookRef,
};
