//! Work service: writes follow unlocked preflight, root lock, recovery, decision, one transaction, then
//! effect publication (architecture §4, storage contract §2). Responses use commit-time snapshots (cli-result/v2).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::Graph;
use sheltie_core::ids::{AttemptId, NodeId, WorkId, WorkName};
use sheltie_core::path::AbsPath;
use sheltie_core::work::{
    Command, Context, Decision, Effect, NextOp, ObservedFile, Reply, StatsJson, StatusCardJson,
    WorkState, WorkStatus, WorkbookRef, decide, legal_next, render_stats_json,
};

use crate::effects::{
    CheckedEffects, EffectOp, RefJson, check_work_effects, decode_effects, encode_effects,
};
use crate::error::{Error, Result};
use crate::fsx::ManagedRelPath;
use crate::home::Home;
use crate::observe::{now, principal};
use crate::request::{InputValue, RequestIntent};
use crate::store::{CommitInput, CommitOutcome, Store};
use crate::workbook_repo::WorkbookRepo;

/// work start arguments: values are literals or @file paths; read contents only after unlocked preflight's
/// replay lookup (storage contract §2.1).
#[derive(Debug, Clone)]
pub struct StartArgs {
    pub workbook_id: String,
    pub version: Option<String>,
    pub flow: String,
    pub name: Option<String>,
    pub inputs: BTreeMap<String, InputValue>,
}

/// Complete commit-time response (GF-15); data is the protocol payload, returned field-for-field on replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub request_id: String,
    pub revision: u64,
    #[serde(default)]
    pub replayed: bool,
    pub reply: Reply,
    #[serde(default)]
    pub data: serde_json::Value,
    pub next: Vec<NextOp>,
}

impl From<crate::snapshot::PersistedResponse> for Response {
    fn from(snapshot: crate::snapshot::PersistedResponse) -> Self {
        Self {
            request_id: snapshot.request_id,
            revision: snapshot.revision,
            replayed: snapshot.replayed,
            reply: snapshot.reply,
            data: snapshot.data,
            next: snapshot.next,
        }
    }
}

/// One work list row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkSummary {
    pub work_id: WorkId,
    pub name: String,
    pub status: WorkStatus,
    pub current: String,
    pub updated_at: String,
}

/// Service handle; holds no long-lived connection.
#[derive(Debug, Clone)]
pub struct WorkService {
    home: Home,
    store: Store,
}

/// Loaded Work: state, revision, and graph.
struct Loaded {
    state: WorkState,
    revision: u64,
    graph: Graph,
    resource_files: BTreeMap<sheltie_core::path::RelPath, ObservedFile>,
    instructions: BTreeMap<sheltie_core::path::RelPath, String>,
    pending_publish: bool,
}

struct PreparedCommand {
    command: Command,
    observed_outputs: BTreeMap<String, crate::fsx::SafeFile>,
}

struct ReadRequestPayload {
    command: Command,
    snapshot: Response,
    effects: Vec<EffectOp>,
    data: crate::snapshot::CheckedData,
}

impl PreparedCommand {
    fn plain(command: Command) -> Self {
        Self {
            command,
            observed_outputs: BTreeMap::new(),
        }
    }
}

/// Create pending sidecars/staging containers (storage contract §3.3). start/add also create payload; remove creates
/// only the container, moving verified final into payload after commit.
pub(crate) fn stage_pending(
    home: &Home,
    lock: &crate::home::HomeLock,
    internal_id: &str,
    request_id: &str,
    op: &str,
) -> Result<AbsPath> {
    let create_payload = match op {
        "start_work" | "add_workbook" => true,
        "remove_workbook" => false,
        _ => {
            return Err(Error::StoreCorrupt {
                detail: format!("Unknown pending operation {op}"),
            });
        }
    };
    let pending_root = ManagedRelPath::new("pending")?;
    let container = ManagedRelPath::new(format!("pending/{internal_id}"))?;
    let sidecar = ManagedRelPath::new(format!("pending/{internal_id}.owner"))?;
    let owner = crate::pending::PendingOwner {
        format: "pending/v1".to_string(),
        internal_id: internal_id.to_string(),
        request_id: request_id.to_string(),
        op: op.to_string(),
    };
    let mut content = serde_json::to_vec(&owner).map_err(|error| Error::StoreCorrupt {
        detail: format!("pending owner serialization failed: {error}"),
    })?;
    content.push(b'\n');
    let fs = crate::fsx::ManagedFs::open_existing(home)?;
    fs.ensure_dir(lock, &pending_root)?;
    fs.write_new(lock, &sidecar, &content)?;
    fs.sync_dir_locked(lock, &pending_root)?;
    fs.ensure_dir(lock, &container)?;
    crate::failpoint::maybe_exit("pending_owner_synced_before_payload");
    let payload = ManagedRelPath::new(format!("pending/{internal_id}/payload"))?;
    if create_payload {
        fs.ensure_dir(lock, &payload)?;
    }
    home.rel(payload.as_str())
}

pub(crate) fn verify_pending_owner(
    home: &Home,
    internal_id: &str,
    request_id: &str,
    op: &str,
) -> Result<()> {
    crate::pending::verify_owner(home, internal_id, request_id, op)
}

impl WorkService {
    pub fn new(home: Home) -> Self {
        let store = Store::deferred_for_home(&home, crate::store::OpenMode::ReadOnly);
        Self { home, store }
    }

    pub(crate) fn with_store(home: Home, store: Store) -> Self {
        Self { home, store }
    }

    /// work start (protocol §3): after side-effect-free preflight (GF-30, T02), enter the write path:
    /// locked staging, copy verification, sequence, decide, transaction, publication.
    pub fn start(&self, args: StartArgs, request_id: Option<String>) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        // Construct intent before resolving targets or reading source files; omitted states and original arguments stay fixed.
        let intent = RequestIntent::StartWork {
            workbook: args.workbook_id.clone(),
            version: args.version.clone(),
            flow: args.flow.clone(),
            name: args.name.clone(),
            inputs: args.inputs.clone(),
        };
        let intent_hash = intent.hash();

        // ── Unlocked preflight: identify existing Store read-only and look up replayable requests ──
        let mut inputs: Option<BTreeMap<String, String>> = None;
        let mut replay_hit = false;
        let preflight_store =
            match Store::open_for_home(&self.home, crate::store::OpenMode::ReadOnly) {
                Ok(ro) => {
                    if let Some(hash) = ro.lookup_request_hash(&request_id)? {
                        if hash != intent_hash.as_str() {
                            return Err(Error::RequestConflict {
                                request_id: request_id.clone(),
                            });
                        }
                        // Replay hit: do not read current Workbook/@file (O04); verify effects under the write lock.
                        replay_hit = true;
                    }
                    ro
                }
                Err(Error::NotFound { .. }) => {
                    return Err(Error::NotFound {
                        what: format!("Workbook {}", args.workbook_id),
                    });
                }
                Err(error) => return Err(error),
            };

        if !replay_hit {
            WorkName::normalize(args.name.as_deref().unwrap_or(&args.flow))?;
            // Continue preflight: load Workbook/Flow, verify registered digest, and validate start keys.
            let ro_store = WorkService::with_store(self.home.clone(), preflight_store);
            let wb = ro_store
                .repo()
                .load(&args.workbook_id, args.version.as_deref())?;
            let flow = wb.flow(&args.flow).ok_or_else(|| Error::NotFound {
                what: format!("Flow {}", args.flow),
            })?;
            sheltie_core::work::validate_start_inputs(&flow.1, args.inputs.keys())?;
            // Read @file after replay lookup (work start step 3); the write path reuses
            // the same observation without rereading.
            let values = materialize_inputs(&args.inputs)?;
            inputs = Some(values);
        }

        // ── Write path: lock, recovery, recheck, staging, decision, transaction, publication ──
        let session = crate::session::WriteSession::open_existing(&self.home)?;
        let svc = Self::with_store(self.home.clone(), session.store.clone());
        svc.recover_before_write(&session.lock, &request_id, intent_hash.as_str())?;
        // Recheck requests under the lock; a writer may have committed after preflight.
        if let Some(row) = svc.store.inspect_request(&request_id)? {
            if row.intent_hash != intent_hash.as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return svc.replay(request_id, row.reply_json, &session.lock);
        }
        let wb = svc
            .repo()
            .load(&args.workbook_id, args.version.as_deref())?;
        let flow = wb.flow(&args.flow).ok_or_else(|| Error::NotFound {
            what: format!("Flow {}", args.flow),
        })?;
        // Recheck input keys under the lock against definitions replaced after preflight.
        sheltie_core::work::validate_start_inputs(&flow.1, args.inputs.keys())?;
        let inputs = match inputs {
            Some(values) => values,
            None => materialize_inputs(&args.inputs)?,
        };

