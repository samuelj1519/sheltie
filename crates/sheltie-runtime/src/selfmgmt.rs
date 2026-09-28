//! 二进制自管理：`self install | update | rollback | uninstall | version`。
//! 规则见 `specs/contracts/storage.md` §9 与协议 `self` 组。

use serde::Deserialize;
use sheltie_core::digest::Sha256Hex;
use sheltie_core::path::AbsPath;

use crate::error::{Error, Result};
use crate::home::Home;

/// 发布清单的来源。默认 GitHub Releases；测试用环境变量 `SHELTIE_RELEASE_BASE` 指向本地目录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSource {
    pub base: String,
}

impl ReleaseSource {
    pub fn from_env() -> Self {
        let base = std::env::var("SHELTIE_RELEASE_BASE")
            .unwrap_or_else(|_| "https://github.com/samuelj1519/sheltie/releases".to_string());
        Self { base }
    }

    fn is_local(&self) -> bool {
        !self.base.starts_with("http://") && !self.base.starts_with("https://")
    }

    /// 发布身份目录下的清单 URL。`tag` 是 `latest` 或 `v<version>`（存储合同 §9 步 1）。
    /// 本地目录按 tag 布局镜像远端；远端 latest 是移动别名，固定 tag 走 download 路径。
    fn manifest_url(&self, tag: &str) -> String {
        if self.is_local() {
            format!("{}/{tag}/dist-manifest.json", self.base)
        } else if tag == "latest" {
            format!("{}/latest/download/dist-manifest.json", self.base)
        } else {
            format!("{}/download/{tag}/dist-manifest.json", self.base)
        }
    }

