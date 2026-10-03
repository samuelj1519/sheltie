//! Pending owner sidecars, request references and locked cleanup.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;

use crate::effects::{EffectOp, decode_effects, valid_digest};
use crate::error::{Error, Result};
use crate::fsx::{self, ManagedEntryKind, ManagedRelPath};
use crate::home::{Home, HomeLock};
use crate::store::Store;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PendingOwner {
    pub(crate) format: String,
    pub(crate) internal_id: String,
    pub(crate) request_id: String,
    pub(crate) op: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingReference {
    pub(crate) request_id: String,
    pub(crate) published: bool,
    pub(crate) operation: String,
    pub(crate) pending: String,
    pub(crate) final_path: String,
    pub(crate) owner: String,
    pub(crate) digest: String,
    pub(crate) digest_root: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PublishLocation {
    pub(crate) pending: String,
    pub(crate) final_path: String,
    pub(crate) pending_publish: bool,
}

/// Every request row contributes to this index, including published rows whose cleanup metadata
/// remains. Building it is all-or-nothing: malformed persisted effects stop cleanup before IO.
#[derive(Debug, Default)]
pub(crate) struct PendingReferenceIndex {
    references: BTreeMap<String, Vec<PendingReference>>,
    request_ids: BTreeSet<String>,
}

#[derive(Default)]
struct PendingRootRecord {
    owner: Option<ManagedEntryKind>,
    container: Option<ManagedEntryKind>,
    marker: Option<ManagedEntryKind>,
}

impl PendingReferenceIndex {
    pub(crate) fn load(store: &Store) -> Result<Self> {
        Self::from_rows(store.all_request_effect_rows()?)
    }

    pub(crate) fn load_work_start(store: &Store, work: &sheltie_core::ids::WorkId) -> Result<Self> {
        Self::from_rows(store.work_start_effect_rows(work)?)
    }

    fn from_rows(rows: Vec<crate::store::read::RequestEffectRow>) -> Result<Self> {
        let mut index = Self::default();
        for request in rows {
            index.request_ids.insert(request.request_id.clone());
            let effects =
                decode_effects(&request.effects_json).map_err(|error| Error::StoreCorrupt {
                    detail: maintenance_warning(
                        &request.request_id,
                        &format!("requests/{}.effects_json", request.request_id),
                        &error.to_string(),
                    ),
                })?;
            let mut seen_internal_ids = BTreeSet::new();
            for effect in effects {
                let (pending, final_path, owner, operation, digest, digest_root) = match effect {
                    EffectOp::PublishDir {
                        pending,
                        final_path,
                        owner,
                        digest,
                        digest_root,
                    } => {
                        validate_publish_reference(&owner, &final_path, &digest, &digest_root)?;
                        validate_publish_request_work_id(&request, &owner)?;
                        (pending, final_path, owner, "publish", digest, digest_root)
                    }
                    EffectOp::DeleteDir {
                        pending,
                        final_path,
                        owner,
                        digest,
                    } => {
                        validate_delete_reference(&owner, &final_path, &digest)?;
                        if request.work_id.is_some() {
                            return Err(Error::StoreCorrupt {
                                detail: format!(
                                    "remove Workbook请求 {} 带有Work归属",
                                    request.request_id
                                ),
                            });
                        }
                        (pending, final_path, owner, "delete", digest, String::new())
                    }
                    other => {
                        validate_other_effect_paths(&request, &other)?;
                        continue;
                    }
                };
                let internal_id = crate::effects::pending_internal_id(&pending)?.to_string();
                if !seen_internal_ids.insert(internal_id.clone())
                    || Sha256Hex::new(digest.clone()).is_err()
                {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "请求 {} 的pending引用有重复internal_id或无效digest",
                            request.request_id
                        ),
                    });
                }
                index
                    .references
                    .entry(internal_id)
                    .or_default()
                    .push(PendingReference {
                        request_id: request.request_id.clone(),
                        published: request.published,
                        operation: operation.to_string(),
                        pending,
                        final_path,
                        owner,
                        digest,
                        digest_root,
                    });
            }
        }
        for (internal_id, references) in &index.references {
            if references.len() != 1 {
                return Err(Error::StoreCorrupt {
                    detail: format!(
                        "pending/{internal_id} 被 {} 个requests效果引用",
                        references.len()
                    ),
                });
            }
        }
        Ok(index)
    }

    pub(crate) fn get(&self, internal_id: &str) -> Option<&PendingReference> {
        self.references
            .get(internal_id)
            .and_then(|references| references.first())
    }

    pub(crate) fn contains_request(&self, request_id: &str) -> bool {
        self.request_ids.contains(request_id)
    }

    pub(crate) fn internal_ids(&self) -> impl Iterator<Item = &String> {
        self.references.keys()
    }

    pub(crate) fn publish_references_for_work(&self, work: &str) -> Vec<&PendingReference> {
        let owner = format!("work:{work}");
        self.references
            .values()
            .flatten()
            .filter(|reference| reference.operation == "publish" && reference.owner == owner)
            .collect()
    }

    pub(crate) fn lifecycle_references_for_workbook(
        &self,
        owner: &str,
        final_path: &str,
    ) -> Vec<&PendingReference> {
        self.references
            .values()
            .flatten()
            .filter(|reference| reference.owner == owner && reference.final_path == final_path)
            .collect()
    }
}

fn sole_directory_exists(candidate_exists: bool, other_exists: bool) -> bool {
    candidate_exists && !other_exists
}

