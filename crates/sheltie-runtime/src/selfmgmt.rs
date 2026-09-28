//! 二进制自管理：`self install | update | rollback | uninstall | version`。
//! 规则见 `specs/contracts/storage.md` §9 与协议 `self` 组。

use std::io::{Read as _, Write as _};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Deserialize;
use sheltie_core::digest::Sha256Hex;
use sheltie_core::path::AbsPath;

use crate::error::{Error, Result};
use crate::fsx::ManagedRelPath;
use crate::home::Home;

const MAX_TOOL_STDERR_BYTES: u64 = 1024 * 1024;

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
    let lock = home.acquire_lock()?;
    let current = std::env::current_exe().map_err(|e| Error::io("current_exe", e))?;
    let bin = home.bin_dir();
    let target = bin.join_segment("sheltie");
    // 建管理根的 store.db（协议 self install：复制之外还要建库）；
    // 放在幂等短路之前，bin/ 里已有同字节二进制但库还没建的场合也能补齐。
    crate::store::Store::open(&home.store_path(), crate::store::OpenMode::ReadWrite)?;
    // 当前可执行文件是根外对象，只读取观察；限额读一次，幂等比较与落位都用这份字节。
    let exe = crate::fsx::ExternalReadFile::open_regular(
        &AbsPath::new(current.to_string_lossy().into_owned()).map_err(Error::Core)?,
    )?;
    let bytes = exe.read_bounded(crate::fsx::MAX_FILE_BYTES)?;
    // 缺失走独占新建；已存在对象必须是受管普通单链接文件，不能把软链当成
    // “不同版本”覆盖。相同字节保持幂等。
    let old = crate::fsx::open_managed_optional(home, &target)?;
    let replace = if let Some(file) = &old {
        if file.read_bounded(crate::fsx::MAX_FILE_BYTES)? == bytes {
            return Ok(InstallOutcome {
                installed_to: target,
                already_installed: true,
                path_hint: path_hint(home),
            });
        }
        true
    } else {
        false
    };
    crate::fsx::ensure_dirs_under(home, &lock, &bin)?;
    let tmp = home
        .tmp_dir()
        .join_segment(&uuid::Uuid::now_v7().to_string());
    let staged = tmp.join_segment("sheltie");
    crate::fsx::ensure_dirs_under(home, &lock, &tmp)?;
    let install_result = (|| {
        crate::fsx::write_new_file(home, &lock, &staged, &bytes)?;
        let staged_file =
            crate::fsx::verify_and_make_executable_confined(home, &lock, &staged, &bytes)?;
        if replace {
            crate::fsx::replace_verified_managed_file(home, &lock, &staged_file, &target, &bytes)?;
        } else {
            crate::fsx::rename_verified_managed_file(home, &lock, &staged_file, &target, &bytes)?;
        }
        Ok::<_, Error>(())
    })();
    if let Err(error) = install_result {
        return Err(cleanup_self_tmp(home, &lock, &tmp, error));
    }
    crate::fsx::remove_tree_no_follow(home, &lock, &tmp)?;
    Ok(InstallOutcome {
        installed_to: target,
        already_installed: false,
        path_hint: path_hint(home),
    })
}

fn cleanup_self_tmp(
    home: &Home,
    lock: &crate::home::HomeLock,
    tmp: &AbsPath,
    original: Error,
) -> Error {
    if matches!(original, Error::RecoveryRequired { .. }) {
        return original;
    }
    match crate::fsx::remove_tree_no_follow(home, lock, tmp) {
        Ok(()) => original,
        Err(cleanup) => Error::io(
            tmp.as_str(),
            std::io::Error::other(format!(
                "self操作失败：{original}；临时目录清理也失败：{cleanup}"
            )),
        ),
    }
}

