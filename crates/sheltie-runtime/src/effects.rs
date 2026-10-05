//! Effect registration and recovery (storage contract §3.2, GF-31).
//!
//! requests.effects_json records I/O completion, without selecting business edges or forming
//! a second Work state. Paths are management-root relative; write_file carries exact bytes, restoring historical briefs and
//! engine/stats.json from commit-time bytes, without recomputing from current state. Recover in audit.seq
//! order; within each request publish_dir/prepare_attempt precede historical files, sealing/deletion, and finally
//! `refresh_status_card`。

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{Graph, InputSource};
use sheltie_core::ids::AttemptId;
use sheltie_core::path::AbsPath;
use sheltie_core::work::{Command, Reply, WorkState, output_paths_for};

use crate::error::{Error, Result};
use crate::fsx::{self, ManagedRelPath};
use crate::home::Home;

/// Effects whose complete batch has been checked against its owning Work and frozen graph.
#[derive(Debug)]
pub(crate) struct CheckedEffects {
    ops: Vec<EffectOp>,
    start_inputs: Option<BTreeMap<String, RefJson>>,
    request_id: Option<String>,
}

impl CheckedEffects {
    pub(crate) fn as_slice(&self) -> &[EffectOp] {
        &self.ops
    }
}

/// One effects_json object; fields match storage contract §3.2, rejecting unknown fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectOp {
    /// Atomically publish pending/<id>/payload/ to its final directory.
    PublishDir {
        pending: String,
        #[serde(rename = "final")]
        final_path: String,
        /// work:<work_id> or workbook:<id>@<version>.
        owner: String,
        /// workbook-digest/v2, shared by Workbooks and Work frozen copies.
        digest: String,
        /// Digest subtree relative to payload; empty means the whole tree. Work payload contains
        /// workbook/ and start-inputs/; hash only workbook/ (storage contract §3.2).
        digest_root: String,
    },
    /// Safely create the committed Attempt's directory skeleton.
    PrepareAttempt {
        work_id: String,
        attempt_id: String,
        /// Root-relative directory paths ordered parent before child.
        dirs: Vec<String>,
    },
    /// Exact-byte historical files: brief and engine/stats.json.
    WriteFile {
        path: String,
        sha256: String,
        content: String,
    },
    /// Set outputs read-only after verifying original artifact references.
    SealOutputs { refs: Vec<RefJson> },
    /// Move and delete directories with verified ownership.
    DeleteDir {
        pending: String,
        #[serde(rename = "final")]
        final_path: String,
        owner: String,
        digest: String,
    },
    /// Regenerate the status card from latest state, as a current projection rather than a historical file.
    RefreshStatusCard { work_id: String },
}

/// Complete artifact reference (protocol §6); seal_outputs uses it to verify the original object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefJson {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeletedMarker {
    pub(crate) format: String,
    pub(crate) internal_id: String,
}

/// Decode an effect array; invalid structure yields STORE_CORRUPT without guessed defaults.
pub fn decode_effects(json: &str) -> Result<Vec<EffectOp>> {
    serde_json::from_str(json).map_err(|e| Error::StoreCorrupt {
        detail: format!("Cannot decode effects_json: {e}"),
    })
}

