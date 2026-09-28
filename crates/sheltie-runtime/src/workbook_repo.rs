//! Workbook 仓库：`add / list / load / remove / verify`。规则见 `specs/contracts/storage.md` §5 与协议 §3。

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{FlowDef, Graph, compile, parse_flow};
use sheltie_core::path::{AbsPath, RelPath};
use sheltie_core::workbook::Manifest;
use sheltie_core::workbook::parse_manifest;

use crate::effects::{EffectOp, decode_effects, encode_effects, execute};
use crate::error::{Error, Result};
use crate::home::Home;
use crate::observe::build_resource_index;
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

fn decode_added_snapshot(request_id: &str, reply_json: &str) -> Result<AddedSnapshot> {
    let mut s: AddedSnapshot =
        serde_json::from_str(reply_json).map_err(|e| Error::StoreCorrupt {
            detail: format!("requests 表里的响应解不开：{e}"),
        })?;
    s.request_id = request_id.to_string();
    s.replayed = true;
    Ok(s)
}

fn decode_removed_snapshot(request_id: &str, reply_json: &str) -> Result<RemovedSnapshot> {
    let mut s: RemovedSnapshot =
        serde_json::from_str(reply_json).map_err(|e| Error::StoreCorrupt {
            detail: format!("requests 表里的响应解不开：{e}"),
        })?;
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
}

/// 仓库句柄。
#[derive(Debug, Clone)]
pub struct WorkbookRepo {
    home: Home,
    store: Store,
}

impl WorkbookRepo {
    pub fn new(home: Home, store: Store) -> Self {
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
        // 无锁预检：命中已提交请求直接按快照重放（发布完成状态下不再读源目录）。
        if let Ok(ro) = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadOnly) {
            if let Some((hash, reply_json)) = ro.lookup_request(&request_id)? {
                if hash != intent.hash().as_str() {
                    return Err(Error::RequestConflict {
                        request_id: request_id.clone(),
                    });
                }
                return decode_added_snapshot(&request_id, &reply_json);
            }
        }

