//! Workbook 仓库：`add / list / load / remove / verify`。规则见 `specs/contracts/storage.md` §5 与协议 §3。

use serde::Serialize;
use sha2::Digest as _;
use sheltie_core::digest::Sha256Hex;
use sheltie_core::flow::{FlowDef, Graph, compile, parse_flow};
use sheltie_core::ids::WorkId;
use sheltie_core::path::{AbsPath, RelPath};
use sheltie_core::workbook::Manifest;
use sheltie_core::workbook::parse_manifest;

use crate::error::{Error, Result};
use crate::home::Home;
use crate::observe::build_resource_index;
use crate::store::{Store, WorkbookRow};

/// 单文件上限 32 MiB。
pub const MAX_FILE_BYTES: u64 = 33_554_432;
/// 目录总量上限 256 MiB。
pub const MAX_TOTAL_BYTES: u64 = 268_435_456;

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

    /// `workbook add <dir>`（存储合同 §5 四步）。
    ///
    /// 1. 校验：`build_resource_index`、`parse_manifest`、每个 Flow `parse_flow` + `compile`。
    /// 2. 复制到 `staging/<uuid>/`（`copy_confined`），fsync，算 `digest_dir`。
    /// 3. `store.insert_workbook`；冲突则删 staging 并报 `WorkbookExists`。
    /// 4. rename 到 `workbooks/<id>/<version>/`，整棵置只读。
    pub fn add(&self, dir: &AbsPath) -> Result<Added> {
        // 1. 校验全部通过才碰管理根（合同 §1：任一步失败则整体拒绝，不落任何文件）。
        let res = build_resource_index(dir)?;
        let manifest = parse_manifest(&read_utf8(&dir.join(&RelPath::new("workbook.toml")?))?)?;
        let mut flow_defs = Vec::new();
        for path in manifest.flows() {
            let def = parse_flow(&read_utf8(&dir.join(path))?)?;
            let _ = compile(&def, &manifest, &res)?;
            flow_defs.push(def.id().as_str().to_string());
        }
        // 2. staging：先顺手清掉上次崩溃留下的残留。
        let staging_root = self.home.staging_dir();
        let _ = std::fs::remove_dir_all(staging_root.as_path());
        std::fs::create_dir_all(staging_root.as_path())
            .map_err(|e| Error::io(staging_root.as_str(), e))?;
        let staging = staging_root.join_segment(&uuid::Uuid::now_v7().to_string());
        let added = self.add_via_staging(dir, &staging, &manifest, &flow_defs);
        if added.is_err() {
            let _ = std::fs::remove_dir_all(staging.as_path());
        }
        added
    }

    /// 四步的后三步：staging 里有东西之后的路径，任何失败都把 staging 清掉。
    fn add_via_staging(
        &self,
        dir: &AbsPath,
        staging: &AbsPath,
        manifest: &Manifest,
        flow_ids: &[String],
    ) -> Result<Added> {
        Self::copy_confined(dir, staging)?;
        let digest = Self::digest_dir(staging)?;
        let final_dir = self
            .home
            .workbook_dir(manifest.id().as_str(), manifest.version());
        let rel_dir = final_dir
            .as_path()
            .strip_prefix(self.home.root().as_path())
            .map(|p| p.to_string())
            .unwrap_or_else(|_| final_dir.as_str().to_string());
        let row = WorkbookRow {
            id: manifest.id().as_str().to_string(),
            version: manifest.version().to_string(),
            digest: digest.as_str().to_string(),
            dir: rel_dir,
            added_at: crate::observe::now().as_str().to_string(),
        };
        self.store.insert_workbook(&row)?;
        if let Some(parent) = final_dir.as_path().parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(final_dir.as_str(), e))?;
        }
        std::fs::rename(staging.as_path(), final_dir.as_path())
            .map_err(|e| Error::io(final_dir.as_str(), e))?;
        set_tree_readonly(&final_dir)?;
        Ok(Added {
            id: row.id,
            version: row.version,
            digest,
            flows: flow_ids.to_vec(),
            requires: manifest
                .requires()
                .iter()
                .map(|r| format!("{}:{}", r.kind().as_str(), r.name()))
                .collect(),
        })
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
        self.load_dir(&self.home.workbook_dir(id, &row.version))
    }

    /// 目录摘要：全部文件按相对路径排序，拼 `路径\0内容` 后 sha256（协议 `workbook add`）。
    pub fn digest_dir(dir: &AbsPath) -> Result<Sha256Hex> {
        let mut files = Vec::new();
        collect_file_bytes(dir, dir, &mut files)?;
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut hasher = sha2::Sha256::new();
        for (path, content) in files {
            hasher.update(path.as_bytes());
            hasher.update([0u8]);
            hasher.update(&content);
        }
        Ok(Sha256Hex::of_bytes(&hasher.finalize()))
    }

    /// `workbook remove <id>@<version>`（协议细则三步）。
    pub fn remove(&self, id: &str, version: &str) -> Result<Removed> {
        if version.is_empty() {
            return Err(Error::InvalidRequest {
                reason: "remove 必须给全版本，不接受「最高版本」默认".to_string(),
            });
        }
        let works = self.works_referencing(id, version)?;
        if !works.is_empty() {
            return Err(Error::WorkbookInUse {
                id: id.to_string(),
                version: version.to_string(),
                works,
            });
        }
        self.store.delete_workbook(id, version)?;
        // 提交后把目录挪到 tmp/ 再删；失败只影响磁盘，不影响库。
        let dir = self.home.workbook_dir(id, version);
        if dir.as_path().exists() {
            std::fs::create_dir_all(self.home.tmp_dir().as_path())
                .map_err(|e| Error::io(self.home.tmp_dir().as_str(), e))?;
            let tmp = self
                .home
                .tmp_dir()
                .join_segment(&uuid::Uuid::now_v7().to_string());
            // macOS 上挪动目录本身要写权限（会更新 ..），先放开再挪。
            make_tree_writable(&dir);
            if std::fs::rename(dir.as_path(), tmp.as_path()).is_ok() {
                let _ = std::fs::remove_dir_all(tmp.as_path());
            }
        }
        Ok(Removed {
            id: id.to_string(),
            version: version.to_string(),
        })
    }

    /// 引用本版本且非终态的 Work。先验证全部持久行，不按冗余 status 预筛。
    pub(crate) fn works_referencing(&self, id: &str, version: &str) -> Result<Vec<WorkId>> {
        let mut out = Vec::new();
        for row in self.store.list_works()? {
            if !row.state.status.is_terminal()
                && row.state.workbook.id.as_str() == id
                && row.state.workbook.version == version
            {
                out.push(row.state.work_id);
            }
        }
        Ok(out)
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

    /// 受限复制：拒绝软链、硬链、非普通文件、含 `..`、单文件超 32 MiB、总量超 256 MiB。
    /// 返回复制的总字节数。每个文件写完即 fsync。
    pub(crate) fn copy_confined(src: &AbsPath, dst: &AbsPath) -> Result<u64> {
        let mut total = 0u64;
        copy_tree_confined(src, dst, &mut total)?;
        Ok(total)
    }
}