pub fn encode_effects(ops: &[EffectOp]) -> String {
    serde_json::to_string(ops).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn check_work_effects(
    home: &Home,
    request_id: &str,
    state: &WorkState,
    graph: &Graph,
    command: &Command,
    reply: &Reply,
    ops: Vec<EffectOp>,
) -> Result<CheckedEffects> {
    let work = state.work_id.as_str();
    let mut seen_prepare = false;
    let command_attempt = match command {
        Command::BeginAttempt { .. } | Command::ReplaceAttempt { .. } => match reply {
            Reply::AttemptBegun { attempt, .. } | Reply::AttemptReplaced { attempt, .. } => {
                Some(attempt.clone())
            }
            _ => None,
        },
        Command::SubmitAttempt { attempt, .. } | Command::FailAttempt { attempt, .. } => {
            Some(attempt.clone())
        }
        _ => None,
    };
    validate_effect_shape(home, state, graph, command, reply, &ops)?;
    for (index, op) in ops.iter().enumerate() {
        let invalid = |field: &str, reason: &str| Error::StoreCorrupt {
            detail: format!("Work {work} requests.effects[{index}].{field} {reason}"),
        };
        match op {
            EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                digest_root,
            } => {
                let Command::Start {
                    work_id, workbook, ..
                } = command
                else {
                    return Err(invalid(
                        "kind",
                        "Only start requests may publish Work directories",
                    ));
                };
                let parts = pending.split('/').collect::<Vec<_>>();
                if parts.len() != 3
                    || parts[0] != "pending"
                    || parts[2] != "payload"
                    || uuid::Uuid::parse_str(parts[1]).is_err()
                {
                    return Err(invalid(
                        "pending",
                        "Not this engine's pending/<id>/payload path",
                    ));
                }
                if work_id.as_str() != work
                    || final_path != &format!("works/{work}")
                    || owner != &format!("work:{work}")
                    || digest != workbook.digest.as_str()
                    || digest_root != "workbook"
                {
                    return Err(invalid(
                        "owner",
                        "Does not match the Start command or committed Work identity",
                    ));
                }
                ManagedRelPath::new(pending.clone())
                    .map_err(|error| invalid("pending", &format!("Invalid: {error}")))?;
                ManagedRelPath::new(final_path.clone())
                    .map_err(|error| invalid("final", &format!("Invalid: {error}")))?;
            }
            EffectOp::PrepareAttempt {
                work_id,
                attempt_id,
                dirs,
            } => {
                if seen_prepare || work_id != work {
                    return Err(invalid(
                        "work_id",
                        "Mismatched or duplicate Attempt directory preparation",
                    ));
                }
                seen_prepare = true;
                let attempt_id = AttemptId::parse(attempt_id)
                    .map_err(|error| invalid("attempt_id", &format!("Invalid: {error}")))?;
                if command_attempt.as_ref() != Some(&attempt_id)
                    || state.attempt(&attempt_id).is_none()
                {
                    return Err(invalid(
                        "attempt_id",
                        "Not the committed Attempt belonging to this request",
                    ));
                }
                let expected = expected_attempt_dirs(home, state, graph, &attempt_id)?;
                if dirs != &expected {
                    return Err(invalid(
                        "dirs",
                        &format!(
                            "Does not match frozen-graph Attempt directories: actual={dirs:?}, expected={expected:?}"
                        ),
                    ));
                }
            }
            EffectOp::WriteFile {
                path,
                sha256,
                content,
            } => {
                let attempt_id = command_attempt
                    .as_ref()
                    .ok_or_else(|| invalid("path", "No owning Attempt"))?;
                let attempt = state
                    .attempt(attempt_id)
                    .ok_or_else(|| invalid("path", "Attempt is absent from verified Work state"))?;
                if !matches!(
                    command,
                    Command::BeginAttempt { .. } | Command::ReplaceAttempt { .. }
                ) {
                    return Err(invalid(
                        "path",
                        "Only begin/replace requests may register historical files",
                    ));
                }
                let expected_brief =
                    home.to_rel(&state.attempt_dir(attempt_id).join_segment("brief.md"))?;
                let expected_stats = home.to_rel(
                    &sheltie_core::work::layout::engine_stats_path(&state.attempt_dir(attempt_id)),
                )?;
                if path != &expected_brief && path != &expected_stats {
                    return Err(invalid(
                        "path",
                        "Does not match this Attempt's brief or engine.stats path",
                    ));
                }
                ManagedRelPath::new(path.clone())
                    .map_err(|error| invalid("path", &format!("Invalid: {error}")))?;
                if Sha256Hex::new(sha256.clone()).is_err()
                    || Sha256Hex::of_bytes(content.as_bytes()).as_str() != sha256
                {
                    return Err(invalid(
                        "sha256",
                        "Does not match exact historical-file bytes",
                    ));
                }
                if path == &expected_stats {
                    let stats_path = state
                        .attempt_dir(attempt_id)
                        .join_segment("engine")
                        .join_segment("stats.json");
                    let references = attempt
                        .inputs
                        .values()
                        .filter_map(Option::as_ref)
                        .filter(|reference| reference.path == stats_path)
                        .collect::<Vec<_>>();
                    let bytes = content.len() as u64;
                    if references.is_empty()
                        || references.iter().any(|reference| {
                            reference.sha256.as_str() != sha256 || reference.bytes != bytes
                        })
                    {
                        return Err(invalid(
                            "content",
                            "engine.stats bytes differ from the ArtifactRef bound by this begin",
                        ));
                    }
                }
            }
            EffectOp::SealOutputs { refs } => {
                let Command::SubmitAttempt { attempt, .. } = command else {
                    return Err(invalid("refs", "Only submit requests may register sealing"));
                };
                let persisted = state
                    .attempt(attempt)
                    .ok_or_else(|| invalid("refs", "Attempt is absent from verified Work state"))?;
                let expected = persisted
                    .outputs
                    .iter()
                    .map(|(name, reference)| {
                        let path = home.to_rel(&reference.path).map_err(|error| {
                            invalid(name, &format!("Output path is not managed: {error}"))
                        })?;
                        Ok((
                            name,
                            RefJson {
                                path,
                                sha256: reference.sha256.as_str().to_string(),
                                bytes: reference.bytes,
                            },
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?;
                let expected_by_path = expected
                    .values()
                    .map(|reference| (reference.path.as_str(), reference))
                    .collect::<BTreeMap<_, _>>();
                let actual = refs
                    .iter()
                    .map(|reference| (reference.path.as_str(), reference))
                    .collect::<BTreeMap<_, _>>();
                if actual.len() != refs.len()
                    || actual.len() != expected_by_path.len()
                    || actual.iter().any(|(path, actual)| {
                        expected_by_path
                            .get(path)
                            .is_none_or(|expected| *actual != *expected)
                    })
                {
                    return Err(invalid(
                        "refs",
                        "Differs from this Attempt's committed artifact references",
                    ));
                }
            }
            EffectOp::RefreshStatusCard { work_id } if work_id == work => {}
            EffectOp::RefreshStatusCard { .. } => {
                return Err(invalid(
                    "work_id",
                    "Status card does not belong to this Work",
                ));
            }
            EffectOp::DeleteDir { .. } => {
                return Err(invalid(
                    "kind",
                    "Work requests must not delete Workbook directories",
                ));
            }
        }
    }
    let start_inputs = matches!(command, Command::Start { .. })
        .then(|| {
            state
                .inputs
                .iter()
                .map(|(key, reference)| {
                    let path = home.to_rel(&reference.path)?;
                    Ok((
                        key.clone(),
                        RefJson {
                            path,
                            sha256: reference.sha256.as_str().to_string(),
                            bytes: reference.bytes,
                        },
                    ))
                })
                .collect::<Result<BTreeMap<_, _>>>()
        })
        .transpose()?;
    Ok(CheckedEffects {
        ops,
        start_inputs,
        request_id: Some(request_id.to_string()),
    })
}

fn validate_effect_shape(
    home: &Home,
    state: &WorkState,
    graph: &Graph,
    command: &Command,
    reply: &Reply,
    ops: &[EffectOp],
) -> Result<()> {
    let work = state.work_id.as_str();
    let valid = match (command, reply) {
        (Command::Start { .. }, Reply::Started { .. }) => {
            matches!(ops, [EffectOp::PublishDir { .. }, EffectOp::RefreshStatusCard { work_id }] if work_id == work)
        }
        (Command::BeginAttempt { .. }, Reply::AttemptBegun { attempt, .. })
        | (Command::ReplaceAttempt { .. }, Reply::AttemptReplaced { attempt, .. }) => {
            let Some(node) = graph.node(&attempt.node) else {
                return Err(Error::StoreCorrupt {
                    detail: format!(
                        "Work {work} begin Attempt node is absent from the frozen graph"
                    ),
                });
            };
            let wants_stats = node
                .inputs()
                .iter()
                .any(|input| matches!(input.source(), InputSource::EngineStats));
            let stats_path = home.to_rel(&sheltie_core::work::layout::engine_stats_path(
                &state.attempt_dir(attempt),
            ))?;
            let brief_path = home.to_rel(&state.attempt_dir(attempt).join_segment("brief.md"))?;
            let prepare_matches = matches!(ops.first(), Some(EffectOp::PrepareAttempt {
                work_id,
                attempt_id,
                ..
            }) if work_id == state.work_id.as_str() && attempt_id == &attempt.to_string());
            let card_matches = matches!(ops.last(), Some(EffectOp::RefreshStatusCard { work_id }) if work_id == state.work_id.as_str());
            let expected_writes = usize::from(wants_stats) + 1;
            let writes_match = ops.len() == expected_writes + 2
                && ops[1..1 + usize::from(wants_stats)].iter().all(
                    |op| matches!(op, EffectOp::WriteFile { path, .. } if path == &stats_path),
                )
                && matches!(&ops[ops.len() - 2], EffectOp::WriteFile { path, .. } if path == &brief_path);
            prepare_matches && writes_match && card_matches
        }
        (
            Command::SubmitAttempt { attempt, .. },
            Reply::AttemptSubmitted {
                attempt: reply_attempt,
                ..
            },
        ) => {
            attempt == reply_attempt
                && matches!(ops, [EffectOp::SealOutputs { .. }, EffectOp::RefreshStatusCard { work_id }] if work_id == work)
        }
        (Command::FailAttempt { .. }, Reply::AttemptFailed { .. })
        | (Command::ApproveGate { .. }, Reply::GateApproved { .. })
        | (Command::Cancel, Reply::Cancelled) => {
            matches!(ops, [EffectOp::RefreshStatusCard { work_id }] if work_id == work)
        }
        _ => false,
    };
    if !valid {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Work {work} requests.effects count, kinds, or order differ from Command"
            ),
        });
    }
    Ok(())
}

pub(crate) enum WorkbookEffectIdentity {
    Add {
        id: String,
        version: String,
        digest: String,
    },
    Remove {
        id: String,
        version: String,
    },
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PublicationTarget {
    PublishDir {
        #[serde(rename = "final")]
        final_path: String,
        owner: String,
        digest: String,
    },
    #[serde(other)]
    Other,
}

fn workbook_target_matches(
    identity: &WorkbookEffectIdentity,
    final_path: &str,
    owner: &str,
    digest: Option<&str>,
) -> bool {
    match identity {
        WorkbookEffectIdentity::Add {
            id,
            version,
            digest: expected_digest,
        } => {
            final_path == format!("workbooks/{id}/{version}")
                && owner == format!("workbook:{id}@{version}")
                && digest == Some(expected_digest.as_str())
        }
        WorkbookEffectIdentity::Remove { id, version } => {
            final_path == format!("workbooks/{id}/{version}")
                && owner == format!("workbook:{id}@{version}")
        }
    }
}

fn invalid_workbook_target() -> Error {
    Error::StoreCorrupt {
        detail: "Workbook snapshot business identity differs from registered effects".into(),
    }
}

/// Read-only publication identity projection; full-payload validation still rejects other invalid effect fields without discarding verified identity.
pub(crate) fn check_workbook_add_snapshot_target(
    identity: &WorkbookEffectIdentity,
    json: &str,
) -> Result<()> {
    let targets: Vec<PublicationTarget> =
        serde_json::from_str(json).map_err(|error| Error::StoreCorrupt {
            detail: format!("Cannot decode Workbook publication identity: {error}"),
        })?;
    let mut publications = targets
        .iter()
        .filter(|target| matches!(target, PublicationTarget::PublishDir { .. }));
    let valid = matches!((publications.next(), publications.next()),
        (Some(PublicationTarget::PublishDir { final_path, owner, digest }), None)
        if matches!(identity, WorkbookEffectIdentity::Add { .. })
            && workbook_target_matches(identity, final_path, owner, Some(digest)));
    if !valid {
        return Err(invalid_workbook_target());
    }
    Ok(())
}

pub(crate) fn check_workbook_snapshot_target(
    identity: &WorkbookEffectIdentity,
    ops: &[EffectOp],
) -> Result<()> {
    let valid = match (identity, ops) {
        (
            WorkbookEffectIdentity::Add { .. },
            [
                EffectOp::PublishDir {
                    final_path,
                    owner,
                    digest,
                    ..
                },
            ],
        ) => workbook_target_matches(identity, final_path, owner, Some(digest)),
        (
            WorkbookEffectIdentity::Remove { .. },
            [
                EffectOp::DeleteDir {
                    final_path, owner, ..
                },
            ],
        ) => workbook_target_matches(identity, final_path, owner, None),
        _ => false,
    };
    if !valid {
        return Err(invalid_workbook_target());
    }
    Ok(())
}

pub(crate) fn check_workbook_effects(
    request_id: &str,
    audit_work_id: &str,
    audit_revision: i64,
    audit_at: &str,
    identity: WorkbookEffectIdentity,
    ops: Vec<EffectOp>,
    registered: Option<&crate::store::WorkbookRow>,
) -> Result<CheckedEffects> {
    if !audit_work_id.is_empty() || audit_revision != 0 || ops.len() != 1 {
        return Err(Error::StoreCorrupt {
            detail: "Workbook request audit ownership or effect count is invalid".to_string(),
        });
    }
    check_workbook_snapshot_target(&identity, &ops)?;
    match (&ops[0], identity) {
        (
            EffectOp::PublishDir {
                pending,
                digest,
                digest_root,
                ..
            },
            WorkbookEffectIdentity::Add {
                id,
                version,
                digest: expected_digest,
            },
        ) => {
            let expected_final = format!("workbooks/{id}/{version}");
            ManagedRelPath::new(expected_final.clone()).map_err(|error| Error::StoreCorrupt {
                detail: format!("Invalid add snapshot Workbook identity path: {error}"),
            })?;
            if registered.is_some_and(|registered| {
                registered.id != id
                    || registered.version != version
                    || registered.digest != expected_digest
                    || registered.dir != expected_final
                    || registered.added_at != audit_at
            }) || !digest_root.is_empty()
                || Sha256Hex::new(digest.clone()).is_err()
            {
                return Err(Error::StoreCorrupt {
                    detail:
                        "add request audit, snapshot, Store row, and PublishDir ownership differ"
                            .to_string(),
                });
            }
            validate_pending_payload(pending)?;
        }
        (
            EffectOp::DeleteDir {
                pending, digest, ..
            },
            WorkbookEffectIdentity::Remove { id, version },
        ) => {
            let expected_final = format!("workbooks/{id}/{version}");
            ManagedRelPath::new(expected_final.clone()).map_err(|error| Error::StoreCorrupt {
                detail: format!("Invalid remove snapshot Workbook identity path: {error}"),
            })?;
            if Sha256Hex::new(digest.clone()).is_err() {
                return Err(Error::StoreCorrupt {
                    detail: "remove request audit, snapshot, and DeleteDir ownership differ"
                        .to_string(),
                });
            }
            validate_pending_payload(pending)?;
        }
        _ => {
            return Err(Error::StoreCorrupt {
                detail: "Workbook request intent does not match effect kind".to_string(),
            });
        }
    }
    Ok(CheckedEffects {
        ops,
        start_inputs: None,
        request_id: Some(request_id.to_string()),
    })
}

fn validate_pending_payload(path: &str) -> Result<()> {
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() != 3
        || segments[0] != "pending"
        || segments[2] != "payload"
        || uuid::Uuid::parse_str(segments[1]).is_err()
    {
        return Err(Error::StoreCorrupt {
            detail: format!("Invalid pending-original path {path}"),
        });
    }
    ManagedRelPath::new(path.to_string()).map_err(|error| Error::StoreCorrupt {
        detail: format!("Invalid pending-original path {path}: {error}"),
    })?;
    Ok(())
}

pub(crate) fn validate_workbook_identity(id: &str, version: &str) -> Result<()> {
    sheltie_core::ids::WorkbookId::new(id).map_err(|error| Error::StoreCorrupt {
        detail: format!("Invalid Workbook effect ID: {error}"),
    })?;
    if !crate::load::valid_workbook_version(version) {
        return Err(Error::StoreCorrupt {
            detail: format!("Workbook effect version {version:?} violates the contract"),
        });
    }
    Ok(())
}

fn expected_attempt_dirs(
    home: &Home,
    state: &WorkState,
    graph: &Graph,
    attempt_id: &AttemptId,
) -> Result<Vec<String>> {
    let mut directories = BTreeSet::new();
    let attempt_dir = state.attempt_dir(attempt_id);
    let output_dir = sheltie_core::work::layout::outputs_dir(&attempt_dir);
    add_directory_chain(home, &output_dir, &mut directories)?;
    let outputs =
        output_paths_for(state, graph, attempt_id).map_err(|error| Error::StoreCorrupt {
            detail: format!("Invalid Attempt {attempt_id} frozen output definitions: {error}"),
        })?;
    for output in outputs.values() {
        let parent = output
            .as_path()
            .parent()
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Attempt {attempt_id} output has no parent directory"),
            })?;
        add_directory_chain(
            home,
            &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
            &mut directories,
        )?;
    }
    let uses_engine_stats = graph.node(&attempt_id.node).is_some_and(|node| {
        node.inputs()
            .iter()
            .any(|input| matches!(input.source(), InputSource::EngineStats))
    });
    if uses_engine_stats {
        let engine_stats = sheltie_core::work::layout::engine_stats_path(&attempt_dir);
        add_directory_chain(
            home,
            &AbsPath::new(
                engine_stats
                    .as_path()
                    .parent()
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!(
                            "Attempt {attempt_id} engine.stats has no parent directory"
                        ),
                    })?
                    .to_string(),
            )
            .map_err(Error::Core)?,
            &mut directories,
        )?;
    }
    Ok(directories.into_iter().collect())
}

