//! One write transaction; order in storage contract §2.

use rusqlite::OptionalExtension;
use sheltie_core::ids::WorkId;
use sheltie_core::work::{Principal, Timestamp, WorkState};

use super::Store;
use crate::error::{Error, Result};

/// Complete write-transaction input (storage §2.3).
#[derive(Debug, Clone)]
pub struct CommitInput {
    /// None for writes without Work ownership, such as workbook add/remove.
    pub work_id: Option<WorkId>,
    /// None for start inserts; other writes use the previously read revision.
    pub expected_revision: Option<u64>,
    /// New state; None leaves works unchanged.
    pub state: Option<WorkState>,
    /// workbooks row to insert (add).
    pub workbook_insert: Option<super::WorkbookRow>,
    /// workbooks row to delete (remove).
    pub workbook_delete: Option<(String, String)>,
    /// Transactional reference check: nonterminal Work references yield WORKBOOK_IN_USE (§5.2).
    /// Corrupt works rows yield STORE_CORRUPT here, without skipping them.
    pub workbook_in_use_check: Option<(String, String)>,
    pub request_id: String,
    /// RequestIntent canonical JSON sha256 (§2.1).
    pub intent_hash: String,
    /// Complete commit-time ResponseSnapshot, returned unchanged on replay (cli-result/v2).
    pub reply_json: String,
    /// Effect registration (§3.2).
    pub effects_json: String,
    pub principal: Principal,
    /// Command JSON with large fields reduced, for audit storage.
    pub command_json: String,
    pub at: Timestamp,
}

/// Transaction result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitOutcome {
    /// Successful write; revision is post-commit, or zero without Work ownership.
    Committed { revision: u64 },
    /// Same request_id/intent replay returns original response/effects; published records the original request's
    /// effect completion, telling callers whether recovery remains necessary.
    Replayed {
        reply_json: String,
        effects_json: String,
        published: bool,
    },
}

