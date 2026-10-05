//! M1 additional literal forms for runtime and CLI: error codes, executor, command names, dates.
//! These functions have no core-internal callers; tests here anchor their output.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use sheltie_core::error::ErrorCode;
use sheltie_core::flow::Executor;
use sheltie_core::ids::NodeId;
use sheltie_core::work::Command;

// Task: T01
#[test]
fn error_code_as_str_matches_serde_and_display() {
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

// Task: T01
#[test]
fn executor_as_str_is_contract_literal() {
    assert_eq!(Executor::Agent.as_str(), "agent");
    assert_eq!(Executor::Human.as_str(), "human");
}

// Task: T01
#[test]
fn command_name_is_cli_verb() {
    assert_eq!(Command::Cancel.name(), "work cancel");
    let approve = Command::ApproveGate {
        node: NodeId::new("review").unwrap(),
    };
    assert_eq!(approve.name(), "gate approve");
}