fn add_directory_chain(
    home: &Home,
    directory: &AbsPath,
    directories: &mut BTreeSet<String>,
) -> Result<()> {
    let relative = home.to_rel(directory)?;
    let mut prefix = String::new();
    for segment in relative.split('/') {
        prefix = if prefix.is_empty() {
            segment.to_string()
        } else {
            format!("{prefix}/{segment}")
        };
        directories.insert(prefix.clone());
    }
    Ok(())
}

/// Execute or recover effects idempotently: verify current state first, accepting the same object as completed,
/// rejecting different objects without overwriting or deleting originals; return Ok only after all succeed, with context on failure.
///
/// publish controls publication; requests already published = 1 replay only write_file verification,
/// without repeating publication, sealing, or deletion (storage contract §3.2 final paragraph).
pub(crate) fn execute(
    home: &Home,
    lock: &crate::home::HomeLock,
    ops: &CheckedEffects,
    publish: bool,
) -> Result<()> {
    execute_with_observed_outputs(home, lock, ops, publish, None)
}

pub(crate) fn execute_with_observed_outputs(
    home: &Home,
    lock: &crate::home::HomeLock,
    ops: &CheckedEffects,
    publish: bool,
    observed: Option<&BTreeMap<String, fsx::SafeFile>>,
) -> Result<()> {
    for op in ops.as_slice() {
        match op {
            EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                digest_root,
            } => {
                if !publish {
                    continue;
                }
                publish_dir(
                    home,
                    lock,
                    PublishSpec {
                        pending,
                        final_path,
                        owner,
                        digest,
                        digest_root,
                        start_inputs: ops.start_inputs.as_ref(),
                        request_id: ops.request_id.as_deref(),
                    },
                )?;
            }
            EffectOp::PrepareAttempt { dirs, .. } => {
                if !publish {
                    continue;
                }
                for rel in dirs {
                    let target = home.rel(rel)?;
                    fsx::ensure_dirs_under(home, lock, &target).map_err(|error| {
                        classify_committed_path_error("Attempt directory", &target, error)
                    })?;
                    fsx::sync_managed_directory_entry(home, lock, rel).map_err(|error| {
                        classify_committed_path_error("Attempt directory", &target, error)
                    })?;
                }
            }
            EffectOp::WriteFile {
                path,
                sha256,
                content,
            } => {
                let target = home.rel(path)?;
                if let Some(f) = fsx::open_managed_optional(home, &target).map_err(|error| {
                    classify_committed_path_error("Historical file", &target, error)
                })? {
                    // No-follow inspection distinguishes a missing leaf from an abnormal object.
                    // A matching existing original is verified and left unchanged.
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES).map_err(|error| {
                        classify_committed_path_error("Historical file", &target, error)
                    })?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!(
                                "Historical file {path} digest differs from registration (modified)"
                            ),
                        });
                    }
                    // A completed request can still have an interrupted explicit history repair.
                    fsx::sync_managed_regular_file_handle(home, lock, path, &f).map_err(
                        |error| classify_committed_path_error("Historical file", &target, error),
                    )?;
                } else {
                    // Do not invent missing or untrusted parent directories.
                    let parent = target
                        .as_path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .ok_or_else(|| Error::StoreCorrupt {
                            detail: format!("Historical file {path} has no parent directory"),
                        })?;
                    if !parent.exists() {
                        return Err(Error::StoreCorrupt {
                            detail: format!(
                                "Historical file {path} parent directory is missing; cannot recover"
                            ),
                        });
                    }
                    fsx::write_new_atomic_file(home, lock, &target, content.as_bytes()).map_err(
                        |error| classify_committed_path_error("Historical file", &target, error),
                    )?;
                    let f = fsx::open_managed_regular(home, &target).map_err(|error| {
                        classify_committed_path_error("Historical file", &target, error)
                    })?;
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES).map_err(|error| {
                        classify_committed_path_error("Historical file", &target, error)
                    })?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!(
                                "Historical file {path} digest differs from registration after recovery"
                            ),
                        });
                    }
                }
                crate::failpoint::maybe_exit("after_first_history_file_write");
            }
            EffectOp::SealOutputs { refs } => {
                if !publish {
                    continue;
                }
                crate::failpoint::maybe_exit("submit_before_seal");
                for r in refs {
                    let target = home.rel(&r.path)?;
                    if let Some(observed) = observed {
                        let file =
                            observed
                                .get(target.as_str())
                                .ok_or_else(|| Error::StoreCorrupt {
                                    detail: format!(
                                        "Ordinary submit lacks an observation handle for output {}",
                                        r.path
                                    ),
                                })?;
                        if file.path() != &target {
                            return Err(Error::StoreCorrupt {
                                detail: format!(
                                    "Sealing handle {} differs from effect path",
                                    r.path
                                ),
                            });
                        }
                        seal_output(home, lock, &target, file, r)?;
                    } else {
                        let file = fsx::open_managed_regular(home, &target)?;
                        seal_output(home, lock, &target, &file, r)?;
                    }
                }
            }
            EffectOp::DeleteDir {
                pending,
                final_path,
                owner,
                digest,
            } => {
                if !publish {
                    continue;
                }
                let request_id = ops
                    .request_id
                    .as_deref()
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: "DeleteDir effect lacks verified request identity".to_string(),
                    })?;
                delete_dir(home, lock, request_id, pending, final_path, owner, digest)?;
            }
            EffectOp::RefreshStatusCard { .. } => {
                // Status cards are current projections, generated by the Store-owning caller from latest state_json;
                // this executor neither reads the database nor stores historical card bytes.
            }
        }
    }
    Ok(())
}

