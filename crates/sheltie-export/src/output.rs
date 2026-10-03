use serde::Serialize;
use sheltie_core::ids::WorkId;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Complete,
    Rejected,
    FailedBeforePublish,
    PublicationUnconfirmed,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub source_exit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub format: &'static str,
    pub status: Status,
    pub work_id: Option<WorkId>,
    pub revision: Option<u64>,
    pub target_path: Option<PathBuf>,
    pub staging_path: Option<PathBuf>,
    pub error: Option<Diagnostic>,
}

impl Report {
    pub fn complete(work_id: WorkId, revision: u64, target_path: PathBuf) -> Self {
        Self {
            format: "work-export/v1",
            status: Status::Complete,
            work_id: Some(work_id),
            revision: Some(revision),
            target_path: Some(target_path),
            staging_path: None,
            error: None,
        }
    }

    pub fn failure(
        work_id: Option<WorkId>,
        revision: Option<u64>,
        staging_path: Option<PathBuf>,
        error: Error,
    ) -> Self {
        let diagnostic = Diagnostic {
            code: error.code(),
            message: error.to_string(),
            source_exit: match &error {
                Error::Source { exit_code, .. } => *exit_code,
                Error::Rejected { .. }
                | Error::Io { .. }
                | Error::Integrity { .. }
                | Error::PublicationUnconfirmed { .. } => None,
            },
        };
        let (status, target_path, staging_path) = match error {
            Error::Rejected { .. } => (Status::Rejected, None, staging_path),
            Error::Source { .. } | Error::Io { .. } | Error::Integrity { .. } => {
                (Status::FailedBeforePublish, None, staging_path)
            }
            Error::PublicationUnconfirmed { target_path, .. } => {
                (Status::PublicationUnconfirmed, target_path, None)
            }
        };
        Self {
            format: "work-export/v1",
            status,
            work_id,
            revision,
            target_path,
            staging_path,
            error: Some(diagnostic),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.status {
            Status::Complete => 0,
            Status::Rejected => 2,
            Status::FailedBeforePublish => 1,
            Status::PublicationUnconfirmed => 3,
        }
    }

    pub fn readable(&self) -> String {
        let mut message = match self.status {
            Status::Complete => "副本已完成".to_string(),
            Status::Rejected => "导出被拒绝".to_string(),
            Status::FailedBeforePublish => "发布前失败".to_string(),
            Status::PublicationUnconfirmed => "可能已发布，需核查".to_string(),
        };
        if let Some(error) = &self.error {
            message.push_str(&format!("：{} ({})", error.message, error.code));
        }
        if let Some(path) = &self.target_path {
            message.push_str(&format!("\n副本目录：{}", path.display()));
        }
        if let Some(path) = &self.staging_path {
            message.push_str(&format!("\n本次暂存目录：{}", path.display()));
        }
        if self.status != Status::Complete {
            message.push_str("\n请核查保留现场；再次执行会建立另一份新副本。");
        }
        message
    }

    pub fn write(&self, json: bool, writer: &mut impl Write) -> io::Result<()> {
        if json {
            serde_json::to_writer(&mut *writer, self).map_err(io::Error::other)?;
            writer.write_all(b"\n")
        } else {
            writeln!(writer, "{}", self.readable())
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // Task: C006-T01
    #[test]
    fn report_error_variants_preserve_stable_status_exit_and_source_diagnostic() {
        let cases = [
            (
                Error::Rejected {
                    code: "INVALID_RESULT",
                    message: "结果不合格".into(),
                },
                Status::Rejected,
                2,
                "INVALID_RESULT",
                None,
            ),
            (
                Error::Source {
                    message: "进程失败".into(),
                    exit_code: Some(17),
                },
                Status::FailedBeforePublish,
                1,
                "SOURCE_FAILED",
                Some(17),
            ),
            (
                Error::Io {
                    path: "/owned/partial".into(),
                    operation: "write",
                    source: io::Error::other("实际写入失败"),
                },
                Status::FailedBeforePublish,
                1,
                "IO",
                None,
            ),
            (
                Error::Integrity {
                    path: "/owned/partial".into(),
                    message: "摘要不同".into(),
                },
                Status::FailedBeforePublish,
                1,
                "INTEGRITY_MISMATCH",
                None,
            ),
        ];
        for (error, status, exit, code, source_exit) in cases {
            let report = Report::failure(None, None, Some("/owned/partial".into()), error);
            assert_eq!(report.status, status);
            assert_eq!(report.exit_code(), exit);
            assert_eq!(report.target_path, None);
            assert_eq!(report.staging_path, Some("/owned/partial".into()));
            let diagnostic = report.error.unwrap();
            assert_eq!(diagnostic.code, code);
            assert_eq!(diagnostic.source_exit, source_exit);
        }
    }

    // Task: C006-T01
    #[test]
    fn unconfirmed_report_only_names_a_target_whose_identity_was_confirmed() {
        for target_path in [None, Some(PathBuf::from("/owned/published"))] {
            let report = Report::failure(
                None,
                Some(9),
                Some("/owned/old-staging-name".into()),
                Error::PublicationUnconfirmed {
                    target_path: target_path.clone(),
                    message: "父目录同步失败".into(),
                },
            );
            assert_eq!(report.status, Status::PublicationUnconfirmed);
            assert_eq!(report.exit_code(), 3);
            assert_eq!(report.target_path, target_path);
            assert_eq!(report.staging_path, None);
            assert!(report.readable().contains("可能已发布，需核查"));
        }
    }

    // Task: C006-T01
    #[test]
    fn complete_report_is_one_json_line_with_exact_identity_and_no_residual_or_error() {
        let work = WorkId::parse("2026-09-24-001-export").unwrap();
        let report = Report::complete(work.clone(), 7, "/owned/副本".into());
        let mut bytes = Vec::new();
        report.write(true, &mut bytes).unwrap();
        assert_eq!(report.exit_code(), 0);
        assert_eq!(bytes.iter().filter(|byte| **byte == b'\n').count(), 1);
        assert_eq!(bytes.last(), Some(&b'\n'));
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["format"], "work-export/v1");
        assert_eq!(value["status"], "complete");
        assert_eq!(value["work_id"], work.as_str());
        assert_eq!(value["revision"], 7);
        assert_eq!(value["target_path"], "/owned/副本");
        assert_eq!(value["staging_path"], serde_json::Value::Null);
        assert_eq!(value["error"], serde_json::Value::Null);
        assert_eq!(report.readable(), "副本已完成\n副本目录：/owned/副本");
    }
}
