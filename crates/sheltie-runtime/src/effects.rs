//! 效果登记与恢复（存储合同 §3.2、GF-31）。
//!
//! `requests.effects_json` 是效果对象数组：I/O 完成情况，不参与业务选边，不构成
//! 第二套 Work 状态。全部路径相对管理根；`write_file` 携带精确字节，历史任务书与
//! `engine/stats.json` 按提交时字节恢复，不从最新状态重算。恢复顺序按 `audit.seq`
//! 递增；同请求先 `publish_dir` / `prepare_attempt`，再历史文件、封存或删除，最后
//! `refresh_status_card`。

use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::path::AbsPath;

use crate::error::{Error, Result};
use crate::fsx;
use crate::home::Home;

/// `effects_json` 的一个效果对象。字段与存储合同 §3.2 的表逐项对应，未知字段拒绝。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectOp {
    /// `pending/<id>/payload/` → 最终目录的原子发布。
    PublishDir {
        pending: String,
        #[serde(rename = "final")]
        final_path: String,
        /// `work:<work_id>` 或 `workbook:<id>@<version>`。
        owner: String,
        /// `workbook-digest/v2`（Workbook 与 Work 冻结副本同口径）。
        digest: String,
        /// 摘要核算的子路径（相对 payload，空串为整棵）。Work 的 payload 含
        /// `workbook/` 与 `start-inputs/`，摘要只核 `workbook/`（存储合同 §3.2）。
        #[serde(default)]
        digest_root: String,
    },
    /// 在已提交 Attempt 下安全建立目录骨架。
    PrepareAttempt {
        work_id: String,
        attempt_id: String,
        /// 按父先于子排序的目录路径（相对管理根）。
        dirs: Vec<String>,
    },
    /// 精确字节的历史文件（任务书、`engine/stats.json`）。
    WriteFile {
        path: String,
        sha256: String,
        content: String,
    },
    /// 对原产物引用核对后置只读。
    SealOutputs { refs: Vec<RefJson> },
    /// 已核归属目录的移入与删除。
    DeleteDir {
        pending: String,
        #[serde(rename = "final")]
        final_path: String,
        owner: String,
        digest: String,
    },
    /// 从最新状态重新生成状态卡（当前投影，不是历史文件）。
    RefreshStatusCard { work_id: String },
}

/// 完整产物引用（协议 §6 形状；`seal_outputs` 用它核对原对象）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefJson {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

/// 把效果数组从 JSON 解出；结构不符报 `STORE_CORRUPT`，不猜默认值。
pub fn decode_effects(json: &str) -> Result<Vec<EffectOp>> {
    serde_json::from_str(json).map_err(|e| Error::StoreCorrupt {
        detail: format!("effects_json 解不开：{e}"),
    })
}

pub fn encode_effects(ops: &[EffectOp]) -> String {
    serde_json::to_string(ops).unwrap_or_else(|_| "[]".to_string())
}

