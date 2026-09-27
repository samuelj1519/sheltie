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
        // 2. staging：先顺手清掉上次崩溃留下的残留（不跟随软链）。
        let staging_root = self.home.staging_dir();
        let _ = crate::fsx::remove_tree_no_follow(&staging_root);
        crate::fsx::ensure_dirs_under(self.home.root(), &staging_root)?;
        let staging = staging_root.join_segment(&uuid::Uuid::now_v7().to_string());
        let added = self.add_via_staging(dir, &staging, &manifest, &flow_defs);
        if added.is_err() {
            let _ = crate::fsx::remove_tree_no_follow(&staging);
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
            crate::fsx::ensure_dirs_under(
                self.home.root(),
                &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
            )?;
        }
        std::fs::rename(staging.as_path(), final_dir.as_path())
            .map_err(|e| Error::io(final_dir.as_str(), e))?;
        crate::fsx::fsync_dir(&final_dir);
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

    /// 目录摘要：全部文件按相对路径排序，拼 `路径\0内容` 后对字节流 sha256 再对摘要
    /// 十六进制串做一次 sha256——这是 schema 1 已登记的旧算法（含 O07 的双重哈希与
    /// `path\0content` 定界歧义），流式化只改读取方式、不改字节语义，已装目录的记录
    /// 才能继续对上。C002-T07 切到 `workbook_digest::digest_dir_v2` 后本函数删除。
    pub fn digest_dir(dir: &AbsPath) -> Result<Sha256Hex> {
        let mut files = Vec::new();
        collect_file_meta(dir, dir, &mut files)?;
        // 限额在读取前核对（存储合同 §5.2）；读取经句柄计数，增长绕不过。
        let mut total = 0u64;
        for (rel, size) in &files {
            if *size > MAX_FILE_BYTES {
                return Err(Error::InvalidRequest {
                    reason: format!("{dir} 下 {rel} 超过 {MAX_FILE_BYTES} 字节"),
                });
            }
            total = total
                .checked_add(*size)
                .ok_or_else(|| Error::InvalidRequest {
                    reason: format!("{dir} 总量超出上限"),
                })?;
            if total > MAX_TOTAL_BYTES {
                return Err(Error::InvalidRequest {
                    reason: format!("{dir} 总量超过 {MAX_TOTAL_BYTES} 字节"),
                });
            }
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut hasher = sha2::Sha256::new();
        for (rel, _) in files {
            let full = dir.join(&RelPath::new(rel.clone()).map_err(Error::Core)?);
            let f = crate::fsx::SafeFile::open_regular(&full)?;
            let bytes = f.read_bounded(MAX_FILE_BYTES)?;
            hasher.update(rel.as_bytes());
            hasher.update([0u8]);
            hasher.update(&bytes);
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
        // 提交后把目录挪到 tmp/ 再删；失败只影响磁盘，不影响库。删除不跟随软链。
        let dir = self.home.workbook_dir(id, version);
        if dir.as_path().exists() {
            crate::fsx::ensure_dirs_under(self.home.root(), &self.home.tmp_dir())?;
            let tmp = self
                .home
                .tmp_dir()
                .join_segment(&uuid::Uuid::now_v7().to_string());
            // macOS 上挪动目录本身要写权限（会更新 ..），先放开再挪。
            crate::fsx::make_tree_writable(&dir);
            if std::fs::rename(dir.as_path(), tmp.as_path()).is_ok() {
                let _ = crate::fsx::remove_tree_no_follow(&tmp);
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

/// 收集 `dir` 下全部普通文件的 `(相对路径, 声明字节数)`。拒绝软链；跳过非常规文件
/// （与旧摘要的语义一致）。字节内容在摘要阶段经句柄流式读取。
fn collect_file_meta(root: &AbsPath, dir: &AbsPath, out: &mut Vec<(String, u64)>) -> Result<()> {
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
            collect_file_meta(root, &path, out)?;
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
        let meta = entry.metadata().map_err(|e| Error::io(path.as_str(), e))?;
        out.push((rel, meta.len()));
    }
    Ok(())
}

/// 整棵含根置只读（目录 0555、文件 0444；存储合同 §5.2）。句柄核对身份后 fchmod，
/// 拒绝软链；`work start` 的冻结副本也用它。挪动/删除前的放开用 `make_tree_writable`。
pub(crate) fn set_tree_readonly(dir: &AbsPath) -> Result<()> {
    crate::fsx::set_tree_readonly_confined(dir)
}