/// Locate the single registered directory through the rename window without creating a lock or
/// Store. The caller still verifies request/owner/digest before exposing loaded bytes.
pub(crate) fn locate_publish_dir(
    home: &Home,
    pending: &str,
    final_path: &str,
    pending_publish: bool,
) -> Result<(sheltie_core::path::AbsPath, bool)> {
    for _ in 0..3 {
        let final_before = fsx::managed_directory_exists_readonly(home, final_path)?;
        if !pending_publish {
            if final_before {
                let final_after = fsx::managed_directory_exists_readonly(home, final_path)?;
                if final_after {
                    return Ok((home.rel(final_path)?, false));
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
            continue;
        }
        let pending_before = fsx::managed_directory_exists_readonly(home, pending)?;
        crate::failpoint::rendezvous("locator_after_initial_observation", final_path)
            .map_err(|error| Error::io(final_path, error))?;
        if sole_directory_exists(pending_before, final_before) {
            let pending_after = fsx::managed_directory_exists_readonly(home, pending)?;
            let final_after = fsx::managed_directory_exists_readonly(home, final_path)?;
            if sole_directory_exists(pending_after, final_after) {
                return Ok((home.rel(pending)?, true));
            }
        }
        if sole_directory_exists(final_before, pending_before) {
            let final_after = fsx::managed_directory_exists_readonly(home, final_path)?;
            let pending_after = fsx::managed_directory_exists_readonly(home, pending)?;
            if sole_directory_exists(final_after, pending_after) {
                return Ok((home.rel(final_path)?, true));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    if !pending_publish && !fsx::managed_directory_exists_readonly(home, final_path)? {
        return Err(Error::NotFound {
            what: final_path.to_string(),
        });
    }
    Err(Error::io(
        final_path,
        std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "发布目录在只读定位期间处于rename交错窗口",
        ),
    ))
}

/// Read one registered tree across its pending-to-final rename without taking the engine lock.
/// A path that disappears after the locator's final observation is retried a bounded number of
/// times; a persistent race becomes a temporary I/O error instead of a fallback to another tree.
pub(crate) fn read_publish_dir<T>(
    home: &Home,
    location: &PublishLocation,
    mut read: impl FnMut(&sheltie_core::path::AbsPath) -> Result<T>,
) -> Result<(T, bool)> {
    for _ in 0..3 {
        let (directory, pending_publish) = locate_publish_dir(
            home,
            &location.pending,
            &location.final_path,
            location.pending_publish,
        )?;
        crate::failpoint::rendezvous("pending_read_after_locate", &location.final_path)
            .map_err(|error| Error::io(&location.final_path, error))?;
        match read(&directory) {
            Ok(value) => return Ok((value, pending_publish)),
            Err(error @ Error::NotFound { .. }) => {
                let relative = home.to_rel(&directory)?;
                if fsx::managed_directory_exists_readonly(home, &relative)? {
                    return Err(error);
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Err(error) => return Err(error),
        }
    }
    Err(Error::io(
        &location.final_path,
        std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "pending/final目录在只读读取期间持续rename交错",
        ),
    ))
}

pub(crate) fn prepare_new_request(
    home: &Home,
    store: &Store,
    lock: &HomeLock,
    request_id: &str,
) -> Result<()> {
    if !fsx::managed_directory_exists(home, lock, "pending")? {
        return Ok(());
    }
    let mut selected = BTreeSet::new();
    for entry in fsx::managed_directory_entries_locked(home, lock, "pending")? {
        let Some(id) = entry
            .name
            .strip_suffix(".owner")
            .filter(|id| valid_internal_id(id))
        else {
            continue;
        };
        if entry.kind != ManagedEntryKind::RegularFile {
            continue;
        }
        if let Ok(owner) = read_owner(home, id) {
            if owner.request_id == request_id {
                selected.insert(id.to_string());
            }
        }
    }
    if selected.is_empty() {
        return Ok(());
    }
    let warnings = cleanup_selected(home, store, lock, Some(&selected))?;
    if !warnings.is_empty() {
        return Err(Error::io(
            home.pending_dir().as_str(),
            std::io::Error::other(format!(
                "当前未提交请求 {request_id} 的原件未能安全清理，尚未登记新请求：{}",
                warnings.join("; ")
            )),
        ));
    }
    for entry in fsx::managed_directory_entries_locked(home, lock, "pending")? {
        let id = entry
            .name
            .strip_suffix(".owner")
            .or_else(|| entry.name.strip_suffix(".deleted"))
            .unwrap_or(&entry.name);
        if selected.contains(id) {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "当前未提交请求 {request_id} 的残留 pending/{} 未清，不能登记新请求",
                    entry.name
                ),
            });
        }
    }
    Ok(())
}

pub(crate) fn cleanup(home: &Home, store: &Store, lock: &HomeLock) -> Result<Vec<String>> {
    cleanup_selected(home, store, lock, None)
}

fn cleanup_selected(
    home: &Home,
    store: &Store,
    lock: &HomeLock,
    selected: Option<&BTreeSet<String>>,
) -> Result<Vec<String>> {
    // Parse every Store reference before deleting any path. A single undecodable row makes
    // absence unprovable for the entire pass.
    let references = PendingReferenceIndex::load(store)?;
    if !fsx::managed_directory_exists(home, lock, "pending")? {
        return Ok(references
            .references
            .iter()
            .flat_map(|(id, refs)| {
                refs.iter()
                    .filter(|reference| !reference.published)
                    .map(move |reference| {
                        maintenance_warning(
                            &reference.request_id,
                            &format!("pending/{id}"),
                            "未发布原件引用存在但pending根缺失",
                        )
                    })
            })
            .collect());
    }
    let entries = fsx::managed_directory_entries_locked(home, lock, "pending")?;
    let mut roots = BTreeMap::<String, PendingRootRecord>::new();
    let mut warnings = Vec::new();
    for entry in entries {
        if let Some(selected) = selected {
            let id = entry
                .name
                .strip_suffix(".owner")
                .or_else(|| entry.name.strip_suffix(".deleted"))
                .unwrap_or(&entry.name);
            if !selected.contains(id) {
                continue;
            }
        }
        if let Some(id) = entry.name.strip_suffix(".owner") {
            if valid_internal_id(id) {
                roots.entry(id.to_string()).or_default().owner = Some(entry.kind);
            } else {
                warnings.push(maintenance_warning(
                    "unknown",
                    &format!("pending/{}", entry.name),
                    "侧车文件名不是合法internal_id，保留原件",
                ));
            }
        } else if let Some(id) = entry.name.strip_suffix(".deleted") {
            if valid_internal_id(id) {
                roots.entry(id.to_string()).or_default().marker = Some(entry.kind);
            } else {
                warnings.push(maintenance_warning(
                    "unknown",
                    &format!("pending/{}", entry.name),
                    "删除marker文件名不是合法internal_id，保留原件",
                ));
            }
        } else if valid_internal_id(&entry.name) {
            roots.entry(entry.name.clone()).or_default().container = Some(entry.kind);
        } else {
            warnings.push(maintenance_warning(
                "unknown",
                &format!("pending/{}", entry.name),
                "未识别pending对象，保留原件",
            ));
        }
    }
    for id in references.internal_ids() {
        roots.entry(id.clone()).or_default();
    }
    for (id, record) in roots {
        let reference = references.get(&id);
        let owner_file = match record.owner {
            Some(ManagedEntryKind::RegularFile) => match read_owner_file(home, &id) {
                Ok(owner) => Some(owner),
                Err(error) => {
                    warnings.push(maintenance_warning(
                        reference
                            .map(|reference| reference.request_id.as_str())
                            .unwrap_or("unknown"),
                        &format!("pending/{id}.owner"),
                        &format!("owner无法读取或校验，保留原件：{error}"),
                    ));
                    continue;
                }
            },
            Some(_) => {
                warnings.push(maintenance_warning(
                    reference
                        .map(|reference| reference.request_id.as_str())
                        .unwrap_or("unknown"),
                    &format!("pending/{id}.owner"),
                    "owner类型异常，保留原件",
                ));
                continue;
            }
            None => None,
        };
        let owner_matches_reference = match (owner_file.as_ref(), reference) {
            (Some((owner, _)), Some(reference)) => owner_matches_reference(owner, &id, reference),
            (None, Some(reference)) if reference.published => true,
            (None, Some(_)) => false,
            (Some((owner, _)), None) => !references.contains_request(&owner.request_id),
            (None, None) => false,
        };
        if !owner_matches_reference {
            let request_id = reference
                .map(|reference| reference.request_id.as_str())
                .or_else(|| {
                    owner_file
                        .as_ref()
                        .map(|(owner, _)| owner.request_id.as_str())
                })
                .unwrap_or("unknown");
            warnings.push(maintenance_warning(
                request_id,
                &format!("pending/{id}"),
                "owner与Store引用不一致或引用缺失，保留原件",
            ));
            continue;
        }
        if reference.is_some_and(|reference| !reference.published) {
            continue;
        }
        if reference.is_none() && owner_file.is_none() {
            if record.container.is_some() || record.marker.is_some() {
                warnings.push(maintenance_warning(
                    "unknown",
                    &format!("pending/{id}"),
                    "没有合法owner，保留原件",
                ));
            }
            continue;
        }
        if reference.is_none()
            && owner_file
                .as_ref()
                .is_some_and(|(owner, _)| references.contains_request(&owner.request_id))
        {
            let request_id = owner_file
                .as_ref()
                .map(|(owner, _)| owner.request_id.as_str())
                .unwrap_or("unknown");
            warnings.push(maintenance_warning(
                request_id,
                &format!("pending/{id}"),
                "owner.request_id已存在但effects未引用该id，保留原件",
            ));
            continue;
        }
        let cleanup_request_id = reference
            .map(|reference| reference.request_id.as_str())
            .or_else(|| {
                owner_file
                    .as_ref()
                    .map(|(owner, _)| owner.request_id.as_str())
            })
            .unwrap_or("pending-orphan")
            .to_string();
        let cleanup_request_id = cleanup_request_id.as_str();
        if let Some(reference) = reference {
            if reference.operation == "publish" && record.marker.is_some() {
                warnings.push(maintenance_warning(
                    &reference.request_id,
                    &format!("pending/{id}.deleted"),
                    "publish请求出现删除marker，保留原件",
                ));
                continue;
            }
            if let Some(marker_kind) = record.marker {
                if marker_kind != ManagedEntryKind::RegularFile {
                    warnings.push(maintenance_warning(
                        &reference.request_id,
                        &format!("pending/{id}.deleted"),
                        "删除marker类型异常，保留原件",
                    ));
                    continue;
                }
                if reference.operation == "delete" {
                    match crate::effects::read_deleted_marker(home, lock, &id) {
                        Ok(Some(_)) => {}
                        Ok(None) => {}
                        Err(error) => {
                            warnings.push(maintenance_warning(
                                &reference.request_id,
                                &format!("pending/{id}.deleted"),
                                &format!("删除marker校验失败，保留原件：{error}"),
                            ));
                            continue;
                        }
                    }
                }
            }
        } else if record.marker.is_some() {
            warnings.push(maintenance_warning(
                "unknown",
                &format!("pending/{id}.deleted"),
                "marker不能证明孤儿归属，保留原件",
            ));
            continue;
        }

        if let Some(container_kind) = record.container {
            if container_kind != ManagedEntryKind::Directory {
                warnings.push(maintenance_warning(
                    cleanup_request_id,
                    &format!("pending/{id}"),
                    "container类型异常，保留owner和原件",
                ));
                continue;
            }
            let path = format!("pending/{id}");
            let tree = match fsx::open_managed_tree(home, lock, &path) {
                Ok(tree) => tree,
                Err(error) => {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &path,
                        &format!("无法取得已核对象，保留owner：{error}"),
                    ));
                    continue;
                }
            };
            if reference.is_some_and(|reference| reference.published) {
                match fsx::managed_tree_is_empty_locked(home, lock, &tree, &path) {
                    Ok(true) => {}
                    Ok(false) => {
                        warnings.push(maintenance_warning(
                            cleanup_request_id,
                            &path,
                            "已完成请求的container仍有内容，保留原件",
                        ));
                        continue;
                    }
                    Err(error) => {
                        warnings.push(maintenance_warning(
                            cleanup_request_id,
                            &path,
                            &format!("container状态无法核验，保留原件：{error}"),
                        ));
                        continue;
                    }
                }
                if let Err(error) = crate::failpoint::rendezvous(
                    "pending_cleanup_after_empty_check",
                    cleanup_request_id,
                ) {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &path,
                        &format!("清理同步点失败，保留原件：{error}"),
                    ));
                    continue;
                }
                if let Err(error) = maybe_fail_cleanup("pending_cleanup_before_remove") {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &path,
                        &format!("清理失败，保留原件：{error}"),
                    ));
                    continue;
                }
                if let Err(error) = fsx::remove_empty_managed_tree(home, lock, &tree, &path) {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &path,
                        &format!("empty-container删除失败，保留变化对象：{error}"),
                    ));
                    continue;
                }
            } else {
                if let Err(error) = fsx::make_managed_tree_writable(home, lock, &tree, &path) {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &path,
                        &format!("合法孤儿无法放开内部权限，保留原件：{error}"),
                    ));
                    continue;
                }
                if let Err(error) = maybe_fail_cleanup("pending_cleanup_before_remove") {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &path,
                        &format!("清理失败，保留原件：{error}"),
                    ));
                    continue;
                }
                if let Err(error) =
                    fsx::remove_managed_tree(home, lock, &tree, &path, cleanup_request_id)
                {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &path,
                        &format!("合法孤儿删除失败，保留原件：{error}"),
                    ));
                    continue;
                }
            }
        }
        if let Some((_, owner_handle)) = owner_file {
            if let Err(error) = maybe_fail_cleanup("pending_cleanup_before_remove") {
                warnings.push(maintenance_warning(
                    cleanup_request_id,
                    &format!("pending/{id}.owner"),
                    &format!("侧车删除失败：{error}"),
                ));
            } else {
                let path = home.rel(&format!("pending/{id}.owner"))?;
                if let Err(error) =
                    fsx::remove_managed_file_if_same(home, lock, &path, &owner_handle)
                {
                    warnings.push(maintenance_warning(
                        cleanup_request_id,
                        &format!("pending/{id}.owner"),
                        &format!("侧车删除失败：{error}"),
                    ));
                }
            }
        }
        if record.marker == Some(ManagedEntryKind::RegularFile)
            && reference.is_some_and(|reference| reference.operation == "delete")
        {
            if let Err(error) = maybe_fail_cleanup("pending_cleanup_before_remove") {
                warnings.push(maintenance_warning(
                    cleanup_request_id,
                    &format!("pending/{id}.deleted"),
                    &format!("marker删除失败：{error}"),
                ));
                continue;
            }
            match crate::effects::read_deleted_marker(home, lock, &id) {
                Ok(Some(marker)) => {
                    let path = home.pending_dir().join_segment(&format!("{id}.deleted"));
                    if let Err(error) = fsx::remove_managed_file_if_same(home, lock, &path, &marker)
                    {
                        warnings.push(maintenance_warning(
                            cleanup_request_id,
                            &format!("pending/{id}.deleted"),
                            &format!("marker删除失败：{error}"),
                        ));
                    }
                }
                Ok(None) => {}
                Err(error) => warnings.push(maintenance_warning(
                    cleanup_request_id,
                    &format!("pending/{id}.deleted"),
                    &format!("marker读取失败：{error}"),
                )),
            }
        }
    }
    Ok(warnings)
}