        let ctx = Context {
            now: now(),
            principal: principal(),
        };
        // Verify works/ ancestor symlinks before sequence allocation or materialization (O01); reject escapes
        // outside the root here, before effect publication.
        crate::fsx::ensure_dirs_under(&self.home, &session.lock, &self.home.works_dir())?;
        let internal_id = uuid::Uuid::now_v7().simple().to_string();
        let payload = stage_pending(
            &self.home,
            &session.lock,
            &internal_id,
            &request_id,
            "start_work",
        )?;

        let day = ctx.now.day().to_string();
        let seq = svc.store.allocate_seq(&day)?;
        let name = WorkName::normalize(args.name.as_deref().unwrap_or(&args.flow))?;
        let work_id = WorkId::new(&day, seq, &name)?;
        let work_dir = self.home.work_dir(&work_id);

        // payload/workbook freezes this Work's definitions; reparse/recompile the final copy
        // and verify identity/digest after copying (GF-17, storage contract §5.4).
        let frozen = payload.join_segment("workbook");
        WorkbookRepo::copy_confined(&self.home, &session.lock, &wb.dir, &frozen)?;
        let copied = svc.repo().load_dir(&frozen)?;
        if copied.manifest.id() != wb.manifest.id()
            || copied.manifest.version() != wb.manifest.version()
        {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Frozen-copy manifest identity {}@{} differs from registered {}@{}",
                    copied.manifest.id(),
                    copied.manifest.version(),
                    wb.manifest.id(),
                    wb.manifest.version(),
                ),
            });
        }
        if copied.digest != wb.digest {
            return Err(Error::StoreCorrupt {
                detail:
                    "Frozen-copy digest differs from registration (source changed during copying)"
                        .to_string(),
            });
        }
        let frozen_flow = copied
            .flow(flow.0.id().as_str())
            .ok_or_else(|| Error::NotFound {
                what: format!("Flow {}", args.flow),
            })?;
        crate::fsx::set_tree_readonly_confined(&self.home, &session.lock, &frozen)?;

        // payload/start-inputs/<key>: materialize in pending, recording final paths effective after publication.
        let inputs_dir = payload.join_segment("start-inputs");
        crate::fsx::ensure_dirs_under(&self.home, &session.lock, &inputs_dir)?;
        let mut input_refs = BTreeMap::new();
        for (key, value) in &inputs {
            let staged = Home::confine(&inputs_dir, key)?;
            crate::fsx::write_new_file(&self.home, &session.lock, &staged, value.as_bytes())?;
            let final_path = sheltie_core::work::start_input_path(&work_dir, key);
            input_refs.insert(
                key.clone(),
                sheltie_core::work::ArtifactRef {
                    sha256: Sha256Hex::of_bytes(value.as_bytes()),
                    bytes: value.len() as u64,
                    path: final_path,
                },
            );
        }

        let cmd = Command::Start {
            work_id: work_id.clone(),
            name,
            workbook: WorkbookRef {
                id: wb.manifest.id().clone(),
                version: wb.manifest.version().to_string(),
                digest: copied.digest.clone(),
            },
            flow: flow.0.id().clone(),
            work_dir,
            inputs: input_refs,
        };
        let decision = decide(None, &frozen_flow.1, &cmd, &ctx)?;
        let effects = vec![
            EffectOp::PublishDir {
                pending: self.home.to_rel(&payload)?,
                final_path: format!("works/{work_id}"),
                owner: format!("work:{work_id}"),
                digest: copied.digest.as_str().to_string(),
                digest_root: "workbook".to_string(),
            },
            EffectOp::RefreshStatusCard {
                work_id: work_id.as_str().to_string(),
            },
        ];
        let resp = svc.commit(
            decision,
            &frozen_flow.1,
            &request_id,
            intent_hash.as_str().to_string(),
            None,
            effects,
            &ctx,
            &cmd,
            &session.lock,
        )?;
        // After COMMIT, the unified entry executes the whole effect closure and records completion.
        svc.recover_finish_request(&session.lock, &request_id, None, false)?;
        Ok(resp)
    }

    pub fn begin(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::BeginAttempt {
            work: work.clone(),
            node: node.clone(),
        };
        self.run_command(work, &intent, request_id, &|loaded, _| {
            let paths = sheltie_core::work::input_paths_for(&loaded.state, &loaded.graph, node)?;
            let mut observed = BTreeMap::new();
            for (name, path) in paths {
                let obs = match path {
                    Some(p) => observe_loaded_input(&self.home, loaded, node, &name, &p)?,
                    None => None,
                };
                observed.insert(name, obs);
            }
            Ok(PreparedCommand::plain(Command::BeginAttempt {
                node: node.clone(),
                observed_inputs: observed,
                instruction_text: instruction_text_of(loaded, node)?,
            }))
        })
    }

    pub fn submit(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        summary: &InputValue,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::SubmitAttempt {
            work: work.clone(),
            attempt: attempt.clone(),
            summary: summary.clone(),
        };
        self.run_command(work, &intent, request_id, &|loaded, _| {
            let summary_text = materialize_summary(summary)?;
            let paths =
                sheltie_core::work::output_paths_for(&loaded.state, &loaded.graph, attempt)?;
            let mut observed = BTreeMap::new();
            let mut safe_files = BTreeMap::new();
            for (name, path) in paths {
                let (obs, safe_file) = observe_output(&self.home, &name, &path)?;
                if let Some(file) = safe_file {
                    safe_files.insert(path.as_str().to_string(), file);
                }
                observed.insert(name, obs);
            }
            Ok(PreparedCommand {
                command: Command::SubmitAttempt {
                    attempt: attempt.clone(),
                    summary: summary_text,
                    observed_outputs: observed,
                },
                observed_outputs: safe_files,
            })
        })
    }

    pub fn fail(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        reason: &InputValue,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::FailAttempt {
            work: work.clone(),
            attempt: attempt.clone(),
            reason: reason.clone(),
        };
        self.run_command(work, &intent, request_id, &|_, _| {
            let reason_text = materialize_summary(reason)?;
            Ok(PreparedCommand::plain(Command::FailAttempt {
                attempt: attempt.clone(),
                reason: reason_text,
            }))
        })
    }

    pub fn replace(
        &self,
        work: &WorkId,
        attempt: &AttemptId,
        reason: &InputValue,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::ReplaceAttempt {
            work: work.clone(),
            attempt: attempt.clone(),
            reason: reason.clone(),
        };
        self.run_command(work, &intent, request_id, &|loaded, lock| {
            let paths = sheltie_core::work::replacement_input_paths_for(
                &loaded.state,
                &loaded.graph,
                attempt,
            )?;
            let reason = materialize_summary(reason)?;
            sheltie_core::text::Summary::new(&reason, "reason").map_err(|_| {
                Error::Core(sheltie_core::Error::SummaryTooLong {
                    max: sheltie_core::text::Summary::max_bytes(),
                    actual: reason.len(),
                })
            })?;
            let mut observed_inputs = BTreeMap::new();
            for (name, path) in paths {
                let observed = match path {
                    Some(path) => {
                        let file = crate::fsx::open_managed_optional(&self.home, &path)?;
                        file.map(|file| {
                            crate::failpoint::rendezvous("replace_after_input_open", path.as_str())
                                .map_err(|error| Error::io(path.as_str(), error))?;
                            let (sha256, bytes) =
                                file.sha256_bounded(crate::fsx::MAX_FILE_BYTES)?;
                            crate::fsx::verify_managed_file_bound(&self.home, lock, &path, &file)?;
                            Ok::<_, Error>(ObservedFile::new(path, sha256, bytes))
                        })
                        .transpose()?
                    }
                    None => None,
                };
                observed_inputs.insert(name, observed);
            }
            Ok(PreparedCommand::plain(Command::ReplaceAttempt {
                attempt: attempt.clone(),
                reason,
                observed_inputs,
                instruction_text: instruction_text_of(loaded, &attempt.node)?,
            }))
        })
    }

    pub fn approve(
        &self,
        work: &WorkId,
        node: &NodeId,
        request_id: Option<String>,
    ) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::ApproveGate {
            work: work.clone(),
            node: node.clone(),
        };
        self.run_command(work, &intent, request_id, &|_, _| {
            Ok(PreparedCommand::plain(Command::ApproveGate {
                node: node.clone(),
            }))
        })
    }

    pub fn cancel(&self, work: &WorkId, request_id: Option<String>) -> Result<Response> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::CancelWork { work: work.clone() };
        self.run_command(work, &intent, request_id, &|_, _| {
            Ok(PreparedCommand::plain(Command::Cancel))
        })
    }

    /// Read-only: text and structured status cards.
    pub fn status(&self, work: &WorkId) -> Result<(String, StatusCardJson)> {
        let (text, view) = self.status_projection(work)?;
        Ok((text, view.card))
    }

    /// Return the status card and publication facts from this load.
    pub fn status_with_publication(&self, work: &WorkId) -> Result<(String, StatusCardJson, bool)> {
        let (text, view) = self.status_projection(work)?;
        Ok((text, view.card, view.pending_publish))
    }

    pub fn status_read(&self, work: &WorkId) -> Result<(String, crate::StatusReadView)> {
        let (text, view) = self.status_projection(work)?;
        Ok((view.render(text), view))
    }

    fn status_projection(&self, work: &WorkId) -> Result<(String, crate::StatusReadView)> {
        let (loaded, effects_pending) = self.load_read_context(work)?;
        let card = sheltie_core::work::render::status_view(&loaded.state, &loaded.graph);
        let text = card.render();
        let view = crate::StatusReadView {
            card: card.into_json(),
            revision: loaded.revision,
            effects_pending,
            pending_publish: loaded.pending_publish,
        };
        Ok((text, view))
    }

    pub fn result(
        &self,
        work: &WorkId,
    ) -> Result<(sheltie_core::work::result::ResultView, Vec<NextOp>)> {
        let (loaded, effects_pending) = self.load_read_context(work)?;
        let view = sheltie_core::work::result::result_view(
            &loaded.state,
            &loaded.graph,
            loaded.revision,
            effects_pending,
        )
        .map_err(|detail| Error::StoreCorrupt {
            detail: format!("Work {work} result projection is invalid: {detail}"),
        })?;
        Ok((view, legal_next(&loaded.state, &loaded.graph)))
    }

    pub fn write_result_artifact(
        &self,
        work: &WorkId,
        key: &str,
        revision: u64,
        writer: &mut impl std::io::Write,
    ) -> Result<()> {
        let (view, _) = self.result(work)?;
        if view.revision != revision {
            return Err(Error::RevisionConflict {
                expected: revision,
                actual: view.revision,
            });
        }
        if !view.r#final || view.effects_pending {
            return Err(Error::InvalidRequest {
                reason: "Final results for this Work are not readable yet".into(),
            });
        }
        let artifact = view
            .artifacts
            .iter()
            .find(|artifact| artifact.key == key)
            .ok_or_else(|| Error::NotFound {
                what: format!("Final result slot {key:?}"),
            })?;
        crate::result::write_artifact(
            &self.home,
            &sheltie_core::work::ArtifactRef {
                path: artifact.path.clone(),
                sha256: artifact.sha256.clone(),
                bytes: artifact.bytes,
            },
            writer,
        )
    }

    /// Read-only: facts and next share one loaded state/revision.
    pub fn stats(&self, work: &WorkId) -> Result<(String, StatsJson, Vec<serde_json::Value>)> {
        let loaded = self.load(work)?;
        crate::failpoint::rendezvous("stats_after_load", work.as_str())
            .map_err(|error| Error::io(loaded.state.work_dir.as_str(), error))?;
        let stats = render_stats_json(&loaded.state, &loaded.graph);
        Ok((
            stats.render(),
            stats,
            legal_next(&loaded.state, &loaded.graph)
                .iter()
                .map(|operation| sheltie_core::work::render::next_item_json(work, operation))
                .collect(),
        ))
    }

    /// Read-only: summaries of all Works.
    pub fn list(&self) -> Result<Vec<WorkSummary>> {
        self.store
            .list_works()?
            .into_iter()
            .map(|row| {
                let work = row.state.work_id.clone();
                let loaded = self.load_row(&work, row)?;
                Ok(WorkSummary {
                    work_id: loaded.state.work_id.clone(),
                    name: loaded.state.name.to_string(),
                    status: loaded.state.status,
                    current: loaded.state.current.to_string(),
                    updated_at: loaded.state.updated_at.as_str().to_string(),
                })
            })
            .collect()
    }

    /// Read-only: resolve a full ID or unique prefix; multiple matches yield InvalidRequest with candidates.
    pub fn resolve_work(&self, prefix: &str) -> Result<WorkId> {
        let matches = self.store.find_works_by_prefix(prefix)?;
        match matches.len() {
            0 => Err(Error::NotFound {
                what: format!("Work {prefix}"),
            }),
            1 => Ok(matches
                .into_iter()
                .next()
                .unwrap_or_else(|| unreachable!("A single-match vector must contain one element"))),
            _ => Err(Error::InvalidRequest {
                reason: format!(
                    "Prefix {prefix} matches multiple Works: {}",
                    matches
                        .iter()
                        .map(|w| w.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }),
        }
    }

    /// Existing requests use requests.work_id as target authority; conflict only if the prefix does not match the historical Work.
    pub fn resolve_work_for_request(
        &self,
        prefix: &str,
        request_id: Option<&str>,
    ) -> Result<WorkId> {
        if let Some(request_id) = request_id {
            if let Some(stored_work) = self.store.lookup_request_work(request_id)? {
                let Some(stored_work) = stored_work else {
                    return Err(Error::RequestConflict {
                        request_id: request_id.to_string(),
                    });
                };
                let work = WorkId::parse(&stored_work).map_err(|error| Error::StoreCorrupt {
                    detail: format!("Invalid requests.work_id {stored_work:?}: {error}"),
                })?;
                if !work.as_str().starts_with(prefix) {
                    return Err(Error::RequestConflict {
                        request_id: request_id.to_string(),
                    });
                }
                return Ok(work);
            }
        }
        self.resolve_work(prefix)
    }

    fn repo(&self) -> WorkbookRepo {
        WorkbookRepo::with_store(self.home.clone(), self.store.clone())
    }

    /// Load state/graph from the frozen copy; missing/mismatched copies yield StoreCorrupt. Unpublished Works
    /// read Store-protected pending originals (storage contract §3.3).
    fn load(&self, work: &WorkId) -> Result<Loaded> {
        let row = self.store.load_work(work)?;
        self.load_row(work, row)
    }

    fn load_read_context(&self, work: &WorkId) -> Result<(Loaded, bool)> {
        for retry in 0..2 {
            let bundle = self.store.read_work_bundle(work)?;
            crate::load::validate_work_root(&self.home, &bundle.work.state)?;
            let decoded = bundle
                .requests
                .iter()
                .map(|request| decode_read_request(request, work))
                .collect::<Result<Vec<_>>>()?;
            let start = bundle.requests.first().ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Work {work} has no Start request"),
            })?;
            let ReadRequestPayload {
                command,
                snapshot,
                effects: ops,
                data,
            } = decoded.first().ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Work {work} has no decoded Start request"),
            })?;
            if !crate::snapshot::start_matches(&bundle.work.state, command, &snapshot.reply, data)
                || start.row.at != bundle.work.state.created_at.as_str()
            {
                return Err(Error::StoreCorrupt {
                    detail: format!("Work {work} Start snapshot differs from frozen identity"),
                });
            }
            let [
                EffectOp::PublishDir {
                    pending,
                    final_path,
                    owner,
                    digest,
                    digest_root,
                },
                EffectOp::RefreshStatusCard { work_id },
            ] = ops.as_slice()
            else {
                return Err(Error::StoreCorrupt {
                    detail: format!("Work {work} Start publication effects are incomplete"),
                });
            };
            let segments = pending.split('/').collect::<Vec<_>>();
            if segments.len() != 3
                || segments[0] != "pending"
                || segments[2] != "payload"
                || uuid::Uuid::parse_str(segments[1]).is_err()
                || final_path != &format!("works/{work}")
                || owner != &format!("work:{work}")
                || digest != bundle.work.state.workbook.digest.as_str()
                || digest_root != "workbook"
                || work_id != work.as_str()
            {
                return Err(Error::StoreCorrupt {
                    detail: format!("Work {work} Start publication ownership does not match"),
                });
            }
            if !start.row.published {
                if let Err(error) =
                    verify_pending_owner(&self.home, segments[1], &start.request_id, "start_work")
                {
                    // Publication can finish and clean its sidecar after the snapshot.
                    // Restart the complete read rather than mixing a new flag into old state.
                    if retry == 0 {
                        continue;
                    }
                    return Err(error);
                }
            }
            let location = crate::pending::PublishLocation {
                pending: pending.clone(),
                final_path: final_path.clone(),
                pending_publish: !start.row.published,
            };
            let effects_pending = bundle.requests.iter().any(|request| !request.row.published);
            let loaded = self.load_row_at(work, bundle.work, &location)?;
            for (
                request,
                ReadRequestPayload {
                    command,
                    snapshot,
                    effects: ops,
                    data,
                },
            ) in bundle.requests.iter().zip(decoded)
            {
                validate_command_owner_data(
                    &request.request_id,
                    work,
                    &loaded.state,
                    &loaded.graph,
                    &command,
                    &snapshot,
                    &data,
                )?;
                validate_audit_execution(
                    &request.request_id,
                    &request.audit,
                    &loaded.state,
                    &snapshot.reply,
                )?;
                check_work_effects(
                    &self.home,
                    &request.request_id,
                    &loaded.state,
                    &loaded.graph,
                    &command,
                    &snapshot.reply,
                    ops,
                )?;
            }
            return Ok((loaded, effects_pending));
        }
        Err(Error::StoreCorrupt {
            detail: format!(
                "Work {work} Start ownership cannot be verified from the complete read snapshot"
            ),
        })
    }

    fn load_row(&self, work: &WorkId, row: crate::store::read::WorkRow) -> Result<Loaded> {
        let location = self.workbook_publication(work, &row.state)?;
        self.load_row_at(work, row, &location)
    }

    fn load_row_at(
        &self,
        work: &WorkId,
        row: crate::store::read::WorkRow,
        location: &crate::pending::PublishLocation,
    ) -> Result<Loaded> {
        let (workbook, pending_publish) =
            crate::pending::read_publish_dir(&self.home, location, |root| {
                let frozen = root.join_segment("workbook");
                let relative = self.home.to_rel(&frozen)?;
                if !crate::fsx::managed_directory_exists_readonly(&self.home, &relative)? {
                    if !location.pending_publish {
                        return Err(Error::StoreCorrupt {
                            detail: format!(
                                "Work {work} published frozen copy {} is missing",
                                frozen
                            ),
                        });
                    }
                    return Err(Error::NotFound {
                        what: format!("Work {work} frozen copy {}", frozen),
                    });
                }
                crate::load::compile_frozen_workbook(&self.home, &frozen, &row.state).map_err(
                    |error| match error {
                        Error::NotFound { .. } if !location.pending_publish => {
                            Error::StoreCorrupt {
                                detail: format!(
                                    "Work {work} published frozen-copy files are missing"
                                ),
                            }
                        }
                        other => other,
                    },
                )
            })
            .map_err(|error| match error {
                Error::NotFound { .. } => Error::StoreCorrupt {
                    detail: format!("Work {work} frozen copy lacks a declared file"),
                },
                other => other,
            })?;
        if workbook.digest != row.state.workbook.digest {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Frozen copy {} digest {} differs from its record",
                    workbook.dir,
                    workbook.digest.as_str()
                ),
            });
        }
        let graph = workbook
            .flow(row.state.flow.as_str())
            .map(|(_, graph)| graph.clone())
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Frozen copy has no Flow {}", row.state.flow),
            })?;
        crate::load::validate_work_paths(&self.home, &row.state, &graph, &workbook.resource_files)?;
        Ok(Loaded {
            state: row.state,
            revision: row.revision,
            graph,
            resource_files: workbook.resource_files,
            instructions: workbook.instructions,
            pending_publish,
        })
    }

    /// Locate this Work's unique frozen copy from its Start request and effect closure.
    fn workbook_publication(
        &self,
        work: &WorkId,
        state: &WorkState,
    ) -> Result<crate::pending::PublishLocation> {
        let references = crate::pending::PendingReferenceIndex::load_work_start(&self.store, work)?;
        crate::failpoint::rendezvous("pending_after_reference_index", &format!("works/{work}"))
            .map_err(|error| Error::io(state.work_dir.as_str(), error))?;
        let work_references = references.publish_references_for_work(work.as_str());
        let [reference] = work_references.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Work {work} requires one Start publish reference; actual {}",
                    work_references.len()
                ),
            });
        };
        let request_id = reference.request_id.as_str();
        let mut request =
            self.store
                .inspect_request(request_id)?
                .ok_or_else(|| Error::StoreCorrupt {
                    detail: format!("Start {request_id} has no requests row"),
                })?;
        if reference.published && !request.published {
            return Err(Error::StoreCorrupt {
                detail: format!("Start {request_id} published flag regressed after index reading"),
            });
        }
        let ops = decode_effects(&request.effects_json)?;
        let publish = ops.iter().find_map(|op| match op {
            EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                digest_root,
            } if owner == &format!("work:{work}") => {
                Some((pending, final_path, owner, digest, digest_root))
            }
            _ => None,
        });
        let Some((pending, final_path, owner, digest, digest_root)) = publish else {
            return Err(Error::StoreCorrupt {
                detail: format!("Work {work} reference index differs from Start effects"),
            });
        };
        if ops.len() != 2
            || ops
                .iter()
                .filter(|op| matches!(op, EffectOp::PublishDir { .. }))
                .count()
                != 1
            || ops
                .iter()
                .filter(|op| matches!(op, EffectOp::RefreshStatusCard { work_id } if work_id == work.as_str()))
                .count()
                != 1
            || final_path != &format!("works/{work}")
            || owner != &format!("work:{work}")
            || digest != state.workbook.digest.as_str()
            || digest_root != "workbook"
        {
            return Err(Error::StoreCorrupt {
                detail: format!("Work {work} unpublished Start effect ownership does not match"),
            });
        }
        let segments = pending.split('/').collect::<Vec<_>>();
        if segments.len() != 3
            || segments[0] != "pending"
            || segments[2] != "payload"
            || uuid::Uuid::parse_str(segments[1]).is_err()
        {
            return Err(Error::StoreCorrupt {
                detail: format!("Work {work} pending path is invalid"),
            });
        }
        if request.work_id.as_deref() != Some(work.as_str()) {
            return Err(Error::StoreCorrupt {
                detail: format!("Unpublished Start {request_id} Work ownership does not match"),
            });
        }
        if request.at != state.created_at.as_str() {
            return Err(Error::StoreCorrupt {
                detail: format!("Start {request_id} timestamp differs from WorkState.created_at"),
            });
        }
        Sha256Hex::new(request.intent_hash.clone()).map_err(|error| Error::StoreCorrupt {
            detail: format!("Unpublished Start {request_id} intent_hash is invalid: {error}"),
        })?;
        let audits = self.store.audit_rows(request_id)?;
        let [audit] = audits.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!("Unpublished Start {request_id} requires exactly one audit row"),
            });
        };
        let response: crate::snapshot::PersistedResponse =
            serde_json::from_str(&request.reply_json).map_err(|error| Error::StoreCorrupt {
                detail: format!("Cannot decode unpublished Start {request_id} snapshot: {error}"),
            })?;
        let command: Command =
            serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                detail: format!(
                    "Cannot decode unpublished Start {request_id} audit command: {error}"
                ),
            })?;
        let data = crate::snapshot::check_data(&response.reply, &response.data, work)?;
        let valid_start = crate::snapshot::start_matches(state, &command, &response.reply, &data)
            && response.request_id == request_id
            && response.revision == 1
            && !response.replayed
            && audit.work_id == work.as_str()
            && audit.revision == 1
            && audit.at == request.at;
        if !valid_start {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Unpublished Start {request_id} Command, snapshot, and Work do not match"
                ),
            });
        }
        if !request.published {
            match verify_pending_owner(&self.home, segments[1], request_id, "start_work") {
                Ok(()) => {}
                Err(owner_error) => {
                    let current = self.store.inspect_request(request_id)?;
                    match current {
                        Some(current)
                            if current.published
                                && current.intent_hash == request.intent_hash
                                && current.reply_json == request.reply_json
                                && current.effects_json == request.effects_json
                                && current.work_id == request.work_id
                                && current.at == request.at =>
                        {
                            request = current;
                        }
                        _ => return Err(owner_error),
                    }
                }
            }
        }
        Ok(crate::pending::PublishLocation {
            pending: pending.clone(),
            final_path: final_path.clone(),
            pending_publish: !request.published,
        })
    }

    /// Shared Work write path: unlocked replay preflight, lock, recovery, locked recheck, load, observe,
    /// decide, transaction, publication. Replay lookup precedes load/observe (protocol §2).
    fn run_command(
        &self,
        work: &WorkId,
        intent: &RequestIntent,
        request_id: String,
        build: &dyn Fn(&Loaded, &crate::home::HomeLock) -> Result<PreparedCommand>,
    ) -> Result<Response> {
        let intent_hash = intent.hash();
        // ── Unlocked read-only replay preflight: historical requests bind complete WorkId (§2.1), without resolving
        // prefixes again; the write path recovers incomplete effects.
        {
            let ro = Store::open_for_home(&self.home, crate::store::OpenMode::ReadOnly)?;
            if let Some(hash) = ro.lookup_request_hash(&request_id)? {
                if hash != intent_hash.as_str() {
                    return Err(Error::RequestConflict {
                        request_id: request_id.clone(),
                    });
                }
                // Verify effects, including missing write_file restoration, under the lock; do not return early here.
            }
        }

        // ── Write path: lock, recovery, locked recheck, load, observe, decide, transaction, publication ──
        let session = crate::session::WriteSession::open_existing(&self.home)?;
        let svc = Self::with_store(self.home.clone(), session.store.clone());
        svc.recover_before_write(&session.lock, &request_id, intent_hash.as_str())?;
        if let Some(row) = svc.store.inspect_request(&request_id)? {
            if row.intent_hash != intent_hash.as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return svc.replay(request_id, row.reply_json, &session.lock);
        }

        let ctx = Context {
            now: now(),
            principal: principal(),
        };
        let loaded = svc.load(work)?;
        let prepared = build(&loaded, &session.lock)?;
        let cmd = &prepared.command;
        let decision = decide(Some(&loaded.state), &loaded.graph, cmd, &ctx)?;
        let effects = core_effects_to_ops(&svc.home, &decision)?;
        let resp = svc.commit(
            decision,
            &loaded.graph,
            &request_id,
            intent_hash.as_str().to_string(),
            Some(loaded.revision),
            effects,
            &ctx,
            cmd,
            &session.lock,
        )?;
        if matches!(cmd, Command::SubmitAttempt { .. }) {
            crate::failpoint::rendezvous("submit_after_commit_before_seal", &request_id)
                .map_err(|error| Error::io("submit sync point", error))?;
        }
        svc.finish_request_with_observed(&request_id, &session.lock, &prepared.observed_outputs)?;
        Ok(resp)
    }

    /// Recover incomplete effects under the lock before new commands, in commit order; failure blocks new
    /// requests with EFFECT_PENDING (committed=false, referencing the old request).
    /// Regenerate affected status cards from latest state, without historical regression (§6).
    pub(crate) fn refresh_cards_of(
        &self,
        ops: &CheckedEffects,
        lock: &crate::home::HomeLock,
    ) -> Result<()> {
        crate::load::refresh_status_cards(&self.home, ops, lock, |id| {
            self.load(id).map(|loaded| (loaded.state, loaded.graph))
        })
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
        observed: Option<&BTreeMap<String, crate::fsx::SafeFile>>,
        verify_published: bool,
    ) -> Result<()> {
        crate::recovery::finish_request(
            &self.home,
            &self.store,
            lock,
            request_id,
            observed,
            verify_published,
            self,
        )
    }

    fn finish_request_with_observed(
        &self,
        request_id: &str,
        lock: &crate::home::HomeLock,
        observed: &BTreeMap<String, crate::fsx::SafeFile>,
    ) -> Result<()> {
        self.recover_finish_request(lock, request_id, Some(observed), false)
    }

    pub(crate) fn load_checked_request(
        &self,
        request_id: &str,
        metadata: &crate::store::read::RequestMetadata,
    ) -> crate::recovery::RequestLoadResult<(crate::recovery::CheckedRequest, Response)> {
        let row = metadata;
        Sha256Hex::new(row.intent_hash.clone()).map_err(|error| Error::StoreCorrupt {
            detail: format!("Request {request_id} intent_hash is invalid: {error}"),
        })?;
        let audits = self.store.audit_rows(request_id)?;
        let [audit] = audits.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Request {request_id} requires exactly one audit row; actual {}",
                    audits.len()
                ),
            }
            .into());
        };
        let work = row
            .work_id
            .as_deref()
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Work request {request_id} has no requests.work_id"),
            })
            .and_then(|value| {
                WorkId::parse(value).map_err(|error| Error::StoreCorrupt {
                    detail: format!("Request {request_id} work_id is invalid: {error}"),
                })
            })?;
        if audit.work_id != work.as_str() || row.at != audit.at {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Request {request_id} audit ownership/time differs from requests row"
                ),
            }
            .into());
        }
        let audit_revision = u64::try_from(audit.revision)
            .ok()
            .filter(|revision| *revision > 0)
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Request {request_id} audit.revision is invalid"),
            })?;
        let persisted: crate::snapshot::PersistedResponse =
            serde_json::from_str(&row.reply_json).map_err(|error| Error::StoreCorrupt {
                detail: format!("Cannot decode request {request_id} reply_json: {error}"),
            })?;
        if persisted.request_id != request_id
            || persisted.revision != audit_revision
            || persisted.replayed
        {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Request {request_id} snapshot identity/revision differs from audit"
                ),
            }
            .into());
        }
        let snapshot = Response::from(persisted);
        let command: Command =
            serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                detail: format!("Cannot decode request {request_id} audit.command_json: {error}"),
            })?;
        let data = crate::snapshot::check_data(&snapshot.reply, &snapshot.data, &work)?;
        validate_audit_snapshot(request_id, audit, &data)?;
        let state = self.load(&work)?;
        if state.revision < audit_revision {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "Request {request_id} audit revision exceeds current Work revision"
                ),
            }
            .into());
        }
        validate_command_owner_data(
            request_id,
            &work,
            &state.state,
            &state.graph,
            &command,
            &snapshot,
            &data,
        )?;
        validate_audit_execution(request_id, audit, &state.state, &snapshot.reply)?;
        let original = crate::recovery::work_original_response(&work, &snapshot);
        let effects_result: Result<_> = (|| {
            let row =
                self.store
                    .inspect_request(request_id)?
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("Missing effect record for request {request_id}"),
                    })?;
            metadata.check_row(request_id, &row)?;
            let ops = decode_effects(&row.effects_json)?;
            let checked = check_work_effects(
                &self.home,
                request_id,
                &state.state,
                &state.graph,
                &command,
                &snapshot.reply,
                ops,
            )?;
            for effect in checked.as_slice() {
                if !row.published {
                    if let EffectOp::PublishDir { pending, .. } = effect {
                        let internal_id =
                            pending
                                .split('/')
                                .nth(1)
                                .ok_or_else(|| Error::StoreCorrupt {
                                    detail: format!(
                                        "Request {request_id} pending owner path is invalid"
                                    ),
                                })?;
                        verify_pending_owner(&self.home, internal_id, request_id, "start_work")?;
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
            snapshot,
        ))
    }

    /// decide, one transaction, and snapshot; assemble data before commit and return unchanged on replay (GF-15).
    #[allow(clippy::too_many_arguments)]
    fn commit(
        &self,
        decision: Decision,
        graph: &Graph,
        request_id: &str,
        intent_hash: String,
        expected_revision: Option<u64>,
        effects: Vec<EffectOp>,
        ctx: &Context,
        cmd: &Command,
        lock: &crate::home::HomeLock,
    ) -> Result<Response> {
        let revision = expected_revision.map_or(1, |r| r + 1);
        let next = legal_next(&decision.state, graph);
        let data = snapshot_data(&decision);
        let snapshot = Response {
            request_id: request_id.to_string(),
            revision,
            replayed: false,
            reply: decision.reply.clone(),
            data: data.clone(),
            next: next.clone(),
        };
        let reply_json = serde_json::to_string(&snapshot).map_err(|e| Error::StoreCorrupt {
            detail: format!("Response serialization failed: {e}"),
        })?;
        let input = CommitInput {
            work_id: Some(decision.state.work_id.clone()),
            workbook_insert: None,
            workbook_delete: None,
            workbook_in_use_check: None,
            expected_revision,
            state: Some(decision.state.clone()),
            request_id: request_id.to_string(),
            intent_hash,
            reply_json,
            effects_json: encode_effects(&effects),
            principal: ctx.principal.clone(),
            command_json: audit_json(cmd)?,
            at: ctx.now.clone(),
        };
        match self.store.commit(input)? {
            CommitOutcome::Committed { revision: rev } => {
                // Crash window: after COMMIT, before effect publication (storage contract §3, fourth row).
                crate::failpoint::maybe_exit("after_commit_before_effects");
                Ok(Response {
                    request_id: request_id.to_string(),
                    revision: rev,
                    replayed: false,
                    reply: decision.reply,
                    data,
                    next,
                })
            }
            CommitOutcome::Replayed { reply_json, .. } => {
                self.replay(request_id.to_string(), reply_json, lock)
            }
        }
    }

    /// Replay returns the original snapshot (replayed = true), first finishing unpublished effects. Historical next is historical
    /// fact; always inspect current state to continue.
    fn replay(
        &self,
        request_id: String,
        reply_json: String,
        lock: &crate::home::HomeLock,
    ) -> Result<Response> {
        let metadata = self
            .store
            .inspect_request_metadata(&request_id)
            .map_err(|cause| crate::recovery::own_pending(&request_id, None, cause))?
            .ok_or_else(|| {
                crate::recovery::own_pending(
                    &request_id,
                    None,
                    Error::StoreCorrupt {
                        detail: format!("Replay request {request_id} disappeared from requests"),
                    },
                )
            })?;
        let (checked, mut resp) = self
            .load_checked_request(&request_id, &metadata)
            .map_err(|failure| failure.into_pending(&request_id))?;
        let row = &checked.row;
        if row.reply_json != reply_json {
            let cause = Error::StoreCorrupt {
                detail: format!(
                    "Request {request_id} historical response differs from requests record"
                ),
            };
            return Err(crate::recovery::own_pending(
                &request_id,
                Some(checked.original.clone()),
                cause,
            ));
        }
        crate::recovery::finish_checked_request(
            &self.home,
            &self.store,
            lock,
            crate::recovery::CheckedRequestEffects {
                request_id: &request_id,
                request: &checked,
            },
            None,
            self,
        )?;
        resp.request_id = request_id.clone();
        resp.replayed = true;
        Ok(resp)
    }
}

