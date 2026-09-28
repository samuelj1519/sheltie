//! `workbook-digest/v2` 的目录摘要单元（存储合同 §5.1）。
//!
//! 精确字节流：域前缀、BE64 文件数、按规范 UTF-8 相对路径字节序排序的每个普通文件
//! `BE64(路径长) || 路径 || BE64(内容长) || 内容`，最后只做一次 SHA256。流式读取并
//! 准确计数，读取间的增长或收缩都会被拒，不能用更大的内容绕过限额。
//!
//! 生产调用方（`WorkbookRepo::digest_dir` 等 schema 1 路径）按 C002 计划在 T07 一次
//! 切换到本单元；切换前旧算法不与新算法并存于同一字段。

use std::io::Read;

use sha2::Digest as _;
use sheltie_core::digest::{Sha256Hex, WORKBOOK_DIGEST_V2_PREFIX, be64, digest_v2_file_frame};
use sheltie_core::path::{AbsPath, RelPath};

use crate::error::{Error, Result};
use crate::workbook_repo::{MAX_FILE_BYTES, MAX_TOTAL_BYTES};

/// 目录摘要 `workbook-digest/v2`。
pub fn digest_dir_v2(dir: &AbsPath) -> Result<Sha256Hex> {
    let files = collect_regular_files(dir)?;
    // 限额在读取前核对（§5.1）；读取间的增长由逐文件计数兜住。
    let mut total = 0u64;
    for (rel, size) in &files {
        if size > &MAX_FILE_BYTES {
            return Err(Error::InvalidRequest {
                reason: format!("{} 下 {rel} 超过 {} 字节", dir, MAX_FILE_BYTES),
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

    let mut hasher = sha2::Sha256::new();
    hasher.update(WORKBOOK_DIGEST_V2_PREFIX);
    hasher.update(be64(files.len() as u64));
    for (rel, declared) in files {
        // 相对路径以 UTF-8 字节排序，不做 Unicode/大小写转换。
        let path_bytes = rel.as_str().as_bytes().to_vec();
        hasher.update(digest_v2_file_frame(&path_bytes, declared));
        stream_file_into(&dir.join(&rel), declared, &mut hasher)?;
    }
    // 单次 SHA256 收口：finalize 的输出直接转十六进制，不再二次哈希。
    Ok(Sha256Hex::from_sha256(hasher.finalize()))
}

/// 收集目录下全部普通文件的 `(相对路径, 声明字节数)`，按相对路径字节序排序。
/// 空目录不参与摘要；拒绝符号链接、硬链接与特殊文件；路径段不得为 `.`、`..` 或空。
fn collect_regular_files(dir: &AbsPath) -> Result<Vec<(RelPath, u64)>> {
    let mut out = Vec::new();
    walk_regular_files(dir, dir, &mut out)?;
    out.sort_by(|a, b| a.0.as_str().as_bytes().cmp(b.0.as_str().as_bytes()));
    Ok(out)
}

fn walk_regular_files(root: &AbsPath, dir: &AbsPath, out: &mut Vec<(RelPath, u64)>) -> Result<()> {
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
            walk_regular_files(root, &path, out)?;
            continue;
        }
        if !ft.is_file() {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 不是普通文件"),
            });
        }
        let meta = entry.metadata().map_err(|e| Error::io(path.as_str(), e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            if meta.nlink() > 1 {
                return Err(Error::InvalidRequest {
                    reason: format!("{path} 是硬链接"),
                });
            }
        }
        let rel = path
            .as_path()
            .strip_prefix(root.as_path())
            .map_err(|e| Error::io(root.as_str(), std::io::Error::other(e.to_string())))?
            .to_string();
        let rel = RelPath::new(rel).map_err(Error::Core)?;
        out.push((rel, meta.len()));
    }
    Ok(())
}

/// 把一个文件的内容流式喂进哈希器，并核对实际字节数与帧头声明一致。
/// 读取间增长或收缩都拒绝：帧头的长度是摘要流的一部分，事后无法补写。
fn stream_file_into(path: &AbsPath, declared: u64, hasher: &mut sha2::Sha256) -> Result<()> {
    let mut file = std::fs::File::open(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
    let mut actual = 0u64;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| Error::io(path.as_str(), e))?;
        if n == 0 {
            break;
        }
        actual = actual
            .checked_add(n as u64)
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{path} 读取字节数溢出"),
            })?;
        if actual > declared || actual > MAX_FILE_BYTES {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 在读取间变大，与帧头不符"),
            });
        }
        hasher.update(&buf[..n]);
    }
    if actual != declared {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 实际 {actual} 字节与帧头声明 {declared} 不符（读取间变化）"),
        });
    }
    Ok(())
}
