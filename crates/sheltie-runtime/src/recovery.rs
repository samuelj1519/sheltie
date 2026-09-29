//! Work 与 Workbook 共用的已提交效果校验、发布和恢复入口。

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::effects::CheckedEffects;
use crate::effects::{execute, execute_with_observed_outputs};
use crate::error::{Error, Result};
use crate::fsx::SafeFile;
use crate::home::{Home, HomeLock};
use crate::store::Store;
use crate::store::read::RequestRow;

pub(crate) trait RecoveryAccess {
    fn load_effects(&self, request_id: &str, row: &RequestRow) -> Result<CheckedEffects>;
    fn refresh_cards(&self, effects: &CheckedEffects, lock: &HomeLock) -> Result<()>;
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedResponse {
    pub(crate) request_id: String,
    pub(crate) revision: u64,
    pub(crate) replayed: bool,
    pub(crate) reply: sheltie_core::work::Reply,
    pub(crate) data: serde_json::Value,
    pub(crate) next: Vec<sheltie_core::work::NextOp>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddedSnapshotData {
    pub(crate) id: String,
    pub(crate) version: String,
    pub(crate) digest: String,
    pub(crate) flows: Vec<String>,
    pub(crate) requires: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RemovedSnapshotData {
    pub(crate) id: String,
    pub(crate) version: String,
}

/// 锁内、在新请求进入事务前恢复所有未完成请求。
pub(crate) fn before_write(
    home: &Home,
    store: &Store,
    lock: &HomeLock,
    request_id: &str,
    intent_hash: &str,
    access: &dyn RecoveryAccess,
) -> Result<()> {
    let current_hash = store.lookup_request_hash(request_id)?;
    if let Some(hash) = &current_hash {
        if hash != intent_hash {
            return Err(Error::RequestConflict {
                request_id: request_id.to_string(),
            });
        }
    }
    let current_identity = match current_hash {
        Some(_) => match store.lookup_request(request_id) {
            Ok(Some(identity)) => Some(identity),
            Ok(None) => {
                return Err(own_pending(
                    request_id,
                    None,
                    Error::StoreCorrupt {
                        detail: format!("已确认的请求 {request_id} 读取快照时消失"),
                    },
                ));
            }
            Err(cause) => return Err(own_pending(request_id, None, cause)),
        },
        None => None,
    };
    let current_work_id = match store.lookup_request_work(request_id) {
        Ok(work_id) => work_id,
        Err(cause) if current_identity.is_some() => {
            let original = current_identity
                .as_ref()
                .and_then(|(_, reply)| original_snapshot_parts(request_id, None, reply));
            return Err(own_pending(request_id, original, cause));
        }
        Err(cause) => return Err(cause),
    };
    let current = match store.inspect_request(request_id) {
        Ok(None) if current_identity.is_some() => {
            let Some((_, reply_json)) = current_identity.as_ref() else {
                return Err(Error::StoreCorrupt {
                    detail: format!("请求 {request_id} 在锁内读取时消失"),
                });
            };
            let original = original_snapshot_parts(
                request_id,
                current_work_id.as_ref().and_then(Option::as_deref),
                reply_json,
            );
            return Err(own_pending(
                request_id,
                original,
                Error::StoreCorrupt {
                    detail: format!("已提交请求 {request_id} 缺少完整requests行"),
                },
            ));
        }
        Ok(current) => current,
        Err(cause) if current_identity.is_some() => {
            let Some((_, reply_json)) = current_identity.as_ref() else {
                return Err(cause);
            };
            let work_id = current_work_id.as_ref().and_then(Option::as_deref);
            let original = original_snapshot_parts(request_id, work_id, reply_json);
            return Err(own_pending(request_id, original, cause));
        }
        Err(cause) => return Err(cause),
    };
    if let Some(row) = &current {
        let original = original_snapshot(request_id, row);
        if let Err(cause) = access.load_effects(request_id, row) {
            return Err(own_pending(request_id, original, cause));
        }
    }
    let current_original = current
        .as_ref()
        .and_then(|row| original_snapshot(request_id, row));
    for pending_id in store.unpublished_request_ids()? {
        let row = match store.inspect_request(&pending_id) {
            Ok(Some(row)) => row,
            Ok(None) => {
                let cause = Error::StoreCorrupt {
                    detail: format!("未发布请求 {pending_id} 缺少requests行"),
                };
                let blocker = own_pending_without_snapshot(&pending_id, cause);
                if pending_id == request_id {
                    return Err(blocker);
                }
                return Err(blocked_by_pending(
                    request_id,
                    current.is_some(),
                    &pending_id,
                    current_original.clone(),
                    blocker,
                ));
            }
            Err(cause) => {
                let pending_snapshot = match store.lookup_request(&pending_id) {
                    Ok(snapshot) => snapshot,
                    Err(identity_error) => {
                        return Err(blocked_by_pending(
                            request_id,
                            current.is_some(),
                            &pending_id,
                            current_original.clone(),
                            own_pending_without_snapshot(&pending_id, identity_error),
                        ));
                    }
                };
                let pending_work_id = match store.lookup_request_work(&pending_id) {
                    Ok(work_id) => work_id,
                    Err(owner_error) => {
                        let original = pending_snapshot.as_ref().and_then(|(_, reply)| {
                            original_snapshot_parts(&pending_id, None, reply)
                        });
                        let blocker = own_pending_optional(&pending_id, original, owner_error);
                        if pending_id == request_id {
                            return Err(blocker);
                        }
                        return Err(blocked_by_pending(
                            request_id,
                            current.is_some(),
                            &pending_id,
                            current_original.clone(),
                            blocker,
                        ));
                    }
                };
                let pending_original = pending_snapshot.as_ref().and_then(|(_, reply)| {
                    original_snapshot_parts(
                        &pending_id,
                        pending_work_id.as_ref().and_then(Option::as_deref),
                        reply,
                    )
                });
                let blocker = own_pending_optional(&pending_id, pending_original, cause);
                if pending_id == request_id {
                    return Err(blocker);
                }
                return Err(blocked_by_pending(
                    request_id,
                    current.is_some(),
                    &pending_id,
                    current_original.clone(),
                    blocker,
                ));
            }
        };
        if pending_id == request_id && row.intent_hash != intent_hash {
            return Err(Error::RequestConflict {
                request_id: request_id.to_string(),
            });
        }
        if let Err(error) = finish_request(home, store, lock, &pending_id, None, false, access) {
            if pending_id == request_id {
                return Err(error);
            }
            return Err(blocked_by_pending(
                request_id,
                current.is_some(),
                &pending_id,
                current_original.clone(),
                error,
            ));
        }
    }
    Ok(())
}

/// 完成一个已提交请求。普通COMMIT后调用完成发布；显式重放也会核对已发布效果。
pub(crate) fn finish_request(
    home: &Home,
    store: &Store,
    lock: &HomeLock,
    request_id: &str,
    observed_outputs: Option<&BTreeMap<String, SafeFile>>,
    verify_published: bool,
    access: &dyn RecoveryAccess,
) -> Result<()> {
    let identity = match store.lookup_request_hash(request_id) {
        Ok(Some(_)) => match store.lookup_request(request_id) {
            Ok(identity) => identity,
            Err(cause) => return Err(own_pending_optional(request_id, None, cause)),
        },
        Ok(None) => None,
        Err(cause) => return Err(own_pending_optional(request_id, None, cause)),
    };
    let work_id = match store.lookup_request_work(request_id) {
        Ok(work_id) => work_id,
        Err(cause) => {
            let original = identity
                .as_ref()
                .and_then(|(_, reply)| original_snapshot_parts(request_id, None, reply));
            if identity.is_some() {
                return Err(own_pending(request_id, original, cause));
            }
            return Err(cause);
        }
    };
    let row = match store.inspect_request(request_id) {
        Ok(Some(row)) => row,
        Ok(None) => {
            let Some((_, reply_json)) = identity.as_ref() else {
                return Err(own_pending_optional(
                    request_id,
                    None,
                    Error::StoreCorrupt {
                        detail: format!("缺少请求 {request_id} 的持久记录"),
                    },
                ));
            };
            let original = original_snapshot_parts(
                request_id,
                work_id.as_ref().and_then(Option::as_deref),
                reply_json,
            );
            return Err(own_pending(
                request_id,
                original,
                Error::StoreCorrupt {
                    detail: format!("已提交请求 {request_id} 缺少完整requests行"),
                },
            ));
        }
        Err(cause) => {
            let Some((_, reply_json)) = identity.as_ref() else {
                return Err(own_pending_optional(request_id, None, cause));
            };
            let original = original_snapshot_parts(
                request_id,
                work_id.as_ref().and_then(Option::as_deref),
                reply_json,
            );
            return Err(own_pending(request_id, original, cause));
        }
    };
    if row.published && !verify_published {
        return Ok(());
    }
    let original = original_snapshot(request_id, &row);
    let effects = access
        .load_effects(request_id, &row)
        .map_err(|cause| own_pending(request_id, original.clone(), cause))?;
    let complete = !row.published;
    let execution = match observed_outputs {
        Some(observed) if complete => {
            execute_with_observed_outputs(home, lock, &effects, true, Some(observed))
        }
        _ => execute(home, lock, &effects, complete),
    };
    execution.map_err(|cause| own_pending(request_id, original.clone(), cause))?;
    if complete {
        access
            .refresh_cards(&effects, lock)
            .map_err(|cause| own_pending(request_id, original.clone(), cause))?;
        store
            .mark_published(request_id)
            .map_err(|cause| own_pending(request_id, original, cause))?;
    }
    Ok(())
}

pub(crate) fn own_pending(request_id: &str, original: Option<String>, cause: Error) -> Error {
    own_pending_optional(request_id, original, cause)
}

fn own_pending_without_snapshot(request_id: &str, cause: Error) -> Error {
    own_pending_optional(request_id, None, cause)
}

fn own_pending_optional(request_id: &str, original: Option<String>, cause: Error) -> Error {
    Error::effect_pending(true, request_id.to_string(), None, &cause, original, None)
}

fn blocked_by_pending(
    request_id: &str,
    committed: bool,
    pending_request_id: &str,
    original: Option<String>,
    blocker: Error,
) -> Error {
    let (cause, cause_detail, pending_original) = match blocker {
        Error::EffectPending {
            cause,
            cause_detail,
            original,
            pending_original,
            ..
        } => (cause, cause_detail, pending_original.or(original)),
        other => (other.code(), other.to_string(), None),
    };
    Error::EffectPending {
        committed,
        request_id: request_id.to_string(),
        pending_request_id: Some(pending_request_id.to_string()),
        cause,
        cause_detail,
        original: original.map(String::into_boxed_str),
        pending_original,
    }
}

pub(crate) fn original_snapshot(request_id: &str, row: &RequestRow) -> Option<String> {
    original_snapshot_parts(request_id, row.work_id.as_deref(), &row.reply_json)
}

fn original_snapshot_parts(
    request_id: &str,
    work_id: Option<&str>,
    reply_json: &str,
) -> Option<String> {
    if let Some(work_id) = work_id {
        return work_original_snapshot(request_id, work_id, reply_json);
    }
    if let Ok(response) = serde_json::from_str::<PersistedResponse>(reply_json) {
        if response.request_id != request_id {
            return None;
        }
        if let sheltie_core::work::Reply::Started { work_id, .. } = &response.reply {
            return work_original_snapshot(request_id, work_id.as_str(), reply_json);
        }
        return None;
    }
    let snapshot: WorkbookSnapshot = serde_json::from_str(reply_json).ok()?;
    if snapshot.request_id != request_id || snapshot.replayed {
        return None;
    }
    if !strict_workbook_data(&snapshot.data) {
        return None;
    }
    let mut data = snapshot.data;
    if let Some(object) = data.as_object_mut() {
        object.insert("replayed".to_string(), serde_json::Value::Bool(false));
    }
    serde_json::json!({
        "ok": true,
        "request_id": snapshot.request_id,
        "data": data,
        "next": [],
    })
    .to_string()
    .into()
}

fn strict_workbook_data(value: &serde_json::Value) -> bool {
    if let Ok(data) = serde_json::from_value::<AddedSnapshotData>(value.clone()) {
        let Ok(id) = sheltie_core::ids::WorkbookId::new(&data.id) else {
            return false;
        };
        if id.as_str() != data.id
            || !crate::load::valid_workbook_version(&data.version)
            || sheltie_core::digest::Sha256Hex::new(data.digest.clone()).is_err()
            || data.flows.is_empty()
            || data
                .flows
                .iter()
                .any(|flow| sheltie_core::ids::FlowId::new(flow).is_err())
            || data
                .flows
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != data.flows.len()
            || data.requires.iter().any(|requirement| {
                requirement
                    .split_once(':')
                    .is_none_or(|(kind, name)| kind.is_empty() || name.is_empty())
            })
        {
            return false;
        }
        return true;
    }
    let Ok(data) = serde_json::from_value::<RemovedSnapshotData>(value.clone()) else {
        return false;
    };
    let Ok(id) = sheltie_core::ids::WorkbookId::new(&data.id) else {
        return false;
    };
    if id.as_str() != data.id || !crate::load::valid_workbook_version(&data.version) {
        return false;
    }
    true
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkbookSnapshot {
    request_id: String,
    replayed: bool,
    data: serde_json::Value,
}

fn work_original_snapshot(request_id: &str, work_id: &str, reply_json: &str) -> Option<String> {
    let response: PersistedResponse = serde_json::from_str(reply_json).ok()?;
    if response.request_id != request_id || response.revision == 0 || response.replayed {
        return None;
    }
    let work = sheltie_core::ids::WorkId::parse(work_id).ok()?;
    if !persisted_response_data_is_well_formed(&response, &work) {
        return None;
    }
    let mut data = response.data;
    if let Some(object) = data.as_object_mut() {
        object.insert("replayed".to_string(), serde_json::Value::Bool(false));
    }
    let next = response
        .next
        .iter()
        .map(|op| next_item(work_id, op))
        .collect::<Vec<_>>();
    Some(
        serde_json::json!({
            "ok": true,
            "request_id": response.request_id,
            "revision": response.revision,
            "data": data,
            "next": next,
        })
        .to_string(),
    )
}

fn snapshot_data_has_exact_fields(data: &serde_json::Value, expected: &[&str]) -> bool {
    let Some(object) = data.as_object() else {
        return false;
    };
    object.len() == expected.len() && expected.iter().all(|name| object.contains_key(*name))
}

fn work_status_value_is_canonical(value: &serde_json::Value) -> bool {
    serde_json::from_value::<sheltie_core::work::WorkStatus>(value.clone())
        .ok()
        .and_then(|status| serde_json::to_value(status).ok())
        .is_some_and(|canonical| canonical == *value)
}

fn persisted_response_data_is_well_formed(
    response: &PersistedResponse,
    work_id: &sheltie_core::ids::WorkId,
) -> bool {
    use sheltie_core::work::{Reply, WorkStatus};

    let data = &response.data;
    let fields_valid = match &response.reply {
        Reply::Started {
            work_id: reply_work,
            work_dir,
            requires,
        } => {
            let workbook =
                serde_json::from_value::<sheltie_core::work::WorkbookRef>(data["workbook"].clone());
            let name = serde_json::from_value::<sheltie_core::ids::WorkName>(data["name"].clone());
            snapshot_data_has_exact_fields(
                data,
                &[
                    "work_id", "name", "workbook", "flow", "work_dir", "requires",
                ],
            ) && reply_work == work_id
                && data["work_id"].as_str() == Some(work_id.as_str())
                && data["work_dir"].as_str() == Some(work_dir.as_str())
                && name.is_ok_and(|name| {
                    work_id
                        .as_str()
                        .get(15..)
                        .is_some_and(|suffix| suffix == name.as_str())
                })
                && data["flow"]
                    .as_str()
                    .is_some_and(|flow| sheltie_core::ids::FlowId::new(flow).is_ok())
                && workbook
                    .is_ok_and(|workbook| crate::load::valid_workbook_version(&workbook.version))
                && serde_json::to_value(requires).is_ok_and(|expected| data["requires"] == expected)
        }
        Reply::AttemptBegun {
            attempt,
            brief_path,
            output_dir,
            inputs,
            outputs,
            requires,
        } => {
            let attempt_text = attempt.to_string();
            let expected_inputs: serde_json::Map<String, serde_json::Value> = inputs
                .iter()
                .map(|(name, path)| {
                    (
                        name.clone(),
                        path.as_ref().map_or(serde_json::Value::Null, |path| {
                            serde_json::json!(path.as_str())
                        }),
                    )
                })
                .collect();
            snapshot_data_has_exact_fields(
                data,
                &[
                    "attempt",
                    "node",
                    "occurrence",
                    "retry",
                    "brief_path",
                    "output_dir",
                    "inputs",
                    "outputs",
                    "requires",
                ],
            ) && data["attempt"].as_str() == Some(attempt_text.as_str())
                && data["node"].as_str() == Some(attempt.node.as_str())
                && data["occurrence"].as_u64() == Some(u64::from(attempt.occurrence))
                && data["retry"].as_u64() == Some(u64::from(attempt.retry))
                && data["brief_path"].as_str() == Some(brief_path.as_str())
                && data["output_dir"].as_str() == Some(output_dir.as_str())
                && data["inputs"] == serde_json::Value::Object(expected_inputs)
                && serde_json::to_value(outputs).is_ok_and(|expected| data["outputs"] == expected)
                && serde_json::to_value(requires).is_ok_and(|expected| data["requires"] == expected)
        }
        Reply::AttemptSubmitted { attempt, outputs } => {
            let attempt_text = attempt.to_string();
            snapshot_data_has_exact_fields(data, &["attempt", "outputs", "work_status"])
                && data["attempt"].as_str() == Some(attempt_text.as_str())
                && serde_json::to_value(outputs).is_ok_and(|expected| data["outputs"] == expected)
        }
        Reply::AttemptFailed { attempt } => {
            let attempt_text = attempt.to_string();
            snapshot_data_has_exact_fields(data, &["attempt", "work_status"])
                && data["attempt"].as_str() == Some(attempt_text.as_str())
        }
        Reply::GateApproved { node, occurrence } => {
            let principal =
                serde_json::from_value::<sheltie_core::work::Principal>(data["by"].clone());
            let at = data["at"]
                .as_str()
                .and_then(|value| sheltie_core::work::Timestamp::parse(value).ok());
            snapshot_data_has_exact_fields(data, &["node", "occurrence", "by", "at", "work_status"])
                && data["node"].as_str() == Some(node.as_str())
                && data["occurrence"].as_u64() == Some(u64::from(*occurrence))
                && principal.is_ok_and(|principal| !principal.0.is_empty())
                && at.is_some()
        }
        Reply::Cancelled => {
            snapshot_data_has_exact_fields(data, &["work_id", "work_status"])
                && data["work_id"].as_str() == Some(work_id.as_str())
                && serde_json::from_value::<WorkStatus>(data["work_status"].clone()).is_ok_and(
                    |status| {
                        status == WorkStatus::Cancelled
                            && work_status_value_is_canonical(&data["work_status"])
                    },
                )
        }
    };
    let has_status = matches!(
        &response.reply,
        Reply::AttemptSubmitted { .. }
            | Reply::AttemptFailed { .. }
            | Reply::GateApproved { .. }
            | Reply::Cancelled
    );
    fields_valid && (!has_status || work_status_value_is_canonical(&data["work_status"]))
}

fn next_item(work_id: &str, op: &sheltie_core::work::NextOp) -> serde_json::Value {
    use sheltie_core::work::NextOp;

    match op {
        NextOp::BeginAttempt {
            node,
            edge,
            executor,
            tier,
        } => {
            let mut item = serde_json::json!({
                "op": "attempt begin",
                "args": { "work": work_id, "node": node.as_str() },
            });
            if let Some(edge) = edge {
                item["edge"] = serde_json::json!(edge.as_str());
            }
            item["executor"] = serde_json::json!(executor.as_str());
            if let Some(tier) = tier {
                item["tier"] = serde_json::json!(tier.as_str());
            }
            item
        }
        NextOp::SubmitAttempt { attempt } => serde_json::json!({
            "op": "attempt submit",
            "args": { "work": work_id, "attempt": attempt.to_string() },
        }),
        NextOp::FailAttempt { attempt } => serde_json::json!({
            "op": "attempt fail",
            "args": { "work": work_id, "attempt": attempt.to_string() },
        }),
        NextOp::ApproveGate { node } => serde_json::json!({
            "op": "gate approve",
            "args": { "work": work_id, "node": node.as_str() },
        }),
        NextOp::Cancel => serde_json::json!({
            "op": "work cancel",
            "args": { "work": work_id },
        }),
    }
}
