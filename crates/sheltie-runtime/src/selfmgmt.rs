//! Binary self-management: self install | update | rollback | uninstall | version.
//! Rules are in storage contract §9 and protocol self commands.

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

/// Release manifest source: GitHub Releases by default; tests point SHELTIE_RELEASE_BASE to a local directory.
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

    /// Manifest URL under a release identity; tag is latest or v<version> (storage contract §9, step 1).
    /// Local tag directories mirror remote releases; remote latest is mutable, pinned tags use download paths.
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

/// Current platform: Rust target triple, matching release filenames (D-30).
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

/// self version: read-only, without requiring an existing management root.
pub fn version_info(home: &Home) -> VersionInfo {
    VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform(),
        home: home.root().clone(),
        schema_version: crate::store::SCHEMA_VERSION,
    }
}

/// self install copies the running executable to bin/sheltie, staging in tmp/ before rename.
/// Identical existing bytes yield already_installed = true. Never change shell configuration; PATH advice
/// is text only (storage contract §9, protocol §3). Writes hold the root lock (§2.2).
pub fn install(home: &Home) -> Result<InstallOutcome> {
    match crate::store::Store::open_for_home(home, crate::store::OpenMode::ReadOnly) {
        Ok(store) => drop(store),
        Err(Error::NotFound { .. }) => {}
        Err(error) => return Err(error),
    }
    let current = std::env::current_exe().map_err(|e| Error::io("current_exe", e))?;
    let exe = crate::fsx::ExternalReadFile::open_regular(
        &AbsPath::new(
            current
                .to_str()
                .ok_or_else(|| Error::InvalidRequest {
                    reason: format!(
                        "Current executable {} is not a UTF-8 path",
                        current.display()
                    ),
                })?
                .to_string(),
        )
        .map_err(Error::Core)?,
    )?;
    let bytes = exe.read_bounded(crate::fsx::MAX_FILE_BYTES)?;
    let session = crate::session::WriteSession::open_or_create(home)?;
    let lock = session.lock;
    let bin = home.bin_dir();
    let target = bin.join_segment("sheltie");
    // Create store.db (self install requires database initialization as well as copying);
    // initialize before the idempotent shortcut so an identical existing binary does not skip missing storage.
    // Exclusively create when absent; existing objects must be managed regular singly linked files, never treating symlinks as
    // a different version to overwrite. Identical bytes remain idempotent.
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
                "self operation failed: {original}; temporary-directory cleanup also failed: {cleanup}"
            )),
        ),
    }
}

/// self update follows six storage §9 steps; None version resolves latest and pins that release.
pub fn update(home: &Home, source: &ReleaseSource, version: Option<&str>) -> Result<UpdateOutcome> {
    let lock = home.acquire_lock()?;
    // Step 1: resolve and pin one tag; obtain both manifest and assets from that identity.
    let tag = resolve_tag(source, version)?;
    // Step 2: read the pinned manifest and locate the platform artifact and sha256.
    let manifest = read_manifest(source, &tag)?;
    let pinned = tag_version(&tag);
    if manifest.version != pinned {
        return Err(Error::UpdateUnavailable {
            reason: format!(
                "Tag {tag} manifest declares version {}, which does not match the tag",
                manifest.version
            ),
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
            reason: format!("Release manifest has no artifact for {platform_str}"),
        });
    };
    // Asset names must join safely beneath tmp/<uuid>/; forged manifests cannot escape the management root.
    let name = checked_asset_name(&asset.name)?;
    // Step 3: download into managed tmp, verify digest, and safely read the archive candidate.
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
                reason: format!("Cannot read release artifact {name}: {error}"),
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
                // Atomic exchange moves current to .prev; keep old .prev in tmp until the new binary is installed.
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
                                "Moving old .prev during update failed: {error}; destination, prev, and temporary backups could not be restored and are preserved: {restore_error}"
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

        // Existing crash tests verify this point: destination may be absent, but .prev holds the original current binary.
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
                            "New binary installation failed: {error}; original endpoint restoration could not be verified; preserve prev and temporary directories: {restore_error}"
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
                                "Original binary restored; old .prev is at {displaced_prev}; preserve temporary directory: {restore_error}"
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

/// self rollback restores sheltie.prev, moving directly if bin/sheltie is absent; no .prev yields NotFound.
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
                        "Rollback replacement failed: {error}; current binary is at {saved_current}; restoration failed: {restore}"
                    ),
                }),
            },
        }
    } else {
        crate::fsx::rename_managed_new(home, &lock, &home.to_rel(&prev)?, &home.to_rel(&target)?)
    }
}