impl crate::recovery::RecoveryAccess for WorkService {
    fn load_request(
        &self,
        request_id: &str,
        metadata: &crate::store::read::RequestMetadata,
    ) -> crate::recovery::RequestLoadResult<crate::recovery::CheckedRequest> {
        if metadata.work_id.is_some() {
            self.load_checked_request(request_id, metadata)
                .map(|(request, _)| request)
        } else {
            self.repo()
                .load_checked_request(request_id, metadata)
                .map(|(request, _)| request)
        }
    }

    fn refresh_cards(&self, effects: &CheckedEffects, lock: &crate::home::HomeLock) -> Result<()> {
        self.refresh_cards_of(effects, lock)
    }
}

fn decode_read_request(
    request: &crate::store::read::WorkReadRequest,
    work: &WorkId,
) -> Result<ReadRequestPayload> {
    let id = &request.request_id;
    Sha256Hex::new(request.row.intent_hash.clone()).map_err(|error| Error::StoreCorrupt {
        detail: format!("Request {id} intent_hash is invalid: {error}"),
    })?;
    sheltie_core::work::Timestamp::parse(&request.row.at).map_err(|error| Error::StoreCorrupt {
        detail: format!("Request {id} timestamp is invalid: {error}"),
    })?;
    let persisted: crate::snapshot::PersistedResponse =
        serde_json::from_str(&request.row.reply_json).map_err(|error| Error::StoreCorrupt {
            detail: format!("Cannot decode request {id} response snapshot: {error}"),
        })?;
    if persisted.request_id != *id
        || persisted.replayed
        || i64::try_from(persisted.revision).ok() != Some(request.audit.revision)
    {
        return Err(Error::StoreCorrupt {
            detail: format!("Request {id} response identity/revision differs from audit"),
        });
    }
    let command =
        serde_json::from_str(&request.audit.command_json).map_err(|error| Error::StoreCorrupt {
            detail: format!("Cannot decode request {id} audit command: {error}"),
        })?;
    if let Command::ReplaceAttempt { reason, .. } = &command {
        sheltie_core::text::Summary::new(reason, "reason").map_err(|error| {
            Error::StoreCorrupt {
                detail: format!(
                    "Request {id} replacement reason is not valid bounded text: {error}"
                ),
            }
        })?;
    }
    let response = Response::from(persisted);
    let data = crate::snapshot::check_data(&response.reply, &response.data, work)?;
    validate_audit_snapshot(&request.request_id, &request.audit, &data)?;
    Ok(ReadRequestPayload {
        command,
        snapshot: response,
        effects: decode_effects(&request.row.effects_json)?,
        data,
    })
}