/// 执行（或恢复）一批效果。幂等：每个动作先核对现状再动手；同对象视为已完成，
/// 不同对象报错，不覆盖、不删原件。全部成功返回 `Ok(())`；失败携带定位信息。
///
/// `publish` 是发布动作的开关：已 `published = 1` 的请求重放只核对 `write_file`，
/// 不重做发布/封存/删除（存储合同 §3.2 末段）。
pub fn execute(home: &Home, ops: &[EffectOp], publish: bool) -> Result<()> {
    for op in ops {
        match op {
            EffectOp::PublishDir {
                pending,
                final_path,
                owner,
                digest,
                digest_root,
            } => {
                if !publish {
                    continue;
                }
                publish_dir(home, pending, final_path, owner, digest, digest_root)?;
            }
            EffectOp::PrepareAttempt { dirs, .. } => {
                if !publish {
                    continue;
                }
                for rel in dirs {
                    fsx::ensure_dirs_under(home.root(), &home.rel(rel))?;
                }
            }
            EffectOp::WriteFile {
                path,
                sha256,
                content,
            } => {
                let target = home.rel(path);
                if target.as_path().exists() {
                    // 存在且摘要相同不写；不同是完整性错误，不掩盖修改。
                    let f = fsx::SafeFile::open_regular(&target)?;
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES)?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 的摘要与登记不符（被修改）"),
                        });
                    }
                } else {
                    // 父目录缺失或不可信时不臆造。
                    let parent = target
                        .as_path()
                        .parent()
                        .map(|p| p.to_path_buf())
                        .ok_or_else(|| Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 没有父目录"),
                        })?;
                    if !parent.exists() {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 的父目录缺失，不能恢复"),
                        });
                    }
                    fsx::write_exclusive_atomic(home.root(), &target, content.as_bytes())?;
                    let f = fsx::SafeFile::open_regular(&target)?;
                    let (got, _) = f.sha256_bounded(fsx::MAX_FILE_BYTES)?;
                    if got.as_str() != sha256 {
                        return Err(Error::StoreCorrupt {
                            detail: format!("历史文件 {path} 恢复后摘要与登记不符"),
                        });
                    }
                }
            }
            EffectOp::SealOutputs { refs } => {
                if !publish {
                    continue;
                }
                for r in refs {
                    let target = home.rel(&r.path);
                    let f = fsx::SafeFile::open_regular(&target)?;
                    let (got, bytes) = f.sha256_bounded(fsx::MAX_FILE_BYTES)?;
                    if got.as_str() != r.sha256 || bytes != r.bytes {
                        return Err(Error::StoreCorrupt {
                            detail: format!(
                                "封存目标 {} 与提交时引用不符（不存在或被改变）",
                                r.path
                            ),
                        });
                    }
                    f.set_readonly()?;
                }
            }
            EffectOp::DeleteDir {
                pending,
                final_path,
                owner,
                digest,
            } => {
                if !publish {
                    continue;
                }
                let _ = owner;
                delete_dir(home, pending, final_path, digest)?;
            }
            EffectOp::RefreshStatusCard { .. } => {
                // 状态卡是当前投影：由调用方（持有 Store）从最新 state_json 生成，
                // 不在本执行器里读库，也不保存历史卡字节。
            }
        }
    }
    Ok(())
}

/// `publish_dir`：`final` 不存在且 `pending` 在 → 核原件归属与摘要后 rename 并置
/// 只读；仅 `final` 在 → 核归属与摘要后视为完成；两者都在或内容不符 → 停止。
fn publish_dir(
    home: &Home,
    pending: &str,
    final_path: &str,
    owner: &str,
    digest: &str,
    digest_root: &str,
) -> Result<()> {
    // 摘要核算的是原件的某个子路径（Work 的 payload 含 workbook/ 与 start-inputs/，
    // 摘要只核 workbook/，存储合同 §3.2）；发布动作移动的是**整个 payload**。
    let payload = home.rel(pending);
    let verify_at = if digest_root.is_empty() {
        payload.clone()
    } else {
        payload.join_segment(digest_root)
    };
    let dst = home.rel(final_path);
    match (payload.as_path().exists(), dst.as_path().exists()) {
        (true, false) => {
            verify_owned_digest(&verify_at, owner, digest)?;
            if let Some(parent) = dst.as_path().parent() {
                fsx::ensure_dirs_under(
                    home.root(),
                    &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
                )?;
            }
            std::fs::rename(payload.as_path(), dst.as_path())
                .map_err(|e| Error::io(dst.as_str(), e))?;
            fsx::fsync_dir(&dst);
            // 只读化只属于 Workbook 目录（合同 §5.2：目录含根 0555、文件 0444）。
            // Work 目录要保持可写——状态卡与 Attempt 目录随后还要写入；冻结副本
            // `workbook/` 子树已在提交前置只读（§5.4）。
            if owner.starts_with("workbook:") {
                fsx::set_tree_readonly_confined(&dst)?;
            }
            Ok(())
        }
        (false, true) => {
            // 仅最终对象在：核最终对象的归属与摘要后视为完成（恢复语义 §3.1）；
            // Work 的摘要只核 `digest_root`（workbook/）子路径。
            let final_verify = if digest_root.is_empty() {
                dst.clone()
            } else {
                dst.join_segment(digest_root)
            };
            verify_owned_digest(&final_verify, owner, digest)?;
            Ok(())
        }
        (false, false) => Err(Error::StoreCorrupt {
            detail: format!("发布对象 {final_path} 与原件 {pending} 都不存在"),
        }),
        (true, true) => Err(Error::StoreCorrupt {
            detail: format!("发布对象 {final_path} 已存在且原件 {pending} 仍在，不能覆盖"),
        }),
    }
}