fn seal_output(
    home: &Home,
    lock: &crate::home::HomeLock,
    target: &AbsPath,
    file: &fsx::SafeFile,
    reference: &RefJson,
) -> Result<()> {
    file.verify_seal_reference(&reference.sha256, reference.bytes)
        .map_err(|error| classify_committed_path_error("sealing", target, error))?;
    fsx::ManagedFs::open_existing(home)?
        .set_readonly(lock, file)
        .map_err(|error| classify_committed_path_error("sealing", target, error))
}

fn classify_committed_path_error(operation: &str, target: &AbsPath, error: Error) -> Error {
    match error {
        Error::Io { source, .. } if source.kind() == std::io::ErrorKind::AlreadyExists => {
            Error::StoreCorrupt {
                detail: format!(
                    "{operation} path {target} gained an existing object before recovery placement: {source}"
                ),
            }
        }
        Error::InvalidRequest { reason } => Error::StoreCorrupt {
            detail: format!("{operation} path {target} object identity or type differs: {reason}"),
        },
        Error::NotFound { what } => Error::StoreCorrupt {
            detail: format!("{operation} path {target} disappeared before completion: {what}"),
        },
        other => other,
    }
}

/// publish_dir: pending only -> verify ownership/digest, rename, and set
/// read-only; final only -> verify ownership/digest and accept completion; both present or mismatched contents -> stop.
struct PublishSpec<'a> {
    pending: &'a str,
    final_path: &'a str,
    owner: &'a str,
    digest: &'a str,
    digest_root: &'a str,
    start_inputs: Option<&'a BTreeMap<String, RefJson>>,
    request_id: Option<&'a str>,
}

fn publish_dir(home: &Home, lock: &crate::home::HomeLock, spec: PublishSpec<'_>) -> Result<()> {
    let PublishSpec {
        pending,
        final_path,
        owner,
        digest,
        digest_root,
        start_inputs,
        request_id,
    } = spec;
    // Hash the original's subtree (Work payload includes workbook/ and start-inputs/,
    // but digest covers only workbook/, storage §3.2); publication moves the whole payload.
    let payload = home.rel(pending)?;
    let verify_at = if digest_root.is_empty() {
        payload.clone()
    } else {
        payload.join_segment(digest_root)
    };
    let dst = home.rel(final_path)?;
    let pending_exists = fsx::managed_directory_exists(home, lock, pending)?;
    let final_exists = fsx::managed_directory_exists(home, lock, final_path)?;
    match (pending_exists, final_exists) {
        (true, false) => {
            let tree = fsx::open_managed_tree(home, lock, pending)?;
            fsx::verify_managed_tree_at(home, &tree, pending).map_err(|error| {
                integrity_error(
                    format!("Publication original {pending} identity recheck failed"),
                    error,
                )
            })?;
            verify_publish_object(home, &verify_at, owner, digest)?;
            verify_start_input_references(home, pending, final_path, owner, start_inputs)?;
            fsx::sync_managed_tree(home, lock, &tree).map_err(|error| {
                integrity_error(format!("Publication original {pending} sync failed"), error)
            })?;
            if let Some(request_id) = request_id {
                crate::failpoint::rendezvous("publish_after_tree_sync", request_id)
                    .map_err(|error| Error::io(pending, error))?;
            }
            fsx::verify_managed_tree_at(home, &tree, pending).map_err(|error| {
                integrity_error(
                    format!("Original {pending} identity recheck failed after publication sync"),
                    error,
                )
            })?;
            if let Some(parent) = dst.as_path().parent() {
                fsx::ensure_dirs_under(
                    home,
                    lock,
                    &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                )?;
            }
            fsx::rename_managed_tree_new(home, lock, &tree, final_path)?;
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(
                    format!("Original {final_path} identity recheck failed after publication"),
                    error,
                )
            })?;
            let final_verify = if digest_root.is_empty() {
                dst.clone()
            } else {
                dst.join_segment(digest_root)
            };
            verify_publish_object(home, &final_verify, owner, digest)?;
            verify_start_input_references(home, final_path, final_path, owner, start_inputs)?;
            fsx::sync_publish_final_root(home, lock, final_path)?;
            finish_publication(
                home,
                lock,
                &spec,
                &tree,
                &dst,
                &final_verify,
                "Original after read-only conversion",
            )
        }
        (false, true) => {
            // Only final exists: verify ownership/digest and accept completion (recovery §3.1);
            // Work digests cover only the digest_root (workbook/) subtree.
            let final_verify = if digest_root.is_empty() {
                dst.clone()
            } else {
                dst.join_segment(digest_root)
            };
            let tree = fsx::open_managed_tree(home, lock, final_path)?;
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(
                    format!("Published original {final_path} identity recheck failed"),
                    error,
                )
            })?;
            verify_publish_object(home, &final_verify, owner, digest)?;
            verify_start_input_references(home, final_path, final_path, owner, start_inputs)?;
            fsx::sync_managed_tree(home, lock, &tree).map_err(|error| {
                integrity_error(
                    format!("Published original {final_path} sync failed"),
                    error,
                )
            })?;
            fsx::verify_managed_tree_at(home, &tree, final_path).map_err(|error| {
                integrity_error(
                    format!("Original {final_path} identity recheck failed after sync"),
                    error,
                )
            })?;
            fsx::sync_publish_parents(home, lock, pending, final_path)?;
            fsx::sync_publish_final_root(home, lock, final_path)?;
            finish_publication(
                home,
                lock,
                &spec,
                &tree,
                &dst,
                &final_verify,
                "Published original",
            )
        }
        (false, false) => Err(Error::StoreCorrupt {
            detail: format!(
                "Publication object {final_path} and original {pending} are both missing"
            ),
        }),
        (true, true) => Err(Error::StoreCorrupt {
            detail: format!(
                "Publication object {final_path} and original {pending} both exist; must not overwrite"
            ),
        }),
    }
}