fn validate_audit_snapshot(
    request_id: &str,
    audit: &crate::store::read::AuditRow,
    data: &crate::snapshot::CheckedData,
) -> Result<()> {
    sheltie_core::work::Timestamp::parse(&audit.at).map_err(|error| Error::StoreCorrupt {
        detail: format!("Request {request_id} audit timestamp format is invalid: {error}"),
    })?;
    let approval_mismatch = matches!(data, crate::snapshot::CheckedData::Approved(approval)
        if approval.by.0 != audit.principal || approval.at.as_str() != audit.at);
    if audit.principal.is_empty() || approval_mismatch {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Request {request_id} audit principal/time differs from response snapshot"
            ),
        });
    }
    Ok(())
}

fn validate_audit_execution(
    request_id: &str,
    audit: &crate::store::read::AuditRow,
    state: &WorkState,
    reply: &Reply,
) -> Result<()> {
    let time_matches = match reply {
        Reply::Started { .. } => state.created_at.as_str() == audit.at,
        Reply::AttemptBegun { attempt, .. } => state
            .attempt(attempt)
            .is_some_and(|attempt| attempt.started_at.as_str() == audit.at),
        Reply::AttemptReplaced {
            replaced_attempt,
            attempt,
            ..
        } => {
            state
                .attempt(replaced_attempt)
                .and_then(|old| old.ended_at.as_ref())
                .is_some_and(|at| at.as_str() == audit.at)
                && state
                    .attempt(attempt)
                    .is_some_and(|new| new.started_at.as_str() == audit.at)
        }
        Reply::AttemptSubmitted { attempt, .. } | Reply::AttemptFailed { attempt } => state
            .attempt(attempt)
            .and_then(|attempt| attempt.ended_at.as_ref())
            .is_some_and(|at| at.as_str() == audit.at),
        Reply::GateApproved { node, occurrence } => state.approvals.iter().any(|approval| {
            approval.node == *node
                && approval.occurrence == *occurrence
                && approval.at.as_str() == audit.at
                && approval.by.0 == audit.principal
        }),
        Reply::Cancelled => state.updated_at.as_str() == audit.at,
    };
    if audit.principal.is_empty() || !time_matches {
        return Err(Error::StoreCorrupt {
            detail: format!(
                "Request {request_id} audit principal/time differs from original execution facts"
            ),
        });
    }
    Ok(())
}

