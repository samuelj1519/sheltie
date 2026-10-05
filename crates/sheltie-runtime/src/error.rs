//! Runtime errors: wrap core errors unchanged; define storage, update, and filesystem errors here.

use sheltie_core::ErrorCode;
use sheltie_core::ids::WorkId;

use crate::workbook_repo::VerifyRow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] sheltie_core::Error),
    #[error("Workbook {id}@{version} is already installed")]
    WorkbookExists { id: String, version: String },
    #[error("Workbook {id}@{version} is referenced by {} unfinished Works", works.len())]
    WorkbookInUse {
        id: String,
        version: String,
        works: Vec<WorkId>,
    },
    #[error("Installed Workbook does not match its stored record")]
    WorkbookTampered { results: Vec<VerifyRow> },
    #[error("No update available: {reason}")]
    UpdateUnavailable { reason: String },
    #[error("Downloaded file digest mismatch: expected {expected}, actual {actual}")]
    UpdateChecksumMismatch { expected: String, actual: String },
    #[error("{what} does not exist")]
    NotFound { what: String },
    #[error("Request {request_id} was already submitted with a different intent")]
    RequestConflict { request_id: String },
    /// Incomplete effects (protocol §5). cause is a stable error code. Keep the original success snapshot
    /// and old blocking-request snapshot separate so request ownership remains unambiguous.
    #[error("Effects incomplete (committed={committed}, cause={cause}): {cause_detail}")]
    EffectPending {
        committed: bool,
        request_id: String,
        pending_request_id: Option<String>,
        cause: ErrorCode,
        cause_detail: String,
        original: Option<Box<str>>,
        pending_original: Option<Box<str>>,
    },
    #[error("Revision conflict: expected {expected}, actual {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    #[error("Database structure does not match SCHEMA_VERSION: {detail}")]
    StoreSchemaMismatch { detail: String },
    #[error("Database contents are corrupt: {detail}")]
    StoreCorrupt { detail: String },
    #[error("{path}：{source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// The filesystem call may already have moved an object; preserve recovery state and do not clean up affected paths.
    #[error("{path}: filesystem state requires recovery: {detail}")]
    RecoveryRequired { path: String, detail: String },
    #[error("{reason}")]
    InvalidRequest { reason: String },
    #[error("Cannot read input file {path}: {reason}")]
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

    /// Wrap `std::io::Error` with its path.
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