    fn asset_url(&self, tag: &str, name: &str) -> String {
        if self.is_local() {
            format!("{}/{tag}/{name}", self.base)
        } else if tag == "latest" {
            format!("{}/latest/download/{name}", self.base)
        } else {
            format!("{}/download/{tag}/{name}", self.base)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InstallOutcome {
    pub installed_to: AbsPath,
    pub already_installed: bool,
    pub path_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct UpdateOutcome {
    pub from: String,
    pub to: String,
    pub up_to_date: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub platform: String,
    pub home: AbsPath,
    pub schema_version: i64,
}

/// 当前平台串，与发布包命名一致：Rust target triple（D-30）。
pub fn platform() -> String {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        (os, arch) => return format!("{arch}-{os}"),
    }
    .to_string()
}

/// `self version`。只读，不需要管理根存在。
pub fn version_info(home: &Home) -> VersionInfo {
    VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform(),
        home: home.root().clone(),
        schema_version: crate::store::SCHEMA_VERSION,
    }
}

/// `self install`：把当前可执行文件复制到 `bin/sheltie`（先写 `tmp/` 再 rename）。
/// 已存在且字节相同 → `already_installed = true`。不写任何 shell 配置，PATH 提示
/// 只是输出文本（存储合同 §9、协议 §3）。写动词持管理根写锁（§2.2）。
pub fn install(home: &Home) -> Result<InstallOutcome> {
    let _lock = home.acquire_lock()?;
    let current = std::env::current_exe().map_err(|e| Error::io("current_exe", e))?;
    let bin = home.bin_dir();
    let target = bin.join_segment("sheltie");
    // 建管理根的 store.db（协议 self install：复制之外还要建库）；
    // 放在幂等短路之前，bin/ 里已有同字节二进制但库还没建的场合也能补齐。
    crate::store::Store::open(&home.store_path(), crate::store::OpenMode::ReadWrite)?;
    // 当前可执行文件是根外对象，只读取观察；限额读一次，幂等比较与落位都用这份字节。
    let exe = crate::fsx::SafeFile::open_regular(
        &AbsPath::new(current.to_string_lossy().into_owned()).map_err(Error::Core)?,
    )?;
    let bytes = exe.read_bounded(crate::fsx::MAX_FILE_BYTES)?;
    // 已存在且字节相同就不动（幂等）。目标同样走 T04 边界读；读不出（缺失、软链、被替换）
    // 都当成不同，走落位覆盖。
    if crate::fsx::SafeFile::open_regular(&target)
        .and_then(|old| old.read_bounded(crate::fsx::MAX_FILE_BYTES))
        .map(|old| old == bytes)
        .unwrap_or(false)
    {
        return Ok(InstallOutcome {
            installed_to: target,
            already_installed: true,
            path_hint: path_hint(home),
        });
    }
    crate::fsx::ensure_dirs_under(home.root(), &bin)?;
    let tmp = home
        .tmp_dir()
        .join_segment(&uuid::Uuid::now_v7().to_string());
    std::fs::create_dir_all(tmp.as_path()).map_err(|e| Error::io(tmp.as_str(), e))?;
    let staged = tmp.join_segment("sheltie");
    // 落位用独占创建 + fsync + rename。
    crate::fsx::write_new_file(&staged, &bytes)?;
    make_executable(&staged)?;
    fsync(&staged)?;
    std::fs::rename(staged.as_path(), target.as_path())
        .map_err(|e| Error::io(target.as_str(), e))?;
    crate::fsx::fsync_dir(&bin);
    let _ = std::fs::remove_dir_all(tmp.as_path());
    Ok(InstallOutcome {
        installed_to: target,
        already_installed: false,
        path_hint: path_hint(home),
    })
}

/// `self update`（存储合同 §9 六步）。`version` 为 `None` 时按 latest 固定出的版本更新。
pub fn update(home: &Home, source: &ReleaseSource, version: Option<&str>) -> Result<UpdateOutcome> {
    let _lock = home.acquire_lock()?;
    // 步 1：解析发布身份，一次固定到 tag；清单与资产都从同一 tag 取，不混用两次解析。
    let tag = resolve_tag(source, version)?;
    // 步 2：读该 tag 的清单，找当前平台的包与 sha256。
    let manifest = read_manifest(source, &tag)?;
    let pinned = tag_version(&tag);
    if manifest.version != pinned {
        return Err(Error::UpdateUnavailable {
            reason: format!("tag {tag} 的清单写着版本 {}，与 tag 不符", manifest.version),
        });
    }
    let current = env!("CARGO_PKG_VERSION");
    if manifest.version == current {
        return Ok(UpdateOutcome {
            from: current.to_string(),
            to: manifest.version,
            up_to_date: true,
        });
    }
    let platform_str = platform();
    let Some(asset) = manifest.assets.iter().find(|a| a.platform == platform_str) else {
        return Err(Error::UpdateUnavailable {
            reason: format!("发布清单里没有 {platform_str} 的包"),
        });
    };
    // 资产名要能安全拼进 tmp/<uuid>/；伪造的清单不能把路径带出管理根。
    let name = checked_asset_name(&asset.name)?;
    // 步 3：下载到 tmp/<uuid>/，核对摘要，必要时解包。
    crate::fsx::ensure_dirs_under(home.root(), &home.tmp_dir())?;
    let tmp = home
        .tmp_dir()
        .join_segment(&uuid::Uuid::now_v7().to_string());
    std::fs::create_dir_all(tmp.as_path()).map_err(|e| Error::io(tmp.as_str(), e))?;
    let downloaded = match download(source, &tag, &name, &tmp) {
        Ok(p) => p,
        Err(e) => {
            let _ = std::fs::remove_dir_all(tmp.as_path());
            return Err(e);
        }
    };
    // 核对发布包文件的摘要（限额读取同一对象）；不符删掉下载文件。
    let bytes = match crate::fsx::SafeFile::open_regular(&downloaded)
        .and_then(|f| f.read_bounded(crate::fsx::MAX_FILE_BYTES))
    {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_dir_all(tmp.as_path());
            return Err(Error::UpdateUnavailable {
                reason: format!("读不了发布包 {name}：{e}"),
            });
        }
    };
    let got = Sha256Hex::of_bytes(&bytes);
    if got.as_str() != asset.sha256 {
        let _ = std::fs::remove_dir_all(tmp.as_path());
        return Err(Error::UpdateChecksumMismatch {
            expected: asset.sha256.clone(),
            actual: got.as_str().to_string(),
        });
    }
    // 压缩包解包取包内的 sheltie 二进制；瘦格式的资产就是二进制本身。
    // 解包失败同样删 tmp/<uuid>/——失败窗口里不留半成品（存储合同 §9 步 3）。
    let new_bin = match unpack_if_archive(&downloaded, &tmp) {
        Ok(Some(bin)) => bin,
        Ok(None) => downloaded,
        Err(e) => {
            let _ = std::fs::remove_dir_all(tmp.as_path());
            return Err(e);
        }
    };
    if let Err(e) = make_executable(&new_bin) {
        let _ = std::fs::remove_dir_all(tmp.as_path());
        return Err(e);
    }
    let bin = home.bin_dir();
    crate::fsx::ensure_dirs_under(home.root(), &bin)?;
    let target = bin.join_segment("sheltie");
    let prev = bin.join_segment("sheltie.prev");
    // 崩溃窗口：第 4 步与第 5 步之间（存储合同 §9 末段）。
    if target.as_path().exists() {
        if let Err(e) = std::fs::rename(target.as_path(), prev.as_path()) {
            // rename 失败同样清 tmp——失败窗口不留半成品（§9 步 3 的口径延伸到替换步）。
            let _ = std::fs::remove_dir_all(tmp.as_path());
            return Err(Error::io(prev.as_str(), e));
        }
    }
    crate::failpoint::maybe_exit("update_between_renames");
    if let Err(e) = std::fs::rename(new_bin.as_path(), target.as_path()) {
        let _ = std::fs::remove_dir_all(tmp.as_path());
        return Err(Error::io(target.as_str(), e));
    }
    crate::fsx::fsync_dir(&bin);
    let _ = std::fs::remove_dir_all(tmp.as_path());
    Ok(UpdateOutcome {
        from: current.to_string(),
        to: manifest.version,
        up_to_date: false,
    })
}

/// `self rollback`：`sheltie.prev` 换回来；`bin/sheltie` 缺失时直接挪回。没有 `.prev` 报 `NotFound`。
pub fn rollback(home: &Home) -> Result<()> {
    let _lock = home.acquire_lock()?;
    let bin = home.bin_dir();
    let prev = bin.join_segment("sheltie.prev");
    let target = bin.join_segment("sheltie");
    if !prev.as_path().exists() {
        return Err(Error::NotFound {
            what: prev.to_string(),
        });
    }
    if target.as_path().exists() {
        // 只保留一级：现在的 sheltie 丢弃，不变成新的 .prev。
        let trash = home
            .tmp_dir()
            .join_segment(&uuid::Uuid::now_v7().to_string());
        if let Some(parent) = trash.as_path().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::rename(target.as_path(), trash.as_path()).is_ok() {
            // trash 是文件不是目录；remove_dir_all 对文件报 ENOTDIR，会把它留在 tmp/ 里。
            let _ = std::fs::remove_file(trash.as_path());
        }
        crate::fsx::fsync_dir(&bin);
    }
    std::fs::rename(prev.as_path(), target.as_path()).map_err(|e| Error::io(target.as_str(), e))?;
    Ok(())
}

/// `self uninstall`：默认只删 `bin/`；`purge` 时删整个管理根，`confirmed` 为假则报 `InvalidRequest`。
/// 返回保留下来的路径（`--purge` 时为空）。写动词持管理根写锁；purge 持锁删根（§2.2），
/// 等待者获锁后复核根与 `.lock` 身份，见 [`crate::home::HomeLock::identity_still_valid`]。
pub fn uninstall(home: &Home, purge: bool, confirmed: bool) -> Result<Vec<AbsPath>> {
    if purge && !confirmed {
        return Err(Error::InvalidRequest {
            reason: "卸载并清空管理根需要确认（交互模式输入 yes，--json 模式给 --yes）".to_string(),
        });
    }
    let _lock = home.acquire_lock()?;
    let bin = home.bin_dir();
    if purge {
        crate::fsx::remove_tree_no_follow(home.root())?;
        return Ok(Vec::new());
    }
    if bin.as_path().exists() {
        crate::fsx::remove_tree_no_follow(&bin)?;
    }
    let mut kept = Vec::new();
    for name in ["store.db", "workbooks", "works"] {
        let p = home.root().join_segment(name);
        if p.as_path().exists() {
            kept.push(p);
        }
    }
    Ok(kept)
}

// ── 发布身份（存储合同 §9 步 1） ────────────────────────────────

/// 解析发布身份，一次固定到 tag：给了 `--version` 就是 `v<version>`；没给则读
/// `latest` 的清单学出版本号，再固定到 `v<版本>`。之后清单与资产都从这个 tag 取
/// ——latest 是移动别名，两次解析之间可能已经换发布，混用会把新旧包拼在一起。
fn resolve_tag(source: &ReleaseSource, version: Option<&str>) -> Result<String> {
    match version {
        Some(want) => {
            let v = checked_version(want)?;
            Ok(format!("v{v}"))
        }
        None => {
            let latest = read_manifest(source, "latest")?;
            let v = checked_version(&latest.version)?;
            Ok(format!("v{v}"))
        }
    }
}

/// tag `v<version>` 去掉 `v` 前缀就是版本号。
fn tag_version(tag: &str) -> String {
    tag.strip_prefix('v').unwrap_or(tag).to_string()
}

/// 版本号只允许一个路径段（无 `/`、`..`、`.`、NUL），防止伪造的版本把 URL/路径带出发布目录。
fn checked_version(raw: &str) -> Result<String> {
    let v = raw.strip_prefix('v').unwrap_or(raw);
    if v.is_empty()
        || v == "."
        || v == ".."
        || v.contains('/')
        || v.contains('\\')
        || v.contains('\0')
    {
        return Err(Error::UpdateUnavailable {
            reason: format!("版本 {raw} 不是合法的版本号"),
        });
    }
    Ok(v.to_string())
}

/// 资产名同理：只接受单个文件名，`..`、分隔符一概拒绝（伪造路径不越界）。
fn checked_asset_name(raw: &str) -> Result<String> {
    if raw.is_empty()
        || raw == "."
        || raw == ".."
        || raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\0')
    {
        return Err(Error::UpdateUnavailable {
            reason: format!("发布包名 {raw} 不是合法的文件名"),
        });
    }
    Ok(raw.to_string())
}

// ── 发布清单 ──────────────────────────────────────────────────

/// 瘦格式清单（存储合同 §9）：本地发布目录与测试的合同。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SlimManifest {
    version: String,
    assets: Vec<SlimAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SlimAsset {
    platform: String,
    name: String,
    sha256: String,
}

/// 适配后的清单：两种来源都归到这个形状。
struct ReleaseManifest {
    version: String,
    assets: Vec<SlimAsset>,
}

/// 读固定 tag 的发布清单。先按瘦格式解析；失败再按 cargo-dist 的完整 `dist-manifest.json` 适配。
/// 完整格式的字段没有自动化测试，T25 真实升级时人工核对（D-30）。
fn read_manifest(source: &ReleaseSource, tag: &str) -> Result<ReleaseManifest> {
    let url = source.manifest_url(tag);
    let text = if source.is_local() {
        std::fs::read_to_string(&url).map_err(|e| Error::UpdateUnavailable {
            reason: format!("读不了发布清单 {url}：{e}"),
        })?
    } else {
        curl(&url).map_err(|reason| Error::UpdateUnavailable { reason })?
    };
    if let Ok(slim) = serde_json::from_str::<SlimManifest>(&text) {
        return Ok(ReleaseManifest {
            version: slim.version,
            assets: slim.assets,
        });
    }
    adapt_cargo_dist_manifest(&text)
}

/// cargo-dist 完整清单 → 瘦格式：版本取 `announcement_tag`（去 `v` 前缀），
/// 资产取 `kind = executable-zip` 的压缩包，平台取 `target_triples` 的第一个。
/// 0.32 的真实形态（T25 用 `dist build` 的产出核过）：`artifacts` 是按产物名索引的
/// 对象（也容忍数组写法）；真哈希在 `checksums.sha256`；`checksum` 字段是同名的
/// 校验文件**名**，不是哈希，裸串只在恰好是 64 位十六进制时才当哈希接受。
fn adapt_cargo_dist_manifest(text: &str) -> Result<ReleaseManifest> {
    let bad = || Error::UpdateUnavailable {
        reason: "发布清单既不是瘦格式也解不出 cargo-dist 的字段".to_string(),
    };
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| Error::UpdateUnavailable {
            reason: format!("发布清单不是合法 JSON：{e}"),
        })?;
    let tag = value
        .get("announcement_tag")
        .and_then(|v| v.as_str())
        .ok_or_else(bad)?;
    let version = tag.strip_prefix('v').ok_or_else(bad)?.to_string();
    let artifacts = value
        .get("artifacts")
        .or_else(|| value.get("assets"))
        .ok_or_else(bad)?;
    let list: Vec<&serde_json::Value> = match artifacts {
        serde_json::Value::Object(map) => map.values().collect(),
        serde_json::Value::Array(arr) => arr.iter().collect(),
        _ => return Err(bad()),
    };
    let mut assets = Vec::new();
    for artifact in list {
        let kind = artifact.get("kind").and_then(|v| v.as_str());
        if kind != Some("executable-zip") {
            continue;
        }
        let Some(name) = artifact.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(triple) = artifact
            .get("target_triples")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        let checksum = artifact
            .get("checksums")
            .and_then(|c| c.get("sha256"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                artifact
                    .get("checksum")
                    .and_then(|c| c.get("sha256"))
                    .and_then(|v| v.as_str())
            })
            .or_else(|| artifact.get("checksum_sha256").and_then(|v| v.as_str()))
            .or_else(|| {
                let raw = artifact.get("checksum").and_then(|v| v.as_str())?;
                let s = raw.strip_prefix("sha256:").unwrap_or(raw);
                (s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())).then_some(s)
            });
        let Some(sha256) = checksum else {
            continue;
        };
        assets.push(SlimAsset {
            platform: triple.to_string(),
            name: name.to_string(),
            sha256: sha256.to_string(),
        });
    }
    Ok(ReleaseManifest { version, assets })
}

