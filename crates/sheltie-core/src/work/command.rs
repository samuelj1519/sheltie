//! Commands, context, effects, and replies: input/output types for `decide`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::state::{ArtifactRef, Principal, Timestamp, WorkState, WorkbookRef};
use crate::digest::Sha256Hex;
use crate::ids::{AttemptId, FlowId, NodeId, WorkId, WorkName};
use crate::path::AbsPath;
use crate::workbook::HostRequire;

/// Runtime's read-only file observation; core checks it against the contract without reading files.
///
/// Runtime constructs production observations through managed reads (`INV-6`); models do not report digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedFile {
    pub path: AbsPath,
    pub sha256: Sha256Hex,
    pub bytes: u64,
}

impl ObservedFile {
    /// For runtime callers only.
    #[doc(hidden)]
    pub fn new(path: AbsPath, sha256: Sha256Hex, bytes: u64) -> Self {
        Self {
            path,
            sha256,
            bytes,
        }
    }

    pub fn into_ref(self) -> ArtifactRef {
        ArtifactRef {
            path: self.path,
            sha256: self.sha256,
            bytes: self.bytes,
        }
    }

    pub fn matches(&self, r: &ArtifactRef) -> bool {
        self.sha256 == r.sha256
    }
}

/// All coordinator write operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "command", deny_unknown_fields)]
pub enum Command {
    /// Create a Work; runtime has written inputs to files and calculated their digests.
    Start {
        work_id: WorkId,
        name: WorkName,
        workbook: WorkbookRef,
        flow: FlowId,
        work_dir: AbsPath,
        inputs: BTreeMap<String, ArtifactRef>,
    },
    /// Enter a node and begin an Attempt; runtime observes inputs according to `input_paths_for`.
    /// Core renders the brief from original instruction_text; runtime reads File instructions from the frozen copy, or passes Text directly.
    BeginAttempt {
        node: NodeId,
        observed_inputs: BTreeMap<String, Option<ObservedFile>>,
        instruction_text: String,
    },
    /// Submit an Attempt; runtime observes outputs via `output_paths_for`, using `None` for missing files.
    SubmitAttempt {
        attempt: AttemptId,
        summary: String,
        observed_outputs: BTreeMap<String, Option<ObservedFile>>,
    },
    FailAttempt {
        attempt: AttemptId,
        reason: String,
    },
    ReplaceAttempt {
        attempt: AttemptId,
        reason: String,
        observed_inputs: BTreeMap<String, Option<ObservedFile>>,
        instruction_text: String,
    },
    ApproveGate {
        node: NodeId,
    },
    Cancel,
}

impl Command {
    /// Command name for auditing and diagnostics.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Start { .. } => "start",
            Self::BeginAttempt { .. } => "attempt begin",
            Self::SubmitAttempt { .. } => "attempt submit",
            Self::FailAttempt { .. } => "attempt fail",
            Self::ReplaceAttempt { .. } => "attempt replace",
            Self::ApproveGate { .. } => "gate approve",
            Self::Cancel => "work cancel",
        }
    }
}

/// Immutable decision context: timestamp and principal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub now: Timestamp,
    pub principal: Principal,
}

/// Idempotent actions core requires runtime to perform after commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "effect")]
pub enum Effect {
    WriteBrief {
        path: AbsPath,
        content: String,
    },
    /// Engine-generated input file (currently engine.stats stats.json); core computes content and records its digest.
    WriteFile {
        path: AbsPath,
        content: String,
    },
    SealOutputs {
        paths: Vec<AbsPath>,
    },
    RefreshStatusCard,
}

/// Successful command reply data, matching protocol §3 operation responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reply", deny_unknown_fields)]
pub enum Reply {
    Started {
        work_id: WorkId,
        work_dir: AbsPath,
        requires: Vec<HostRequire>,
    },
    AttemptBegun {
        attempt: AttemptId,
        brief_path: AbsPath,
        output_dir: AbsPath,
        inputs: BTreeMap<String, Option<AbsPath>>,
        outputs: BTreeMap<String, AbsPath>,
        requires: Vec<HostRequire>,
    },
    AttemptSubmitted {
        attempt: AttemptId,
        outputs: BTreeMap<String, ArtifactRef>,
    },
    AttemptFailed {
        attempt: AttemptId,
    },
    AttemptReplaced {
        replaced_attempt: AttemptId,
        attempt: AttemptId,
        brief_path: AbsPath,
        output_dir: AbsPath,
        inputs: BTreeMap<String, Option<AbsPath>>,
        outputs: BTreeMap<String, AbsPath>,
        requires: Vec<HostRequire>,
    },
    GateApproved {
        node: NodeId,
        occurrence: u32,
    },
    Cancelled,
}

/// Output of `decide`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub state: WorkState,
    pub effects: Vec<Effect>,
    pub reply: Reply,
}
