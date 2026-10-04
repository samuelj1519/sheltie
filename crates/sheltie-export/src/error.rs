use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}")]
    Rejected { code: &'static str, message: String },
    #[error("源进程失败：{message}，exit={exit_code:?}")]
    Source {
        message: String,
        exit_code: Option<i32>,
    },
    #[error("{operation} {path:?}：{source}")]
    Io {
        path: PathBuf,
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("完整性不符 {path:?}：{message}")]
    Integrity { path: PathBuf, message: String },
    #[error("可能已发布，需核查：{message}")]
    PublicationUnconfirmed {
        target_path: Option<PathBuf>,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Rejected { code, .. } => code,
            Self::Source { .. } => "SOURCE_FAILED",
            Self::Io { .. } => "IO",
            Self::Integrity { .. } => "INTEGRITY_MISMATCH",
            Self::PublicationUnconfirmed { .. } => "PUBLICATION_UNCONFIRMED",
        }
    }
}