/// 核对目录摘要与归属标记（侧车）；不符报 `STORE_CORRUPT`。
fn verify_owned_digest(dir: &AbsPath, owner: &str, digest: &str) -> Result<()> {
    let got = crate::workbook_digest::digest_dir_v2(dir).map_err(|e| Error::StoreCorrupt {
        detail: format!("发布原件 {dir} 读不了：{e}"),
    })?;
    if got.as_str() != digest {
        return Err(Error::StoreCorrupt {
            detail: format!("发布原件 {dir} 的摘要与登记不符"),
        });
    }
    // 归属由 pending 侧车与 Store 引用共同承担；owner 串只做存在性核对。
    if owner.is_empty() {
        return Err(Error::StoreCorrupt {
            detail: "发布效果缺归属".to_string(),
        });
    }
    Ok(())
}

/// `delete_dir`（存储合同 §3.2/§3.3）：最终目录仍在时核身份与摘要——**只有登记的
/// 那个对象**才移入本操作 pending 并删除；摘要不符说明那是别人的新生命周期对象
///（同版本重新 add），本操作的删除视为已完成，不碰它。移入后只删同一对象；完成
/// 后写 `.deleted` 持久标记，两处都缺时只有合法标记才能证明完成。
fn delete_dir(home: &Home, pending: &str, final_path: &str, digest: &str) -> Result<()> {
    let fin = home.rel(final_path);
    let pen = home.rel(pending);
    let internal_id = pen
        .as_path()
        .file_name()
        .map(|n| n.to_string())
        .unwrap_or_default();
    if fin.as_path().exists() && !digest.is_empty() {
        let got = crate::workbook_digest::digest_dir_v2(&fin).map_err(|e| Error::StoreCorrupt {
            detail: format!("删除对象 {fin} 读不了：{e}"),
        })?;
        if got.as_str() != digest {
            // 不是登记要删的对象（新生命周期或外部替换）：不删、不覆盖。
            write_deleted_marker(home, &internal_id)?;
            return Ok(());
        }
    }
    if fin.as_path().exists() {
        fsx::make_tree_writable(&fin);
        if let Some(parent) = pen.as_path().parent() {
            fsx::ensure_dirs_under(
                home.root(),
                &AbsPath::new(parent.to_string()).map_err(Error::Core)?,
            )?;
        }
        std::fs::rename(fin.as_path(), pen.as_path()).map_err(|e| Error::io(fin.as_str(), e))?;
    }
    if pen.as_path().exists() {
        fsx::remove_tree_no_follow(&pen)?;
    }
    // 两处都缺时本函数的执行本身就是完成证明：写持久标记（§3.3）。
    write_deleted_marker(home, &internal_id)?;
    Ok(())
}

/// 独占创建并 fsync `pending/<internal_id>.deleted` 完成标记（§3.3）。
fn write_deleted_marker(home: &Home, internal_id: &str) -> Result<()> {
    if internal_id.is_empty() {
        return Ok(());
    }
    let marker = home
        .pending_dir()
        .join_segment(&format!("{internal_id}.deleted"));
    if marker.as_path().exists() {
        return Ok(());
    }
    let content =
        format!("{{\"format\":\"delete-complete/v1\",\"internal_id\":\"{internal_id}\"}}\n");
    fsx::write_new_file(&marker, content.as_bytes())?;
    fsx::fsync_dir(&home.pending_dir());
    Ok(())
}

/// 独立校验一个 sha256 串。
pub fn valid_digest(s: &str) -> bool {
    Sha256Hex::new(s).is_ok()
}