fn validate_command_owner_data(
    request_id: &str,
    work_id: &WorkId,
    state: &WorkState,
    graph: &Graph,
    command: &Command,
    snapshot: &Response,
    data: &crate::snapshot::CheckedData,
) -> Result<()> {
    use crate::snapshot::CheckedData;

    let status = match data {
        CheckedData::Submitted(status) | CheckedData::Failed(status) => Some(*status),
        CheckedData::Approved(data) => Some(data.work_status),
        _ => None,
    };
    if status.is_some_and(|status| {
        !sheltie_core::work::reply_status_matches(&snapshot.reply, status, state, graph)
    }) {
        return Err(Error::StoreCorrupt {
            detail: format!("Request {request_id} snapshot status differs from original operation"),
        });
    }

    let valid = match (command, &snapshot.reply, data) {
        (Command::Start { .. }, Reply::Started { requires, .. }, CheckedData::Started(_)) => {
            crate::snapshot::start_matches(state, command, &snapshot.reply, data)
                && requires == graph.requires()
        }
        (
            Command::BeginAttempt { node, .. },
            Reply::AttemptBegun {
                attempt,
                brief_path,
                output_dir,
                inputs,
                outputs,
                requires,
                ..
            },
            CheckedData::Begun,
        ) => {
            let record = state.attempt(attempt).ok_or_else(|| Error::StoreCorrupt {
                detail: format!(
                    "Request {request_id} snapshot references unknown Attempt {attempt}"
                ),
            })?;
            let expected_outputs = sheltie_core::work::output_paths_for(state, graph, attempt)
                .map_err(|error| Error::StoreCorrupt {
                    detail: format!(
                        "Work {work_id} frozen output definitions are invalid: {error}"
                    ),
                })?;
            let expected_inputs = record
                .inputs
                .iter()
                .map(|(name, reference)| {
                    (
                        name.clone(),
                        reference.as_ref().map(|item| item.path.clone()),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            record.id.node == *node
                && brief_path == &state.attempt_dir(attempt).join_segment("brief.md")
                && output_dir
                    == &sheltie_core::work::layout::outputs_dir(&state.attempt_dir(attempt))
                && inputs == &expected_inputs
                && outputs == &expected_outputs
                && graph.node_requires(node).as_ref() == Some(requires)
        }
        (
            Command::ReplaceAttempt {
                attempt: replaced_attempt,
                reason,
                observed_inputs,
                ..
            },
            Reply::AttemptReplaced {
                replaced_attempt: reply_old,
                attempt,
                brief_path,
                output_dir,
                inputs,
                outputs,
                requires,
            },
            CheckedData::Replaced,
        ) => {
            let old = state
                .attempt(replaced_attempt)
                .ok_or_else(|| Error::StoreCorrupt {
                    detail: format!(
                        "Request {request_id} references unknown old Attempt {replaced_attempt}"
                    ),
                })?;
            let new = state.attempt(attempt).ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Request {request_id} references unknown new Attempt {attempt}"),
            })?;
            let node = graph
                .node(&attempt.node)
                .ok_or_else(|| Error::StoreCorrupt {
                    detail: format!(
                        "Request {request_id} replacement node is absent from the frozen graph"
                    ),
                })?;
            let input_paths = new
                .inputs
                .iter()
                .map(|(name, reference)| {
                    (
                        name.clone(),
                        reference.as_ref().map(|reference| reference.path.clone()),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let bound = node.inputs().iter().all(|declaration| {
                let observed = observed_inputs
                    .get(declaration.name())
                    .and_then(Option::as_ref);
                if matches!(
                    declaration.source(),
                    sheltie_core::flow::InputSource::EngineStats
                ) {
                    observed.is_none()
                } else {
                    match old.inputs.get(declaration.name()).and_then(Option::as_ref) {
                        Some(reference) => observed.is_some_and(|observed| {
                            observed.path == reference.path
                                && observed.sha256 == reference.sha256
                                && observed.bytes == reference.bytes
                        }),
                        None => observed.is_none(),
                    }
                }
            });
            reply_old == replaced_attempt
                && old.status == sheltie_core::work::AttemptStatus::Superseded
                && old
                    .replacement_reason
                    .as_ref()
                    .is_some_and(|recorded| recorded.as_str() == reason)
                && old.occurrence() == new.occurrence()
                && old.id.number.checked_add(1) == Some(new.id.number)
                && old.ended_at.as_ref() == Some(&new.started_at)
                && old.entered_from == new.entered_from
                && observed_inputs.keys().eq(old.inputs.keys())
                && bound
                && brief_path == &state.attempt_dir(attempt).join_segment("brief.md")
                && output_dir
                    == &sheltie_core::work::layout::outputs_dir(&state.attempt_dir(attempt))
                && inputs == &input_paths
                && outputs == &sheltie_core::work::output_paths_for(state, graph, attempt)?
                && graph.node_requires(&attempt.node).as_ref() == Some(requires)
        }
        (
            Command::SubmitAttempt { attempt, .. },
            Reply::AttemptSubmitted {
                attempt: reply_attempt,
                outputs,
            },
            CheckedData::Submitted(_),
        ) => state.attempt(attempt).is_some_and(|record| {
            reply_attempt == attempt
                && record.status == sheltie_core::work::AttemptStatus::Succeeded
                && outputs == &record.outputs
        }),
        (
            Command::FailAttempt { attempt, .. },
            Reply::AttemptFailed {
                attempt: reply_attempt,
            },
            CheckedData::Failed(_),
        ) => {
            reply_attempt == attempt
                && state.attempt(attempt).is_some_and(|record| {
                    record.status == sheltie_core::work::AttemptStatus::Failed
                })
        }
        (
            Command::ApproveGate { node },
            Reply::GateApproved {
                node: reply_node,
                occurrence,
            },
            CheckedData::Approved(data),
        ) => {
            reply_node == node
                && state.approvals.iter().any(|approval| {
                    approval.node == *node
                        && approval.occurrence == *occurrence
                        && approval.by == data.by
                        && approval.at == data.at
                })
        }
        (Command::Cancel, Reply::Cancelled, CheckedData::Cancelled) => {
            state.status == WorkStatus::Cancelled
        }
        _ => false,
    };
    if !valid {
        return Err(Error::StoreCorrupt {
            detail: format!("Request {request_id} audit command differs from Work {work_id} state"),
        });
    }
    Ok(())
}

/// Read @file content; intent records only the path, contents are observations from first execution.
fn materialize_inputs(inputs: &BTreeMap<String, InputValue>) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (key, value) in inputs {
        out.insert(key.clone(), materialize_summary(value)?);
    }
    Ok(out)
}

fn materialize_summary(value: &InputValue) -> Result<String> {
    match value {
        InputValue::Literal { text } => Ok(text.clone()),
        InputValue::AtFile { path } => {
            let f = crate::fsx::ExternalReadFile::open_regular_no_follow(
                &AbsPath::new(path.clone()).map_err(Error::Core)?,
            )
            .map_err(|error| Error::InputFileInvalid {
                path: path.clone(),
                reason: error.to_string(),
            })?;
            let bytes = f
                .read_bounded(crate::fsx::MAX_FILE_BYTES)
                .map_err(|error| Error::InputFileInvalid {
                    path: path.clone(),
                    reason: error.to_string(),
                })?;
            String::from_utf8(bytes).map_err(|_| Error::InputFileInvalid {
                path: path.clone(),
                reason: "Not UTF-8".to_string(),
            })
        }
    }
}

/// Convert core effects to persistent records; paths are root-relative, write_file carries exact bytes.
fn core_effects_to_ops(home: &Home, decision: &Decision) -> Result<Vec<EffectOp>> {
    let state = &decision.state;
    let mut ops = Vec::new();
    // begin directory skeleton: Attempt root, engine/, outputs/, and declared output parents, parents before children.
    if let Reply::AttemptBegun {
        attempt,
        output_dir,
        outputs,
        ..
    }
    | Reply::AttemptReplaced {
        attempt,
        output_dir,
        outputs,
        ..
    } = &decision.reply
    {
        let mut dirs: Vec<String> = Vec::new();
        let push_chain = |abs: &AbsPath, dirs: &mut Vec<String>| -> Result<()> {
            let rel = home.to_rel(abs)?;
            let mut acc = String::new();
            for seg in rel.split('/') {
                if seg.is_empty() {
                    continue;
                }
                acc = if acc.is_empty() {
                    seg.to_string()
                } else {
                    format!("{acc}/{seg}")
                };
                dirs.push(acc.clone());
            }
            Ok(())
        };
        push_chain(output_dir, &mut dirs)?;
        for p in outputs.values() {
            if let Some(parent) = p.as_path().parent() {
                push_chain(
                    &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                    &mut dirs,
                )?;
            }
        }
        // Include parents of engine-generated brief.md and engine/stats.json in the skeleton.
        for effect in &decision.effects {
            if let Effect::WriteBrief { path, .. } | Effect::WriteFile { path, .. } = effect {
                if let Some(parent) = path.as_path().parent() {
                    push_chain(
                        &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                        &mut dirs,
                    )?;
                }
            }
        }
        dirs.sort();
        dirs.dedup();
        ops.push(EffectOp::PrepareAttempt {
            work_id: state.work_id.as_str().to_string(),
            attempt_id: attempt.to_string(),
            dirs,
        });
    }
    for effect in &decision.effects {
        match effect {
            Effect::WriteBrief { path, content } | Effect::WriteFile { path, content } => {
                ops.push(EffectOp::WriteFile {
                    path: home.to_rel(path)?,
                    sha256: Sha256Hex::of_bytes(content.as_bytes()).as_str().to_string(),
                    content: content.clone(),
                });
            }
            Effect::SealOutputs { .. } => {
                // Read refs from committed state, including exact ArtifactRef values.
                let attempt = state.attempts.last().ok_or_else(|| Error::StoreCorrupt {
                    detail: "Sealing effect has no corresponding Attempt".to_string(),
                })?;
                let mut refs = Vec::new();
                for r in attempt.outputs.values() {
                    refs.push(RefJson {
                        path: home.to_rel(&r.path)?,
                        sha256: r.sha256.as_str().to_string(),
                        bytes: r.bytes,
                    });
                }
                ops.push(EffectOp::SealOutputs { refs });
            }
            Effect::RefreshStatusCard => {
                ops.push(EffectOp::RefreshStatusCard {
                    work_id: state.work_id.as_str().to_string(),
                });
            }
        }
    }
    Ok(ops)
}

/// Commit-time snapshot payload (protocol §3 returns), replayed unchanged field-for-field.
fn snapshot_data(decision: &Decision) -> serde_json::Value {
    let state = &decision.state;
    match &decision.reply {
        Reply::Started {
            work_id,
            work_dir,
            requires,
        } => serde_json::json!({
            "work_id": work_id.as_str(),
            "name": state.name.as_str(),
            "workbook": {
                "id": state.workbook.id.as_str(),
                "version": state.workbook.version,
                "digest": state.workbook.digest.as_str(),
            },
            "flow": state.flow.as_str(),
            "work_dir": work_dir.as_str(),
            "requires": requires,
        }),
        Reply::AttemptBegun {
            attempt,
            brief_path,
            output_dir,
            inputs,
            outputs,
            requires,
        }
        | Reply::AttemptReplaced {
            attempt,
            brief_path,
            output_dir,
            inputs,
            outputs,
            requires,
            ..
        } => {
            let inputs: serde_json::Map<String, serde_json::Value> = inputs
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_ref()
                            .map_or(serde_json::Value::Null, |p| serde_json::json!(p.as_str())),
                    )
                })
                .collect();
            let outputs: serde_json::Map<String, serde_json::Value> = outputs
                .iter()
                .map(|(k, p)| (k.clone(), serde_json::json!(p.as_str())))
                .collect();
            let mut data = serde_json::json!({
                "attempt": attempt.to_string(),
                "node": attempt.node.as_str(),
                "occurrence": attempt.occurrence,
                "number": attempt.number,
                "brief_path": brief_path.as_str(),
                "output_dir": output_dir.as_str(),
                "inputs": inputs,
                "outputs": outputs,
                "requires": requires,
            });
            if let Reply::AttemptReplaced {
                replaced_attempt, ..
            } = &decision.reply
            {
                data["replaced_attempt"] = serde_json::json!(replaced_attempt.to_string());
            }
            data
        }
        Reply::AttemptSubmitted { attempt, outputs } => {
            let outs: serde_json::Map<String, serde_json::Value> = outputs
                .iter()
                .map(|(k, r)| {
                    (
                        k.clone(),
                        serde_json::json!({
                            "path": r.path.as_str(),
                            "sha256": r.sha256.as_str(),
                            "bytes": r.bytes,
                        }),
                    )
                })
                .collect();
            serde_json::json!({
                "attempt": attempt.to_string(),
                "outputs": outs,
                "work_status": state.status,
            })
        }
        Reply::AttemptFailed { attempt } => serde_json::json!({
            "attempt": attempt.to_string(),
            "work_status": state.status,
        }),
        Reply::GateApproved { node, occurrence } => {
            let approval = state
                .approvals
                .last()
                .cloned()
                .unwrap_or_else(|| unreachable!("Approval reply must have an approval record"));
            serde_json::json!({
                "node": node.as_str(),
                "occurrence": occurrence,
                "by": approval.by.0,
                "at": approval.at.as_str(),
                "work_status": state.status,
            })
        }
        Reply::Cancelled => serde_json::json!({
            "work_id": state.work_id.as_str(),
            "work_status": state.status,
        }),
    }
}