fn valid_internal_id(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}

fn maintenance_warning(request_id: &str, object: &str, reason: &str) -> String {
    format!("request_id={request_id} object={object} reason={reason}")
}

fn owner_matches_reference(
    owner: &PendingOwner,
    internal_id: &str,
    reference: &PendingReference,
) -> bool {
    owner.format == "pending/v1"
        && owner.internal_id == internal_id
        && owner.request_id == reference.request_id
        && owner.op
            == match reference.operation.as_str() {
                "delete" => "remove_workbook",
                "publish" if reference.owner.starts_with("work:") => "start_work",
                "publish" => "add_workbook",
                _ => "",
            }
}

fn maybe_fail_cleanup(name: &str) -> Result<()> {
    crate::failpoint::cleanup_error(name).map_err(|error| Error::io("pending cleanup", error))
}

fn validate_publish_request_work_id(
    request: &crate::store::read::RequestEffectRow,
    owner: &str,
) -> Result<()> {
    if let Some(work) = owner.strip_prefix("work:") {
        let parsed =
            sheltie_core::ids::WorkId::parse(work).map_err(|error| Error::StoreCorrupt {
                detail: format!("publish owner {owner:?} 的WorkId无效：{error}"),
            })?;
        if request.work_id.as_deref() != Some(parsed.as_str()) {
            return Err(Error::StoreCorrupt {
                detail: format!("publish请求 {} 的work_id与owner不一致", request.request_id),
            });
        }
    } else if request.work_id.is_some() {
        return Err(Error::StoreCorrupt {
            detail: format!("Workbook publish请求 {} 带有Work归属", request.request_id),
        });
    }
    Ok(())
}