/// 读一个必须存在的 UTF-8 文本文件，失败按 `WORKBOOK_INVALID` 报。
fn read_utf8(path: &AbsPath) -> Result<String> {
    std::fs::read_to_string(path.as_path()).map_err(|e| {
        Error::Core(sheltie_core::Error::WorkbookInvalid {
            field: path.to_string(),
            reason: format!("读不了或不是 UTF-8：{e}"),
        })
    })
}

fn copy_tree_confined(src: &AbsPath, dst: &AbsPath, total: &mut u64) -> Result<()> {
    std::fs::create_dir_all(dst.as_path()).map_err(|e| Error::io(dst.as_str(), e))?;
    for entry in std::fs::read_dir(src.as_path()).map_err(|e| Error::io(src.as_str(), e))? {
        let entry = entry.map_err(|e| Error::io(src.as_str(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let s = src.join_segment(&name);
        let d = dst.join_segment(&name);
        let ft = entry.file_type().map_err(|e| Error::io(s.as_str(), e))?;
        if ft.is_symlink() {
            return Err(Error::InvalidRequest {
                reason: format!("{s} 是符号链接"),
            });
        }
        if ft.is_dir() {
            copy_tree_confined(&s, &d, total)?;
            continue;
        }
        if !ft.is_file() {
            return Err(Error::InvalidRequest {
                reason: format!("{s} 不是普通文件"),
            });
        }
        let meta = entry.metadata().map_err(|e| Error::io(s.as_str(), e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            if meta.nlink() > 1 {
                return Err(Error::InvalidRequest {
                    reason: format!("{s} 是硬链接"),
                });
            }
        }
        if meta.len() > MAX_FILE_BYTES {
            return Err(Error::InvalidRequest {
                reason: format!("{s} 超过 {MAX_FILE_BYTES} 字节"),
            });
        }
        if total
            .checked_add(meta.len())
            .is_none_or(|t| t > MAX_TOTAL_BYTES)
        {
            return Err(Error::InvalidRequest {
                reason: format!("{src} 总量超过 {MAX_TOTAL_BYTES} 字节"),
            });
        }
        std::fs::copy(s.as_path(), d.as_path()).map_err(|e| Error::io(d.as_str(), e))?;
        let f = std::fs::File::open(d.as_path()).map_err(|e| Error::io(d.as_str(), e))?;
        f.sync_all().map_err(|e| Error::io(d.as_str(), e))?;
        *total += meta.len();
    }
    Ok(())
}

/// 收集 `dir` 下全部普通文件的 `(相对路径, 内容)`。拒绝软链。
fn collect_file_bytes(
    root: &AbsPath,
    dir: &AbsPath,
    out: &mut Vec<(String, Vec<u8>)>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir.as_path()).map_err(|e| Error::io(dir.as_str(), e))? {
        let entry = entry.map_err(|e| Error::io(dir.as_str(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = dir.join_segment(&name);
        let ft = entry.file_type().map_err(|e| Error::io(path.as_str(), e))?;
        if ft.is_symlink() {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 是符号链接"),
            });
        }
        if ft.is_dir() {
            collect_file_bytes(root, &path, out)?;
            continue;
        }
        if !ft.is_file() {
            continue;
        }
        let rel = path
            .as_path()
            .strip_prefix(root.as_path())
            .map_err(|e| Error::io(root.as_str(), std::io::Error::other(e.to_string())))?
            .to_string();
        let bytes = std::fs::read(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
        out.push((rel, bytes));
    }
    Ok(())
}

/// 内容置只读：子目录 0555、文件 0444。传入的目录本身保持原样——macOS 挪动或
/// 删除目录需要它可写；防篡改靠文件只读位加 `verify` 的摘要核对。
/// `work start` 的冻结副本也用它。
pub(crate) fn set_tree_readonly(dir: &AbsPath) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    for entry in std::fs::read_dir(dir.as_path()).map_err(|e| Error::io(dir.as_str(), e))? {
        let entry = entry.map_err(|e| Error::io(dir.as_str(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = dir.join_segment(&name);
        if entry
            .file_type()
            .map_err(|e| Error::io(path.as_str(), e))?
            .is_dir()
        {
            std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o555))
                .map_err(|e| Error::io(path.as_str(), e))?;
            set_tree_readonly(&path)?;
        } else {
            std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o444))
                .map_err(|e| Error::io(path.as_str(), e))?;
        }
    }
    Ok(())
}

/// 整棵放开写权限：文件 0644，目录 0755。删除只读目录前用；尽力而为。
pub(crate) fn make_tree_writable(dir: &AbsPath) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(meta) = std::fs::metadata(dir.as_path()) else {
        return;
    };
    let mode = if meta.is_dir() { 0o755 } else { 0o644 };
    let _ = std::fs::set_permissions(dir.as_path(), std::fs::Permissions::from_mode(mode));
    if meta.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir.as_path()) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                make_tree_writable(&dir.join_segment(&name));
            }
        }
    }
}
