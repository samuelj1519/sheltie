//! 管理根内的受限文件操作（`INV-3`、GF-32、存储合同 §4）。
//!
//! 统一入口：祖先链软链检查、句柄身份核对、限额读取与同一句柄封存、
//! 独占临时名原子写、不跟随软链的删除。观察、限额、摘要与封存作用在
//! 同一个打开的文件对象上；不能「检查路径、重开另一对象、再 chmod」。

use std::io::{Read, Seek, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use sheltie_core::digest::Sha256Hex;
use sheltie_core::path::AbsPath;

use crate::error::{Error, Result};

/// 单文件上限 32 MiB（存储合同 §5.2，与 workbook_repo 一致）。
pub const MAX_FILE_BYTES: u64 = 33_554_432;
/// 目录总量上限 256 MiB。
pub const MAX_TOTAL_BYTES: u64 = 268_435_456;
/// 明确列举的宿主元数据文件（存储合同 §5.3）：Finder 一类工具生成，不属于 Workbook
/// 字节。复制与读取都**准确拒绝并点名文件**，不静默忽略字节。
pub const HOST_METADATA_FILES: &[&str] = &[".DS_Store"];

/// 安全打开的叶文件句柄。
///
/// 打开即核对身份：`symlink_metadata` 拒绝软链与非常规文件并核对硬链计数，随后
/// 打开并在**句柄上** `metadata()` 比对 dev/ino——只有打开到的就是刚才核对的那个
/// 对象才可用。之后的大小、读取、摘要与置只读全部用同一句柄；路径上再怎么替换，
/// 读到的仍是核对过的对象（GF-32 的「同一观测对象」）。
#[derive(Debug)]
pub struct SafeFile {
    file: std::fs::File,
    path: AbsPath,
    meta: std::fs::Metadata,
}

impl SafeFile {
    /// 打开并核对一个普通文件。不存在返回 `Err(NotFound)`。
    pub fn open_regular(path: &AbsPath) -> Result<Self> {
        let stat = std::fs::symlink_metadata(path.as_path()).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error::NotFound {
                what: path.to_string(),
            },
            _ => Error::io(path.as_str(), e),
        })?;
        let ft = stat.file_type();
        if ft.is_symlink() {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 是符号链接"),
            });
        }
        if ft.is_dir() {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 是目录"),
            });
        }
        if !ft.is_file() {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 不是普通文件"),
            });
        }
        #[cfg(unix)]
        if stat.nlink() > 1 {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 是硬链接（nlink = {}）", stat.nlink()),
            });
        }
        let file = std::fs::File::open(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
        // 句柄上的事实；与打开前的 stat 比对身份，堵住 stat→open 之间的替换窗口。
        let meta = file.metadata().map_err(|e| Error::io(path.as_str(), e))?;
        #[cfg(unix)]
        if meta.dev() != stat.dev() || meta.ino() != stat.ino() {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 在打开期间被替换（对象身份不符）"),
            });
        }
        Ok(Self {
            file,
            path: path.clone(),
            meta,
        })
    }

    /// 不存在返回 `Ok(None)`，其他错误照常。
    pub fn open_optional(path: &AbsPath) -> Result<Option<Self>> {
        match Self::open_regular(path) {
            Ok(f) => Ok(Some(f)),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 句柄元数据（句柄上的事实，不是路径上的）。
    pub fn metadata(&self) -> &std::fs::Metadata {
        &self.meta
    }

    pub fn path(&self) -> &AbsPath {
        &self.path
    }

    /// 流式读取全部内容，先按句柄大小核对上限，读到的字节数必须与句柄元数据一致
    ///（读取间增长或收缩都拒绝）。每次读都从对象起点开始，同一句柄可重复读取。
    pub fn read_bounded(&self, max_bytes: u64) -> Result<Vec<u8>> {
        if self.meta.len() > max_bytes {
            return Err(Error::InvalidRequest {
                reason: format!("{} 超过 {} 字节", self.path, max_bytes),
            });
        }
        let mut buf = Vec::with_capacity(self.meta.len().min(1 << 20) as usize);
        let mut chunk = vec![0u8; 64 * 1024];
        let mut handle = &self.file;
        handle
            .rewind()
            .map_err(|e| Error::io(self.path.as_str(), e))?;
        loop {
            let n = handle
                .read(&mut chunk)
                .map_err(|e| Error::io(self.path.as_str(), e))?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if buf.len() as u64 > max_bytes {
                return Err(Error::InvalidRequest {
                    reason: format!("{} 在读取间增长，超过 {} 字节", self.path, max_bytes),
                });
            }
        }
        if buf.len() as u64 != self.meta.len() {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "{} 实际 {} 字节与句柄元数据 {} 不符（读取间变化）",
                    self.path,
                    buf.len(),
                    self.meta.len()
                ),
            });
        }
        Ok(buf)
    }

    /// 同一对象上的摘要与字节数（先限额再流式）。
    pub fn sha256_bounded(&self, max_bytes: u64) -> Result<(Sha256Hex, u64)> {
        let bytes = self.read_bounded(max_bytes)?;
        let n = bytes.len() as u64;
        Ok((Sha256Hex::of_bytes(&bytes), n))
    }

    /// 在同一句柄上置只读（0444）。封存以记录的 sha256 为准，只读位只是减少误写。
    pub fn set_readonly(&self) -> Result<()> {
        self.file
            .set_permissions(std::fs::Permissions::from_mode(0o444))
            .map_err(|e| Error::io(self.path.as_str(), e))
    }
}