/// `self update`（存储合同 §9 六步）。`version` 为 `None` 时按 latest 固定出的版本更新。
pub fn update(home: &Home, source: &ReleaseSource, version: Option<&str>) -> Result<UpdateOutcome> {
    let lock = home.acquire_lock()?;
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
    // 步 3：在受管 tmp 中下载、核对摘要，并安全读取压缩包候选。
    crate::fsx::ensure_dirs_under(home, &lock, &home.tmp_dir())?;
    let tmp = home
        .tmp_dir()
        .join_segment(&uuid::Uuid::now_v7().to_string());
    let bin = home.bin_dir();
    crate::fsx::ensure_dirs_under(home, &lock, &bin)?;
    let target = bin.join_segment("sheltie");
    let prev = bin.join_segment("sheltie.prev");
    crate::fsx::ensure_dirs_under(home, &lock, &tmp)?;
    let result = (|| {
        let downloaded = download(home, &lock, source, &tag, &name, &tmp)?;
        let bytes = crate::fsx::open_managed_regular(home, &downloaded)?
            .read_bounded(crate::fsx::MAX_FILE_BYTES)
            .map_err(|error| Error::UpdateUnavailable {
                reason: format!("读不了发布包 {name}：{error}"),
            })?;
        let got = Sha256Hex::of_bytes(&bytes);
        if got.as_str() != asset.sha256 {
            return Err(Error::UpdateChecksumMismatch {
                expected: asset.sha256.clone(),
                actual: got.as_str().to_string(),
            });
        }
        let (new_bin, candidate_bytes) = unpack_if_archive(home, &lock, &name, &bytes, &tmp)?;
        let candidate_handle = crate::fsx::verify_and_make_executable_confined(
            home,
            &lock,
            &new_bin,
            &candidate_bytes,
        )?;
        crate::failpoint::rendezvous("update_after_candidate_verify", home.root().as_str())
            .map_err(|error| Error::io(home.root().as_str(), error))?;

        let current_binary = crate::fsx::open_managed_optional(home, &target)?;
        let old_prev = if current_binary.is_some() {
            crate::fsx::open_managed_optional(home, &prev)?
        } else {
            None
        };
        let displaced_prev = tmp.join_segment("previous-sheltie.prev");
        if current_binary.is_some() {
            if old_prev.is_some() {
                // 原子交换后 current 已到 .prev；旧 .prev 暂留在 tmp，直到新 binary 到位。
                crate::fsx::replace_managed_regular_file(
                    home,
                    &lock,
                    &home.to_rel(&prev)?,
                    &home.to_rel(&target)?,
                )?;
                if let Err(error) = crate::fsx::rename_managed_new(
                    home,
                    &lock,
                    &home.to_rel(&target)?,
                    &home.to_rel(&displaced_prev)?,
                ) {
                    if matches!(error, Error::RecoveryRequired { .. }) {
                        return Err(error);
                    }
                    let restore = crate::fsx::replace_managed_regular_file(
                        home,
                        &lock,
                        &home.to_rel(&prev)?,
                        &home.to_rel(&target)?,
                    );
                    return Err(match restore {
                        Ok(()) => error,
                        Err(restore_error) => Error::RecoveryRequired {
                            path: target.to_string(),
                            detail: format!(
                                "更新移动旧 .prev 失败：{error}；目标、prev 与临时备份状态未能恢复，均保留：{restore_error}"
                            ),
                        },
                    });
                }
            } else {
                crate::fsx::rename_managed_new(
                    home,
                    &lock,
                    &home.to_rel(&target)?,
                    &home.to_rel(&prev)?,
                )?;
            }
        }

        // 既有 crash test 在此验证：目标可缺失，但 .prev 是原当前 binary。
        crate::failpoint::maybe_exit("update_between_renames");
        if let Err(error) = crate::fsx::rename_verified_managed_file(
            home,
            &lock,
            &candidate_handle,
            &target,
            &candidate_bytes,
        ) {
            if matches!(error, Error::RecoveryRequired { .. }) {
                return Err(error);
            }
            if current_binary.is_some() {
                let restore_current = crate::fsx::rename_managed_new(
                    home,
                    &lock,
                    &home.to_rel(&prev)?,
                    &home.to_rel(&target)?,
                );
                if let Err(restore_error) = restore_current {
                    return Err(Error::RecoveryRequired {
                        path: target.to_string(),
                        detail: format!(
                            "安装新binary失败：{error}；原binary端点未能确认恢复，保留prev与临时目录：{restore_error}"
                        ),
                    });
                }
                if old_prev.is_some() {
                    if let Err(restore_error) = crate::fsx::rename_managed_new(
                        home,
                        &lock,
                        &home.to_rel(&displaced_prev)?,
                        &home.to_rel(&prev)?,
                    ) {
                        return Err(Error::RecoveryRequired {
                            path: prev.to_string(),
                            detail: format!(
                                "原binary已恢复；旧 .prev 位于 {displaced_prev}，保留临时目录：{restore_error}"
                            ),
                        });
                    }
                }
            }
            return Err(error);
        }
        Ok(UpdateOutcome {
            from: current.to_string(),
            to: manifest.version.clone(),
            up_to_date: false,
        })
    })();
    match result {
        Ok(outcome) => {
            crate::fsx::remove_tree_no_follow(home, &lock, &tmp)?;
            Ok(outcome)
        }
        Err(error) => Err(cleanup_self_tmp(home, &lock, &tmp, error)),
    }
}

