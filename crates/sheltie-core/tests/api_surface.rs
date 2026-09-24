//! M1 补测：骨架里给 runtime 与 CLI 用的字面形式（错误码、执行者、命令名、日期）。
//! 这些函数在 core 内部没有调用者，只有这里钉住它们的输出。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use sheltie_core::error::ErrorCode;
use sheltie_core::flow::Executor;
use sheltie_core::ids::NodeId;
use sheltie_core::work::{Command, Timestamp};

#[test]
fn t01_error_code_as_str_matches_serde_and_display() {
    use ErrorCode::*;
    let all = [
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
    ];
    for code in all {
        let serde = serde_json::to_value(code).unwrap();
        assert_eq!(serde.as_str().unwrap(), code.as_str());
        assert_eq!(code.to_string(), code.as_str());
    }
    assert_eq!(ErrorCode::IllegalNext.as_str(), "ILLEGAL_NEXT");
}

#[test]
fn t01_executor_as_str_is_contract_literal() {
    assert_eq!(Executor::Agent.as_str(), "agent");
    assert_eq!(Executor::Human.as_str(), "human");
}

#[test]
fn t01_command_name_is_cli_verb() {
    assert_eq!(Command::Cancel.name(), "work cancel");
    let approve = Command::ApproveGate {
        node: NodeId::new("review").unwrap(),
    };
    assert_eq!(approve.name(), "gate approve");
}

#[test]
fn t01_timestamp_day_is_date_prefix() {
    assert_eq!(
        Timestamp("2026-09-24T03:00:00Z".to_string()).day(),
        "2026-09-24"
    );
}
