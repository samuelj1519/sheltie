//! Workbook 仓库：`add / list / load / remove / verify`。规则见 `specs/contracts/storage.md` §5 与协议 §3。

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

/// 单文件上限 32 MiB。
pub const MAX_FILE_BYTES: u64 = 33_554_432;
/// 目录总量上限 256 MiB。
pub const MAX_TOTAL_BYTES: u64 = 268_435_456;

/// `workbook add` 的提交时快照（GF-15）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddedSnapshot {
    pub request_id: String,
    pub replayed: bool,
    pub data: serde_json::Value,
}

/// `workbook remove` 的提交时快照。
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
            detail: format!("requests 表里的响应解不开：{e}"),
        })?;
    if s.request_id != request_id || s.replayed {
        return Err(Error::StoreCorrupt {
            detail: format!("Workbook add请求 {request_id} 的历史snapshot身份无效"),
        });
    }
    s.request_id = request_id.to_string();
    s.replayed = true;
    Ok(s)
}

fn decode_removed_snapshot(request_id: &str, reply_json: &str) -> Result<RemovedSnapshot> {
    let mut s: RemovedSnapshot =
        serde_json::from_str(reply_json).map_err(|e| Error::StoreCorrupt {
            detail: format!("requests 表里的响应解不开：{e}"),
        })?;
    if s.request_id != request_id || s.replayed {
        return Err(Error::StoreCorrupt {
            detail: format!("Workbook remove请求 {request_id} 的历史snapshot身份无效"),
        });
    }
    s.request_id = request_id.to_string();
    s.replayed = true;
    Ok(s)
}

/// `workbook add` 的返回。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Added {
    pub id: String,
    pub version: String,
    pub digest: Sha256Hex,
    pub flows: Vec<String>,
    pub requires: Vec<String>,
}

/// 装好的 Workbook：manifest、每张图、目录。
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

/// `workbook list` 的登记身份及只读发布状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbookListItem {
    pub id: String,
    pub version: String,
    pub digest: String,
    pub pending_publish: bool,
}

