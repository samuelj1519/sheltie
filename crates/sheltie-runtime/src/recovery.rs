//! Work 与 Workbook 共用的已提交效果校验、发布和恢复入口。

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::effects::CheckedEffects;
use crate::effects::{execute, execute_with_observed_outputs};
use crate::error::{Error, Result};
use crate::fsx::SafeFile;
use crate::home::{Home, HomeLock};
use crate::snapshot::PersistedResponse;
use crate::store::Store;
use crate::store::read::RequestRow;

pub(crate) trait RecoveryAccess {
    fn load_effects(&self, request_id: &str, row: &RequestRow) -> Result<CheckedEffects>;
    fn refresh_cards(&self, effects: &CheckedEffects, lock: &HomeLock) -> Result<()>;
}

pub(crate) struct CheckedRequestEffects<'a> {
    pub(crate) request_id: &'a str,
    pub(crate) row: &'a RequestRow,
    pub(crate) effects: &'a CheckedEffects,
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
    // A retry must retire its own uncommitted originals before registering the same id.
    if current.is_none() {
        crate::pending::prepare_new_request(home, store, lock, request_id)?;
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
    finish_checked_request(
        home,
        store,
        lock,
        CheckedRequestEffects {
            request_id,
            row: &row,
            effects: &effects,
        },
        observed_outputs,
        access,
    )
}

/// 调用者已在同一写锁内核过请求与整组效果；不重复装入冻结图和历史载荷。
pub(crate) fn finish_checked_request(
    home: &Home,
    store: &Store,
    lock: &HomeLock,
    checked: CheckedRequestEffects<'_>,
    observed_outputs: Option<&BTreeMap<String, SafeFile>>,
    access: &dyn RecoveryAccess,
) -> Result<()> {
    let CheckedRequestEffects {
        request_id,
        row,
        effects,
    } = checked;
    let original = original_snapshot(request_id, row);
    let complete = !row.published;
    let execution = match observed_outputs {
        Some(observed) if complete => {
            execute_with_observed_outputs(home, lock, effects, true, Some(observed))
        }
        _ => execute(home, lock, effects, complete),
    };
    execution.map_err(|cause| own_pending(request_id, original.clone(), cause))?;
    if complete {
        access
            .refresh_cards(effects, lock)
            .map_err(|cause| own_pending(request_id, original.clone(), cause))?;
        store
            .mark_published(request_id)
            .map_err(|cause| own_pending(request_id, original, cause))?;
        crate::failpoint::maybe_exit("request_published_before_cleanup");
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
    if crate::snapshot::check_data(&response.reply, &response.data, &work).is_err() {
        return None;
    }
    let mut data = response.data;
    if let Some(object) = data.as_object_mut() {
        object.insert("replayed".to_string(), serde_json::Value::Bool(false));
    }
    let next = response
        .next
        .iter()
        .map(|op| sheltie_core::work::render::next_item_json(&work, op))
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