fn validate_other_effect_paths(
    request: &crate::store::read::RequestEffectRow,
    effect: &EffectOp,
) -> Result<()> {
    let validate_path = |path: &str, allow_ancestor: bool| -> Result<()> {
        let relative =
            ManagedRelPath::new(path.to_string()).map_err(|error| Error::StoreCorrupt {
                detail: format!("请求 {} effects路径无效：{error}", request.request_id),
            })?;
        let work = request
            .work_id
            .as_deref()
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Workbook请求 {} 含Work效果路径", request.request_id),
            })?;
        let parsed =
            sheltie_core::ids::WorkId::parse(work).map_err(|error| Error::StoreCorrupt {
                detail: format!("请求 {} 的work_id无效：{error}", request.request_id),
            })?;
        let prefix = format!("works/{parsed}/");
        let ancestor = allow_ancestor
            && (relative.as_str() == "works" || relative.as_str() == format!("works/{parsed}"));
        if !relative.as_str().starts_with(&prefix) && !ancestor {
            return Err(Error::StoreCorrupt {
                detail: format!(
                    "请求 {} effects路径 {} 不属于Work目录 {}",
                    request.request_id, path, parsed
                ),
            });
        }
        Ok(())
    };

    match effect {
        EffectOp::PrepareAttempt { work_id, dirs, .. } => {
            if request.work_id.as_deref() != Some(work_id.as_str()) {
                return Err(Error::StoreCorrupt {
                    detail: format!("请求 {} PrepareAttempt.work_id不一致", request.request_id),
                });
            }
            for path in dirs {
                validate_path(path, true)?;
            }
        }
        EffectOp::WriteFile { path, sha256, .. } => {
            if !valid_digest(sha256) {
                return Err(Error::StoreCorrupt {
                    detail: format!("请求 {} WriteFile摘要无效", request.request_id),
                });
            }
            validate_path(path, false)?;
        }
        EffectOp::SealOutputs { refs } => {
            for reference in refs {
                if !valid_digest(&reference.sha256) {
                    return Err(Error::StoreCorrupt {
                        detail: format!("请求 {} SealOutputs摘要无效", request.request_id),
                    });
                }
                validate_path(&reference.path, false)?;
            }
        }
        EffectOp::RefreshStatusCard { work_id } => {
            let parsed =
                sheltie_core::ids::WorkId::parse(work_id).map_err(|error| Error::StoreCorrupt {
                    detail: format!(
                        "请求 {} RefreshStatusCard.work_id无效：{error}",
                        request.request_id
                    ),
                })?;
            if request.work_id.as_deref() != Some(parsed.as_str()) {
                return Err(Error::StoreCorrupt {
                    detail: format!(
                        "请求 {} RefreshStatusCard.work_id与requests不一致",
                        request.request_id
                    ),
                });
            }
        }
        EffectOp::PublishDir { .. } | EffectOp::DeleteDir { .. } => {
            return Err(Error::StoreCorrupt {
                detail: format!("请求 {} 目录效果进入非目录校验入口", request.request_id),
            });
        }
    }
    Ok(())
}