/// self uninstall removes only bin/ by default; purge clears managed data, preserving the root and the same .lock.
/// Unconfirmed requests yield InvalidRequest; return retained user-data paths, empty after successful purge.
/// Writes hold the root lock; waiting callers recheck root and .lock identity after acquisition.
pub fn uninstall(home: &Home, purge: bool, confirmed: bool) -> Result<Vec<AbsPath>> {
    if purge && !confirmed {
        return Err(Error::InvalidRequest {
            reason: "Uninstall and purge require confirmation (type yes interactively; use --yes with --json)".to_string(),
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

// ── Release identity (storage contract §9, step 1) ────────────────────────────────

/// Resolve release identity once: --version yields v<version>; otherwise read
/// latest's manifest, extract the version, and pin v<version>. Fetch all subsequent manifest/assets from that tag,
/// because latest may change between requests, causing inconsistent release assets.
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

/// Strip v from the v<version> tag to obtain the version.
fn tag_version(tag: &str) -> String {
    tag.strip_prefix('v').unwrap_or(tag).to_string()
}

/// Versions allow one path segment only, without /, .., ., or NUL, preventing URL/path escapes.
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
            reason: format!("Version {raw} is not valid"),
        });
    }
    Ok(v.to_string())
}

/// Similarly, accept only one asset filename; reject .. and separators to confine forged paths.
fn checked_asset_name(raw: &str) -> Result<String> {
    if raw.is_empty()
        || raw == "."
        || raw == ".."
        || raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\0')
    {
        return Err(Error::UpdateUnavailable {
            reason: format!("Release artifact name {raw} is not a valid filename"),
        });
    }
    Ok(raw.to_string())
}

// ── Release manifest ──────────────────────────────────────────────────

/// Compact manifest format (storage contract §9), used by local releases and tests.
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

/// Adapted manifest: both sources converge on this shape.
struct ReleaseManifest {
    version: String,
    assets: Vec<SlimAsset>,
}