/// Original instruction text: read File from the frozen copy; use Text directly.
fn instruction_text_of(loaded: &Loaded, node: &NodeId) -> Result<String> {
    let def = loaded.graph.node(node).ok_or_else(|| Error::NotFound {
        what: format!("Node {node}"),
    })?;
    match def.instruction() {
        sheltie_core::flow::Instruction::Text(t) => Ok(t.clone()),
        sheltie_core::flow::Instruction::File(rel) => loaded
            .instructions
            .get(rel)
            .cloned()
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Frozen-copy load lacks Node {node} instruction file {rel}"),
            }),
    }
}

fn observe_loaded_input(
    home: &Home,
    loaded: &Loaded,
    node: &NodeId,
    name: &str,
    path: &AbsPath,
) -> Result<Option<ObservedFile>> {
    let declaration = loaded
        .graph
        .node(node)
        .and_then(|definition| {
            definition
                .inputs()
                .iter()
                .find(|input| input.name() == name)
        })
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("Frozen Flow lacks Node {node} input declaration {name}"),
        })?;
    if let sheltie_core::flow::InputSource::Resource { path: relative } = declaration.source() {
        if path != &loaded.state.workbook_dir().join(relative) {
            return Err(Error::StoreCorrupt {
                detail: format!("Resource input {name} path differs from frozen Workbook"),
            });
        }
        return loaded
            .resource_files
            .get(relative)
            .cloned()
            .map(Some)
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Frozen-copy load lacks resource {relative}"),
            });
    }
    if !path.as_path().starts_with(home.root().as_path()) {
        return Err(Error::StoreCorrupt {
            detail: format!("Input {name} path is outside the management root"),
        });
    }
    let Some(file) = crate::fsx::open_managed_optional(home, path)? else {
        return Ok(None);
    };
    let (sha256, bytes) = file.sha256_bounded(crate::fsx::MAX_FILE_BYTES)?;
    Ok(Some(ObservedFile::new(path.clone(), sha256, bytes)))
}