/// 确保从管理根 `root` 到 `dir` 的路径可用：根本身（连同缺失的宿主祖先）可以创建
/// ——宿主层的软链不归引擎管，`Home` 已在入口把根规范到真实形式；根**以下**已存在的
/// 段必须是真实目录（软链拒绝），缺失的段逐段 `create_dir`。这样 `works`、`bin`、
/// `pending` 之类的管理子目录被换成软链时，引擎不会沿它写到根外（O01）。
pub fn ensure_dirs_under(root: &AbsPath, dir: &AbsPath) -> Result<()> {
    if dir.as_path() != root.as_path() && !dir.as_path().starts_with(root.as_path()) {
        return Err(Error::InvalidRequest {
            reason: format!("{dir} 不在管理根 {root} 之下"),
        });
    }
    if !root.as_path().exists() {
        std::fs::create_dir_all(root.as_path()).map_err(|e| Error::io(root.as_str(), e))?;
    }
    let rel = dir
        .as_path()
        .strip_prefix(root.as_path())
        .map_err(|e| Error::io(root.as_str(), std::io::Error::other(e.to_string())))?;
    let mut current = root.as_path().to_path_buf();
    for segment in rel.components() {
        let camino::Utf8Component::Normal(name) = segment else {
            continue;
        };
        current = current.join(name);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(Error::InvalidRequest {
                        reason: format!("{current} 是符号链接，不能作为管理目录"),
                    });
                }
                if !meta.is_dir() {
                    return Err(Error::InvalidRequest {
                        reason: format!("{current} 不是目录"),
                    });
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&current).map_err(|e| Error::io(current.as_str(), e))?;
            }
            Err(e) => return Err(Error::io(current.as_str(), e)),
        }
    }
    Ok(())
}

/// 原子写：在目标同目录**独占创建**唯一临时名（已存在或软链都直接失败，不跟随），
/// 写入并 `fsync` 临时文件，`rename` 到目标，再 `fsync` 父目录使目录项持久。
/// 临时名是随机 UUID，不在路径上可预测，也不保留固定后缀（O01 的 tmp-pending 缺口）。
/// `base` 是可信基点（管理根或其下已验证的目录）；目标父目录在基点之下逐段核对。
pub fn write_exclusive_atomic(base: &AbsPath, path: &AbsPath, content: &[u8]) -> Result<()> {
    let parent = path
        .as_path()
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| Error::InvalidRequest {
            reason: format!("{path} 没有父目录"),
        })?;
    let parent = AbsPath::new(parent.to_string()).map_err(Error::Core)?;
    ensure_dirs_under(base, &parent)?;
    let tmp = parent
        .as_path()
        .join(format!(".{}.tmp", uuid::Uuid::now_v7().simple()));
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
    {
        Ok(f) => f,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return Err(Error::io(tmp.to_string(), e));
        }
    };
    if let Err(e) = file.write_all(content) {
        let _ = std::fs::remove_file(&tmp);
        return Err(Error::io(tmp.to_string(), e));
    }
    if let Err(e) = file.sync_all() {
        let _ = std::fs::remove_file(&tmp);
        return Err(Error::io(tmp.to_string(), e));
    }
    drop(file);
    if let Err(e) = std::fs::rename(&tmp, path.as_path()) {
        let _ = std::fs::remove_file(&tmp);
        return Err(Error::io(path.as_str(), e));
    }
    fsync_dir(&parent);
    Ok(())
}

/// fsync 一个目录（让 rename/create 的目录项持久）。尽力而为：不支持的平台或权限
/// 不足不阻断主流程。
pub fn fsync_dir(dir: &AbsPath) {
    if let Ok(f) = std::fs::File::open(dir.as_path()) {
        let _ = f.sync_all();
    }
}

/// 受限复制一棵树到 `dst`（`dst` 尚不存在）：拒绝软链、硬链与非常规文件，
/// 每个文件经 `SafeFile` 句柄复制、独占创建目标并 `fsync`；单文件与总量上限
/// 在复制前按句柄元数据核对，读到的字节数再核对一次。
pub fn copy_tree_confined(src: &AbsPath, dst: &AbsPath) -> Result<u64> {
    let mut total = 0u64;
    copy_tree_into(src, dst, &mut total)?;
    Ok(total)
}

