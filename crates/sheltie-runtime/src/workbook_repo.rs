//! Workbook repository: add/list/load/remove/verify; rules in storage contract §5 and protocol §3.

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{FlowDef, Graph, compile, parse_flow};
use sheltie_core::path::{AbsPath, RelPath};
use sheltie_core::workbook::Manifest;
use sheltie_core::workbook::parse_manifest;

use crate::effects::{EffectOp, decode_effects, encode_effects};
use crate::error::{Error, Result};
use crate::fsx::{ExternalReadTree, MAX_FILE_BYTES as FS_MAX_FILE_BYTES, ManagedRelPath};
use crate::home::Home;
use crate::request::{RequestIntent, lexical_abs};
use crate::service::stage_pending;
use crate::store::{CommitOutcome, Store, WorkbookRow};
use sheltie_core::work::Context;

/// workbook add commit-time snapshot (GF-15).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddedSnapshot {
    pub request_id: String,
    pub replayed: bool,
    pub data: serde_json::Value,
}

/// workbook remove commit-time snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemovedSnapshot {
    pub request_id: String,
    pub replayed: bool,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case", deny_unknown_fields)]
enum WorkbookAuditCommand {
    AddWorkbook { source: String },
    RemoveWorkbook { target: String },
}

fn decode_added_snapshot(request_id: &str, reply_json: &str) -> Result<AddedSnapshot> {
    let mut s: AddedSnapshot =
        serde_json::from_str(reply_json).map_err(|e| Error::StoreCorrupt {
            detail: format!("Cannot decode response in requests table: {e}"),
        })?;
    if s.request_id != request_id || s.replayed {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Workbook add request {request_id} historical snapshot identity is invalid"
            ),
        });
    }
    s.replayed = true;
    Ok(s)
}

fn decode_removed_snapshot(request_id: &str, reply_json: &str) -> Result<RemovedSnapshot> {
    let mut s: RemovedSnapshot =
        serde_json::from_str(reply_json).map_err(|e| Error::StoreCorrupt {
            detail: format!("Cannot decode response in requests table: {e}"),
        })?;
    if s.request_id != request_id || s.replayed {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Workbook remove request {request_id} historical snapshot identity is invalid"
            ),
        });
    }
    s.replayed = true;
    Ok(s)
}

/// workbook add return data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Added {
    pub id: String,
    pub version: String,
    pub digest: Sha256Hex,
    pub flows: Vec<String>,
    pub requires: Vec<String>,
}

/// Installed Workbook: manifest, graphs, and directory.
#[derive(Debug, Clone)]
pub struct LoadedWorkbook {
    pub manifest: Manifest,
    pub flows: Vec<(FlowDef, Graph)>,
    pub dir: AbsPath,
    pub digest: Sha256Hex,
    pub resource_files: std::collections::BTreeMap<RelPath, sheltie_core::work::ObservedFile>,
    pub instructions: std::collections::BTreeMap<RelPath, String>,
    pub pending_publish: bool,
}

