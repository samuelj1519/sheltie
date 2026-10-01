//! Work 与 Workbook 共用的已提交效果校验、发布和恢复入口。

use crate::effects::{CheckedEffects, execute, execute_with_observed_outputs};
use crate::error::{Error, Result};
use crate::fsx::SafeFile;
use crate::home::{Home, HomeLock};
use crate::store::Store;
use crate::store::read::{RequestMetadata, RequestRow};
use serde::Deserialize;
use std::collections::BTreeMap;

pub(crate) struct CheckedRequest {
    pub(crate) row: RequestRow,
    pub(crate) original: String,
    pub(crate) effects: CheckedEffects,
}

#[derive(Debug)]
pub(crate) struct RequestLoadError {
    pub(crate) cause: Box<Error>,
    pub(crate) original: Option<String>,
}

impl From<Error> for RequestLoadError {
    fn from(cause: Error) -> Self {
        Self {
            cause: Box::new(cause),
            original: None,
        }
    }
}

impl RequestLoadError {
    pub(crate) fn with_original(cause: Error, original: String) -> Self {
        Self {
            cause: Box::new(cause),
            original: Some(original),
        }
    }
    pub(crate) fn into_error(self) -> Error {
        *self.cause
    }
    pub(crate) fn into_pending(self, request_id: &str) -> Error {
        own_pending(request_id, self.original, *self.cause)
    }
}

pub(crate) type RequestLoadResult<T> = std::result::Result<T, RequestLoadError>;

pub(crate) trait RecoveryAccess {
    fn load_request(
        &self,
        request_id: &str,
        metadata: &RequestMetadata,
    ) -> RequestLoadResult<CheckedRequest>;
    fn refresh_cards(&self, effects: &CheckedEffects, lock: &HomeLock) -> Result<()>;
}

pub(crate) struct CheckedRequestEffects<'a> {
    pub(crate) request_id: &'a str,
    pub(crate) request: &'a CheckedRequest,
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

fn request_metadata(store: &Store, request_id: &str) -> Result<RequestMetadata> {
    store
        .inspect_request_metadata(request_id)?
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("缺少请求 {request_id} 的持久记录"),
        })
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
    if current_hash
        .as_ref()
        .is_some_and(|hash| hash != intent_hash)
    {
        return Err(Error::RequestConflict {
            request_id: request_id.to_string(),
        });
    }
    let mut current = if current_hash.is_some() {
        let metadata = request_metadata(store, request_id)
            .map_err(|cause| own_pending(request_id, None, cause))?;
        Some(
            access
                .load_request(request_id, &metadata)
                .map_err(|failure| failure.into_pending(request_id))?,
        )
    } else {
        None
    };
    let committed = current.is_some();
    let current_original = current.as_ref().map(|request| request.original.clone());
    for pending_id in store.unpublished_request_ids()? {
        let loaded = if pending_id == request_id {
            match current.take() {
                Some(request) => Ok(request),
                None => request_metadata(store, &pending_id)
                    .map_err(RequestLoadError::from)
                    .and_then(|metadata| access.load_request(&pending_id, &metadata)),
            }
        } else {
            request_metadata(store, &pending_id)
                .map_err(RequestLoadError::from)
                .and_then(|metadata| access.load_request(&pending_id, &metadata))
        };
        let request = match loaded {
            Ok(request) => request,
            Err(failure) => {
                let blocker = failure.into_pending(&pending_id);
                if pending_id == request_id {
                    return Err(blocker);
                }
                return Err(blocked_by_pending(
                    request_id,
                    committed,
                    &pending_id,
                    current_original.clone(),
                    blocker,
                ));
            }
        };
        if pending_id == request_id && request.row.intent_hash != intent_hash {
            return Err(Error::RequestConflict {
                request_id: request_id.to_string(),
            });
        }
        if let Err(error) = finish_checked_request(
            home,
            store,
            lock,
            CheckedRequestEffects {
                request_id: &pending_id,
                request: &request,
            },
            None,
            access,
        ) {
            if pending_id == request_id {
                return Err(error);
            }
            return Err(blocked_by_pending(
                request_id,
                committed,
                &pending_id,
                current_original.clone(),
                error,
            ));
        }
    }
    if !committed {
        crate::pending::prepare_new_request(home, store, lock, request_id)?;
    }
    Ok(())
}

/// 完成一个已提交请求。快照校验成功后，效果失败保留已核原响应。
pub(crate) fn finish_request(
    home: &Home,
    store: &Store,
    lock: &HomeLock,
    request_id: &str,
    observed_outputs: Option<&BTreeMap<String, SafeFile>>,
    verify_published: bool,
    access: &dyn RecoveryAccess,
) -> Result<()> {
    let metadata = request_metadata(store, request_id)
        .map_err(|cause| own_pending(request_id, None, cause))?;
    let request = access
        .load_request(request_id, &metadata)
        .map_err(|failure| failure.into_pending(request_id))?;
    if request.row.published && !verify_published {
        return Ok(());
    }
    finish_checked_request(
        home,
        store,
        lock,
        CheckedRequestEffects {
            request_id,
            request: &request,
        },
        observed_outputs,
        access,
    )
}

/// 调用者在同一写锁内核过快照与整组效果；复用已装入的上下文。
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
        request,
    } = checked;
    let original = Some(request.original.clone());
    let complete = !request.row.published;
    let execution = match observed_outputs {
        Some(observed) if complete => {
            execute_with_observed_outputs(home, lock, &request.effects, true, Some(observed))
        }
        _ => execute(home, lock, &request.effects, complete),
    };
    execution.map_err(|cause| own_pending(request_id, original.clone(), cause))?;
    if complete {
        access
            .refresh_cards(&request.effects, lock)
            .map_err(|cause| own_pending(request_id, original.clone(), cause))?;
        store
            .mark_published(request_id)
            .map_err(|cause| own_pending(request_id, original, cause))?;
        crate::failpoint::maybe_exit("request_published_before_cleanup");
    }
    Ok(())
}

pub(crate) fn own_pending(request_id: &str, original: Option<String>, cause: Error) -> Error {
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

/// 已通过元数据与业务绑定校验的Work快照转成协议原响应。
pub(crate) fn work_original_response(
    work: &sheltie_core::ids::WorkId,
    response: &crate::service::Response,
) -> String {
    let mut data = response.data.clone();
    if let Some(object) = data.as_object_mut() {
        object.insert("replayed".to_string(), serde_json::Value::Bool(false));
    }
    let next = response
        .next
        .iter()
        .map(|op| sheltie_core::work::render::next_item_json(work, op))
        .collect::<Vec<_>>();
    serde_json::json!({"ok":true, "request_id":response.request_id,
        "revision":response.revision, "data":data, "next":next})
    .to_string()
}

/// 已通过元数据与业务绑定校验的Workbook快照转成协议原响应。
pub(crate) fn workbook_original_response(request_id: &str, mut data: serde_json::Value) -> String {
    if let Some(object) = data.as_object_mut() {
        object.insert("replayed".to_string(), serde_json::Value::Bool(false));
    }
    serde_json::json!({"ok":true, "request_id":request_id, "data":data, "next":[]}).to_string()
}