fn copy_tree_into(src: &AbsPath, dst: &AbsPath, total: &mut u64) -> Result<()> {
    let stat = std::fs::symlink_metadata(src.as_path()).map_err(|e| Error::io(src.as_str(), e))?;
    if stat.file_type().is_symlink() {
        return Err(Error::InvalidRequest {
            reason: format!("{src} 是符号链接"),
        });
    }
    if !stat.is_dir() {
        return Err(Error::InvalidRequest {
            reason: format!("{src} 不是目录"),
        });
    }
    std::fs::create_dir(dst.as_path()).map_err(|e| Error::io(dst.as_str(), e))?;
    fsync_dir(dst);
    for entry in std::fs::read_dir(src.as_path()).map_err(|e| Error::io(src.as_str(), e))? {
        let entry = entry.map_err(|e| Error::io(src.as_str(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let s = src.join_segment(&name);
        let d = dst.join_segment(&name);
        if HOST_METADATA_FILES.contains(&name.as_str()) {
            return Err(Error::InvalidRequest {
                reason: format!("{s} 是宿主元数据文件（如 Finder 生成），先清理再装"),
            });
        }
        let ft = entry.file_type().map_err(|e| Error::io(s.as_str(), e))?;
        if ft.is_symlink() {
            return Err(Error::InvalidRequest {
                reason: format!("{s} 是符号链接"),
            });
        }
        if ft.is_dir() {
            copy_tree_into(&s, &d, total)?;
            continue;
        }
        if !ft.is_file() {
            return Err(Error::InvalidRequest {
                reason: format!("{s} 不是普通文件"),
            });
        }
        let from = SafeFile::open_regular(&s)?;
        let bytes = from.metadata().len();
        if bytes > MAX_FILE_BYTES {
            return Err(Error::InvalidRequest {
                reason: format!("{s} 超过 {MAX_FILE_BYTES} 字节"),
            });
        }
        *total = total
            .checked_add(bytes)
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{src} 总量超出上限"),
            })?;
        if *total > MAX_TOTAL_BYTES {
            return Err(Error::InvalidRequest {
                reason: format!("{src} 总量超过 {MAX_TOTAL_BYTES} 字节"),
            });
        }
        let content = from.read_bounded(MAX_FILE_BYTES)?;
        write_new_file(&d, &content)?;
    }
    Ok(())
}

/// 独占创建一个新文件并写入、fsync。目标已存在（含软链占位）即失败。
pub fn write_new_file(path: &AbsPath, content: &[u8]) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.as_path())
        .map_err(|e| Error::io(path.as_str(), e))?;
    file.write_all(content)
        .map_err(|e| Error::io(path.as_str(), e))?;
    file.sync_all().map_err(|e| Error::io(path.as_str(), e))?;
    Ok(())
}

/// 不跟随软链的删除：叶子是软链时只删链接本身，目录才递归。清理不得沿链接
/// 删到根外（存储合同 §3.3 的 tmp 清理语义）。
pub fn remove_tree_no_follow(path: &AbsPath) -> Result<()> {
    let stat = match std::fs::symlink_metadata(path.as_path()) {
        Ok(stat) => stat,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(Error::io(path.as_str(), e)),
    };
    if stat.file_type().is_symlink() || stat.is_file() {
        std::fs::remove_file(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
        return Ok(());
    }
    if !stat.is_dir() {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 不是目录或链接"),
        });
    }
    for entry in std::fs::read_dir(path.as_path()).map_err(|e| Error::io(path.as_str(), e))? {
        let entry = entry.map_err(|e| Error::io(path.as_str(), e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        remove_tree_no_follow(&path.join_segment(&name))?;
    }
    std::fs::remove_dir(path.as_path()).map_err(|e| Error::io(path.as_str(), e))?;
    Ok(())
}

/// 整棵置只读：目录 0555、文件 0444，含传入根本身。先拒软链，再对每个文件用
/// `SafeFile` 句柄核对身份后 fchmod；目录在内容处理完后最后置只读。
pub fn set_tree_readonly_confined(dir: &AbsPath) -> Result<()> {
    let stat = std::fs::symlink_metadata(dir.as_path()).map_err(|e| Error::io(dir.as_str(), e))?;
    if stat.file_type().is_symlink() {
        return Err(Error::InvalidRequest {
            reason: format!("{dir} 是符号链接"),
        });
    }
    if !stat.is_dir() {
        return Err(Error::InvalidRequest {
            reason: format!("{dir} 不是目录"),
        });
    }
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
            set_tree_readonly_confined(&path)?;
        } else {
            SafeFile::open_regular(&path)?.set_readonly()?;
        }
    }
    std::fs::set_permissions(dir.as_path(), std::fs::Permissions::from_mode(0o555))
        .map_err(|e| Error::io(dir.as_str(), e))?;
    Ok(())
}

/// 整棵放开写权限：文件 0644、目录 0755（含根）。删除只读目录前用；尽力而为，
/// 因为它可能要先把父目录放开才能继续列目录。
pub fn make_tree_writable(dir: &AbsPath) {
    use std::fs::Permissions;
    let Ok(meta) = std::fs::symlink_metadata(dir.as_path()) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    let mode = if meta.is_dir() { 0o755 } else { 0o644 };
    let _ = std::fs::set_permissions(dir.as_path(), Permissions::from_mode(mode));
    if meta.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir.as_path()) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                make_tree_writable(&dir.join_segment(&name));
            }
        }
    }
}