impl LoadedWorkbook {
    pub fn flow(&self, id: &str) -> Option<&(FlowDef, Graph)> {
        self.flows.iter().find(|(f, _)| f.id().as_str() == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Removed {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyStatus {
    Ok,
    Tampered,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyRow {
    pub id: String,
    pub version: String,
    pub status: VerifyStatus,
    pub pending_publish: bool,
}

/// workbook list registered identity and read-only publication state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbookListItem {
    pub id: String,
    pub version: String,
    pub digest: String,
    pub pending_publish: bool,
}

/// Repository handle.
#[derive(Debug, Clone)]
pub struct WorkbookRepo {
    home: Home,
    store: Store,
}

impl WorkbookRepo {
    pub fn new(home: Home) -> Self {
        let store = Store::deferred_for_home(&home, crate::store::OpenMode::ReadOnly);
        Self { home, store }
    }

    pub(crate) fn with_store(home: Home, store: Store) -> Self {
        Self { home, store }
    }

    /// workbook add <dir> (storage §5.2), using the same write path as Work:
    /// unlocked replay preflight, root lock, recovery, pending staging, then only on the final copy
    /// parse/compile/digest, one transaction (requests/workbooks/audit), and publication.
    /// Same request-id replay returns the original snapshot; source changes do not reinstall (§2.1).
    pub fn add(&self, dir: &AbsPath, request_id: Option<String>) -> Result<AddedSnapshot> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::AddWorkbook {
            source: lexical_abs(dir.as_str())?,
        };
        let intent_hash = intent.hash();
        let command_json = match &intent {
            RequestIntent::AddWorkbook { .. } => {
                serde_json::json!({"intent": "add_workbook", "source": intent_hash.as_str()})
                    .to_string()
            }
            _ => unreachable!("AddWorkbook method must construct AddWorkbook intent"),
        };
        // Unlocked preflight only determines whether to read the source; replay first locks and recovers prior effects.
        let replay_exists = match Store::open_for_home(&self.home, crate::store::OpenMode::ReadOnly)
        {
            Ok(ro) => {
                if let Some(hash) = ro.lookup_request_hash(&request_id)? {
                    if hash != intent_hash.as_str() {
                        return Err(Error::RequestConflict {
                            request_id: request_id.clone(),
                        });
                    }
                    true
                } else {
                    false
                }
            }
            Err(Error::NotFound { .. }) => false,
            Err(error) => return Err(error),
        };

        if !replay_exists {
            let source = ExternalReadTree::open(dir)?;
            source.validate_sizes()?;
            if !source
                .files()
                .iter()
                .any(|file| file.relative.as_str() == "workbook.toml")
            {
                return Err(Error::NotFound {
                    what: dir.join_segment("workbook.toml").to_string(),
                });
            }
        }
        let session = crate::session::WriteSession::open_or_create(&self.home)?;
        let lock = session.lock;
        let repo = Self::with_store(self.home.clone(), session.store.clone());
        repo.recover_before_write(&lock, &request_id, intent_hash.as_str())?;
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if row.intent_hash != intent_hash.as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            // Recovery completed effects; return the snapshot here.
            return decode_added_snapshot(&request_id, &row.reply_json);
        }

        // Check source structure before locking; create root lock/Store only for valid new additions.
        let internal_id = uuid::Uuid::now_v7().simple().to_string();
        let payload = stage_pending(&self.home, &lock, &internal_id, &request_id, "add_workbook")?;

        // Parse/compile/digest only the final copy (§5.2, step 3).
        Self::copy_confined(&self.home, &lock, dir, &payload)?;
        let loaded = repo.load_dir(&payload)?;
        let flow_ids: Vec<String> = loaded
            .flows
            .iter()
            .map(|(def, _)| def.id().as_str().to_string())
            .collect();
        let final_dir = self
            .home
            .workbook_dir(loaded.manifest.id().as_str(), loaded.manifest.version());
        let rel_dir = self.home.to_rel(&final_dir)?;
        let ctx = Context {
            now: crate::observe::now(),
            principal: crate::observe::principal(),
        };
        let added = Added {
            id: loaded.manifest.id().as_str().to_string(),
            version: loaded.manifest.version().to_string(),
            digest: loaded.digest.clone(),
            flows: flow_ids,
            requires: loaded
                .manifest
                .requires()
                .iter()
                .map(|r| format!("{}:{}", r.kind().as_str(), r.name()))
                .collect(),
        };
        let snapshot = AddedSnapshot {
            request_id: request_id.clone(),
            replayed: false,
            data: serde_json::to_value(&added).unwrap_or(serde_json::Value::Null),
        };
        let input = crate::store::CommitInput {
            work_id: None,
            workbook_in_use_check: None,
            workbook_insert: Some(WorkbookRow {
                id: loaded.manifest.id().as_str().to_string(),
                version: loaded.manifest.version().to_string(),
                digest: loaded.digest.as_str().to_string(),
                dir: rel_dir.clone(),
                added_at: ctx.now.as_str().to_string(),
            }),
            workbook_delete: None,
            expected_revision: None,
            state: None,
            request_id: request_id.clone(),
            intent_hash: intent_hash.as_str().to_string(),
            reply_json: serde_json::to_string(&snapshot).unwrap_or_default(),
            effects_json: encode_effects(&[EffectOp::PublishDir {
                pending: self.home.to_rel(&payload)?,
                final_path: rel_dir,
                owner: format!(
                    "workbook:{}@{}",
                    loaded.manifest.id(),
                    loaded.manifest.version()
                ),
                digest: loaded.digest.as_str().to_string(),
                digest_root: String::new(),
            }]),
            principal: ctx.principal,
            command_json,
            at: ctx.now,
        };
        let replayed = matches!(repo.store.commit(input)?, CommitOutcome::Replayed { .. });
        if !replayed {
            crate::failpoint::maybe_exit("after_commit_before_effects");
        }
        repo.recover_finish_request(&lock, &request_id, false)?;
        if replayed {
            let row =
                repo.store
                    .inspect_request(&request_id)?
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("Replay request {request_id} disappeared from requests"),
                    })?;
            decode_added_snapshot(&request_id, &row.reply_json)
        } else {
            Ok(snapshot)
        }
    }