/// `self rollback`：`sheltie.prev` 换回来；`bin/sheltie` 缺失时直接挪回。没有 `.prev` 报 `NotFound`。
pub fn rollback(home: &Home) -> Result<()> {
    let lock = home.acquire_lock()?;
    let bin = home.bin_dir();
    let prev = bin.join_segment("sheltie.prev");
    let target = bin.join_segment("sheltie");
    let _previous = crate::fsx::open_managed_regular(home, &prev)?;
    let current = crate::fsx::open_managed_optional(home, &target)?;
    if current.is_some() {
        crate::fsx::ensure_dirs_under(home, &lock, &home.tmp_dir())?;
        let tmp = home
            .tmp_dir()
            .join_segment(&uuid::Uuid::now_v7().to_string());
        crate::fsx::ensure_dirs_under(home, &lock, &tmp)?;
        let saved_current = tmp.join_segment("current-sheltie");
        crate::fsx::rename_managed_new(
            home,
            &lock,
            &home.to_rel(&target)?,
            &home.to_rel(&saved_current)?,
        )?;
        match crate::fsx::rename_managed_new(
            home,
            &lock,
            &home.to_rel(&prev)?,
            &home.to_rel(&target)?,
        ) {
            Ok(()) => crate::fsx::remove_tree_no_follow(home, &lock, &tmp),
            Err(error) => match crate::fsx::rename_managed_new(
                home,
                &lock,
                &home.to_rel(&saved_current)?,
                &home.to_rel(&target)?,
            ) {
                Ok(()) => Err(cleanup_self_tmp(home, &lock, &tmp, error)),
                Err(restore) => Err(Error::RecoveryRequired {
                    path: target.to_string(),
                    detail: format!(
                        "rollback替换失败：{error}；当前binary位于 {saved_current}，恢复失败：{restore}"
                    ),
                }),
            },
        }
    } else {
        crate::fsx::rename_managed_new(home, &lock, &home.to_rel(&prev)?, &home.to_rel(&target)?)
    }
}