/// 仓库句柄。
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

    /// `workbook add <dir>`（存储合同 §5.2）。走与 Work 相同的写路径：
    /// 无锁预检查重 → 管理根写锁 → 恢复 → pending staging → 只对最终副本
    /// parse/compile/digest → 一个事务（requests + workbooks + audit）→ 发布。
    /// 相同 request-id 的重放返回原快照；源目录变化不重新安装（§2.1）。
    pub fn add(&self, dir: &AbsPath, request_id: Option<String>) -> Result<AddedSnapshot> {
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::AddWorkbook {
            source: lexical_abs(dir.as_str()),
        };
        let command_json = match &intent {
            RequestIntent::AddWorkbook { .. } => {
                serde_json::json!({"intent": "add_workbook", "source": intent.hash().as_str()})
                    .to_string()
            }
            _ => unreachable!("AddWorkbook方法必构造AddWorkbook intent"),
        };
        // 无锁预检只决定是否需要读取源目录；实际重放必须先拿写锁并恢复旧效果。
        let replay_exists = match Store::open_for_home(&self.home, crate::store::OpenMode::ReadOnly)
        {
            Ok(ro) => {
                if let Some(hash) = ro.lookup_request_hash(&request_id)? {
                    if hash != intent.hash().as_str() {
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
        repo.recover_before_write(&lock, &request_id, intent.hash().as_str())?;
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if row.intent_hash != intent.hash().as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            // 恢复步已补完效果；这里统一按快照返回。
            return decode_added_snapshot(&request_id, &row.reply_json);
        }

        // 源目录粗检在锁前完成；只有合法新添加才创建管理锁和Store。
        let internal_id = uuid::Uuid::now_v7().simple().to_string();
        let payload = stage_pending(&self.home, &lock, &internal_id, &request_id, "add_workbook")?;

        // 只对最终副本 parse/compile/digest（§5.2 第 3 步）。
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
            intent_hash: intent.hash().as_str().to_string(),
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
                        detail: format!("重放请求 {request_id} 在requests中消失"),
                    })?;
            decode_added_snapshot(&request_id, &row.reply_json)
        } else {
            Ok(snapshot)
        }
    }

    /// 锁内恢复未完成的 Workbook 效果（先于新命令，存储合同 §3.2）。
    pub(crate) fn checked_effects_for(
        &self,
        request_id: &str,
    ) -> Result<(
        crate::store::read::RequestRow,
        crate::effects::CheckedEffects,
    )> {
        let mut row =
            self.store
                .inspect_request(request_id)?
                .ok_or_else(|| Error::StoreCorrupt {
                    detail: format!("缺少Workbook请求 {request_id} 的requests行"),
                })?;
        if row.work_id.is_some() {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 不应归属Work"),
            });
        }
        let audits = self.store.audit_rows(request_id)?;
        let [audit] = audits.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 应有且仅有一条audit"),
            });
        };
        if !audit.work_id.is_empty() || audit.revision != 0 {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 的audit包含Work归属"),
            });
        }
        if row.at != audit.at {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 的audit时间与requests行不一致"),
            });
        }
        Sha256Hex::new(row.intent_hash.clone()).map_err(|error| Error::StoreCorrupt {
            detail: format!("Workbook请求 {request_id} 的intent_hash不合法：{error}"),
        })?;
        let command: WorkbookAuditCommand =
            serde_json::from_str(&audit.command_json).map_err(|error| Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 的audit命令不符合合同：{error}"),
            })?;
        let (identity, registered) = match command {
            WorkbookAuditCommand::AddWorkbook { source } => {
                if Sha256Hex::new(source.clone()).is_err() || row.intent_hash != source {
                    return Err(Error::StoreCorrupt {
                        detail: format!("Workbook add请求 {request_id} 的intent_hash与audit不一致"),
                    });
                }
                let snapshot: AddedSnapshot =
                    serde_json::from_str(&row.reply_json).map_err(|error| Error::StoreCorrupt {
                        detail: format!("Workbook add请求 {request_id} snapshot解不开：{error}"),
                    })?;
                if snapshot.request_id != request_id || snapshot.replayed {
                    return Err(Error::StoreCorrupt {
                        detail: format!("Workbook add请求 {request_id} snapshot身份无效"),
                    });
                }
                let data: crate::recovery::AddedSnapshotData =
                    serde_json::from_value(snapshot.data).map_err(|error| Error::StoreCorrupt {
                        detail: format!(
                            "Workbook add请求 {request_id} snapshot.data不符合合同：{error}"
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
                        detail: format!("Workbook add请求 {request_id} snapshot.data字段无效"),
                    });
                }
                let registered = if row.published {
                    None
                } else {
                    let registered = self
                        .store
                        .workbook_versions(&data.id)?
                        .into_iter()
                        .find(|candidate| candidate.version == data.version)
                        .ok_or_else(|| Error::StoreCorrupt {
                            detail: format!("Workbook add请求 {request_id} 缺少登记行"),
                        })?;
                    crate::load::validate_workbook_row(&self.home, &registered)?;
                    Some(registered)
                };
                (
                    crate::effects::WorkbookEffectIdentity::Add {
                        id: data.id,
                        version: data.version,
                        digest: data.digest,
                    },
                    registered,
                )
            }
            WorkbookAuditCommand::RemoveWorkbook { target } => {
                let (id, version) = target.split_once('@').ok_or_else(|| Error::StoreCorrupt {
                    detail: format!("Workbook remove请求 {request_id} 的audit target无效"),
                })?;
                if target != format!("{id}@{version}") {
                    return Err(Error::StoreCorrupt {
                        detail: format!("Workbook remove请求 {request_id} 的audit target不规范"),
                    });
                }
                let request_intent = RequestIntent::RemoveWorkbook {
                    id: id.to_string(),
                    version: version.to_string(),
                };
                if row.intent_hash != request_intent.hash().as_str() {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook remove请求 {request_id} 的intent_hash与audit不一致"
                        ),
                    });
                }
                let snapshot: RemovedSnapshot =
                    serde_json::from_str(&row.reply_json).map_err(|error| Error::StoreCorrupt {
                        detail: format!("Workbook remove请求 {request_id} snapshot解不开：{error}"),
                    })?;
                if snapshot.request_id != request_id || snapshot.replayed {
                    return Err(Error::StoreCorrupt {
                        detail: format!("Workbook remove请求 {request_id} snapshot身份无效"),
                    });
                }
                let data: crate::recovery::RemovedSnapshotData =
                    serde_json::from_value(snapshot.data).map_err(|error| Error::StoreCorrupt {
                        detail: format!(
                            "Workbook remove请求 {request_id} snapshot.data不符合合同：{error}"
                        ),
                    })?;
                if data.id != id || data.version != version {
                    return Err(Error::StoreCorrupt {
                        detail: format!(
                            "Workbook remove请求 {request_id} snapshot与audit身份不一致"
                        ),
                    });
                }
                (
                    crate::effects::WorkbookEffectIdentity::Remove {
                        id: id.to_string(),
                        version: version.to_string(),
                    },
                    None,
                )
            }
        };
        let effects = decode_effects(&row.effects_json)?;
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
                    detail: format!("Workbook请求 {request_id} 的Checked效果形状无效"),
                });
            }
        };
        let internal_id = pending
            .split('/')
            .nth(1)
            .ok_or_else(|| Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 的pending路径无效"),
            })?;
        if !row.published {
            if let Err(owner_error) =
                crate::service::verify_pending_owner(&self.home, internal_id, request_id, operation)
            {
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
    }

    fn recovery_load_effects(
        &self,
        request_id: &str,
        expected: &crate::store::read::RequestRow,
    ) -> Result<crate::effects::CheckedEffects> {
        let (row, effects) = if expected.work_id.is_some() {
            let service =
                crate::service::WorkService::with_store(self.home.clone(), self.store.clone());
            let (row, _, effects) = service.load_checked_request(request_id)?;
            (row, effects)
        } else {
            self.checked_effects_for(request_id)?
        };
        if row.reply_json != expected.reply_json {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 的响应在校验期间改变"),
            });
        }
        Ok(effects)
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
                    detail: format!("audit {} 命令JSON无效：{error}", audit.request_id),
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
                    detail: format!("Workbook audit {} 命令解不开：{error}", audit.request_id),
                })?;
            let (matches, is_add) = match &command {
                WorkbookAuditCommand::AddWorkbook { .. } => {
                    let request =
                        self.store
                            .inspect_request(&audit.request_id)?
                            .ok_or_else(|| Error::StoreCorrupt {
                                detail: format!(
                                    "Workbook audit {} 缺少requests行",
                                    audit.request_id
                                ),
                            })?;
                    let snapshot: AddedSnapshot = serde_json::from_str(&request.reply_json)
                        .map_err(|error| Error::StoreCorrupt {
                            detail: format!(
                                "Workbook add请求 {} snapshot解不开：{error}",
                                audit.request_id
                            ),
                        })?;
                    let data: crate::recovery::AddedSnapshotData =
                        serde_json::from_value(snapshot.data).map_err(|error| {
                            Error::StoreCorrupt {
                                detail: format!(
                                    "Workbook add请求 {} snapshot身份无效：{error}",
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
                    self.checked_effects_for(&request_id)?;
                    format!("Workbook {owner} 的最新生命周期是remove，但workbooks行仍存在")
                }
                _ => format!("Workbook {owner} 缺少对应成功add请求"),
            };
            return Err(Error::StoreCorrupt { detail });
        };

        let (request, checked) = self.checked_effects_for(&request_id)?;
        let audits = self.store.audit_rows(&request_id)?;
        let [audit] = audits.as_slice() else {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook add请求 {request_id} 应有且仅有一条audit"),
            });
        };
        if request.work_id.is_some()
            || !audit.work_id.is_empty()
            || audit.revision != 0
            || audit.at != row.added_at
            || request.at != row.added_at
        {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook {owner} 的workbooks.added_at与当前add审计不一致"),
            });
        }
        let snapshot: AddedSnapshot =
            serde_json::from_str(&request.reply_json).map_err(|error| Error::StoreCorrupt {
                detail: format!("Workbook add请求 {request_id} snapshot解不开：{error}"),
            })?;
        if snapshot.request_id != request_id || snapshot.replayed {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook add请求 {request_id} snapshot身份无效"),
            });
        }
        let data: crate::recovery::AddedSnapshotData = serde_json::from_value(snapshot.data)
            .map_err(|error| Error::StoreCorrupt {
                detail: format!("Workbook add请求 {request_id} snapshot.data解不开：{error}"),
            })?;
        if data.id != row.id || data.version != row.version || data.digest != row.digest {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook {owner} 与当前add响应身份/摘要不一致"),
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
                detail: format!("Workbook add请求 {request_id} 的发布效果形状无效"),
            });
        };
        if effect_owner != &owner
            || final_path != &row.dir
            || digest != &row.digest
            || !digest_root.is_empty()
        {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook add请求 {request_id} 的effect与当前workbooks行不一致"),
            });
        }
        let internal_id = crate::effects::pending_internal_id(pending)?;
        let Some(reference) = references.get(internal_id) else {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook add请求 {request_id} 缺少pending引用索引项"),
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
                detail: format!("Workbook add请求 {request_id} 与pending引用索引不一致"),
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
                    "Workbook {}@{} 的manifest身份与Store行不一致",
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

    /// 从已装目录（或任意目录，用于冻结副本）重新解析并编译。
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
        for path in manifest.flows() {
            let bytes = tree.read_file(path, FS_MAX_FILE_BYTES)?;
            let def = parse_flow(&read_tree_utf8(&bytes, path.as_str())?)?;
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
                        detail: format!("ResourceIndex 缺少已摘要文件 {relative}"),
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
                    detail: format!("冻结副本缺少说明书 {path}"),
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

    /// 按 `id` 与版本加载；`version` 为 `None` 取字面最高版本。不存在报 `NotFound`。
    /// 加载即核对登记身份：重算目录摘要与 `workbooks.digest` 比较，不符报
    /// `WORKBOOK_TAMPERED`（GF-32；O03：verify 发现的篡改不能被 start 静默接受）。
    pub fn load(&self, id: &str, version: Option<&str>) -> Result<LoadedWorkbook> {
        sheltie_core::ids::WorkbookId::new(id).map_err(Error::Core)?;
        if let Some(version) = version {
            if !crate::load::valid_workbook_version(version) {
                return Err(Error::InvalidRequest {
                    reason: format!("Workbook version {version:?} 不符合合同"),
                });
            }
            ManagedRelPath::new(format!("workbooks/{id}/{version}")).map_err(|error| {
                Error::InvalidRequest {
                    reason: format!("Workbook身份不能用于受管路径：{error}"),
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
            // workbook_versions 按版本字面升序，最后一个就是最高版本。
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

    /// 目录摘要 `workbook-digest/v2`（存储合同 §5.1）。schema 2 的唯一摘要口径，
    /// add/load/verify/冻结副本核验与发布效果全部用它。
    pub fn digest_dir(dir: &AbsPath) -> Result<Sha256Hex> {
        crate::workbook_digest::digest_dir_v2(dir)
    }

    /// `workbook remove <id>@<version>`（协议细则；引用检查的事务化归 T08）。
    /// 同样走写锁与请求表：相同 request-id 重放返回原快照，不重复删除。
    pub fn remove(
        &self,
        id: &str,
        version: &str,
        request_id: Option<String>,
    ) -> Result<RemovedSnapshot> {
        if version.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "remove 必须给全版本，不接受「最高版本」默认".to_string(),
            });
        }
        sheltie_core::ids::WorkbookId::new(id).map_err(Error::Core)?;
        if !crate::load::valid_workbook_version(version) {
            return Err(Error::InvalidRequest {
                reason: format!("Workbook version {version:?} 不符合合同"),
            });
        }
        ManagedRelPath::new(format!("workbooks/{id}/{version}")).map_err(|error| {
            Error::InvalidRequest {
                reason: format!("Workbook身份不能用于受管路径：{error}"),
            }
        })?;
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::RemoveWorkbook {
            id: id.to_string(),
            version: version.to_string(),
        };
        let ro = Store::open_for_home(&self.home, crate::store::OpenMode::ReadOnly).map_err(
            |error| match error {
                Error::NotFound { .. } => Error::NotFound {
                    what: format!("Workbook {id}@{version}"),
                },
                other => other,
            },
        )?;
        let replay_exists = if let Some(hash) = ro.lookup_request_hash(&request_id)? {
            if hash != intent.hash().as_str() {
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
        repo.recover_before_write(&lock, &request_id, intent.hash().as_str())?;
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if row.intent_hash != intent.hash().as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return decode_removed_snapshot(&request_id, &row.reply_json);
        }
        if replay_exists {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook请求 {request_id} 重放预检与写锁内记录不一致"),
            });
        }

        // 清理前核归属（T05）：目录当前摘要必须仍与登记值相符。
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
                    detail: format!("Workbook目录 {dir} 存在但没有对应登记行，拒绝删除"),
                });
            }
            return Err(Error::NotFound {
                what: format!("Workbook {id}@{version}"),
            });
        };
        let registered_dir = crate::load::validate_workbook_row(&self.home, &registered)?;
        if registered_dir != dir {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook登记路径 {registered_dir} 与请求路径 {dir} 不一致"),
            });
        }
        let registered_digest = registered.digest.clone();
        sheltie_core::digest::Sha256Hex::new(registered_digest.clone()).map_err(|error| {
            Error::StoreCorrupt {
                detail: format!("Workbook {id}@{version} 登记摘要无效：{error}"),
            }
        })?;
        let dir_rel = self.home.to_rel(&dir)?;
        if !crate::fsx::managed_directory_exists(&self.home, &lock, &dir_rel)? {
            return Err(Error::StoreCorrupt {
                detail: format!("Workbook {id}@{version} 有登记行但最终目录缺失，拒绝删除"),
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
        // 引用检查与删行在同一个事务（存储合同 §5.2）；损坏引用行在事务内停止。
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
            intent_hash: intent.hash().as_str().to_string(),
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
                        detail: format!("重放请求 {request_id} 在requests中消失"),
                    })?;
            decode_removed_snapshot(&request_id, &row.reply_json)
        } else {
            Ok(snapshot)
        }
    }

    /// `workbook verify`。`filter` 为 `Some((id, version))` 只核对一个。
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

    /// 清理写操作留下的pending元数据；失败只返回维护诊断，不回滚已提交业务结果。
    pub fn cleanup_pending(&self) -> Result<Vec<String>> {
        let session = crate::session::WriteSession::open_existing(&self.home)?;
        crate::pending::cleanup(&self.home, &session.store, &session.lock)
    }

    /// 受限复制：拒绝软链、硬链、非普通文件、单文件超 32 MiB、总量超 256 MiB。
    /// 逐文件句柄复制、独占创建目标并 fsync（`fsx`）。
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
    fn load_effects(
        &self,
        request_id: &str,
        row: &crate::store::read::RequestRow,
    ) -> Result<crate::effects::CheckedEffects> {
        self.recovery_load_effects(request_id, row)
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

/// 读一个必须存在的 UTF-8 文本文件：句柄核对身份并限额读取，失败按 `WORKBOOK_INVALID` 报。
fn read_tree_utf8(bytes: &[u8], path: &str) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        Error::Core(sheltie_core::Error::WorkbookInvalid {
            field: path.to_string(),
            reason: "不是 UTF-8".to_string(),
        })
    })
}
