//! runtime 的错误。core 的错误原样包进来；存储、更新、文件系统的错误在这里定义。

use sheltie_core::ErrorCode;
use sheltie_core::ids::WorkId;

use crate::workbook_repo::VerifyRow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] sheltie_core::Error),
    #[error("Workbook {id}@{version} 已装")]
    WorkbookExists { id: String, version: String },
    #[error("Workbook {id}@{version} 被 {} 个未结束的 Work 引用", works.len())]
    WorkbookInUse {
        id: String,
        version: String,
        works: Vec<WorkId>,
    },
    #[error("已装 Workbook 与记录不符")]
    WorkbookTampered { results: Vec<VerifyRow> },
    #[error("没有可用的更新：{reason}")]
    UpdateUnavailable { reason: String },
    #[error("下载文件摘要不符：期望 {expected}，实际 {actual}")]
    UpdateChecksumMismatch { expected: String, actual: String },
    #[error("{what} 不存在")]
    NotFound { what: String },
    #[error("请求 {request_id} 已用不同意图提交过")]
    RequestConflict { request_id: String },
    /// 效果未完成（协议 §5）。cause 是稳定错误码，原始成功快照和旧阻断请求快照
    /// 分别保留，避免把两个请求的归属混在一个字段里。
    #[error("效果未完成（committed={committed}，cause={cause}）：{cause_detail}")]
    EffectPending {
        committed: bool,
        request_id: String,
        pending_request_id: Option<String>,
        cause: ErrorCode,
        cause_detail: String,
        original: Option<Box<str>>,
        pending_original: Option<Box<str>>,
    },
    #[error("revision 冲突：期望 {expected}，实际 {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    #[error("数据库结构与 SCHEMA_VERSION 不符：{detail}")]
    StoreSchemaMismatch { detail: String },
    #[error("数据库内容损坏：{detail}")]
    StoreCorrupt { detail: String },
    #[error("{path}：{source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// 文件系统调用可能已经移动对象；调用方必须保留恢复现场，不得清理相关路径。
    #[error("{path}：文件系统状态需要恢复：{detail}")]
    RecoveryRequired { path: String, detail: String },
    #[error("{reason}")]
    InvalidRequest { reason: String },
    #[error("读不了输入文件 {path}：{reason}")]
    InputFileInvalid { path: String, reason: String },
}

impl Error {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Core(e) => e.code(),
            Self::WorkbookExists { .. } => ErrorCode::WorkbookExists,
            Self::WorkbookInUse { .. } => ErrorCode::WorkbookInUse,
            Self::WorkbookTampered { .. } => ErrorCode::WorkbookTampered,
            Self::UpdateUnavailable { .. } => ErrorCode::UpdateUnavailable,
            Self::UpdateChecksumMismatch { .. } => ErrorCode::UpdateChecksumMismatch,
            Self::NotFound { .. } => ErrorCode::NotFound,
            Self::RequestConflict { .. } => ErrorCode::RequestConflict,
            Self::EffectPending { .. } => ErrorCode::EffectPending,
            Self::RevisionConflict { .. } => ErrorCode::RevisionConflict,
            Self::StoreSchemaMismatch { .. } => ErrorCode::StoreSchemaMismatch,
            Self::StoreCorrupt { .. } => ErrorCode::StoreCorrupt,
            Self::Io { .. } => ErrorCode::Io,
            Self::RecoveryRequired { .. } => ErrorCode::Io,
            Self::InvalidRequest { .. } => ErrorCode::InvalidRequest,
            Self::InputFileInvalid { .. } => ErrorCode::InvalidRequest,
        }
    }

    /// 把 `std::io::Error` 连同路径包起来。
    pub fn io(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn effect_pending(
        committed: bool,
        request_id: impl Into<String>,
        pending_request_id: Option<String>,
        cause: &Error,
        original: Option<String>,
        pending_original: Option<String>,
    ) -> Self {
        Self::EffectPending {
            committed,
            request_id: request_id.into(),
            pending_request_id,
            cause: cause.code(),
            cause_detail: cause.to_string(),
            original: original.map(String::into_boxed_str),
            pending_original: pending_original.map(String::into_boxed_str),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        use rusqlite::ffi::ErrorCode as SqliteErrorCode;

        match e.sqlite_error_code() {
            Some(
                SqliteErrorCode::PermissionDenied
                | SqliteErrorCode::DatabaseBusy
                | SqliteErrorCode::DatabaseLocked
                | SqliteErrorCode::ReadOnly
                | SqliteErrorCode::SystemIoFailure
                | SqliteErrorCode::DiskFull
                | SqliteErrorCode::CannotOpen
                | SqliteErrorCode::FileLockingProtocolFailed,
            ) => Self::io("store.db", std::io::Error::other(e)),
            _ => Self::StoreCorrupt {
                detail: e.to_string(),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::Error;
    use sheltie_core::ErrorCode;

    // Task: C002-T25
    #[test]
    fn sqlite_environment_failures_keep_io_error_code() {
        let sqlite = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_FULL),
            Some("database or disk is full".to_string()),
        );
        let error = Error::from(sqlite);
        assert_eq!(error.code(), ErrorCode::Io);
        assert!(error.to_string().contains("store.db"));
        assert!(error.to_string().contains("database or disk is full"));
    }
}