    fn checked_request_for(
        &self,
        request_id: &str,
    ) -> Result<(
        crate::recovery::CheckedRequest,
        Option<crate::recovery::AddedSnapshotData>,
    )> {
        let metadata = self
            .store
            .inspect_request_metadata(request_id)?
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Missing persistent record for Workbook request {request_id}"),
            })?;
        self.load_checked_request(request_id, &metadata)
            .map_err(crate::recovery::RequestLoadError::into_error)
    }

    pub(crate) fn load_checked_request(
        &self,
        request_id: &str,
        metadata: &crate::store::read::RequestMetadata,
    ) -> crate::recovery::RequestLoadResult<(
        crate::recovery::CheckedRequest,
        Option<crate::recovery::AddedSnapshotData>,
    )> {
        let row = metadata;
        if row.work_id.is_some() {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook request {request_id} must not belong to Work"),
            }
            .into());
        }
        let audits = self.store.audit_rows(request_id)?;
        let [audit] = audits.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook request {request_id} requires exactly one audit row"),
            }
            .into());
        };
        if !audit.work_id.is_empty() || audit.revision != 0 {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook request {request_id} audit contains Work ownership"),
            }
            .into());
        }
        if row.at != audit.at {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook request {request_id} audit time differs from requests row"
                ),
            }
            .into());
        }
        Sha256Hex::new(row.intent_hash.clone()).map_err(|error| Error::StoreCorrupt {
            detail: format!("WorkbookRequest {request_id} intent_hash is invalid: {error}"),
        })?;
        let command: WorkbookAuditCommand =
            serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                detail: format!(
                    "Workbook request {request_id} audit command violates the contract: {error}"
                ),
            })?;
        let (identity, response_data, added_data) = match command {
            WorkbookAuditCommand::AddWorkbook { source } => {
                if Sha256Hex::new(source.clone()).is_err() || row.intent_hash != source {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook add request {request_id} intent_hash differs from audit"
                        ),
                    }
                    .into());
                }
                let snapshot: AddedSnapshot =
                    serde_json::from_str(&row.reply_json).map_err(|error| Error::StoreCorrupt {
                        detail: format!(
                            "Cannot decode Workbook add request {request_id} snapshot: {error}"
                        ),
                    })?;
                if snapshot.request_id != request_id || snapshot.replayed {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook add request {request_id} snapshot identity is invalid"
                        ),
                    }
                    .into());
                }
                let response_data = snapshot.data.clone();
                let data: crate::recovery::AddedSnapshotData =
                    serde_json::from_value(snapshot.data).map_err(|error| Error::StoreCorrupt {
                        detail: format!(
                            "Workbook add request {request_id} snapshot.data violates the contract: {error}"
                        ),
                    })?;
                let unique_flows = data.flows.iter().collect::<std::collections::BTreeSet<_>>();
                if data.flows.is_empty()
                    || unique_flows.len() != data.flows.len()
                    || data
                        .flows
                        .iter()
                        .any(|flow| sheltie_core::ids::FlowId::new(flow).is_err())
                    || data.requires.iter().any(|requirement| {
                        requirement
                            .split_once(':')
                            .is_none_or(|(kind, name)| kind.is_empty() || name.is_empty())
                    })
                {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook add request {request_id} snapshot.data fields are invalid"
                        ),
                    }
                    .into());
                }
                crate::effects::validate_workbook_identity(&data.id, &data.version)?;
                Sha256Hex::new(data.digest.clone()).map_err(|error| Error::StoreCorrupt {
                    detail: format!(
                        "Workbook add request {request_id} snapshot.digest is invalid: {error}"
                    ),
                })?;
                (
                    crate::effects::WorkbookEffectIdentity::Add {
                        id: data.id.clone(),
                        version: data.version.clone(),
                        digest: data.digest.clone(),
                    },
                    response_data,
                    Some(data),
                )
            }
            WorkbookAuditCommand::RemoveWorkbook { target } => {
                let (id, version) = target.split_once('@').ok_or_else(|| Error::StoreCorrupt {
                    detail: format!("Workbook remove request {request_id} audit target is invalid"),
                })?;
                let request_intent = RequestIntent::RemoveWorkbook {
                    id: id.to_string(),
                    version: version.to_string(),
                };
                if row.intent_hash != request_intent.hash().as_str() {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook remove request {request_id} intent_hash differs from audit"
                        ),
                    }
                    .into());
                }
                let snapshot: RemovedSnapshot =
                    serde_json::from_str(&row.reply_json).map_err(|error| Error::StoreCorrupt {
                        detail: format!(
                            "Cannot decode Workbook remove request {request_id} snapshot: {error}"
                        ),
                    })?;
                if snapshot.request_id != request_id || snapshot.replayed {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook remove request {request_id} snapshot identity is invalid"
                        ),
                    }
                    .into());
                }
                let response_data = snapshot.data.clone();
                let data: crate::recovery::RemovedSnapshotData =
                    serde_json::from_value(snapshot.data).map_err(|error| Error::StoreCorrupt {
                        detail: format!(
                            "Workbook remove request {request_id} snapshot.data violates the contract: {error}"
                        ),
                    })?;
                if data.id != id || data.version != version {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook remove request {request_id} snapshot identity differs from audit"
                        ),
                    }
                    .into());
                }
                crate::effects::validate_workbook_identity(&data.id, &data.version)?;
                (
                    crate::effects::WorkbookEffectIdentity::Remove {
                        id: id.to_string(),
                        version: version.to_string(),
                    },
                    response_data,
                    None,
                )
            }
        };
        let add_effects = if matches!(
            &identity,
            crate::effects::WorkbookEffectIdentity::Add { .. }
        ) {
            let effects_json = self
                .store
                .lookup_request_effects(request_id)?
                .ok_or_else(|| Error::StoreCorrupt {
                    detail: format!(
                        "Workbook add request {request_id} lacks publication registration"
                    ),
                })?;
            crate::effects::check_workbook_add_snapshot_target(&identity, &effects_json)?;
            Some(effects_json)
        } else {
            None
        };
        let original = crate::recovery::workbook_original_response(request_id, response_data);
        let effects_result: Result<_> = (|| {
            let mut row =
                self.store
                    .inspect_request(request_id)?
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("Missing effect record for Workbook request {request_id}"),
                    })?;
            metadata.check_row(request_id, &row)?;
            let registered = if let crate::effects::WorkbookEffectIdentity::Add {
                id,
                version,
                ..
            } = &identity
            {
                if row.published {
                    None
                } else {
                    let registered = self
                        .store
                        .workbook_versions(id)?
                        .into_iter()
                        .find(|candidate| candidate.version == *version)
                        .ok_or_else(|| Error::StoreCorrupt {
                            detail: format!(
                                "Workbook add request {request_id} lacks a registration row"
                            ),
                        })?;
                    crate::load::validate_workbook_row(&self.home, &registered)?;
                    Some(registered)
                }
            } else {
                None
            };
            let effects = match add_effects {
                Some(effects_json) => {
                    if effects_json != row.effects_json {
                        return Err(Error::StoreCorrupt {
                            detail: format!(
                                "Workbook add request {request_id} publication registration changed during verification"
                            ),
                        });
                    }
                    decode_effects(&effects_json)?
                }
                None => decode_effects(&row.effects_json)?,
            };
            let checked = crate::effects::check_workbook_effects(
                request_id,
                &audit.work_id,
                audit.revision,
                &audit.at,
                identity,
                effects,
                if row.published {
                    None
                } else {
                    registered.as_ref()
                },
            )?;
            let (pending, operation) = match checked.as_slice() {
                [EffectOp::PublishDir { pending, .. }] => (pending, "add_workbook"),
                [EffectOp::DeleteDir { pending, .. }] => (pending, "remove_workbook"),
                _ => {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook request {request_id} Checked effect shape is invalid"
                        ),
                    });
                }
            };
            let internal_id = pending
                .split('/')
                .nth(1)
                .ok_or_else(|| Error::StoreCorrupt {
                    detail: format!("Workbook request {request_id} pending path is invalid"),
                })?;
            if !row.published {
                if let Err(owner_error) = crate::service::verify_pending_owner(
                    &self.home,
                    internal_id,
                    request_id,
                    operation,
                ) {
                    let current = self.store.inspect_request(request_id)?;
                    match current {
                        Some(current)
                            if current.published
                                && current.intent_hash == row.intent_hash
                                && current.reply_json == row.reply_json
                                && current.effects_json == row.effects_json
                                && current.work_id == row.work_id
                                && current.at == row.at =>
                        {
                            row = current;
                        }
                        _ => return Err(owner_error),
                    }
                }
            }
            Ok((row, checked))
        })();
        let (row, effects) = effects_result.map_err(|cause| {
            crate::recovery::RequestLoadError::with_original(cause, original.clone())
        })?;
        Ok((
            crate::recovery::CheckedRequest {
                row,
                original,
                effects,
            },
            added_data,
        ))
    }

    fn recover_before_write(
        &self,
        lock: &crate::home::HomeLock,
        request_id: &str,
        intent_hash: &str,
    ) -> Result<()> {
        crate::recovery::before_write(&self.home, &self.store, lock, request_id, intent_hash, self)
    }

    fn recover_finish_request(
        &self,
        lock: &crate::home::HomeLock,
        request_id: &str,
        verify_published: bool,
    ) -> Result<()> {
        crate::recovery::finish_request(
            &self.home,
            &self.store,
            lock,
            request_id,
            None,
            verify_published,
            self,
        )
    }

    pub fn list(&self) -> Result<Vec<WorkbookListItem>> {
        let references = crate::pending::PendingReferenceIndex::load(&self.store)?;
        self.store
            .list_workbooks()?
            .into_iter()
            .map(|row| {
                let loaded = self.load_row(&row, &references)?;
                Ok(WorkbookListItem {
                    id: row.id,
                    version: row.version,
                    digest: row.digest,
                    pending_publish: loaded.pending_publish,
                })
            })
            .collect()
    }

    fn publication_for_row(
        &self,
        row: &WorkbookRow,
        references: &crate::pending::PendingReferenceIndex,
    ) -> Result<crate::pending::PublishLocation> {
        let owner = format!("workbook:{}@{}", row.id, row.version);
        crate::failpoint::rendezvous("pending_after_reference_index", &row.dir)
            .map_err(|error| Error::io(&row.dir, error))?;
        let publisher_ids = references
            .lifecycle_references_for_workbook(&owner, &row.dir)
            .into_iter()
            .map(|reference| reference.request_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let mut latest = None;
        for audit in self.store.audit_history()? {
            let raw: serde_json::Value =
                serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                    detail: format!("Invalid audit {} command JSON: {error}", audit.request_id),
                })?;
            let workbook_command = matches!(
                raw.get("intent").and_then(serde_json::Value::as_str),
                Some("add_workbook" | "remove_workbook")
            );
            if !workbook_command && !publisher_ids.contains(audit.request_id.as_str()) {
                continue;
            }
            let command: WorkbookAuditCommand =
                serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                    detail: format!(
                        "Cannot decode Workbook audit {} command: {error}",
                        audit.request_id
                    ),
                })?;
            let (matches, is_add) = match &command {
                WorkbookAuditCommand::AddWorkbook { .. } => {
                    let request =
                        self.store
                            .inspect_request(&audit.request_id)?
                            .ok_or_else(|| Error::StoreCorrupt {
                                detail: format!(
                                    "Workbook audit {} lacks a requests row",
                                    audit.request_id
                                ),
                            })?;
                    let snapshot: AddedSnapshot = serde_json::from_str(&request.reply_json)
                        .map_err(|error| Error::StoreCorrupt {
                            detail: format!(
                                "Cannot decode Workbook add request {} snapshot: {error}",
                                audit.request_id
                            ),
                        })?;
                    let data: crate::recovery::AddedSnapshotData =
                        serde_json::from_value(snapshot.data).map_err(|error| {
                            Error::StoreCorrupt {
                                detail: format!(
                                    "Workbook add request {} snapshot identity is invalid: {error}",
                                    audit.request_id
                                ),
                            }
                        })?;
                    (
                        (data.id == row.id && data.version == row.version)
                            || publisher_ids.contains(audit.request_id.as_str()),
                        true,
                    )
                }
                WorkbookAuditCommand::RemoveWorkbook { target } => (
                    target == &format!("{}@{}", row.id, row.version)
                        || publisher_ids.contains(audit.request_id.as_str()),
                    false,
                ),
            };
            if matches {
                latest = Some((audit.seq, audit.request_id, is_add));
            }
        }
        let Some((_, request_id, true)) = latest else {
            let detail = match latest {
                Some((_, request_id, false)) => {
                    // Validate the latest removal's own persisted closure before rejecting a row
                    // that claims the removed lifecycle still exists.
                    self.checked_request_for(&request_id)?;
                    format!(
                        "Workbook {owner} latest lifecycle is remove, but its workbooks row remains"
                    )
                }
                _ => format!("Workbook {owner} lacks a corresponding successful add request"),
            };
            return Err(Error::StoreCorrupt { detail });
        };

        let (checked_request, added_data) = self.checked_request_for(&request_id)?;
        let request = &checked_request.row;
        let checked = &checked_request.effects;
        let audits = self.store.audit_rows(&request_id)?;
        let [audit] = audits.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook add request {request_id} requires exactly one audit row"),
            });
        };
        if request.work_id.is_some()
            || !audit.work_id.is_empty()
            || audit.revision != 0
            || audit.at != row.added_at
            || request.at != row.added_at
        {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook {owner} workbooks.added_at differs from current add audit"
                ),
            });
        }
        let data = added_data.ok_or_else(|| Error::StoreCorrupt {
            detail: format!(
                "Workbook add request {request_id} verified payload lacks add identity"
            ),
        })?;
        if data.id != row.id || data.version != row.version || data.digest != row.digest {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook {owner} identity/digest differs from current add response"
                ),
            });
        }
        let [
            EffectOp::PublishDir {
                pending,
                final_path,
                owner: effect_owner,
                digest,
                digest_root,
            },
        ] = checked.as_slice()
        else {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook add request {request_id} publication effect shape is invalid"
                ),
            });
        };
        if effect_owner != &owner
            || final_path != &row.dir
            || digest != &row.digest
            || !digest_root.is_empty()
        {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook add request {request_id} effect differs from current workbooks row"
                ),
            });
        }
        let internal_id = crate::effects::pending_internal_id(pending)?;
        let Some(reference) = references.get(internal_id) else {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook add request {request_id} lacks a pending-reference index entry"
                ),
            });
        };
        if reference.request_id != request_id
            || reference.pending != *pending
            || reference.final_path != *final_path
            || reference.owner != *effect_owner
            || reference.digest != *digest
            || reference.digest_root != *digest_root
        {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook add request {request_id} differs from pending-reference index"
                ),
            });
        }
        Ok(crate::pending::PublishLocation {
            pending: pending.clone(),
            final_path: final_path.clone(),
            pending_publish: !request.published,
        })
    }

    fn load_row_from_directory(
        &self,
        row: &WorkbookRow,
        directory: &AbsPath,
        pending_publish: bool,
    ) -> Result<LoadedWorkbook> {
        let mut loaded = match Self::load_managed_dir(&self.home, directory) {
            Err(error @ Error::NotFound { .. }) => {
                let relative = self.home.to_rel(directory)?;
                if !crate::fsx::managed_directory_exists_readonly(&self.home, &relative)? {
                    return Err(error);
                }
                return Err(Error::WorkbookTampered {
                    results: vec![VerifyRow {
                        id: row.id.clone(),
                        version: row.version.clone(),
                        status: VerifyStatus::Tampered,
                        pending_publish,
                    }],
                });
            }
            result => result?,
        };
        if loaded.digest.as_str() != row.digest {
            return Err(Error::WorkbookTampered {
                results: vec![VerifyRow {
                    id: row.id.clone(),
                    version: row.version.clone(),
                    status: VerifyStatus::Tampered,
                    pending_publish,
                }],
            });
        }
        if loaded.manifest.id().as_str() != row.id || loaded.manifest.version() != row.version {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook {}@{} manifest identity differs from Store row",
                    row.id, row.version
                ),
            });
        }
        loaded.pending_publish = pending_publish;
        Ok(loaded)
    }

    fn load_row(
        &self,
        row: &WorkbookRow,
        references: &crate::pending::PendingReferenceIndex,
    ) -> Result<LoadedWorkbook> {
        crate::load::validate_workbook_row(&self.home, row)?;
        let location = self.publication_for_row(row, references)?;
        self.load_row_from_publish_location(row, &location)
    }

    fn load_row_from_publish_location(
        &self,
        row: &WorkbookRow,
        location: &crate::pending::PublishLocation,
    ) -> Result<LoadedWorkbook> {
        let (loaded, _) = crate::pending::read_publish_dir(&self.home, location, |directory| {
            self.load_row_from_directory(row, directory, location.pending_publish)
        })?;
        Ok(loaded)
    }

    /// Reparse/recompile an installed or arbitrary directory, including frozen copies.
    pub fn load_dir(&self, dir: &AbsPath) -> Result<LoadedWorkbook> {
        Self::load_tree(&ExternalReadTree::open_managed(&self.home, dir)?)
    }

    pub(crate) fn load_managed_dir(home: &Home, dir: &AbsPath) -> Result<LoadedWorkbook> {
        Self::load_tree(&ExternalReadTree::open_managed(home, dir)?)
    }

    fn load_tree(tree: &ExternalReadTree) -> Result<LoadedWorkbook> {
        tree.validate_sizes()?;
        let manifest_path = RelPath::new("workbook.toml")?;
        let manifest_bytes = tree.read_file(&manifest_path, FS_MAX_FILE_BYTES)?;
        let mut captured =
            std::collections::BTreeMap::from([(manifest_path, manifest_bytes.clone())]);
        let manifest = parse_manifest(&read_tree_utf8(&manifest_bytes, "workbook.toml")?)?;
        let mut definitions = Vec::new();
        let mut flow_ids = std::collections::BTreeSet::new();
        for path in manifest.flows() {
            let bytes = tree.read_file(path, FS_MAX_FILE_BYTES)?;
            let def = parse_flow(&read_tree_utf8(&bytes, path.as_str())?)?;
            if !flow_ids.insert(def.id().clone()) {
                return Err(Error::Core(sheltie_core::Error::FlowInvalid {
                    rule: "1",
                    path: format!("{}.id", path.as_str()),
                    reason: format!("Duplicate Workbook Flow ID {}", def.id()),
                }));
            }
            captured.insert(path.clone(), bytes);
            definitions.push(def);
        }
        let mut instruction_paths = std::collections::BTreeSet::new();
        for definition in &definitions {
            for node in definition.nodes() {
                if let sheltie_core::flow::Instruction::File(path) = node.instruction() {
                    instruction_paths.insert(path.clone());
                }
            }
        }
        for path in &instruction_paths {
            if !captured.contains_key(path) {
                captured.insert(path.clone(), tree.read_file(path, FS_MAX_FILE_BYTES)?);
            }
        }
        let (digest, res, hashes) = crate::workbook_digest::inspect_tree_v2(tree, &captured)?;
        let flows = definitions
            .into_iter()
            .map(|def| {
                let graph = compile(&def, &manifest, &res)?;
                Ok((def, graph))
            })
            .collect::<Result<Vec<_>>>()?;
        let resource_files = hashes
            .into_iter()
            .map(|(relative, sha256)| {
                let bytes = res
                    .files
                    .get(&relative)
                    .map(|meta| meta.bytes)
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("ResourceIndex lacks digested file {relative}"),
                    })?;
                Ok((
                    relative.clone(),
                    sheltie_core::work::ObservedFile::new(
                        tree.root().join(&relative),
                        sha256,
                        bytes,
                    ),
                ))
            })
            .collect::<Result<_>>()?;
        let instructions = instruction_paths
            .into_iter()
            .map(|path| {
                let bytes = captured.get(&path).ok_or_else(|| Error::StoreCorrupt {
                    detail: format!("Frozen copy lacks instruction file {path}"),
                })?;
                let text = read_tree_utf8(bytes, path.as_str())?;
                Ok((path, text))
            })
            .collect::<Result<_>>()?;
        Ok(LoadedWorkbook {
            manifest,
            flows,
            dir: tree.root().clone(),
            digest,
            resource_files,
            instructions,
            pending_publish: false,
        })
    }

    /// Load by id/version; omitted version selects the lexicographic maximum. Missing versions yield NotFound.
    /// Verify registration on load: recompute directory digest and compare with workbooks.digest; mismatch yields
    /// WORKBOOK_TAMPERED (GF-32, O03: start must not silently accept tampering detected by verify).
    pub fn load(&self, id: &str, version: Option<&str>) -> Result<LoadedWorkbook> {
        sheltie_core::ids::WorkbookId::new(id).map_err(Error::Core)?;
        if let Some(version) = version {
            if !crate::load::valid_workbook_version(version) {
                return Err(Error::InvalidRequest {
                    reason: format!("Workbook version {version:?} violates the contract"),
                });
            }
            ManagedRelPath::new(format!("workbooks/{id}/{version}")).map_err(|error| {
                Error::InvalidRequest {
                    reason: format!("Workbook identity cannot form a managed path: {error}"),
                }
            })?;
        }
        let rows = self.store.workbook_versions(id)?;
        let row = match version {
            Some(v) => {
                rows.into_iter()
                    .find(|r| r.version == v)
                    .ok_or_else(|| Error::NotFound {
                        what: format!("Workbook {id}@{v}"),
                    })?
            }
            // workbook_versions is lexicographically ascending; its last entry is the highest version.
            None => rows
                .into_iter()
                .next_back()
                .ok_or_else(|| Error::NotFound {
                    what: format!("Workbook {id}"),
                })?,
        };
        let references = crate::pending::PendingReferenceIndex::load(&self.store)?;
        self.load_row(&row, &references)
    }

    /// Directory digest workbook-digest/v2 (storage §5.1), the sole schema 2 digest definition,
    /// shared by add/load/verify, frozen-copy verification, and publication effects.
    pub fn digest_dir(dir: &AbsPath) -> Result<Sha256Hex> {
        crate::workbook_digest::digest_dir_v2(dir)
    }

    /// workbook remove <id>@<version>; T08 makes reference checks transactional.
    /// Use the same lock/request table; replay returns the original snapshot without repeated deletion.
    pub fn remove(
        &self,
        id: &str,
        version: &str,
        request_id: Option<String>,
    ) -> Result<RemovedSnapshot> {
        if version.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "remove requires an explicit version, without a highest-version default"
                    .to_string(),
            });
        }
        sheltie_core::ids::WorkbookId::new(id).map_err(Error::Core)?;
        if !crate::load::valid_workbook_version(version) {
            return Err(Error::InvalidRequest {
                reason: format!("Workbook version {version:?} violates the contract"),
            });
        }
        ManagedRelPath::new(format!("workbooks/{id}/{version}")).map_err(|error| {
            Error::InvalidRequest {
                reason: format!("Workbook identity cannot form a managed path: {error}"),
            }
        })?;
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::RemoveWorkbook {
            id: id.to_string(),
            version: version.to_string(),
        };
        let intent_hash = intent.hash();
        let ro = Store::open_for_home(&self.home, crate::store::OpenMode::ReadOnly).map_err(
            |error| match error {
                Error::NotFound { .. } => Error::NotFound {
                    what: format!("Workbook {id}@{version}"),
                },
                other => other,
            },
        )?;
        let replay_exists = if let Some(hash) = ro.lookup_request_hash(&request_id)? {
            if hash != intent_hash.as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            true
        } else {
            false
        };

        let session = crate::session::WriteSession::open_existing(&self.home)?;
        let lock = session.lock;
        let repo = Self::with_store(self.home.clone(), session.store.clone());
        repo.recover_before_write(&lock, &request_id, intent_hash.as_str())?;
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if row.intent_hash != intent_hash.as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return decode_removed_snapshot(&request_id, &row.reply_json);
        }
        if replay_exists {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook request {request_id} replay preflight differs from the locked record"
                ),
            });
        }

        // Verify ownership before cleanup (T05): current directory digest must still match registration.
        let dir = self.home.workbook_dir(id, version);
        let registered = repo
            .store
            .workbook_versions(id)?
            .into_iter()
            .find(|row| row.version == version);
        let Some(registered) = registered else {
            let dir_rel = self.home.to_rel(&dir)?;
            if crate::fsx::managed_directory_exists(&self.home, &lock, &dir_rel)? {
                return Err(Error::StoreCorrupt {
                    detail: format!(
                        "Workbook directory {dir} exists without a corresponding registration row; deletion rejected"
                    ),
                });
            }
            return Err(Error::NotFound {
                what: format!("Workbook {id}@{version}"),
            });
        };
        let registered_dir = crate::load::validate_workbook_row(&self.home, &registered)?;
        if registered_dir != dir {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook registered path {registered_dir} differs from requested path {dir}"
                ),
            });
        }
        let registered_digest = registered.digest.clone();
        sheltie_core::digest::Sha256Hex::new(registered_digest.clone()).map_err(|error| {
            Error::StoreCorrupt {
                detail: format!("Workbook {id}@{version} registered digest is invalid: {error}"),
            }
        })?;
        let dir_rel = self.home.to_rel(&dir)?;
        if !crate::fsx::managed_directory_exists(&self.home, &lock, &dir_rel)? {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Workbook {id}@{version} is registered but its final directory is missing; deletion rejected"
                ),
            });
        }
        let current = crate::workbook_digest::digest_managed_dir_v2(&self.home, &dir)?;
        if current.as_str() != registered_digest {
            return Err(Error::WorkbookTampered {
                results: vec![VerifyRow {
                    id: id.to_string(),
                    version: version.to_string(),
                    status: VerifyStatus::Tampered,
                    pending_publish: false,
                }],
            });
        }
        // Reference checks and row deletion share one transaction (storage §5.2); corrupt references stop within it.
        let ctx = Context {
            now: crate::observe::now(),
            principal: crate::observe::principal(),
        };
        let internal_id = uuid::Uuid::now_v7().simple().to_string();
        let payload = stage_pending(
            &self.home,
            &lock,
            &internal_id,
            &request_id,
            "remove_workbook",
        )?;
        let snapshot = RemovedSnapshot {
            request_id: request_id.clone(),
            replayed: false,
            data: serde_json::to_value(&Removed {
                id: id.to_string(),
                version: version.to_string(),
            })
            .unwrap_or(serde_json::Value::Null),
        };
        let input = crate::store::CommitInput {
            work_id: None,
            workbook_insert: None,
            workbook_in_use_check: Some((id.to_string(), version.to_string())),
            workbook_delete: Some((id.to_string(), version.to_string())),
            expected_revision: None,
            state: None,
            request_id: request_id.clone(),
            intent_hash: intent_hash.as_str().to_string(),
            reply_json: serde_json::to_string(&snapshot).unwrap_or_default(),
            effects_json: encode_effects(&[EffectOp::DeleteDir {
                pending: self.home.to_rel(&payload)?,
                final_path: self.home.to_rel(&dir)?,
                owner: format!("workbook:{id}@{version}"),
                digest: registered_digest,
            }]),
            principal: ctx.principal,
            command_json: serde_json::json!({
                "intent": "remove_workbook",
                "target": format!("{id}@{version}"),
            })
            .to_string(),
            at: ctx.now,
        };
        let replayed = matches!(repo.store.commit(input)?, CommitOutcome::Replayed { .. });
        if !replayed {
            crate::failpoint::maybe_exit("after_commit_before_effects");
        }
        repo.recover_finish_request(&lock, &request_id, false)?;
        if replayed {
            let row =
                repo.store
                    .inspect_request(&request_id)?
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("Replay request {request_id} disappeared from requests"),
                    })?;
            decode_removed_snapshot(&request_id, &row.reply_json)
        } else {
            Ok(snapshot)
        }
    }

    /// workbook verify; Some((id, version)) filters to one Workbook version.
    pub fn verify(&self, filter: Option<(&str, &str)>) -> Result<Vec<VerifyRow>> {
        let references = crate::pending::PendingReferenceIndex::load(&self.store)?;
        let rows = match filter {
            Some((id, version)) => {
                let row = self
                    .store
                    .workbook_versions(id)?
                    .into_iter()
                    .find(|r| r.version == version)
                    .ok_or_else(|| Error::NotFound {
                        what: format!("Workbook {id}@{version}"),
                    })?;
                vec![row]
            }
            None => self.store.list_workbooks()?,
        };
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            match self.load_row(&row, &references) {
                Ok(loaded) => out.push(VerifyRow {
                    id: row.id,
                    version: row.version,
                    status: VerifyStatus::Ok,
                    pending_publish: loaded.pending_publish,
                }),
                Err(Error::NotFound { .. }) => out.push(VerifyRow {
                    id: row.id,
                    version: row.version,
                    status: VerifyStatus::Missing,
                    pending_publish: false,
                }),
                Err(Error::WorkbookTampered { results }) => out.extend(results),
                Err(error) => return Err(error),
            };
        }
        Ok(out)
    }

    /// Clean pending metadata after writes; failures return maintenance diagnostics without rolling back committed business results.
    pub fn cleanup_pending(&self) -> Result<Vec<String>> {
        let session = crate::session::WriteSession::open_existing(&self.home)?;
        crate::pending::cleanup(&self.home, &session.store, &session.lock)
    }

    /// Confined copying rejects symlinks, hardlinks, special files, files above 32 MiB, and totals above 256 MiB.
    /// Copy through per-file handles, exclusively create destinations, and fsync (fsx).
    pub(crate) fn copy_confined(
        home: &Home,
        lock: &crate::home::HomeLock,
        src: &AbsPath,
        dst: &AbsPath,
    ) -> Result<u64> {
        crate::fsx::copy_tree_confined(home, lock, src, dst)
    }
}

impl crate::recovery::RecoveryAccess for WorkbookRepo {
    fn load_request(
        &self,
        request_id: &str,
        metadata: &crate::store::read::RequestMetadata,
    ) -> crate::recovery::RequestLoadResult<crate::recovery::CheckedRequest> {
        if metadata.work_id.is_some() {
            let service =
                crate::service::WorkService::with_store(self.home.clone(), self.store.clone());
            service
                .load_checked_request(request_id, metadata)
                .map(|(request, _)| request)
        } else {
            self.load_checked_request(request_id, metadata)
                .map(|(request, _)| request)
        }
    }

    fn refresh_cards(
        &self,
        effects: &crate::effects::CheckedEffects,
        lock: &crate::home::HomeLock,
    ) -> Result<()> {
        let service =
            crate::service::WorkService::with_store(self.home.clone(), self.store.clone());
        service.refresh_cards_of(effects, lock)
    }
}

/// Read a required UTF-8 file through verified handles with limits; report failures as WORKBOOK_INVALID.
fn read_tree_utf8(bytes: &[u8], path: &str) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        Error::Core(sheltie_core::Error::WorkbookInvalid {
            field: path.to_string(),
            reason: "Not UTF-8".to_string(),
        })
    })
}