/// `self uninstall`：默认只删 `bin/`；`purge` 清理管理数据但保留管理根和同一 `.lock`。
/// `confirmed` 为假时报 `InvalidRequest`。返回保留下来的用户数据路径；purge 成功时为空。
/// 写动词持管理根写锁；等待者获锁后复核根与 `.lock` 身份。
pub fn uninstall(home: &Home, purge: bool, confirmed: bool) -> Result<Vec<AbsPath>> {
    if purge && !confirmed {
        return Err(Error::InvalidRequest {
            reason: "卸载并清空管理根需要确认（交互模式输入 yes，--json 模式给 --yes）".to_string(),
        });
    }
    let lock = home.acquire_lock()?;
    let bin = home.bin_dir();
    if purge {
        crate::fsx::remove_tree_no_follow(home, &lock, home.root())?;
        return Ok(vec![home.root().clone(), home.lock_path()]);
    }
    if bin.as_path().exists() {
        crate::fsx::remove_tree_no_follow(home, &lock, &bin)?;
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
        let path = AbsPath::new(crate::request::lexical_abs(&url)).map_err(Error::Core)?;
        let file = crate::fsx::ExternalReadFile::open_regular(&path).map_err(|error| {
            Error::UpdateUnavailable {
                reason: format!("读不了发布清单 {url}：{error}"),
            }
        })?;
        let bytes = file
            .read_bounded(crate::fsx::MAX_FILE_BYTES)
            .map_err(|error| Error::UpdateUnavailable {
                reason: format!("发布清单 {url} 超过读取限额：{error}"),
            })?;
        String::from_utf8(bytes).map_err(|error| Error::UpdateUnavailable {
            reason: format!("发布清单 {url} 不是UTF-8：{error}"),
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
fn download(
    home: &Home,
    lock: &crate::home::HomeLock,
    source: &ReleaseSource,
    tag: &str,
    name: &str,
    tmp: &AbsPath,
) -> Result<AbsPath> {
    let url = source.asset_url(tag, name);
    let dst = tmp.join_segment(name);
    let bytes = if source.is_local() {
        let path = AbsPath::new(crate::request::lexical_abs(&url)).map_err(Error::Core)?;
        let file = crate::fsx::ExternalReadFile::open_regular(&path).map_err(|error| {
            Error::UpdateUnavailable {
                reason: format!("读不了发布包 {url}：{error}"),
            }
        })?;
        file.read_bounded(crate::fsx::MAX_FILE_BYTES)
            .map_err(|error| Error::UpdateUnavailable {
                reason: format!("发布包 {url} 超过读取限额：{error}"),
            })?
    } else {
        let mut command = std::process::Command::new("curl");
        command.args(["-fsSL", &url]);
        let output = run_child_bounded(
            &mut command,
            None,
            &format!("下载 {url}"),
            crate::fsx::MAX_FILE_BYTES,
            MAX_TOOL_STDERR_BYTES,
        )?;
        if !output.status.success() {
            return Err(Error::UpdateUnavailable {
                reason: format!(
                    "下载 {url} 失败（curl 退出 {}）：{}",
                    output
                        .status
                        .code()
                        .map_or_else(|| "?".to_string(), |code| code.to_string()),
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }
        if output.stdout.len() as u64 > crate::fsx::MAX_FILE_BYTES {
            return Err(Error::UpdateUnavailable {
                reason: format!("发布包 {url} 超过读取限额"),
            });
        }
        output.stdout
    };
    crate::fsx::write_new_file(home, lock, &dst, &bytes)?;
    Ok(dst)
}

/// `curl -fsSL <url>` 取文本。
fn curl(url: &str) -> std::result::Result<String, String> {
    let mut command = std::process::Command::new("curl");
    command.args(["-fsSL", url]);
    let out = run_child_bounded(
        &mut command,
        None,
        &format!("取清单 {url}"),
        crate::fsx::MAX_FILE_BYTES,
        MAX_TOOL_STDERR_BYTES,
    )
    .map_err(|error| error.to_string())?;
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
/// 读取压缩包索引并只把唯一普通 `sheltie` 成员写入受管 tmp；从不让 tar 在管理根解包。
fn unpack_if_archive(
    home: &Home,
    lock: &crate::home::HomeLock,
    name: &str,
    archive: &[u8],
    tmp: &AbsPath,
) -> Result<(AbsPath, Vec<u8>)> {
    if !(name.ends_with(".tar.gz") || name.ends_with(".tgz") || name.ends_with(".tar.xz")) {
        let candidate = tmp.join_segment("sheltie-candidate");
        crate::fsx::write_new_file(home, lock, &candidate, archive)?;
        return Ok((candidate, archive.to_vec()));
    }
    let listed = run_tar_stdout(&["-tf", "-"], archive, crate::fsx::MAX_TOTAL_BYTES)?;
    let verbose = run_tar_stdout(&["-tvf", "-"], archive, crate::fsx::MAX_TOTAL_BYTES)?;
    let names = std::str::from_utf8(&listed).map_err(|error| Error::UpdateUnavailable {
        reason: format!("压缩包 {name} 的索引不是UTF-8：{error}"),
    })?;
    let details = std::str::from_utf8(&verbose).map_err(|error| Error::UpdateUnavailable {
        reason: format!("压缩包 {name} 的详细索引不是UTF-8：{error}"),
    })?;
    let entries = names.lines().collect::<Vec<_>>();
    let detail_lines = details.lines().collect::<Vec<_>>();
    if entries.len() != detail_lines.len() {
        return Err(Error::UpdateUnavailable {
            reason: format!("压缩包 {name} 的索引格式不一致"),
        });
    }
    let mut binary = None;
    for (entry, detail) in entries.into_iter().zip(detail_lines) {
        let kind = detail.as_bytes().first().copied();
        if !matches!(kind, Some(b'-' | b'd')) {
            return Err(Error::UpdateUnavailable {
                reason: format!("压缩包 {name} 含链接或特殊对象，拒绝解包"),
            });
        }
        let entry = entry.trim_end_matches('/');
        let rel =
            ManagedRelPath::new(entry.to_string()).map_err(|error| Error::UpdateUnavailable {
                reason: format!("压缩包成员路径 {entry:?} 不安全：{error}"),
            })?;
        if kind == Some(b'-')
            && entry.rsplit('/').next() == Some("sheltie")
            && binary.replace(rel).is_some()
        {
            return Err(Error::UpdateUnavailable {
                reason: format!("解包 {name} 后找到多个 sheltie 二进制"),
            });
        }
    }
    let binary = binary.ok_or_else(|| Error::UpdateUnavailable {
        reason: format!("解包 {name} 后找不到普通 sheltie 二进制"),
    })?;
    let binary = run_tar_stdout(
        &["-xOf", "-", "--", binary.as_str()],
        archive,
        crate::fsx::MAX_FILE_BYTES,
    )?;
    let extracted = tmp.join_segment("sheltie-unpacked");
    crate::fsx::write_new_file(home, lock, &extracted, &binary)?;
    Ok((extracted, binary))
}

fn run_tar_stdout(args: &[&str], archive: &[u8], max_bytes: u64) -> Result<Vec<u8>> {
    let mut child = std::process::Command::new("tar")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| Error::UpdateUnavailable {
            reason: format!("启动tar失败：{error}"),
        })?;
    let output = wait_bounded_child(&mut child, Some(archive), max_bytes, MAX_TOOL_STDERR_BYTES)
        .map_err(|error| Error::UpdateUnavailable {
            reason: format!("tar执行失败：{error}"),
        })?;
    if !output.status.success() {
        return Err(Error::UpdateUnavailable {
            reason: format!("tar失败：{}", String::from_utf8_lossy(&output.stderr)),
        });
    }
    Ok(output.stdout)
}

fn run_child_bounded(
    command: &mut std::process::Command,
    input: Option<&[u8]>,
    label: &str,
    max_stdout: u64,
    max_stderr: u64,
) -> Result<std::process::Output> {
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| Error::UpdateUnavailable {
        reason: format!("启动{label}失败：{error}"),
    })?;
    wait_bounded_child(&mut child, input, max_stdout, max_stderr).map_err(|error| {
        Error::UpdateUnavailable {
            reason: format!("{label}失败：{error}"),
        }
    })
}

fn wait_bounded_child(
    child: &mut std::process::Child,
    input: Option<&[u8]>,
    max_stdout: u64,
    max_stderr: u64,
) -> std::io::Result<std::process::Output> {
    let stop = Arc::new(AtomicBool::new(false));
    let stdout_stop = Arc::clone(&stop);
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("missing child stdout pipe"))?;
    let stdout_limit = usize::try_from(max_stdout.saturating_add(1)).unwrap_or(usize::MAX);
    let stdout_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Err(error) = stdout.take(stdout_limit as u64).read_to_end(&mut bytes) {
            stdout_stop.store(true, Ordering::Release);
            return Err(error);
        }
        if bytes.len() > stdout_limit.saturating_sub(1) {
            stdout_stop.store(true, Ordering::Release);
        }
        Ok::<_, std::io::Error>(bytes)
    });
    let stderr_stop = Arc::clone(&stop);
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("missing child stderr pipe"))?;
    let stderr_limit = usize::try_from(max_stderr.saturating_add(1)).unwrap_or(usize::MAX);
    let stderr_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Err(error) = stderr.take(stderr_limit as u64).read_to_end(&mut bytes) {
            stderr_stop.store(true, Ordering::Release);
            return Err(error);
        }
        if bytes.len() > stderr_limit.saturating_sub(1) {
            stderr_stop.store(true, Ordering::Release);
        }
        Ok::<_, std::io::Error>(bytes)
    });
    let input_thread = input.map(|bytes| {
        let mut stdin = child.stdin.take();
        let bytes = bytes.to_vec();
        let input_stop = Arc::clone(&stop);
        std::thread::spawn(move || {
            let result = stdin
                .take()
                .ok_or_else(|| std::io::Error::other("missing child stdin pipe"))?
                .write_all(&bytes);
            if result.is_err() {
                input_stop.store(true, Ordering::Release);
            }
            result
        })
    });
    let mut wait_error = None;
    let status = loop {
        if stop.load(Ordering::Acquire) {
            let _ = child.kill();
            match child.wait() {
                Ok(status) => break Some(status),
                Err(error) => {
                    wait_error = Some(error);
                    break None;
                }
            }
        }
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                let _ = child.kill();
                wait_error = Some(match child.wait() {
                    Ok(_) => error,
                    Err(wait_error) => std::io::Error::other(format!(
                        "child try_wait 失败：{error}；kill后wait也失败：{wait_error}"
                    )),
                });
                break None;
            }
        }
    };
    let stdout_result = match stdout_thread.join() {
        Ok(result) => result,
        Err(_) => Err(std::io::Error::other("stdout reader panicked")),
    };
    let stderr_result = match stderr_thread.join() {
        Ok(result) => result,
        Err(_) => Err(std::io::Error::other("stderr reader panicked")),
    };
    let input_result = input_thread.map(|thread| match thread.join() {
        Ok(result) => result,
        Err(_) => Err(std::io::Error::other("stdin writer panicked")),
    });
    if let Some(error) = wait_error {
        return Err(error);
    }
    let stdout = stdout_result?;
    let stderr = stderr_result?;
    if let Some(input_result) = input_result {
        input_result?;
    }
    if stdout.len() as u64 > max_stdout {
        return Err(std::io::Error::other(
            "child stdout exceeded configured limit",
        ));
    }
    if stderr.len() as u64 > max_stderr {
        return Err(std::io::Error::other(
            "child stderr exceeded configured limit",
        ));
    }
    Ok(std::process::Output {
        status: status.ok_or_else(|| std::io::Error::other("child status unavailable"))?,
        stdout,
        stderr,
    })
}

// ── 小件 ──────────────────────────────────────────────────────

fn path_hint(home: &Home) -> String {
    format!("把 {} 加进 PATH", home.bin_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task: C002-T23
    #[test]
    fn bounded_child_accepts_exact_stdout_limit_and_rejects_one_more() {
        let mut exact = std::process::Command::new("sh");
        exact.args(["-c", "printf 1234"]);
        let output = run_child_bounded(&mut exact, None, "测试子进程", 4, 16).unwrap();
        assert_eq!(output.stdout, b"1234");

        let mut oversized = std::process::Command::new("sh");
        oversized.args(["-c", "printf 12345"]);
        let error = run_child_bounded(&mut oversized, None, "测试子进程", 4, 16).unwrap_err();
        assert!(error.to_string().contains("stdout"));
    }

    // Task: C002-T23
    #[test]
    fn bounded_child_limits_stderr_as_well_as_stdout() {
        let mut oversized = std::process::Command::new("sh");
        oversized.args(["-c", "printf 1234 >&2"]);
        let error = run_child_bounded(&mut oversized, None, "测试子进程", 16, 3).unwrap_err();
        assert!(error.to_string().contains("stderr"));
    }
}