/// Read a pinned manifest; try compact format, then adapt cargo-dist's full dist-manifest.json.
/// Full-format fields lack automated coverage; T25's actual update manually checked them (D-30).
fn read_manifest(source: &ReleaseSource, tag: &str) -> Result<ReleaseManifest> {
    let url = source.manifest_url(tag);
    let text = if source.is_local() {
        let path = AbsPath::new(crate::request::lexical_abs(&url)?).map_err(Error::Core)?;
        let file = crate::fsx::ExternalReadFile::open_regular(&path).map_err(|error| {
            Error::UpdateUnavailable {
                reason: format!("Cannot read release manifest {url}: {error}"),
            }
        })?;
        let bytes = file
            .read_bounded(crate::fsx::MAX_FILE_BYTES)
            .map_err(|error| Error::UpdateUnavailable {
                reason: format!("Release manifest {url} exceeds the read limit: {error}"),
            })?;
        String::from_utf8(bytes).map_err(|error| Error::UpdateUnavailable {
            reason: format!("Release manifest {url} is not UTF-8: {error}"),
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

/// Adapt cargo-dist to compact format: version from announcement_tag without v,
/// executable-zip assets, and the first target_triples platform.
/// Actual 0.32 shape, checked against T25 dist build: artifacts is a name-keyed
/// object (arrays also accepted); the real digest is checksums.sha256, while checksum names the
/// checksum file, not the hash. Accept bare strings as hashes only for exactly 64 hexadecimal digits.
fn adapt_cargo_dist_manifest(text: &str) -> Result<ReleaseManifest> {
    let bad = || Error::UpdateUnavailable {
        reason: "Release manifest is neither compact format nor valid cargo-dist fields"
            .to_string(),
    };
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| Error::UpdateUnavailable {
            reason: format!("Release manifest is not valid JSON: {e}"),
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

/// Download a pinned release artifact to tmp/<name>; copy local releases directly, use system curl remotely.
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
        let path = AbsPath::new(crate::request::lexical_abs(&url)?).map_err(Error::Core)?;
        let file = crate::fsx::ExternalReadFile::open_regular(&path).map_err(|error| {
            Error::UpdateUnavailable {
                reason: format!("Cannot read release artifact {url}: {error}"),
            }
        })?;
        file.read_bounded(crate::fsx::MAX_FILE_BYTES)
            .map_err(|error| Error::UpdateUnavailable {
                reason: format!("Release artifact {url} exceeds the read limit: {error}"),
            })?
    } else {
        let mut command = std::process::Command::new("curl");
        command.args(["-fsSL", &url]);
        let output = run_child_bounded(
            &mut command,
            None,
            &format!("Download {url}"),
            crate::fsx::MAX_FILE_BYTES,
            MAX_TOOL_STDERR_BYTES,
        )?;
        if !output.status.success() {
            return Err(Error::UpdateUnavailable {
                reason: format!(
                    "Download {url} failed (curl exit {}): {}",
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
                reason: format!("Release artifact {url} exceeds the read limit"),
            });
        }
        output.stdout
    };
    crate::fsx::write_new_file(home, lock, &dst, &bytes)?;
    Ok(dst)
}

/// Fetch text with curl -fsSL <url>.
fn curl(url: &str) -> std::result::Result<String, String> {
    let mut command = std::process::Command::new("curl");
    command.args(["-fsSL", url]);
    let out = run_child_bounded(
        &mut command,
        None,
        &format!("Fetch manifest {url}"),
        crate::fsx::MAX_FILE_BYTES,
        MAX_TOOL_STDERR_BYTES,
    )
    .map_err(|error| error.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "curl exit {}: {}",
            out.status
                .code()
                .map_or_else(|| "?".to_string(), |c| c.to_string()),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("Manifest is not UTF-8: {e}"))
}

/// Fetch a file with curl -fsSL <url> -o <dst>.
/// Inspect the archive index and write only the unique regular sheltie member into managed tmp; never extract tar into the root.
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
        reason: format!("Archive {name} index is not UTF-8: {error}"),
    })?;
    let details = std::str::from_utf8(&verbose).map_err(|error| Error::UpdateUnavailable {
        reason: format!("Archive {name} detailed index is not UTF-8: {error}"),
    })?;
    let entries = names.lines().collect::<Vec<_>>();
    let detail_lines = details.lines().collect::<Vec<_>>();
    if entries.len() != detail_lines.len() {
        return Err(Error::UpdateUnavailable {
            reason: format!("Archive {name} index formats do not match"),
        });
    }
    let mut binary = None;
    for (entry, detail) in entries.into_iter().zip(detail_lines) {
        let kind = detail.as_bytes().first().copied();
        if !matches!(kind, Some(b'-' | b'd')) {
            return Err(Error::UpdateUnavailable {
                reason: format!(
                    "Archive {name} contains links or special objects; extraction rejected"
                ),
            });
        }
        let entry = entry.trim_end_matches('/');
        let rel =
            ManagedRelPath::new(entry.to_string()).map_err(|error| Error::UpdateUnavailable {
                reason: format!("Unsafe archive member path {entry:?}: {error}"),
            })?;
        if kind == Some(b'-')
            && entry.rsplit('/').next() == Some("sheltie")
            && binary.replace(rel).is_some()
        {
            return Err(Error::UpdateUnavailable {
                reason: format!("Archive {name} contains multiple sheltie binaries"),
            });
        }
    }
    let binary = binary.ok_or_else(|| Error::UpdateUnavailable {
        reason: format!("Archive {name} contains no regular sheltie binary"),
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
            reason: format!("Failed to start tar: {error}"),
        })?;
    let output = wait_bounded_child(&mut child, Some(archive), max_bytes, MAX_TOOL_STDERR_BYTES)
        .map_err(|error| Error::UpdateUnavailable {
            reason: format!("tar execution failed: {error}"),
        })?;
    if !output.status.success() {
        return Err(Error::UpdateUnavailable {
            reason: format!("tar failed: {}", String::from_utf8_lossy(&output.stderr)),
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
        reason: format!("Failed to start {label}: {error}"),
    })?;
    wait_bounded_child(&mut child, input, max_stdout, max_stderr).map_err(|error| {
        Error::UpdateUnavailable {
            reason: format!("{label} failed: {error}"),
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
                        "child try_wait failed: {error}; wait after kill also failed: {wait_error}"
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

// ── Helpers ──────────────────────────────────────────────────────

fn path_hint(home: &Home) -> String {
    format!("Add {} to PATH", home.bin_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task: C002-T47
    #[test]
    fn asset_names_preserve_valid_single_segments_and_reject_forbidden_paths() {
        for valid in [
            "artifact",
            "sheltie-cli-1.2.3-aarch64-apple-darwin.tar.xz",
            "结果.tar",
        ] {
            assert_eq!(checked_asset_name(valid).unwrap(), valid);
        }
        for invalid in ["", ".", "..", "dir/name", "dir\\name", "nul\0name"] {
            assert!(
                matches!(
                    checked_asset_name(invalid),
                    Err(Error::UpdateUnavailable { .. })
                ),
                "{invalid:?}"
            );
        }
    }

    // Task: C002-T47
    #[test]
    fn exact_child_pipe_limits_do_not_kill_a_child_waiting_for_explicit_release() {
        use std::time::{Duration, Instant};
        struct ReleaseOnDrop<'a>(&'a std::path::Path);
        impl Drop for ReleaseOnDrop<'_> {
            fn drop(&mut self) {
                let _ = std::fs::write(self.0, b"release");
            }
        }
        for (emit, stdout, stderr) in [
            ("printf abcd", b"abcd".as_slice(), b"".as_slice()),
            ("printf wxyz >&2", b"".as_slice(), b"wxyz".as_slice()),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let reached = directory.path().join("pipes-closed");
            let release = directory.path().join("release");
            let script = format!(
                "{emit}; exec 1>&-; exec 2>&-; touch \"$1\"; n=0; while [ ! -f \"$2\" ]; do n=$((n+1)); [ \"$n\" -le 300 ] || exit 73; sleep 0.01; done"
            );
            let (reached_before_release, ended_early, release_result, result) =
                std::thread::scope(|scope| {
                    let _release_on_drop = ReleaseOnDrop(&release);
                    let reached_arg = &reached;
                    let release_arg = &release;
                    let worker = scope.spawn(move || {
                        let mut command = std::process::Command::new("sh");
                        command
                            .arg("-c")
                            .arg(script)
                            .arg("fixture")
                            .arg(reached_arg)
                            .arg(release_arg);
                        run_child_bounded(&mut command, None, "held fixture", 4, 4)
                    });
                    let deadline = Instant::now() + Duration::from_secs(3);
                    while !reached.exists() && !worker.is_finished() && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    let reached_before_release = reached.exists();
                    let deadline = Instant::now() + Duration::from_millis(250);
                    while !worker.is_finished() && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    let ended_early = worker.is_finished();
                    let release_result = std::fs::write(&release, b"release");
                    (
                        reached_before_release,
                        ended_early,
                        release_result,
                        worker.join(),
                    )
                });
            release_result.unwrap();
            let result = result.unwrap();
            assert!(reached_before_release, "fixture never closed its pipes");
            assert!(
                !ended_early,
                "the child was terminated before explicit release: {result:?}"
            );
            let output = result.unwrap();
            assert!(output.status.success(), "{output:?}");
            assert_eq!(output.stdout, stdout);
            assert_eq!(output.stderr, stderr);
        }
    }

    // Task: C002-T23
    #[test]
    fn bounded_child_accepts_exact_stdout_limit_and_rejects_one_more() {
        let mut exact = std::process::Command::new("sh");
        exact.args(["-c", "printf 1234"]);
        let output = run_child_bounded(&mut exact, None, "test subprocess", 4, 16).unwrap();
        assert_eq!(output.stdout, b"1234");

        let mut oversized = std::process::Command::new("sh");
        oversized.args(["-c", "printf 12345"]);
        let error = run_child_bounded(&mut oversized, None, "test subprocess", 4, 16).unwrap_err();
        assert!(error.to_string().contains("stdout"));
    }

    // Task: C002-T23
    #[test]
    fn bounded_child_limits_stderr_as_well_as_stdout() {
        let mut oversized = std::process::Command::new("sh");
        oversized.args(["-c", "printf 1234 >&2"]);
        let error = run_child_bounded(&mut oversized, None, "test subprocess", 16, 3).unwrap_err();
        assert!(error.to_string().contains("stderr"));
    }
}