/// Observe a declared output: verify handle identity, then hash through that handle. Before reading all contents,
/// reject files above the absolute 32 MiB output ceiling (workbook §3.2 max_bytes) with
/// OUTPUT_TOO_LARGE; core decide still compares the precise declared limit.
fn observe_output(
    home: &Home,
    name: &str,
    path: &AbsPath,
) -> Result<(Option<ObservedFile>, Option<crate::fsx::SafeFile>)> {
    let Some(f) = crate::fsx::open_managed_optional(home, path)? else {
        return Ok((None, None));
    };
    let len = f.metadata().len();
    if len > crate::fsx::MAX_FILE_BYTES {
        return Err(Error::Core(sheltie_core::Error::OutputTooLarge {
            output: name.to_string(),
            max_bytes: crate::fsx::MAX_FILE_BYTES,
            actual: len,
        }));
    }
    let (sha256, bytes) = f.sha256_bounded(crate::fsx::MAX_FILE_BYTES)?;
    Ok((
        Some(ObservedFile::new(path.clone(), sha256, bytes)),
        Some(f),
    ))
}

/// Audit Command JSON replaces large fields such as instruction_text with length (CommitInput convention).
fn audit_json(cmd: &Command) -> Result<String> {
    let mut value = serde_json::to_value(cmd).map_err(|e| Error::StoreCorrupt {
        detail: format!("Command serialization failed: {e}"),
    })?;
    if let serde_json::Value::Object(map) = &mut value {
        for field in ["instruction_text", "summary", "reason"] {
            if field == "reason" && matches!(cmd, Command::ReplaceAttempt { .. }) {
                continue;
            }
            if let Some(text) = map.get(field).and_then(|value| value.as_str()) {
                map.insert(
                    field.to_string(),
                    // Preserve this persisted audit marker: historical equality uses its exact bytes.
                    serde_json::Value::String(format!("<{} 字节>", text.len())),
                );
            }
        }
    }
    serde_json::to_string(&value).map_err(|e| Error::StoreCorrupt {
        detail: format!("Command serialization failed: {e}"),
    })
}