impl Store {
    /// Execute one write transaction.
    ///
    /// ```text
    /// BEGIN IMMEDIATE
    ///   Lookup requests: same hash -> ROLLBACK/Replayed; different hash -> ROLLBACK/RequestConflict
    ///   With expected_revision, compare works.revision; mismatch -> ROLLBACK/RevisionConflict
    ///   With state, UPDATE/INSERT works, incrementing revision and deriving status from state
    ///   INSERT audit, using an empty work_id for None
    ///   INSERT requests
    /// COMMIT
    /// ```
    /// No file reads or digest computation inside the transaction.
    pub fn commit(&self, input: CommitInput) -> Result<CommitOutcome> {
        if let Some(state) = &input.state {
            let id = input.work_id.as_ref().ok_or_else(|| Error::StoreCorrupt {
                detail: "Missing work_id when writing WorkState".to_string(),
            })?;
            if &state.work_id != id {
                return Err(Error::StoreCorrupt {
                    detail: format!(
                        "Write target {id} differs from state_json.work_id {}",
                        state.work_id
                    ),
                });
            }
            state
                .validate_persisted()
                .map_err(|detail| Error::StoreCorrupt {
                    detail: format!("Invalid WorkState write: {detail}"),
                })?;
        }
        crate::failpoint::maybe_exit("before_commit");
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let prior: Option<(String, String, String, bool)> = tx
            .query_row(
                "SELECT intent_hash, reply_json, effects_json, published FROM requests WHERE request_id = ?1",
                [&input.request_id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get::<_, i64>(3)? != 0,
                    ))
                },
            )
            .optional()?;
        if let Some((hash, reply, effects, published)) = prior {
            // Both replay hits return directly; drop rolls back the transaction.
            if hash == input.intent_hash {
                return Ok(CommitOutcome::Replayed {
                    reply_json: reply,
                    effects_json: effects,
                    published,
                });
            }
            return Err(Error::RequestConflict {
                request_id: input.request_id,
            });
        }
        if let Some(expected) = input.expected_revision {
            let actual: Option<i64> = match &input.work_id {
                Some(id) => tx
                    .query_row(
                        "SELECT revision FROM works WHERE work_id = ?1",
                        [id.as_str()],
                        |r| r.get(0),
                    )
                    .optional()?,
                None => None,
            };
            let actual = match actual {
                Some(value) => u64::try_from(value)
                    .ok()
                    .filter(|revision| *revision > 0)
                    .ok_or_else(|| Error::StoreCorrupt {
                        detail: format!("Invalid Work revision {value}"),
                    })?,
                None => 0,
            };
            if actual != expected {
                return Err(Error::RevisionConflict { expected, actual });
            }
        }
        // start with None expected_revision inserts revision 1; other writes increment the expected value.
        let mut revision = 0u64;
        if let Some(state) = &input.state {
            revision = match input.expected_revision {
                None => 1,
                Some(previous) => previous.checked_add(1).ok_or_else(|| Error::StoreCorrupt {
                    detail: "Work revision cannot be incremented safely".to_string(),
                })?,
            };
            let db_revision = i64::try_from(revision).map_err(|_| Error::StoreCorrupt {
                detail: "Work revision exceeds SQLite INTEGER limit".to_string(),
            })?;
            let state_json = serde_json::to_string(state).map_err(|e| Error::StoreCorrupt {
                detail: format!("WorkState serialization failed: {e}"),
            })?;
            let work_id = input
                .work_id
                .as_ref()
                .map_or_else(String::new, |w| w.as_str().to_string());
            tx.execute(
                "INSERT INTO works (work_id, revision, status, state_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(work_id) DO UPDATE SET
                   revision = excluded.revision,
                   status = excluded.status,
                   state_json = excluded.state_json,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    work_id,
                    db_revision,
                    state.status.column(),
                    state_json,
                    state.created_at.as_str(),
                    state.updated_at.as_str(),
                ],
            )?;
        }
        if let Some(row) = &input.workbook_insert {
            tx.execute(
                "INSERT INTO workbooks (id, version, digest, dir, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![row.id, row.version, row.digest, row.dir, row.added_at],
            )
            .map_err(|e| match &e {
                rusqlite::Error::SqliteFailure(code, _)
                    if code.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    Error::WorkbookExists {
                        id: row.id.clone(),
                        version: row.version.clone(),
                    }
                }
                _ => e.into(),
            })?;
        }
        if let Some((id, version)) = &input.workbook_in_use_check {
            // Validate rows before checking references; corrupt rows stop the whole operation (GF-16).
            let mut stmt = tx.prepare("SELECT work_id, revision, status, state_json FROM works")?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })?;
            let mut referencing = Vec::new();
            for row in rows {
                let (work_id, revision, status, state_json) = row?;
                let row = super::read::decode_row(&work_id, revision, &status, &state_json)?;
                if !row.state.status.is_terminal()
                    && row.state.workbook.id.as_str() == id
                    && row.state.workbook.version == *version
                {
                    referencing.push(row.state.work_id);
                }
            }
            if !referencing.is_empty() {
                return Err(Error::WorkbookInUse {
                    id: id.clone(),
                    version: version.clone(),
                    works: referencing,
                });
            }
        }
        if let Some((id, version)) = &input.workbook_delete {
            let n = tx.execute(
                "DELETE FROM workbooks WHERE id = ?1 AND version = ?2",
                rusqlite::params![id, version],
            )?;
            if n == 0 {
                return Err(Error::NotFound {
                    what: format!("Workbook {id}@{version}"),
                });
            }
        }
        let audit_revision = i64::try_from(revision).map_err(|_| Error::StoreCorrupt {
            detail: "Audit revision exceeds SQLite INTEGER limit".to_string(),
        })?;
        tx.execute(
            "INSERT INTO audit (work_id, revision, request_id, principal, command_json, at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                input
                    .work_id
                    .as_ref()
                    .map_or_else(String::new, |w| w.as_str().to_string()),
                audit_revision,
                input.request_id,
                input.principal.0,
                input.command_json,
                input.at.as_str(),
            ],
        )?;
        tx.execute(
            "INSERT INTO requests (request_id, intent_hash, work_id, reply_json, effects_json, published, at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
            rusqlite::params![
                input.request_id,
                input.intent_hash,
                input.work_id.as_ref().map(|w| w.as_str()),
                input.reply_json,
                input.effects_json,
                input.at.as_str(),
            ],
        )?;
        tx.commit()?;
        Ok(CommitOutcome::Committed { revision })
    }
}
