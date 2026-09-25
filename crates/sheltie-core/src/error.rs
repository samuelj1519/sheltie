//! 错误类型与错误码。
//!
//! 错误码闭集见 `specs/contracts/protocol.md` §7。core 只产生其中与规则有关的那部分；
//! 存储、更新、文件系统的错误由 `sheltie-runtime` 定义并映射到同一组码。

use serde::Serialize;

use crate::ids::{AttemptId, NodeId};

/// 协议 §7 的全部错误码。CLI 用 `as_str` 输出 `SCREAMING_SNAKE_CASE`。
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
    SummaryTooLong,
    OutputMissing,
    OutputTooLarge,
    RequestConflict,
    RevisionConflict,
    StoreSchemaMismatch,
    StoreCorrupt,
    Io,
}

impl ErrorCode {
    /// 协议里的字面形式，例如 `ILLEGAL_NEXT`。
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
            Self::SummaryTooLong => "SUMMARY_TOO_LONG",
            Self::OutputMissing => "OUTPUT_MISSING",
            Self::OutputTooLarge => "OUTPUT_TOO_LARGE",
            Self::RequestConflict => "REQUEST_CONFLICT",
            Self::RevisionConflict => "REVISION_CONFLICT",
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

/// core 产生的错误。每个变体带足够定位的字段。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// ID 不合规。`field` 是字段路径，例如 `nodes[2].id`。
    #[error("{field} 的值 {value:?} 不合规：{reason}")]
    InvalidId {
        field: String,
        value: String,
        reason: &'static str,
    },
    /// 相对路径含 `..`、绝对、空段，或绝对路径不是绝对的。
    #[error("路径 {path:?} 不合规：{reason}")]
    InvalidPath { path: String, reason: &'static str },
    /// 有界文本超限。按字节计。
    #[error("{field} 超过 {max} 字节，实际 {actual}")]
    TextTooLong {
        field: &'static str,
        max: usize,
        actual: usize,
    },
    /// 摘要不是 64 位小写十六进制。
    #[error("摘要 {value:?} 不是 64 位小写十六进制")]
    InvalidDigest { value: String },
    /// `workbook.toml` 不合规（协议 `WORKBOOK_INVALID`）。
    #[error("workbook.toml 的 {field} 不合规：{reason}")]
    WorkbookInvalid { field: String, reason: String },
    /// Flow 解析或编译失败（协议 `FLOW_INVALID`）。`rule` 是合同 §4 的规则编号，解析错误为 `"parse"`。
    #[error("Flow 不合规（规则 {rule}，{path}）：{reason}")]
    FlowInvalid {
        rule: &'static str,
        path: String,
        reason: String,
    },
    /// `work start` 的起始输入缺键或多键。
    #[error("起始输入缺 {missing:?}，多 {extra:?}")]
    InputMissing {
        missing: Vec<String>,
        extra: Vec<String>,
    },
    /// Work 已是终态。
    #[error("Work 已结束（{status}）")]
    WorkTerminal { status: String },
    /// 操作不在当前 `next` 里。`next` 把当前合法集合列成命令行。
    #[error("{requested} 不在合法下一步里")]
    IllegalNext {
        requested: String,
        next: Vec<String>,
    },
    /// 必需输入的上游还没有成功产出。
    #[error("输入 {input} 的上游 {node} 还没有成功产出")]
    InputUnavailable { input: String, node: NodeId },
    /// 输入文件当前摘要与记录不符。
    #[error("输入 {input} 的文件 {path} 已被修改")]
    ArtifactModified { input: String, path: String },
    /// 对非 `running` 的 Attempt 提交或标失败。
    #[error("Attempt {attempt} 不在运行中")]
    AttemptNotRunning { attempt: AttemptId },
    /// 摘要超过 4096 字节。
    #[error("摘要超过 {max} 字节，实际 {actual}")]
    SummaryTooLong { max: usize, actual: usize },
    /// 必需输出文件不存在。
    #[error("必需输出 {output} 不存在：{path}")]
    OutputMissing { output: String, path: String },
    /// 输出超过 `max_bytes`。
    #[error("输出 {output} 超过 {max_bytes} 字节，实际 {actual}")]
    OutputTooLarge {
        output: String,
        max_bytes: u64,
        actual: u64,
    },
    /// 其他格式或取值错误。
    #[error("{reason}")]
    InvalidRequest { reason: String },
}

impl Error {
    /// 映射到协议错误码。
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
            Self::SummaryTooLong { .. } => ErrorCode::SummaryTooLong,
            Self::OutputMissing { .. } => ErrorCode::OutputMissing,
            Self::OutputTooLarge { .. } => ErrorCode::OutputTooLarge,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