        let _lock = self.home.acquire_lock()?;
        let store = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadWrite)?;
        let repo = Self::new(self.home.clone(), store);
        repo.recover_workbook_effects()?;
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if row.intent_hash != intent.hash().as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            // 恢复步已补完效果；这里统一按快照返回。
            return decode_added_snapshot(&request_id, &row.reply_json);
        }

        // 源目录粗检（§5.2 第 1 步）：可读、拒绝软链与宿主元数据。
        let _ = build_resource_index(dir)?;
        let internal_id = uuid::Uuid::now_v7().simple().to_string();
        let payload = stage_pending(&self.home, &internal_id, &request_id, "add_workbook")?;

        // 只对最终副本 parse/compile/digest（§5.2 第 3 步）。
        Self::copy_confined(dir, &payload)?;
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
            command_json: format!(
                "{{\"intent\":\"add_workbook\",\"source\":{:?}}}",
                intent.hash().as_str()
            ),
            at: ctx.now,
        };
        match repo.store.commit(input)? {
            CommitOutcome::Committed { .. } => {}
            CommitOutcome::Replayed { reply_json, .. } => {
                return decode_added_snapshot(&request_id, &reply_json);
            }
        }
        // 发布效果；失败返回 committed=true 的 EFFECT_PENDING（协议 §5）。
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if !row.published {
                let ops = decode_effects(&row.effects_json)?;
                if let Err(e) = execute(&self.home, &ops, true) {
                    return Err(Error::EffectPending {
                        committed: true,
                        request_id,
                        pending_request_id: None,
                        detail: e.to_string(),
                        original: None,
                    });
                }
                repo.store.mark_published(&request_id)?;
            }
        }
        Ok(snapshot)
    }

    /// 锁内恢复未完成的 Workbook 效果（先于新命令，存储合同 §3.2）。
    fn recover_workbook_effects(&self) -> Result<()> {
        for (request_id, effects_json) in self.store.unpublished_requests()? {
            let ops = decode_effects(&effects_json)?;
            if let Err(e) = execute(&self.home, &ops, true) {
                return Err(Error::EffectPending {
                    committed: false,
                    request_id: String::new(),
                    pending_request_id: Some(request_id),
                    detail: e.to_string(),
                    original: None,
                });
            }
            self.store.mark_published(&request_id)?;
        }
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<WorkbookRow>> {
        self.store.list_workbooks()
    }

    /// 从已装目录（或任意目录，用于冻结副本）重新解析并编译。
    pub fn load_dir(&self, dir: &AbsPath) -> Result<LoadedWorkbook> {
        let res = build_resource_index(dir)?;
        let manifest = parse_manifest(&read_utf8(&dir.join(&RelPath::new("workbook.toml")?))?)?;
        let mut flows = Vec::new();
        for path in manifest.flows() {
            let def = parse_flow(&read_utf8(&dir.join(path))?)?;
            let graph = compile(&def, &manifest, &res)?;
            flows.push((def, graph));
        }
        let digest = Self::digest_dir(dir)?;
        Ok(LoadedWorkbook {
            manifest,
            flows,
            dir: dir.clone(),
            digest,
        })
    }

    /// 按 `id` 与版本加载；`version` 为 `None` 取字面最高版本。不存在报 `NotFound`。
    /// 加载即核对登记身份：重算目录摘要与 `workbooks.digest` 比较，不符报
    /// `WORKBOOK_TAMPERED`（GF-32；O03：verify 发现的篡改不能被 start 静默接受）。
    pub fn load(&self, id: &str, version: Option<&str>) -> Result<LoadedWorkbook> {
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
        let loaded = self.load_dir(&self.home.workbook_dir(id, &row.version))?;
        if loaded.digest.as_str() != row.digest {
            return Err(Error::WorkbookTampered {
                results: vec![VerifyRow {
                    id: row.id,
                    version: row.version,
                    status: VerifyStatus::Tampered,
                }],
            });
        }
        Ok(loaded)
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
        let request_id = request_id.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let intent = RequestIntent::RemoveWorkbook {
            id: id.to_string(),
            version: version.to_string(),
        };
        if let Ok(ro) = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadOnly) {
            if let Some((hash, reply_json)) = ro.lookup_request(&request_id)? {
                if hash != intent.hash().as_str() {
                    return Err(Error::RequestConflict {
                        request_id: request_id.clone(),
                    });
                }
                return decode_removed_snapshot(&request_id, &reply_json);
            }
        }

        let _lock = self.home.acquire_lock()?;
        let store = Store::open(&self.home.store_path(), crate::store::OpenMode::ReadWrite)?;
        let repo = Self::new(self.home.clone(), store);
        repo.recover_workbook_effects()?;
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if row.intent_hash != intent.hash().as_str() {
                return Err(Error::RequestConflict {
                    request_id: request_id.clone(),
                });
            }
            return decode_removed_snapshot(&request_id, &row.reply_json);
        }

        // 清理前核归属（T05）：目录当前摘要必须仍与登记值相符。
        let dir = self.home.workbook_dir(id, version);
        let registered_digest;
        if dir.as_path().exists() {
            let registered = repo
                .store
                .workbook_versions(id)?
                .into_iter()
                .find(|r| r.version == version)
                .ok_or_else(|| Error::NotFound {
                    what: format!("Workbook {id}@{version}"),
                })?;
            registered_digest = registered.digest.clone();
            let current = Self::digest_dir(&dir)?;
            if current.as_str() != registered.digest {
                return Err(Error::WorkbookTampered {
                    results: vec![VerifyRow {
                        id: id.to_string(),
                        version: version.to_string(),
                        status: VerifyStatus::Tampered,
                    }],
                });
            }
        } else {
            registered_digest = String::new();
        }
        // 引用检查与删行在同一个事务（存储合同 §5.2）；损坏引用行在事务内停止。
        let ctx = Context {
            now: crate::observe::now(),
            principal: crate::observe::principal(),
        };
        let internal_id = uuid::Uuid::now_v7().simple().to_string();
        let payload = stage_pending(&self.home, &internal_id, &request_id, "remove_workbook")?;
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
            command_json: format!(
                "{{\"intent\":\"remove_workbook\",\"target\":\"{id}@{version}\"}}"
            ),
            at: ctx.now,
        };
        match repo.store.commit(input)? {
            CommitOutcome::Committed { .. } => {}
            CommitOutcome::Replayed { reply_json, .. } => {
                return decode_removed_snapshot(&request_id, &reply_json);
            }
        }
        if let Some(row) = repo.store.inspect_request(&request_id)? {
            if !row.published {
                let ops = decode_effects(&row.effects_json)?;
                if let Err(e) = execute(&self.home, &ops, true) {
                    return Err(Error::EffectPending {
                        committed: true,
                        request_id,
                        pending_request_id: None,
                        detail: e.to_string(),
                        original: None,
                    });
                }
                repo.store.mark_published(&request_id)?;
            }
        }
        Ok(snapshot)
    }

    /// `workbook verify`。`filter` 为 `Some((id, version))` 只核对一个。
    pub fn verify(&self, filter: Option<(&str, &str)>) -> Result<Vec<VerifyRow>> {
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
            let dir = self.home.workbook_dir(&row.id, &row.version);
            let status = if !dir.as_path().exists() {
                VerifyStatus::Missing
            } else {
                match Self::digest_dir(&dir) {
                    Ok(d) if d.as_str() == row.digest => VerifyStatus::Ok,
                    _ => VerifyStatus::Tampered,
                }
            };
            out.push(VerifyRow {
                id: row.id,
                version: row.version,
                status,
            });
        }
        Ok(out)
    }

    /// 受限复制：拒绝软链、硬链、非普通文件、单文件超 32 MiB、总量超 256 MiB。
    /// 逐文件句柄复制、独占创建目标并 fsync（`fsx`）。
    pub(crate) fn copy_confined(src: &AbsPath, dst: &AbsPath) -> Result<u64> {
        crate::fsx::copy_tree_confined(src, dst)
    }
}

/// 读一个必须存在的 UTF-8 文本文件：句柄核对身份并限额读取，失败按 `WORKBOOK_INVALID` 报。
fn read_utf8(path: &AbsPath) -> Result<String> {
    let f = crate::fsx::SafeFile::open_regular(path)?;
    let bytes = f.read_bounded(MAX_FILE_BYTES).map_err(|e| {
        Error::Core(sheltie_core::Error::WorkbookInvalid {
            field: path.to_string(),
            reason: e.to_string(),
        })
    })?;
    String::from_utf8(bytes).map_err(|_| {
        Error::Core(sheltie_core::Error::WorkbookInvalid {
            field: path.to_string(),
            reason: "不是 UTF-8".to_string(),
        })
    })
}

/// 整棵含根置只读（目录 0555、文件 0444；存储合同 §5.2）。句柄核对身份后 fchmod，
/// 拒绝软链；`work start` 的冻结副本也用它。挪动/删除前的放开用 `make_tree_writable`。
pub(crate) fn set_tree_readonly(dir: &AbsPath) -> Result<()> {
    crate::fsx::set_tree_readonly_confined(dir)
}