/// 下载固定 tag 的发布包到 `tmp/<name>`。本地发布目录直接复制；远端用系统 `curl`。
fn download(source: &ReleaseSource, tag: &str, name: &str, tmp: &AbsPath) -> Result<AbsPath> {
    let url = source.asset_url(tag, name);
    let dst = tmp.join_segment(name);
    if source.is_local() {
        std::fs::copy(&url, dst.as_path()).map_err(|e| Error::UpdateUnavailable {
            reason: format!("读不了发布包 {url}：{e}"),
        })?;
    } else {
        curl_to(&url, &dst).map_err(|reason| Error::UpdateUnavailable { reason })?;
    }
    Ok(dst)
}

/// `curl -fsSL <url>` 取文本。
fn curl(url: &str) -> std::result::Result<String, String> {
    let out = std::process::Command::new("curl")
        .args(["-fsSL", url])
        .output()
        .map_err(|e| format!("起不了 curl：{e}"))?;
    if !out.status.success() {
        return Err(format!(
            "curl 退出 {}：{}",
            out.status
                .code()
                .map_or_else(|| "?".to_string(), |c| c.to_string()),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("清单不是 UTF-8：{e}"))
}

/// `curl -fsSL <url> -o <dst>` 取文件。
fn curl_to(url: &str, dst: &AbsPath) -> std::result::Result<(), String> {
    let out = std::process::Command::new("curl")
        .args(["-fsSL", url, "-o", dst.as_str()])
        .output()
        .map_err(|e| format!("起不了 curl：{e}"))?;
    if !out.status.success() {
        return Err(format!(
            "下载 {url} 失败（curl 退出 {}）：{}",
            out.status
                .code()
                .map_or_else(|| "?".to_string(), |c| c.to_string()),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}

/// 压缩包解包，返回包内唯一名为 `sheltie` 的普通文件；不是压缩包返回 `None`。
fn unpack_if_archive(asset: &AbsPath, tmp: &AbsPath) -> Result<Option<AbsPath>> {
    let name = asset
        .as_path()
        .file_name()
        .map(|n| n.to_string())
        .unwrap_or_default();
    if !(name.ends_with(".tar.gz") || name.ends_with(".tgz") || name.ends_with(".tar.xz")) {
        return Ok(None);
    }
    let dir = tmp.join_segment("unpacked");
    std::fs::create_dir_all(dir.as_path()).map_err(|e| Error::io(dir.as_str(), e))?;
    let out = std::process::Command::new("tar")
        .args(["-xf", asset.as_str(), "-C", dir.as_str()])
        .output()
        .map_err(|e| Error::io(asset.as_str(), e))?;
    if !out.status.success() {
        return Err(Error::UpdateUnavailable {
            reason: format!("解包 {name} 失败：{}", String::from_utf8_lossy(&out.stderr)),
        });
    }
    // cargo-dist 0.32 的包内布局是 <产物名去掉扩展>/sheltie（二进制在内层目录根部，
    // T25 用 dist build 的真实产出核过）；旧假设 sheltie/bin/sheltie 也接受：
    // 在解包目录里找恰好一个名为 sheltie 的普通文件。
    let mut found = Vec::new();
    find_named_file(dir.as_path().as_std_path(), "sheltie", 3, &mut found);
    if found.len() == 1 {
        return Ok(found.into_iter().next());
    }
    let detail = if found.is_empty() {
        "找不到"
    } else {
        "找到多个"
    };
    Err(Error::UpdateUnavailable {
        reason: format!("解包 {name} 后{detail} sheltie 二进制"),
    })
}

/// 递归收集 `dir` 下名为 `name` 的普通文件，最多下潜 `depth` 层；读不了的目录跳过。
fn find_named_file(dir: &std::path::Path, name: &str, depth: u8, out: &mut Vec<AbsPath>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            find_named_file(&entry.path(), name, depth - 1, out);
        } else if meta.is_file() && entry.file_name() == name {
            if let Ok(p) = AbsPath::new(entry.path().to_string_lossy().into_owned()) {
                out.push(p);
            }
        }
    }
}

// ── 小件 ──────────────────────────────────────────────────────

fn path_hint(home: &Home) -> String {
    format!("把 {} 加进 PATH", home.bin_dir())
}

fn make_executable(path: &AbsPath) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o755))
        .map_err(|e| Error::io(path.as_str(), e))
}

fn fsync(path: &AbsPath) -> Result<()> {
    let f = std::fs::File::open(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
    f.sync_all().map_err(|e| Error::io(path.as_str(), e))?;
    Ok(())
}