#[cfg(test)]
mod pending_stage_tests {
    use super::stage_pending;
    use crate::home::Home;
    use sheltie_core::path::AbsPath;

    // Task: C002-T19
    #[test]
    fn remove_stage_persists_owner_and_container_without_payload() {
        let temp = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(
            (AbsPath::new(temp.path().to_string_lossy().into_owned()).unwrap()).as_str(),
        ))
        .unwrap();
        let lock = home.acquire_lock().unwrap();
        let payload =
            stage_pending(&home, &lock, "internal-1", "request-1", "remove_workbook").unwrap();
        let container = temp.path().join("pending/internal-1");
        let owner = temp.path().join("pending/internal-1.owner");

        assert!(container.is_dir());
        assert!(!container.join("payload").exists());
        assert_eq!(
            std::fs::read_to_string(owner).unwrap(),
            "{\"format\":\"pending/v1\",\"internal_id\":\"internal-1\",\"request_id\":\"request-1\",\"op\":\"remove_workbook\"}\n"
        );
        assert_eq!(
            payload,
            home.root()
                .join_segment("pending")
                .join_segment("internal-1")
                .join_segment("payload")
        );
    }

    // Task: C002-T19
    #[test]
    fn publish_stage_persists_owner_container_and_payload() {
        let temp = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(
            (AbsPath::new(temp.path().to_string_lossy().into_owned()).unwrap()).as_str(),
        ))
        .unwrap();
        let lock = home.acquire_lock().unwrap();
        let payload = stage_pending(&home, &lock, "internal-2", "request-2", "start_work").unwrap();
        assert!(std::path::Path::new(payload.as_str()).is_dir());
    }
}

#[cfg(test)]
mod input_file_tests {
    use super::materialize_summary;
    use crate::error::Error;
    use crate::request::InputValue;

    // Task: C002-T24
    #[test]
    fn input_file_accepts_exact_32_mib_and_rejects_one_more() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("input.txt");
        let bytes = vec![b'x'; crate::fsx::MAX_FILE_BYTES as usize];
        std::fs::write(&path, &bytes).unwrap();
        let path = path.to_string_lossy().into_owned();
        let input = InputValue::AtFile { path: path.clone() };
        assert_eq!(
            materialize_summary(&input).unwrap().len() as u64,
            crate::fsx::MAX_FILE_BYTES
        );

        let over = vec![b'x'; crate::fsx::MAX_FILE_BYTES as usize + 1];
        std::fs::write(&path, over).unwrap();
        assert!(matches!(
            materialize_summary(&input),
            Err(Error::InputFileInvalid { path: rejected, .. }) if rejected == path
        ));
    }
}