fn validate_publish_reference(
    owner: &str,
    final_path: &str,
    digest: &str,
    digest_root: &str,
) -> Result<()> {
    ManagedRelPath::new(final_path.to_string()).map_err(|error| Error::StoreCorrupt {
        detail: format!("publish final路径无效：{error}"),
    })?;
    if let Some(work) = owner.strip_prefix("work:") {
        let parsed =
            sheltie_core::ids::WorkId::parse(work).map_err(|error| Error::StoreCorrupt {
                detail: format!("publish Work owner {owner:?} 无效：{error}"),
            })?;
        if owner != format!("work:{parsed}")
            || final_path != format!("works/{parsed}")
            || digest_root != "workbook"
            || !valid_digest(digest)
        {
            return Err(Error::StoreCorrupt {
                detail: format!("publish Work {owner:?} 的final/digest闭包无效"),
            });
        }
        return Ok(());
    }
    let identity = owner
        .strip_prefix("workbook:")
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("publish owner {owner:?} 不属于Work或Workbook"),
        })?;
    let (id, version) = identity
        .split_once('@')
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("publish Workbook owner {owner:?} 缺version"),
        })?;
    let parsed_id =
        sheltie_core::ids::WorkbookId::new(id).map_err(|error| Error::StoreCorrupt {
            detail: format!("publish Workbook owner {owner:?} 的id无效：{error}"),
        })?;
    if !crate::load::valid_workbook_version(version)
        || owner != format!("workbook:{}@{version}", parsed_id.as_str())
        || final_path != format!("workbooks/{}/{version}", parsed_id.as_str())
        || !digest_root.is_empty()
        || !valid_digest(digest)
    {
        return Err(Error::StoreCorrupt {
            detail: format!("publish Workbook {owner:?} 的final/digest闭包无效"),
        });
    }
    Ok(())
}

fn validate_delete_reference(owner: &str, final_path: &str, digest: &str) -> Result<()> {
    ManagedRelPath::new(final_path.to_string()).map_err(|error| Error::StoreCorrupt {
        detail: format!("delete final路径无效：{error}"),
    })?;
    let identity = owner
        .strip_prefix("workbook:")
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("delete owner {owner:?} 不是Workbook"),
        })?;
    let (id, version) = identity
        .split_once('@')
        .ok_or_else(|| Error::StoreCorrupt {
            detail: format!("delete Workbook owner {owner:?} 缺version"),
        })?;
    let parsed_id =
        sheltie_core::ids::WorkbookId::new(id).map_err(|error| Error::StoreCorrupt {
            detail: format!("delete Workbook owner {owner:?} 的id无效：{error}"),
        })?;
    if !crate::load::valid_workbook_version(version)
        || owner != format!("workbook:{}@{version}", parsed_id.as_str())
        || final_path != format!("workbooks/{}/{version}", parsed_id.as_str())
        || !valid_digest(digest)
    {
        return Err(Error::StoreCorrupt {
            detail: format!("delete Workbook {owner:?} 的final/digest闭包无效"),
        });
    }
    Ok(())
}

pub(crate) fn read_owner(home: &Home, internal_id: &str) -> Result<PendingOwner> {
    read_owner_file(home, internal_id).map(|(owner, _)| owner)
}

pub(crate) fn read_owner_file(
    home: &Home,
    internal_id: &str,
) -> Result<(PendingOwner, crate::fsx::SafeFile)> {
    let path = ManagedRelPath::new(format!("pending/{internal_id}.owner"))?;
    let absolute = home.rel(path.as_str())?;
    let file = crate::fsx::open_managed_regular(home, &absolute).map_err(|error| match error {
        error @ Error::Io { .. } => error,
        other => Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner缺失或无效：{other}"),
        },
    })?;
    let mut bytes = file.read_bounded(4096).map_err(|error| match error {
        error @ Error::Io { .. } => error,
        other => Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner不可读：{other}"),
        },
    })?;
    if bytes.pop() != Some(b'\n') {
        return Err(Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner缺少结尾换行"),
        });
    }
    let owner: PendingOwner =
        serde_json::from_slice(&bytes).map_err(|error| Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner JSON无效：{error}"),
        })?;
    if owner.format != "pending/v1"
        || owner.internal_id != internal_id
        || !matches!(
            owner.op.as_str(),
            "start_work" | "add_workbook" | "remove_workbook"
        )
    {
        return Err(Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner字段与路径不一致"),
        });
    }
    Ok((owner, file))
}