fn finish_publication(
    home: &Home,
    lock: &crate::home::HomeLock,
    spec: &PublishSpec<'_>,
    tree: &fsx::ManagedTree,
    destination: &AbsPath,
    verify_at: &AbsPath,
    identity_context: &str,
) -> Result<()> {
    // Only Workbook directories become read-only (contract §5.2: directories/root 0555, files 0444).
    // Keep Work directories writable for later status-card and Attempt writes; the frozen-copy
    // workbook/ subtree became read-only before commit (§5.4).
    if spec.owner.starts_with("workbook:") {
        fsx::set_tree_readonly_confined(home, lock, destination)?;
    } else {
        let workbook = destination.join_segment("workbook");
        fsx::set_tree_readonly_confined(home, lock, &workbook)?;
    }
    fsx::verify_managed_tree_at(home, tree, spec.final_path).map_err(|error| {
        integrity_error(
            format!(
                "{identity_context} {} identity recheck failed",
                spec.final_path
            ),
            error,
        )
    })?;
    verify_publish_object(home, verify_at, spec.owner, spec.digest)?;
    verify_start_input_references(
        home,
        spec.final_path,
        spec.final_path,
        spec.owner,
        spec.start_inputs,
    )?;
    verify_publish_final_state(home, lock, spec.pending, spec.final_path, tree)
}

fn verify_publish_final_state(
    home: &Home,
    lock: &crate::home::HomeLock,
    pending: &str,
    final_path: &str,
    tree: &fsx::ManagedTree,
) -> Result<()> {
    let pending_exists = fsx::managed_directory_exists(home, lock, pending)?;
    let final_exists = fsx::managed_directory_exists(home, lock, final_path)?;
    if pending_exists || !final_exists {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Four-case state changed before publication completed: pending={pending_exists}, final={final_exists}"
            ),
        });
    }
    fsx::verify_managed_tree_at(home, tree, final_path).map_err(|error| {
        integrity_error(
            format!("Publication final state {final_path} no longer binds the original"),
            error,
        )
    })?;
    Ok(())
}

fn integrity_error(context: String, error: Error) -> Error {
    match error {
        Error::Io { .. } | Error::RecoveryRequired { .. } | Error::StoreCorrupt { .. } => error,
        other => Error::StoreCorrupt {
            detail: format!("{context}：{other}"),
        },
    }
}

fn verify_publish_object(home: &Home, dir: &AbsPath, owner: &str, digest: &str) -> Result<()> {
    let Some(identity) = owner.strip_prefix("workbook:") else {
        return verify_owned_digest(home, dir, owner, digest);
    };
    let (expected_id, expected_version) =
        identity
            .split_once('@')
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Invalid Workbook publication owner {owner:?} format"),
            })?;
    let loaded =
        crate::workbook_repo::WorkbookRepo::load_managed_dir(home, dir).map_err(|error| {
            integrity_error(
                format!("Workbook publication directory {dir} manifest validation failed"),
                error,
            )
        })?;
    if loaded.manifest.id().as_str() != expected_id
        || loaded.manifest.version() != expected_version
        || loaded.digest.as_str() != digest
    {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Workbook publication directory {dir} manifest identity or digest differs from request owner"
            ),
        });
    }
    Ok(())
}

fn verify_start_input_references(
    home: &Home,
    source_root: &str,
    final_root: &str,
    owner: &str,
    start_inputs: Option<&BTreeMap<String, RefJson>>,
) -> Result<()> {
    if !owner.starts_with("work:") {
        if start_inputs.is_some() {
            return Err(Error::StoreCorrupt {
                detail: "Workbook publication effects unexpectedly carry Work start inputs"
                    .to_string(),
            });
        }
        return Ok(());
    }
    let inputs = start_inputs.ok_or_else(|| Error::StoreCorrupt {
        detail: "Work publication lacks verified start-input references".to_string(),
    })?;
    for (key, reference) in inputs {
        let expected_final = format!("{final_root}/start-inputs/{key}");
        if reference.path != expected_final || Sha256Hex::new(reference.sha256.clone()).is_err() {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Work publication start input {key} path or digest ownership is invalid"
                ),
            });
        }
        let source_path = format!("{source_root}/start-inputs/{key}");
        let path = home.rel(&source_path)?;
        let file = fsx::open_managed_regular(home, &path).map_err(|error| {
            integrity_error(
                format!(
                    "Work publication start input {key} is missing or not a managed regular file"
                ),
                error,
            )
        })?;
        let (digest, bytes) = file.sha256_bounded(fsx::MAX_FILE_BYTES).map_err(|error| {
            integrity_error(
                format!("Work publication start input {key} read failed"),
                error,
            )
        })?;
        if digest.as_str() != reference.sha256 || bytes != reference.bytes {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Work publication start input {key} actual bytes differ from committed reference"
                ),
            });
        }
    }
    Ok(())
}

/// Verify directory digest and sidecar ownership; mismatches yield STORE_CORRUPT.
fn verify_owned_digest(home: &Home, dir: &AbsPath, owner: &str, digest: &str) -> Result<()> {
    let got = crate::workbook_digest::digest_managed_dir_v2(home, dir).map_err(|error| {
        integrity_error(
            format!("Publication original {dir} digest read failed"),
            error,
        )
    })?;
    if got.as_str() != digest {
        return Err(Error::StoreCorrupt {
            detail: format!("Publication original {dir} digest differs from registration"),
        });
    }
    // Pending sidecar and Store references establish ownership jointly; owner strings only check presence.
    if owner.is_empty() {
        return Err(Error::StoreCorrupt {
            detail: "Publication effect has no owner".to_string(),
        });
    }
    Ok(())
}

