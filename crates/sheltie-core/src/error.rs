//! Error types and codes.
//!
//! The closed error-code set is defined in `specs/contracts/protocol.md` §7; core emits rule-related errors,
//! while `sheltie-runtime` defines storage, update, and filesystem errors mapped to the same codes.

use serde::Serialize;

use crate::ids::{AttemptId, NodeId};

/// All protocol §7 error codes; CLI uses `as_str` to emit `SCREAMING_SNAKE_CASE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidRequest,
    NotFound,
    WorkbookInvalid,
    FlowInvalid,
    WorkbookExists,
    WorkbookInUse,
    WorkbookTampered,
    UpdateUnavailable,
    UpdateChecksumMismatch,
    InputMissing,
    WorkTerminal,
    IllegalNext,
    InputUnavailable,
    ArtifactModified,
    AttemptNotRunning,
    ReplacementsExhausted,
    SummaryTooLong,
    OutputMissing,
    OutputTooLarge,
    RequestConflict,
    RevisionConflict,
    EffectPending,
    StoreSchemaMismatch,
    StoreCorrupt,
    Io,
}

impl ErrorCode {
    /// Protocol literal, such as `ILLEGAL_NEXT`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::NotFound => "NOT_FOUND",
            Self::WorkbookInvalid => "WORKBOOK_INVALID",
            Self::FlowInvalid => "FLOW_INVALID",
            Self::WorkbookExists => "WORKBOOK_EXISTS",
            Self::WorkbookInUse => "WORKBOOK_IN_USE",
            Self::WorkbookTampered => "WORKBOOK_TAMPERED",
            Self::UpdateUnavailable => "UPDATE_UNAVAILABLE",
            Self::UpdateChecksumMismatch => "UPDATE_CHECKSUM_MISMATCH",
            Self::InputMissing => "INPUT_MISSING",
            Self::WorkTerminal => "WORK_TERMINAL",
            Self::IllegalNext => "ILLEGAL_NEXT",
            Self::InputUnavailable => "INPUT_UNAVAILABLE",
            Self::ArtifactModified => "ARTIFACT_MODIFIED",
            Self::AttemptNotRunning => "ATTEMPT_NOT_RUNNING",
            Self::ReplacementsExhausted => "REPLACEMENTS_EXHAUSTED",
            Self::SummaryTooLong => "SUMMARY_TOO_LONG",
            Self::OutputMissing => "OUTPUT_MISSING",
            Self::OutputTooLarge => "OUTPUT_TOO_LARGE",
            Self::RequestConflict => "REQUEST_CONFLICT",
            Self::RevisionConflict => "REVISION_CONFLICT",
            Self::EffectPending => "EFFECT_PENDING",
            Self::StoreSchemaMismatch => "STORE_SCHEMA_MISMATCH",
            Self::StoreCorrupt => "STORE_CORRUPT",
            Self::Io => "IO",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Core errors; each variant includes enough context to locate the problem.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Invalid ID; `field` is a field path such as `nodes[2].id`.
    #[error("Invalid {field} value {value:?}: {reason}")]
    InvalidId {
        field: String,
        value: String,
        reason: &'static str,
    },
    /// A relative path contains `..`, an absolute prefix, or empty segments; or an absolute path is not absolute.
    #[error("Invalid path {path:?}: {reason}")]
    InvalidPath { path: String, reason: &'static str },
    /// Bounded text exceeds its byte limit.
    #[error("{field} exceeds {max} bytes; actual {actual}")]
    TextTooLong {
        field: &'static str,
        max: usize,
        actual: usize,
    },
    /// Digest is not 64 lowercase hexadecimal digits.
    #[error("Digest {value:?} must be 64 lowercase hexadecimal digits")]
    InvalidDigest { value: String },
    /// Invalid `workbook.toml` (protocol `WORKBOOK_INVALID`).
    #[error("Invalid workbook.toml {field}: {reason}")]
    WorkbookInvalid { field: String, reason: String },
    /// Flow parse or compilation failure (`FLOW_INVALID`); `rule` is the contract §4 number, or `"parse"`.
    #[error("Invalid Flow (rule {rule}, {path}): {reason}")]
    FlowInvalid {
        rule: &'static str,
        path: String,
        reason: String,
    },
    /// `work start` inputs contain missing or extra keys.
    #[error("Start inputs: missing {missing:?}, extra {extra:?}")]
    InputMissing {
        missing: Vec<String>,
        extra: Vec<String>,
    },
    /// Work is already terminal.
    #[error("Work has ended ({status})")]
    WorkTerminal { status: String },
    /// The operation is absent from current `next`, which lists legal actions as command lines.
    #[error("{requested} is not a legal next action")]
    IllegalNext {
        requested: String,
        next: Vec<String>,
    },
    /// The upstream node has not successfully produced a required input.
    #[error("Upstream node {node} has not successfully produced input {input}")]
    InputUnavailable { input: String, node: NodeId },
    /// The input file's current digest differs from its recorded digest.
    #[error("Input {input} file {path} has been modified")]
    ArtifactModified { input: String, path: String },
    /// Submission or failure reporting for a non-`running` Attempt.
    #[error("Attempt {attempt} is not running")]
    AttemptNotRunning { attempt: AttemptId },
    #[error("Attempt {attempt} does not exist")]
    AttemptNotFound { attempt: AttemptId },
    #[error("The Occurrence of Attempt {attempt} has exhausted its replacement allowance")]
    ReplacementsExhausted { attempt: AttemptId },
    /// Summary exceeds 4096 bytes.
    #[error("Summary exceeds {max} bytes; actual {actual}")]
    SummaryTooLong { max: usize, actual: usize },
    /// A required output file is missing.
    #[error("Required output {output} does not exist: {path}")]
    OutputMissing { output: String, path: String },
    /// Output exceeds `max_bytes`.
    #[error("Output {output} exceeds {max_bytes} bytes; actual {actual}")]
    OutputTooLarge {
        output: String,
        max_bytes: u64,
        actual: u64,
    },
    /// Other format or value errors.
    #[error("{reason}")]
    InvalidRequest { reason: String },
}

impl Error {
    /// Map to a protocol error code.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidId { .. }
            | Self::InvalidPath { .. }
            | Self::TextTooLong { .. }
            | Self::InvalidDigest { .. }
            | Self::InvalidRequest { .. } => ErrorCode::InvalidRequest,
            Self::WorkbookInvalid { .. } => ErrorCode::WorkbookInvalid,
            Self::FlowInvalid { .. } => ErrorCode::FlowInvalid,
            Self::InputMissing { .. } => ErrorCode::InputMissing,
            Self::WorkTerminal { .. } => ErrorCode::WorkTerminal,
            Self::IllegalNext { .. } => ErrorCode::IllegalNext,
            Self::InputUnavailable { .. } => ErrorCode::InputUnavailable,
            Self::ArtifactModified { .. } => ErrorCode::ArtifactModified,
            Self::AttemptNotRunning { .. } => ErrorCode::AttemptNotRunning,
            Self::AttemptNotFound { .. } => ErrorCode::NotFound,
            Self::ReplacementsExhausted { .. } => ErrorCode::ReplacementsExhausted,
            Self::SummaryTooLong { .. } => ErrorCode::SummaryTooLong,
            Self::OutputMissing { .. } => ErrorCode::OutputMissing,
            Self::OutputTooLarge { .. } => ErrorCode::OutputTooLarge,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