pub(crate) fn verify_owner(
    home: &Home,
    internal_id: &str,
    request_id: &str,
    operation: &str,
) -> Result<()> {
    let owner = read_owner(home, internal_id)?;
    if owner.request_id != request_id || owner.op != operation {
        return Err(Error::StoreCorrupt {
            detail: format!("pending/{internal_id}.owner与Store请求归属不一致"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PublishLocation, read_publish_dir};
    use crate::error::Error;
    use crate::home::Home;
    use sheltie_core::path::AbsPath;

    // Task: C002-T28
    #[test]
    fn read_publish_dir_retries_when_final_renames_after_location() {
        let temporary = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(
            (AbsPath::new(temporary.path().to_string_lossy().into_owned()).unwrap()).as_str(),
        ))
        .unwrap();
        let internal_id = "0198f01a7f0070008000000000000004";
        let pending = format!("pending/{internal_id}/payload");
        let final_path = "works/2026-09-29-001-test";
        std::fs::create_dir_all(home.pending_dir().join_segment(internal_id).as_path()).unwrap();
        std::fs::create_dir_all(home.rel(final_path).unwrap().as_path()).unwrap();
        std::fs::write(
            home.rel(final_path)
                .unwrap()
                .join_segment("marker")
                .as_path(),
            b"same-tree-bytes",
        )
        .unwrap();
        let location = PublishLocation {
            pending: pending.clone(),
            final_path: final_path.to_string(),
            pending_publish: true,
        };
        let mut attempts = 0;

        let (bytes, pending_publish) = read_publish_dir(&home, &location, |directory| {
            attempts += 1;
            if attempts == 1 {
                std::fs::rename(directory.as_path(), home.rel(&pending)?.as_path())
                    .map_err(|error| Error::io(final_path, error))?;
                return Err(Error::NotFound {
                    what: final_path.to_string(),
                });
            }
            std::fs::read(directory.as_path().join("marker"))
                .map_err(|error| Error::io(directory.as_str(), error))
        })
        .unwrap();

        assert_eq!(bytes, b"same-tree-bytes");
        assert!(pending_publish);
        assert_eq!(attempts, 2);
        assert!(!home.lock_path().as_path().exists(), "只读定位不创建引擎锁");
        assert!(!home.store_path().as_path().exists(), "只读定位不创建Store");
    }
}

#[cfg(all(test, feature = "failpoint"))]
mod locator_contract_tests {
    use super::*;
    use crate::fsx::controlled_object_tests::observe_change;
    use std::os::unix::fs::MetadataExt as _;

    fn fixture() -> (tempfile::TempDir, Home) {
        let directory = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(directory.path().to_str().unwrap())).unwrap();
        std::fs::create_dir(directory.path().join("pending")).unwrap();
        std::fs::create_dir(directory.path().join("works")).unwrap();
        (directory, home)
    }

    // Task: C002-T55
    #[test]
    fn only_one_existing_directory_qualifies_as_the_selected_candidate() {
        for (candidate, other, answer) in [
            (false, false, false),
            (false, true, false),
            (true, false, true),
            (true, true, false),
        ] {
            assert_eq!(sole_directory_exists(candidate, other), answer);
        }
    }

    // Task: C002-T55
    #[test]
    fn readonly_locator_accepts_only_the_single_qualified_registered_tree() {
        for (pending_exists, final_exists, pending_publish, answer) in [
            (true, false, true, Some("pending/source")),
            (false, true, true, Some("works/final")),
            (false, true, false, Some("works/final")),
            (true, true, true, None),
            (false, false, true, None),
        ] {
            let (_directory, home) = fixture();
            for (exists, name) in [
                (pending_exists, "pending/source"),
                (final_exists, "works/final"),
            ] {
                if exists {
                    std::fs::create_dir(home.rel(name).unwrap().as_path()).unwrap();
                    std::fs::write(
                        home.rel(name).unwrap().as_path().join("original"),
                        name.as_bytes(),
                    )
                    .unwrap();
                }
            }
            let result =
                locate_publish_dir(&home, "pending/source", "works/final", pending_publish);
            if let Some(expected) = answer {
                let (path, flag) = result.unwrap();
                assert_eq!(path, home.rel(expected).unwrap());
                assert_eq!(flag, pending_publish);
            } else {
                let Error::Io { path, source } = result.unwrap_err() else {
                    panic!("unqualified endpoints remain temporary IO, not false absence")
                };
                assert_eq!(path, "works/final");
                assert_eq!(source.kind(), std::io::ErrorKind::WouldBlock);
            }
            assert!(!home.lock_path().as_path().exists());
            assert!(!home.store_path().as_path().exists());
            for (exists, name) in [
                (pending_exists, "pending/source"),
                (final_exists, "works/final"),
            ] {
                if exists {
                    assert_eq!(
                        std::fs::read(home.rel(name).unwrap().as_path().join("original")).unwrap(),
                        name.as_bytes()
                    );
                }
            }
        }
    }

    // Task: C002-T55
    #[test]
    fn a_competing_endpoint_created_after_the_first_read_invalidates_both_pending_and_final_selection()
     {
        for initially_pending in [true, false] {
            let (_directory, home) = fixture();
            let selected = if initially_pending {
                "pending/source"
            } else {
                "works/final"
            };
            let competitor = if initially_pending {
                "works/final"
            } else {
                "pending/source"
            };
            let source = home.rel(selected).unwrap();
            let other = home.rel(competitor).unwrap();
            std::fs::create_dir(source.as_path()).unwrap();
            std::fs::write(source.as_path().join("original"), b"registered original").unwrap();
            let before = std::fs::metadata(source.as_path()).unwrap();
            let worker_home = home.clone();
            let result = observe_change(
                "locator_after_initial_observation",
                "works/final",
                move || locate_publish_dir(&worker_home, "pending/source", "works/final", true),
                || {
                    std::fs::create_dir(other.as_path()).unwrap();
                    std::fs::write(other.as_path().join("competitor"), b"competitor unchanged")
                        .unwrap();
                },
            );
            let Error::Io { source: error, .. } = result.unwrap_err() else {
                panic!("after-read double presence must not select either directory")
            };
            assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
            assert_eq!(
                std::fs::read(source.as_path().join("original")).unwrap(),
                b"registered original"
            );
            assert_eq!(
                std::fs::read(other.as_path().join("competitor")).unwrap(),
                b"competitor unchanged"
            );
            let after = std::fs::metadata(source.as_path()).unwrap();
            assert_eq!(
                (after.dev(), after.ino(), after.mode(), after.nlink()),
                (before.dev(), before.ino(), before.mode(), before.nlink())
            );
            assert!(!home.lock_path().as_path().exists());
            assert!(!home.store_path().as_path().exists());
        }
    }

    // Task: C002-T55
    #[test]
    fn a_locator_observation_failure_preserves_the_registered_tree_without_opening_a_store() {
        let _serial = crate::failpoint::RENDEZVOUS_TEST_LOCK.lock().unwrap();
        let (directory, home) = fixture();
        let source = home.rel("pending/source").unwrap();
        std::fs::create_dir(source.as_path()).unwrap();
        std::fs::write(source.as_path().join("original"), b"preserved bytes").unwrap();
        let carrier = directory.path().join("not-a-sync-directory");
        std::fs::write(&carrier, b"preserved carrier").unwrap();
        crate::failpoint::arm_rendezvous(
            "locator_after_initial_observation",
            "works/final",
            &carrier,
        )
        .unwrap();
        let result = locate_publish_dir(&home, "pending/source", "works/final", true);
        crate::failpoint::disarm_rendezvous().unwrap();
        let Error::Io { path, .. } = result.unwrap_err() else {
            panic!("failed locator observation must not select bytes")
        };
        assert_eq!(path, "works/final");
        assert_eq!(
            std::fs::read(source.as_path().join("original")).unwrap(),
            b"preserved bytes"
        );
        assert_eq!(std::fs::read(&carrier).unwrap(), b"preserved carrier");
        assert!(!home.lock_path().as_path().exists());
        assert!(!home.store_path().as_path().exists());
    }
}

#[cfg(all(test, feature = "failpoint"))]
mod pending_qualification_contract_tests {
    use super::*;
    use crate::fsx::owned_test_directory::OwnedTempDir;
    use crate::store::{OpenMode, Store};
    use std::os::unix::fs::MetadataExt as _;

    fn fixture() -> (OwnedTempDir, Home) {
        let directory = OwnedTempDir::new();
        let home = Home::resolve(Some(directory.path().join("home").to_str().unwrap())).unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/two-step")
            .canonicalize()
            .unwrap();
        crate::WorkbookRepo::new(home.clone())
            .add(
                &sheltie_core::path::AbsPath::new(source.to_str().unwrap()).unwrap(),
                Some("qualified-add".into()),
            )
            .unwrap();
        (directory, home)
    }

    fn original_tree(
        root: &std::path::Path,
    ) -> BTreeMap<std::path::PathBuf, (u64, u64, u32, u64, Vec<u8>)> {
        fn visit(
            base: &std::path::Path,
            path: &std::path::Path,
            result: &mut BTreeMap<std::path::PathBuf, (u64, u64, u32, u64, Vec<u8>)>,
        ) {
            let meta = std::fs::symlink_metadata(path).unwrap();
            let bytes = if meta.is_file() {
                std::fs::read(path).unwrap()
            } else if meta.file_type().is_symlink() {
                std::fs::read_link(path)
                    .unwrap()
                    .as_os_str()
                    .as_encoded_bytes()
                    .to_vec()
            } else {
                Vec::new()
            };
            result.insert(
                path.strip_prefix(base).unwrap().to_owned(),
                (meta.dev(), meta.ino(), meta.mode(), meta.nlink(), bytes),
            );
            if meta.is_dir() {
                for child in std::fs::read_dir(path).unwrap() {
                    visit(base, &child.unwrap().path(), result);
                }
            }
        }
        let mut result = BTreeMap::new();
        visit(root, root, &mut result);
        result
    }

    fn persistent_rows(connection: &rusqlite::Connection) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
        ["workbooks", "works", "work_sequence", "requests", "audit"]
            .into_iter()
            .map(|table| {
                let mut query = connection
                    .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
                    .unwrap();
                let columns = query.column_count();
                query
                    .query_map([], |row| {
                        (0..columns).map(|column| row.get(column)).collect()
                    })
                    .unwrap()
                    .map(|row| row.unwrap())
                    .collect()
            })
            .collect()
    }

    // Task: C002-T56
    #[test]
    fn malformed_global_pending_paths_stop_cleanup_before_deleting_a_legitimate_uncommitted_original()
     {
        for shape in ["prefix", "leaf", "uuid", "extra", "short"] {
            let (_directory, home) = fixture();
            let lock = home.acquire_lock().unwrap();
            let orphan = "0198f01a7f00700080000000000000bb";
            crate::service::stage_pending(&home, &lock, orphan, "uncommitted-orphan", "start_work")
                .unwrap();
            let original = home
                .rel(&format!("pending/{orphan}/payload/original"))
                .unwrap();
            std::fs::write(original.as_path(), b"legitimate orphan original").unwrap();
            let before = std::fs::metadata(original.as_path()).unwrap();
            let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
            let raw: String = connection
                .query_row(
                    "SELECT effects_json FROM requests WHERE request_id='qualified-add'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            let mut effects: serde_json::Value = serde_json::from_str(&raw).unwrap();
            let publish = effects
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|effect| effect["kind"] == "publish_dir")
                .unwrap();
            let id = publish["pending"]
                .as_str()
                .unwrap()
                .split('/')
                .nth(1)
                .unwrap()
                .to_owned();
            publish["pending"] = serde_json::json!(match shape {
                "prefix" => format!("other/{id}/payload"),
                "leaf" => format!("pending/{id}/other"),
                "uuid" => "pending/not-a-uuid/payload".into(),
                "extra" => format!("pending/{id}/payload/extra"),
                _ => "pending".into(),
            });
            connection
                .execute(
                    "UPDATE requests SET effects_json=?1 WHERE request_id='qualified-add'",
                    [effects.to_string()],
                )
                .unwrap();
            let store = Store::open_for_home(&home, OpenMode::ReadWrite).unwrap();
            let error = cleanup(&home, &store, &lock).unwrap_err();
            assert!(
                matches!(error, Error::StoreCorrupt { .. }),
                "{shape}: {error:?}"
            );
            assert_eq!(
                std::fs::read(original.as_path()).unwrap(),
                b"legitimate orphan original"
            );
            let after = std::fs::metadata(original.as_path()).unwrap();
            assert_eq!(
                (after.dev(), after.ino(), after.mode(), after.nlink()),
                (before.dev(), before.ino(), before.mode(), before.nlink())
            );
            let persisted: String = connection
                .query_row(
                    "SELECT effects_json FROM requests WHERE request_id='qualified-add'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(persisted, effects.to_string());
        }
    }

    // Task: C002-T56
    #[test]
    fn a_coupled_non_uuid_owner_and_container_is_unknown_instead_of_cleanup_authority() {
        let (_directory, home) = fixture();
        let lock = home.acquire_lock().unwrap();
        let valid = "0198f01a7f00700080000000000000bb";
        crate::service::stage_pending(&home, &lock, valid, "uncommitted-owner", "start_work")
            .unwrap();
        let source = home.rel(&format!("pending/{valid}")).unwrap();
        let source_owner = home.rel(&format!("pending/{valid}.owner")).unwrap();
        let content = std::fs::read(source_owner.as_path()).unwrap();
        let mut owner: serde_json::Value = serde_json::from_slice(&content).unwrap();
        owner["internal_id"] = serde_json::json!("not-a-uuid");
        let invalid = home.rel("pending/not-a-uuid").unwrap();
        let invalid_owner = home.rel("pending/not-a-uuid.owner").unwrap();
        std::fs::rename(source.as_path(), invalid.as_path()).unwrap();
        std::fs::rename(source_owner.as_path(), invalid_owner.as_path()).unwrap();
        let mut new_content = serde_json::to_vec(&owner).unwrap();
        new_content.push(b'\n');
        std::fs::write(invalid_owner.as_path(), &new_content).unwrap();
        let original = invalid.join_segment("payload").join_segment("original");
        std::fs::write(original.as_path(), b"unknown original must stay").unwrap();
        let before = std::fs::metadata(original.as_path()).unwrap();
        let store = Store::open_for_home(&home, OpenMode::ReadWrite).unwrap();
        let warnings = cleanup(&home, &store, &lock).unwrap();
        assert!(
            warnings
                .iter()
                .any(|warning| warning.contains("not-a-uuid"))
        );
        assert_eq!(
            std::fs::read(original.as_path()).unwrap(),
            b"unknown original must stay"
        );
        assert_eq!(std::fs::read(invalid_owner.as_path()).unwrap(), new_content);
        let after = std::fs::metadata(original.as_path()).unwrap();
        assert_eq!(
            (after.dev(), after.ino(), after.mode(), after.nlink()),
            (before.dev(), before.ino(), before.mode(), before.nlink())
        );
    }

    // Task: C002-T56
    #[test]
    fn cleanup_without_a_pending_root_warns_only_for_unpublished_registered_requests() {
        let _serial = crate::failpoint::RENDEZVOUS_TEST_LOCK.lock().unwrap();
        let (directory, home) = fixture();
        crate::failpoint::arm_sync_error(home.root().as_str(), "publish_source_parent_sync")
            .unwrap();
        let service = crate::WorkService::new(home.clone());
        let error = service
            .start(
                crate::StartArgs {
                    workbook_id: "two-step".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: BTreeMap::from([(
                        "topic".into(),
                        crate::request::InputValue::Literal {
                            text: "unfinished source".into(),
                        },
                    )]),
                },
                Some("unpublished-start".into()),
            )
            .unwrap_err();
        crate::failpoint::disarm_sync_error().unwrap();
        assert!(matches!(
            error,
            Error::EffectPending {
                committed: true,
                ..
            }
        ));
        let saved = directory.path().join("saved-pending");
        std::fs::rename(home.pending_dir().as_path(), &saved).unwrap();
        let before_saved = original_tree(&saved);
        let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
        let before_rows = persistent_rows(&connection);
        let lock = home.acquire_lock().unwrap();
        let store = Store::open_for_home(&home, OpenMode::ReadWrite).unwrap();
        let warnings = cleanup(&home, &store, &lock).unwrap();
        assert!(!warnings.is_empty());
        assert!(
            warnings
                .iter()
                .all(|warning| warning.contains("unpublished-start")
                    && !warning.contains("qualified-add")),
            "{warnings:?}"
        );
        assert_eq!(original_tree(&saved), before_saved);
        assert_eq!(persistent_rows(&connection), before_rows);
        assert!(saved.is_dir());
        assert!(!home.pending_dir().as_path().exists());
    }

    // Task: C002-T56
    #[test]
    fn a_cleaned_completed_delete_does_not_warn_for_a_marker_that_no_longer_exists() {
        let _serial = crate::failpoint::RENDEZVOUS_TEST_LOCK.lock().unwrap();
        let (_directory, home) = fixture();
        crate::WorkbookRepo::new(home.clone())
            .remove("two-step", "1.0.0", Some("completed-delete".into()))
            .unwrap();
        let lock = home.acquire_lock().unwrap();
        let store = Store::open_for_home(&home, OpenMode::ReadWrite).unwrap();
        let raw: String = store
            .connect()
            .unwrap()
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id='completed-delete'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let effects: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let delete = effects
            .as_array()
            .unwrap()
            .iter()
            .find(|effect| effect["kind"] == "delete_dir")
            .unwrap();
        let id = delete["pending"]
            .as_str()
            .unwrap()
            .split('/')
            .nth(1)
            .unwrap();
        cleanup(&home, &store, &lock).unwrap();
        assert!(
            !home
                .pending_dir()
                .join_segment(&format!("{id}.deleted"))
                .as_path()
                .exists()
        );
        let warnings = cleanup(&home, &store, &lock).unwrap();
        assert!(
            !warnings
                .iter()
                .any(|warning| warning.contains("completed-delete") && warning.contains("marker")),
            "{warnings:?}"
        );
    }
}