/// delete_dir (storage §3.2/§3.3) handles only registered final/pending objects with matching digest and owner;
/// if both are absent, only this request's valid .deleted marker proves completion. Digest mismatch, both endpoints
/// present, or both endpoints/marker absent means unknown ownership/result; stop and preserve state.
fn delete_dir(
    home: &Home,
    lock: &crate::home::HomeLock,
    request_id: &str,
    pending: &str,
    final_path: &str,
    owner: &str,
    digest: &str,
) -> Result<()> {
    let internal_id = pending_internal_id(pending)?;
    if !owner.starts_with("workbook:") {
        return Err(Error::StoreCorrupt {
            detail: format!("remove request {request_id} owner is not Workbook"),
        });
    }
    crate::service::verify_pending_owner(home, internal_id, request_id, "remove_workbook")?;
    let marker_file = read_deleted_marker(home, lock, internal_id)?;
    let pending_exists = fsx::managed_directory_exists(home, lock, pending)?;
    let final_exists = fsx::managed_directory_exists(home, lock, final_path)?;
    if let Some(marker_file) = marker_file {
        if pending_exists || final_exists {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Deletion request {request_id} completion marker and directory object both exist"
                ),
            });
        }
        sync_deleted_marker(home, lock, internal_id, request_id, &marker_file)?;
        if fsx::managed_directory_exists(home, lock, pending)?
            || fsx::managed_directory_exists(home, lock, final_path)?
        {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Deletion request {request_id} gained a directory object during marker sync"
                ),
            });
        }
        return Ok(());
    }
    if pending_exists && final_exists {
        return Err(Error::StoreCorrupt {
            detail: format!("Deletion request {request_id} final and pending both exist"),
        });
    }
    if !pending_exists && !final_exists {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Deletion request {request_id} endpoints and valid completion marker are missing; result unknown"
            ),
        });
    }

    if final_exists {
        let tree = fsx::open_managed_tree(home, lock, final_path)?;
        fsx::verify_managed_tree_at(home, &tree, final_path)?;
        let final_dir = home.rel(final_path)?;
        verify_publish_object(home, &final_dir, owner, digest)?;
        fsx::make_managed_tree_writable(home, lock, &tree, final_path)?;
        fsx::verify_managed_tree_at(home, &tree, final_path)?;
        fsx::rename_managed_tree_new(home, lock, &tree, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        let pending_dir = home.rel(pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        fsx::make_managed_tree_writable(home, lock, &tree, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        crate::failpoint::maybe_exit("delete_payload_verified_before_remove");
        fsx::remove_managed_tree(home, lock, &tree, pending, request_id)?;
    } else {
        let tree = fsx::open_managed_tree(home, lock, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        // A crash may have moved the original before either rename parent was synced.
        fsx::sync_publish_parents(home, lock, final_path, pending)?;
        let pending_dir = home.rel(pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        fsx::make_managed_tree_writable(home, lock, &tree, pending)?;
        fsx::verify_managed_tree_at(home, &tree, pending)?;
        verify_publish_object(home, &pending_dir, owner, digest)?;
        crate::failpoint::maybe_exit("delete_payload_verified_before_remove");
        fsx::remove_managed_tree(home, lock, &tree, pending, request_id)?;
    }
    crate::failpoint::maybe_exit("delete_after_tree_removed_before_marker");
    if fsx::managed_directory_exists(home, lock, pending)?
        || fsx::managed_directory_exists(home, lock, final_path)?
    {
        return Err(Error::StoreCorrupt {
            detail: format!("Deletion request {request_id} still has a final or pending object"),
        });
    }
    write_deleted_marker(home, lock, internal_id, request_id)?;
    crate::failpoint::maybe_exit("delete_marker_synced_before_mark");
    if fsx::managed_directory_exists(home, lock, pending)?
        || fsx::managed_directory_exists(home, lock, final_path)?
    {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Deletion request {request_id} gained a directory object while writing its marker"
            ),
        });
    }
    Ok(())
}

pub(crate) fn pending_internal_id(pending: &str) -> Result<&str> {
    let segments = pending.split('/').collect::<Vec<_>>();
    if segments.len() != 3
        || segments[0] != "pending"
        || segments[2] != "payload"
        || uuid::Uuid::parse_str(segments[1]).is_err()
    {
        return Err(Error::StoreCorrupt {
            detail: format!("Invalid deletion-effect pending path {pending:?}"),
        });
    }
    Ok(segments[1])
}

/// Exclusively create and fsync pending/<internal_id>.deleted completion marker (§3.3).
fn write_deleted_marker(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
    request_id: &str,
) -> Result<()> {
    let _ = uuid::Uuid::parse_str(internal_id).map_err(|error| Error::StoreCorrupt {
        detail: format!("Invalid deletion marker internal ID: {error}"),
    })?;
    let relative = format!("pending/{internal_id}.deleted");
    if let Some(file) = read_deleted_marker(home, lock, internal_id)? {
        return sync_deleted_marker(home, lock, internal_id, request_id, &file);
    }
    let marker = DeletedMarker {
        format: "delete-complete/v1".to_string(),
        internal_id: internal_id.to_string(),
    };
    let mut content = serde_json::to_vec(&marker).map_err(|error| Error::StoreCorrupt {
        detail: format!("Deletion marker serialization failed: {error}"),
    })?;
    content.push(b'\n');
    let path = home.rel(&relative)?;
    let file = fsx::write_new_file_observed(home, lock, &path, &content)?;
    sync_deleted_marker(home, lock, internal_id, request_id, &file)?;
    Ok(())
}

pub(crate) fn read_deleted_marker(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
) -> Result<Option<fsx::SafeFile>> {
    let marker_path = home
        .pending_dir()
        .join_segment(&format!("{internal_id}.deleted"));
    let Some(file) =
        fsx::open_managed_optional_locked(home, lock, &marker_path).map_err(|error| {
            integrity_error(
                format!("Deletion marker {marker_path} identity validation failed"),
                error,
            )
        })?
    else {
        return Ok(None);
    };
    validate_deleted_marker_file(&file, &marker_path, internal_id)?;
    Ok(Some(file))
}

fn validate_deleted_marker_file(
    file: &fsx::SafeFile,
    marker_path: &AbsPath,
    internal_id: &str,
) -> Result<()> {
    let mut bytes = file.read_bounded(4096).map_err(|error| {
        integrity_error(format!("Deletion marker {marker_path} read failed"), error)
    })?;
    if bytes.pop() != Some(b'\n') {
        return Err(Error::StoreCorrupt {
            detail: format!("Deletion marker {marker_path} lacks a final newline"),
        });
    }
    let marker: DeletedMarker =
        serde_json::from_slice(&bytes).map_err(|error| Error::StoreCorrupt {
            detail: format!("Invalid deletion marker {marker_path} JSON: {error}"),
        })?;
    if marker.format != "delete-complete/v1" || marker.internal_id != internal_id {
        return Err(Error::StoreCorrupt {
            detail: format!("Deletion marker {marker_path} differs from this request internal_id"),
        });
    }
    let mut expected = serde_json::to_vec(&DeletedMarker {
        format: "delete-complete/v1".to_string(),
        internal_id: internal_id.to_string(),
    })
    .map_err(|error| Error::StoreCorrupt {
        detail: format!("Expected deletion marker serialization failed: {error}"),
    })?;
    expected.push(b'\n');
    let mut actual = bytes;
    actual.push(b'\n');
    if actual != expected {
        return Err(Error::StoreCorrupt {
            detail: format!("Deletion marker {marker_path} bytes violate the contract format"),
        });
    }
    Ok(())
}

fn sync_deleted_marker(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
    request_id: &str,
    file: &fsx::SafeFile,
) -> Result<()> {
    let relative = format!("pending/{internal_id}.deleted");
    let path = home.rel(&relative)?;
    validate_deleted_marker_file(file, &path, internal_id)?;
    crate::failpoint::rendezvous("delete_marker_after_validation_before_sync", request_id)
        .map_err(|error| Error::io(path.as_str(), error))?;
    fsx::sync_managed_regular_file_handle(home, lock, &relative, file).map_err(|error| {
        integrity_error(
            format!("Deletion marker {internal_id}.deleted sync failed"),
            error,
        )
    })?;
    validate_deleted_marker_file(file, &path, internal_id)?;
    fsx::verify_managed_file_bound(home, lock, &path, file).map_err(|error| {
        integrity_error(
            format!("Deletion marker {internal_id}.deleted path binding failed after sync"),
            error,
        )
    })
}

/// Independently validate a sha256 string.
pub fn valid_digest(s: &str) -> bool {
    Sha256Hex::new(s).is_ok()
}

#[cfg(test)]
mod seal_output_tests {
    use super::*;
    use crate::fsx::{ManagedFs, SafeFile};
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    fn checked_submit(path: &ManagedRelPath, sha256: &str, bytes: u64) -> CheckedEffects {
        CheckedEffects {
            ops: vec![
                EffectOp::SealOutputs {
                    refs: vec![RefJson {
                        path: path.as_str().to_string(),
                        sha256: sha256.to_string(),
                        bytes,
                    }],
                },
                EffectOp::RefreshStatusCard {
                    work_id: "w20260929-001".to_string(),
                },
            ],
            start_inputs: None,
            request_id: None,
        }
    }

    fn observed_file() -> (
        tempfile::TempDir,
        Home,
        crate::home::HomeLock,
        ManagedFs,
        ManagedRelPath,
        SafeFile,
        Sha256Hex,
        u64,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(
            (AbsPath::new(dir.path().to_str().unwrap()).unwrap()).as_str(),
        ))
        .unwrap();
        let lock = home.acquire_lock().unwrap();
        let fs = ManagedFs::open_existing(&home).unwrap();
        let path = ManagedRelPath::new(
            "works/w20260929-001/attempts/n/occurrence-001/attempt-000/outputs/out.md",
        )
        .unwrap();
        fs.ensure_dir(
            &lock,
            &ManagedRelPath::new(
                "works/w20260929-001/attempts/n/occurrence-001/attempt-000/outputs",
            )
            .unwrap(),
        )
        .unwrap();
        fs.write_new(&lock, &path, b"committed bytes").unwrap();
        let file = fs.open_regular(&path).unwrap();
        let (digest, bytes) = file.sha256_bounded(fsx::MAX_FILE_BYTES).unwrap();
        (dir, home, lock, fs, path, file, digest, bytes)
    }

    fn file_set(path: &AbsPath, file: SafeFile) -> BTreeMap<String, SafeFile> {
        BTreeMap::from([(path.as_str().to_string(), file)])
    }

    // Task: C002-T22
    #[test]
    fn same_inode_byte_change_stops_before_permission_change() {
        let (_dir, home, lock, _fs, relative, held, expected, bytes) = observed_file();
        let absolute = home.rel(relative.as_str()).unwrap();
        std::fs::write(absolute.as_path(), b"tampered bytes!").unwrap();
        let metadata = std::fs::metadata(absolute.as_path()).unwrap();
        assert_eq!(metadata.ino(), held.metadata().ino());
        assert_eq!(metadata.len(), bytes);
        let effects = checked_submit(&relative, expected.as_str(), bytes);

        assert!(
            execute_with_observed_outputs(
                &home,
                &lock,
                &effects,
                true,
                Some(&file_set(&absolute, held)),
            )
            .is_err()
        );
        assert_ne!(
            std::fs::metadata(absolute.as_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o222,
            0,
            "Digest changes must stop before chmod"
        );
    }

    // Task: C002-T22
    #[test]
    fn path_replacement_seals_observed_object_and_leaves_new_target_untouched() {
        let (_dir, home, lock, _fs, relative, held, expected, bytes) = observed_file();
        let target = home.rel(relative.as_str()).unwrap();
        let outside = tempfile::tempdir().unwrap();
        let replacement = outside.path().join("sentinel");
        let moved_original = outside.path().join("observed-original");
        std::fs::rename(target.as_path(), &moved_original).unwrap();
        std::fs::write(&replacement, b"outside sentinel").unwrap();
        let replacement_mode = replacement.metadata().unwrap().permissions().mode();
        std::os::unix::fs::symlink(&replacement, target.as_path()).unwrap();
        let effects = checked_submit(&relative, expected.as_str(), bytes);

        assert!(
            execute_with_observed_outputs(
                &home,
                &lock,
                &effects,
                true,
                Some(&file_set(&target, held)),
            )
            .is_err()
        );
        assert_eq!(
            moved_original.metadata().unwrap().permissions().mode() & 0o777,
            0o444
        );
        assert_eq!(std::fs::read(&replacement).unwrap(), b"outside sentinel");
        assert_eq!(
            replacement.metadata().unwrap().permissions().mode(),
            replacement_mode
        );
    }

    // Task: C002-T22
    #[test]
    fn normal_submit_requires_every_committed_ref_to_have_its_observed_handle() {
        let (_dir, home, lock, _fs, relative, _held, expected, bytes) = observed_file();
        let effects = checked_submit(&relative, expected.as_str(), bytes);

        assert!(matches!(
            execute_with_observed_outputs(&home, &lock, &effects, true, Some(&BTreeMap::new())),
            Err(Error::StoreCorrupt { detail }) if detail.contains("lacks an observation handle for output")
        ));
        assert_ne!(
            std::fs::metadata(home.rel(relative.as_str()).unwrap().as_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o222,
            0
        );
    }
}

#[cfg(test)]
mod publication_contract_tests {
    use super::*;

    // Task: C002-T48
    #[test]
    fn committed_start_publication_binds_each_identity_and_pending_component() {
        let directory = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(directory.path().to_str().unwrap())).unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/two-step")
            .canonicalize()
            .unwrap();
        let repo = crate::WorkbookRepo::new(home.clone());
        repo.add(&AbsPath::new(source.to_str().unwrap()).unwrap(), None)
            .unwrap();
        let service = crate::WorkService::new(home.clone());
        let response = service
            .start(
                crate::StartArgs {
                    workbook_id: "two-step".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: BTreeMap::from([(
                        "topic".into(),
                        crate::request::InputValue::Literal {
                            text: "real source".into(),
                        },
                    )]),
                },
                Some("publication-contract".into()),
            )
            .unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let state_json: String = connection
            .query_row("SELECT state_json FROM works", [], |row| row.get(0))
            .unwrap();
        let state: WorkState = serde_json::from_str(&state_json).unwrap();
        let audit: String = connection
            .query_row(
                "SELECT command_json FROM audit WHERE request_id='publication-contract'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let command: Command = serde_json::from_str(&audit).unwrap();
        let effects: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id='publication-contract'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let operations = decode_effects(&effects).unwrap();
        let loaded = repo.load("two-step", None).unwrap();
        let graph = &loaded.flow("default").unwrap().1;
        check_work_effects(
            &home,
            &response.request_id,
            &state,
            graph,
            &command,
            &response.reply,
            operations.clone(),
        )
        .unwrap();
        for field in [
            "pending_prefix",
            "pending_leaf",
            "pending_uuid",
            "command_work",
            "final",
            "owner",
            "digest",
        ] {
            let mut altered = operations.clone();
            let mut altered_command = command.clone();
            let EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                ..
            } = &mut altered[0]
            else {
                panic!("real start omitted publication")
            };
            let parts = pending.split('/').map(str::to_owned).collect::<Vec<_>>();
            match field {
                "pending_prefix" => *pending = format!("other/{}/payload", parts[1]),
                "pending_leaf" => *pending = format!("pending/{}/other", parts[1]),
                "pending_uuid" => *pending = "pending/not-a-uuid/payload".into(),
                "command_work" => {
                    let Command::Start { work_id, .. } = &mut altered_command else {
                        panic!("not start")
                    };
                    *work_id = sheltie_core::ids::WorkId::parse("2026-10-03-999-other").unwrap();
                }
                "final" => *final_path = "works/2026-10-03-999-other".into(),
                "owner" => *owner = "work:2026-10-03-999-other".into(),
                "digest" => *digest = "a".repeat(64),
                _ => unreachable!(),
            }
            assert!(
                matches!(
                    check_work_effects(
                        &home,
                        &response.request_id,
                        &state,
                        graph,
                        &altered_command,
                        &response.reply,
                        altered
                    ),
                    Err(Error::StoreCorrupt { .. })
                ),
                "{field}"
            );
        }
    }
}

#[cfg(all(test, feature = "failpoint"))]
mod terminal_effect_contract_tests {
    use super::*;
    use crate::fsx::controlled_object_tests::observe_change;
    use crate::fsx::owned_test_directory::OwnedTempDir;
    use std::os::unix::fs::PermissionsExt as _;

    fn registered_add() -> (OwnedTempDir, Home) {
        let directory = OwnedTempDir::new();
        let home = Home::resolve(Some(directory.path().join("home").to_str().unwrap())).unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/two-step")
            .canonicalize()
            .unwrap();
        crate::WorkbookRepo::new(home.clone())
            .add(
                &AbsPath::new(source.to_str().unwrap()).unwrap(),
                Some("effect-add".into()),
            )
            .unwrap();
        (directory, home)
    }

    fn registered_effect(home: &Home, id: &str) -> EffectOp {
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let raw: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id=?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        let ops: Vec<EffectOp> = serde_json::from_str(&raw).unwrap();
        ops.into_iter()
            .find(|op| matches!(op, EffectOp::PublishDir { .. } | EffectOp::DeleteDir { .. }))
            .unwrap()
    }

    fn unfinished_remove(home: &Home, id: &str) {
        crate::failpoint::arm_sync_error(home.root().as_str(), "publish_source_parent_sync")
            .unwrap();
        let error = crate::WorkbookRepo::new(home.clone())
            .remove("two-step", "1.0.1", Some(id.into()))
            .unwrap_err();
        crate::failpoint::disarm_sync_error().unwrap();
        assert!(
            matches!(
                error,
                Error::EffectPending {
                    committed: true,
                    ..
                }
            ),
            "{error:?}"
        );
    }

    // Task: C002-T55
    #[test]
    fn final_publication_refuses_an_additional_pending_tree_even_when_final_is_the_verified_original()
     {
        let (_directory, home) = registered_add();
        let EffectOp::PublishDir {
            pending,
            final_path,
            ..
        } = registered_effect(&home, "effect-add")
        else {
            panic!("actual add must publish")
        };
        let lock = home.acquire_lock().unwrap();
        let tree = fsx::open_managed_tree(&home, &lock, &final_path).unwrap();
        verify_publish_final_state(&home, &lock, &pending, &final_path, &tree).unwrap();
        let final_file = home.rel(&final_path).unwrap().join_segment("workbook.toml");
        let original = std::fs::read(final_file.as_path()).unwrap();
        let competitor = home.rel(&pending).unwrap();
        std::fs::create_dir_all(competitor.as_path()).unwrap();
        std::fs::write(
            competitor.as_path().join("competitor"),
            b"extra pending bytes",
        )
        .unwrap();
        let Error::StoreCorrupt { detail } =
            verify_publish_final_state(&home, &lock, &pending, &final_path, &tree).unwrap_err()
        else {
            panic!("both endpoints cannot be completed publication")
        };
        assert!(detail.contains("Four-case state changed"), "{detail}");
        assert_eq!(std::fs::read(final_file.as_path()).unwrap(), original);
        assert_eq!(
            std::fs::read(competitor.as_path().join("competitor")).unwrap(),
            b"extra pending bytes"
        );
    }

    // Task: C002-T55
    #[test]
    fn a_changed_owned_digest_stops_removal_before_moving_or_deleting_the_original() {
        let _serial = crate::failpoint::RENDEZVOUS_TEST_LOCK.lock().unwrap();
        let (_directory, home) = registered_add();
        unfinished_remove(&home, "digest-remove");
        let EffectOp::DeleteDir {
            pending,
            final_path,
            owner,
            digest,
        } = registered_effect(&home, "digest-remove")
        else {
            panic!("actual remove must delete")
        };
        let file = home.rel(&pending).unwrap().join_segment("README.md");
        let existed = file.as_path().exists();
        if existed {
            std::fs::set_permissions(file.as_path(), std::fs::Permissions::from_mode(0o600))
                .unwrap();
        }
        std::fs::write(file.as_path(), b"changed registered content").unwrap();
        let lock = home.acquire_lock().unwrap();
        let result = delete_dir(
            &home,
            &lock,
            "digest-remove",
            &pending,
            &final_path,
            &owner,
            &digest,
        );
        assert!(
            home.rel(&pending).unwrap().as_path().is_dir(),
            "digest refusal must precede deleting the only registered original"
        );
        assert!(result.is_err());
        assert_eq!(
            std::fs::read(file.as_path()).unwrap(),
            b"changed registered content"
        );
    }

    // Task: C002-T55
    #[test]
    fn a_valid_deleted_marker_never_authorizes_a_pending_or_final_competitor() {
        for endpoint in ["pending", "final"] {
            let (_directory, home) = registered_add();
            crate::WorkbookRepo::new(home.clone())
                .remove("two-step", "1.0.1", Some("marker-remove".into()))
                .unwrap();
            let EffectOp::DeleteDir {
                pending,
                final_path,
                owner,
                digest,
            } = registered_effect(&home, "marker-remove")
            else {
                panic!("actual remove must delete")
            };
            let lock = home.acquire_lock().unwrap();
            delete_dir(
                &home,
                &lock,
                "marker-remove",
                &pending,
                &final_path,
                &owner,
                &digest,
            )
            .unwrap();
            let path = home
                .rel(if endpoint == "pending" {
                    &pending
                } else {
                    &final_path
                })
                .unwrap();
            std::fs::create_dir_all(path.as_path()).unwrap();
            std::fs::write(path.as_path().join("competitor"), b"preserved competitor").unwrap();
            let Error::StoreCorrupt { detail } = delete_dir(
                &home,
                &lock,
                "marker-remove",
                &pending,
                &final_path,
                &owner,
                &digest,
            )
            .unwrap_err() else {
                panic!("a valid completion marker cannot coexist with an endpoint")
            };
            assert!(
                detail.contains("completion marker and directory object both exist"),
                "{detail}"
            );
            assert_eq!(
                std::fs::read(path.as_path().join("competitor")).unwrap(),
                b"preserved competitor"
            );
        }
    }

    // Task: C002-T55
    #[test]
    fn deleted_marker_checks_each_late_endpoint_after_sync_without_discarding_the_competitor() {
        for endpoint in ["pending", "final"] {
            let (_directory, home) = registered_add();
            crate::WorkbookRepo::new(home.clone())
                .remove("two-step", "1.0.1", Some("marker-sync-remove".into()))
                .unwrap();
            let EffectOp::DeleteDir {
                pending,
                final_path,
                owner,
                digest,
            } = registered_effect(&home, "marker-sync-remove")
            else {
                panic!("actual remove must delete")
            };
            let lock = home.acquire_lock().unwrap();
            let path = home
                .rel(if endpoint == "pending" {
                    &pending
                } else {
                    &final_path
                })
                .unwrap();
            let worker_home = home.clone();
            let result = observe_change(
                "delete_marker_after_validation_before_sync",
                "marker-sync-remove",
                move || {
                    delete_dir(
                        &worker_home,
                        &lock,
                        "marker-sync-remove",
                        &pending,
                        &final_path,
                        &owner,
                        &digest,
                    )
                },
                || {
                    std::fs::create_dir_all(path.as_path()).unwrap();
                    std::fs::write(
                        path.as_path().join("competitor"),
                        b"preserved late competitor",
                    )
                    .unwrap();
                },
            );
            assert!(matches!(result, Err(Error::StoreCorrupt { .. })));
            assert_eq!(
                std::fs::read(path.as_path().join("competitor")).unwrap(),
                b"preserved late competitor"
            );
        }
    }

    // Task: C002-T55
    #[test]
    fn deletion_rechecks_single_late_endpoints_after_tree_removal_and_after_marker_publication() {
        for checkpoint in [
            "delete_after_tree_removed_before_marker",
            "delete_marker_synced_before_mark",
        ] {
            for endpoint in ["pending", "final"] {
                let (_directory, home) = registered_add();
                let _serial = crate::failpoint::RENDEZVOUS_TEST_LOCK.lock().unwrap();
                unfinished_remove(&home, "late-remove");
                drop(_serial);
                let EffectOp::DeleteDir {
                    pending,
                    final_path,
                    owner,
                    digest,
                } = registered_effect(&home, "late-remove")
                else {
                    panic!("actual remove must delete")
                };
                let marker = home.pending_dir().join_segment(&format!(
                    "{}.deleted",
                    pending_internal_id(&pending).unwrap()
                ));
                assert!(!marker.as_path().exists());
                let lock = home.acquire_lock().unwrap();
                let path = home
                    .rel(if endpoint == "pending" {
                        &pending
                    } else {
                        &final_path
                    })
                    .unwrap();
                let worker_home = home.clone();
                let result = observe_change(
                    checkpoint,
                    checkpoint,
                    move || {
                        delete_dir(
                            &worker_home,
                            &lock,
                            "late-remove",
                            &pending,
                            &final_path,
                            &owner,
                            &digest,
                        )
                    },
                    || {
                        std::fs::create_dir_all(path.as_path()).unwrap();
                        std::fs::write(
                            path.as_path().join("competitor"),
                            b"preserved after-removal competitor",
                        )
                        .unwrap();
                    },
                );
                if checkpoint == "delete_after_tree_removed_before_marker" {
                    assert!(
                        !marker.as_path().exists(),
                        "endpoint refusal must precede creating a false completion marker"
                    );
                } else {
                    assert!(marker.as_path().is_file());
                }
                assert!(matches!(result, Err(Error::StoreCorrupt { .. })));
                assert_eq!(
                    std::fs::read(path.as_path().join("competitor")).unwrap(),
                    b"preserved after-removal competitor"
                );
            }
        }
    }
}

#[cfg(all(test, feature = "failpoint"))]
mod owned_work_digest_contract_tests {
    use super::*;
    use crate::fsx::owned_test_directory::OwnedTempDir;
    use std::os::unix::fs::PermissionsExt as _;

    // Task: C002-T55
    #[test]
    fn a_work_owned_frozen_tree_checks_its_actual_bytes_against_the_registered_digest() {
        let directory = OwnedTempDir::new();
        let home = Home::resolve(Some(directory.path().join("home").to_str().unwrap())).unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/two-step")
            .canonicalize()
            .unwrap();
        crate::WorkbookRepo::new(home.clone())
            .add(&AbsPath::new(source.to_str().unwrap()).unwrap(), None)
            .unwrap();
        let service = crate::WorkService::new(home.clone());
        service
            .start(
                crate::StartArgs {
                    workbook_id: "two-step".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: BTreeMap::from([(
                        "topic".into(),
                        crate::request::InputValue::Literal {
                            text: "real owned source".into(),
                        },
                    )]),
                },
                Some("owned-work-start".into()),
            )
            .unwrap();
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let raw: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id='owned-work-start'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let ops: Vec<EffectOp> = serde_json::from_str(&raw).unwrap();
        let EffectOp::PublishDir {
            final_path,
            owner,
            digest,
            digest_root,
            ..
        } = ops
            .into_iter()
            .find(|op| matches!(op, EffectOp::PublishDir { .. }))
            .unwrap()
        else {
            unreachable!()
        };
        assert!(owner.starts_with("work:"));
        assert_eq!(digest_root, "workbook");
        let tree = home.rel(&final_path).unwrap().join_segment("workbook");
        verify_owned_digest(&home, &tree, &owner, &digest).unwrap();
        let file = tree.join_segment("workbook.toml");
        std::fs::set_permissions(file.as_path(), std::fs::Permissions::from_mode(0o600)).unwrap();
        let mut bytes = std::fs::read(file.as_path()).unwrap();
        bytes.extend_from_slice(b"\n# genuine changed frozen source\n");
        std::fs::write(file.as_path(), &bytes).unwrap();
        let Error::StoreCorrupt { detail } =
            verify_owned_digest(&home, &tree, &owner, &digest).unwrap_err()
        else {
            panic!("registered Work digest must reject changed actual bytes")
        };
        assert!(
            detail.contains("digest differs from registration"),
            "{detail}"
        );
        assert_eq!(std::fs::read(file.as_path()).unwrap(), bytes);
    }
}
