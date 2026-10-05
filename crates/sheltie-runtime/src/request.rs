//! RequestIntent and intent fingerprints (storage contract §2.1, GF-15).
//!
//! Intent contains only operation kind, resolved target, and user arguments, excluding observations, clocks, and model-reported
//! facts. intent_hash is canonical intent JSON sha256; observed digests or changed @file
//! contents do not change it. Replaying the same path returns the original response; changed literals or paths conflict.

use std::collections::BTreeMap;

use serde::Serialize;
use sheltie_core::digest::Sha256Hex;
use sheltie_core::ids::{AttemptId, NodeId, WorkId};

/// Start-input value: literal or lexically normalized absolute @file path, without filesystem access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "value")]
pub enum InputValue {
    Literal { text: String },
    AtFile { path: String },
}

/// Values for --summary / --reason, like [`InputValue`].
pub type SummarySource = InputValue;

/// Closed set of write-operation intents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "intent")]
pub enum RequestIntent {
    StartWork {
        workbook: String,
        version: Option<String>,
        flow: String,
        name: Option<String>,
        /// Stable bytewise BTreeMap key order.
        inputs: BTreeMap<String, InputValue>,
    },
    BeginAttempt {
        work: WorkId,
        node: NodeId,
    },
    SubmitAttempt {
        work: WorkId,
        attempt: AttemptId,
        summary: SummarySource,
    },
    FailAttempt {
        work: WorkId,
        attempt: AttemptId,
        reason: SummarySource,
    },
    ReplaceAttempt {
        work: WorkId,
        attempt: AttemptId,
        reason: SummarySource,
    },
    ApproveGate {
        work: WorkId,
        node: NodeId,
    },
    CancelWork {
        work: WorkId,
    },
    AddWorkbook {
        /// Lexically normalized absolute source-directory path; no canonicalize, so replay does not require source existence.
        source: String,
    },
    RemoveWorkbook {
        id: String,
        version: String,
    },
}

impl RequestIntent {
    /// Canonical JSON sha256, 64 bare lowercase hexadecimal digits, from serde_json::to_vec
    /// bytes: fixed enum tags and field order, bytewise input-key sorting, no floating-point values.
    pub fn hash(&self) -> Sha256Hex {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        Sha256Hex::of_bytes(&bytes)
    }
}

/// Expand a path lexically to absolute form without reading targets or resolving symlinks; replay does not require the source.
/// Preserve errors when the working directory is unavailable or non-UTF-8, without guessing locations.
pub fn lexical_abs(path: &str) -> crate::Result<String> {
    let p = camino::Utf8Path::new(path);
    let base = if p.is_absolute() {
        camino::Utf8PathBuf::from("/")
    } else {
        // Expand relative paths lexically against the working directory.
        crate::failpoint::rendezvous("lexical_before_cwd", path)
            .map_err(|error| crate::Error::io("current_dir", error))?;
        let cwd =
            std::env::current_dir().map_err(|error| crate::Error::io("current_dir", error))?;
        camino::Utf8PathBuf::from_path_buf(cwd).map_err(|path| crate::Error::InvalidRequest {
            reason: format!(
                "Path-expansion working directory {} is not a UTF-8 path",
                path.display()
            ),
        })?
    };
    let mut out = base;
    for seg in p.components() {
        match seg {
            camino::Utf8Component::RootDir | camino::Utf8Component::Prefix(_) => {}
            camino::Utf8Component::CurDir => {}
            camino::Utf8Component::ParentDir => {
                out.pop();
            }
            camino::Utf8Component::Normal(name) => out = out.join(name),
        }
    }
    Ok(out.to_string())
}
