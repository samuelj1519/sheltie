//! 管理根 `~/.sheltie` 与路径约束。见 `specs/contracts/storage.md` §6、`specs/architecture.md` §5。

use sheltie_core::ids::WorkId;
use sheltie_core::path::{AbsPath, RelPath};

use crate::error::{Error, Result};

/// 把路径规范化到「最深已存在祖先的真实位置 + 余下原样段」。
/// 全路径已存在时等价于 `canonicalize`；尚不存在的尾部保持词法形式。
fn canonicalize_deepest(path: &camino::Utf8Path) -> camino::Utf8PathBuf {
    let mut probe = path.to_path_buf();
    let mut tail: Vec<String> = Vec::new();
    loop {
        match std::fs::canonicalize(&probe) {
            Ok(real) => {
                let mut out = camino::Utf8PathBuf::from(real.to_string_lossy().into_owned());
                for seg in tail.iter().rev() {
                    out = out.join(seg);
                }
                return out;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let Some(name) = probe.file_name().map(|n| n.to_string()) else {
                    return path.to_path_buf();
                };
                tail.push(name);
                let Some(parent) = probe.parent().map(|p| p.to_path_buf()) else {
                    return path.to_path_buf();
                };
                probe = parent;
            }
            Err(_) => return path.to_path_buf(),
        }
    }
}

/// 管理根。只有 runtime 能在它下面写东西。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    root: AbsPath,
}

impl Home {
    /// 解析管理根：`cli` 参数 > 环境变量 `SHELTIE_HOME` > `$HOME/.sheltie`。
    /// 相对路径按当前目录转成绝对路径。不建目录。
    ///
    /// 根在入口一次规范化：对最深已存在祖先做 `canonicalize`，再拼回余下段。
    /// 之后所有 managed 路径都从这个规范根派生，祖先软链（如 `/tmp` 一类）不会
    /// 让派生路径与 `confine` 的前缀比较失真（架构 §5）。
    pub fn resolve(cli: Option<&str>) -> Result<Self> {
        let given = match cli.map(str::to_string) {
            Some(p) => p,
            None => std::env::var("SHELTIE_HOME")
                .or_else(|_| std::env::var("HOME").map(|h| format!("{h}/.sheltie")))
                .map_err(|_| Error::InvalidRequest {
                    reason: "既没有 --home 与 SHELTIE_HOME，也取不到 $HOME".to_string(),
                })?,
        };
        let root = if camino::Utf8Path::new(&given).is_absolute() {
            camino::Utf8PathBuf::from(given)
        } else {
            let cwd = std::env::current_dir().map_err(|e| Error::io(".", e))?;
            camino::Utf8PathBuf::from(cwd.join(&given).to_string_lossy().into_owned())
        };
        Ok(Self {
            root: AbsPath::new(canonicalize_deepest(&root).to_string())?,
        })
    }

    /// 直接用一个绝对路径当根（测试用）。与 `resolve` 一样把根规范化到真实形式，
    /// 否则父进程记录的词法路径与子进程（`--home` 走 `resolve`）派生的路径对不上。
    pub fn at(root: AbsPath) -> Self {
        Self {
            root: AbsPath::new(canonicalize_deepest(root.as_path()).to_string()).unwrap_or(root),
        }
    }

    pub fn root(&self) -> &AbsPath {
        &self.root
    }

    pub fn store_path(&self) -> AbsPath {
        self.root.join_segment("store.db")
    }

    pub fn workbooks_dir(&self) -> AbsPath {
        self.root.join_segment("workbooks")
    }

    pub fn staging_dir(&self) -> AbsPath {
        self.workbooks_dir().join_segment(".staging")
    }

    pub fn works_dir(&self) -> AbsPath {
        self.root.join_segment("works")
    }

    pub fn tmp_dir(&self) -> AbsPath {
        self.root.join_segment("tmp")
    }

    /// 管理根写锁 `<root>/.lock`（存储合同 §2.2，D-035 的 `fs4`）。
    pub fn lock_path(&self) -> AbsPath {
        self.root.join_segment(".lock")
    }

    /// `pending/`：引擎持锁创建的私有暂存（未提交准备区、已提交未发布原件、待删除目录）。
    pub fn pending_dir(&self) -> AbsPath {
        self.root.join_segment("pending")
    }

    /// 把相对管理根的路径拼回绝对路径。效果登记里的路径都是相对形式；
    /// 拼回后仍受 `ensure_dirs_under` 一类的根内检查约束。
    pub fn rel(&self, rel: &str) -> AbsPath {
        AbsPath::new(format!(
            "{}/{}",
            self.root.as_str().trim_end_matches('/'),
            rel
        ))
        .unwrap_or_else(|_| self.root.clone())
    }

    /// 绝对路径相对管理根的形式；不在根内时报错。
    pub fn to_rel(&self, path: &AbsPath) -> Result<String> {
        path.as_path()
            .strip_prefix(self.root.as_path())
            .map(|p| p.to_string())
            .map_err(|_| Error::InvalidRequest {
                reason: format!("{path} 不在管理根 {} 之内", self.root),
            })
    }

    pub fn bin_dir(&self) -> AbsPath {
        self.root.join_segment("bin")
    }

    pub fn work_dir(&self, id: &WorkId) -> AbsPath {
        self.works_dir().join_segment(id.as_str())
    }

    pub fn workbook_dir(&self, id: &str, version: &str) -> AbsPath {
        self.workbooks_dir().join_segment(id).join_segment(version)
    }

    /// 把外部给的相对路径限制在 `base` 之下。
    ///
    /// 拒绝：绝对路径、含 `..`、空段。若拼出的路径已存在，`canonicalize` 后必须仍以 `base` 的
    /// 规范形式为前缀（防符号链接逃逸）。失败返回 `Error::Core(InvalidPath)`。
    pub fn confine(base: &AbsPath, rel: &str) -> Result<AbsPath> {
        // 先按写法拒绝：`RelPath` 的构造就是这套检查。
        let rel = RelPath::new(rel)?;
        let joined = base.join(&rel);
        // base 不存在时其下不可能有已存在路径，写法检查已足够。
        let Ok(base_canon) = std::fs::canonicalize(base.as_path()) else {
            return Ok(joined);
        };
        // 找最近的存在祖先（叶与中间段可能还没建出来），对它 canonicalize 比前缀。
        let mut probe = joined.as_path().to_path_buf();
        loop {
            match std::fs::symlink_metadata(probe.as_std_path()) {
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(Error::io(probe.as_str(), e)),
            }
            match probe.parent() {
                Some(p) if p != base.as_path() => probe = p.to_path_buf(),
                _ => return Ok(joined),
            }
        }
        let canon =
            std::fs::canonicalize(probe.as_std_path()).map_err(|e| Error::io(probe.as_str(), e))?;
        if !canon.starts_with(&base_canon) {
            return Err(Error::Core(sheltie_core::Error::InvalidPath {
                path: joined.to_string(),
                reason: "经符号链接逃出了根",
            }));
        }
        Ok(joined)
    }
}

/// 管理根写锁的守卫（存储合同 §2.2，D-035 的 `fs4`）。取得锁前只创建管理根与
/// `.lock` 本身；进程退出由 OS 释放。drop 时解锁。
pub struct HomeLock {
    _file: std::fs::File,
}

impl Home {
    /// 排他取得管理根写锁（阻塞等待本地协作进程）。根不存在时先建根与 `.lock`。
    pub fn acquire_lock(&self) -> Result<HomeLock> {
        crate::fsx::ensure_dirs_under(&self.root, &self.root)?;
        let lock_path = self.lock_path();
        // 锁文件内容无关紧要（存在即锁对象），不得截断已存在的文件。
        #[allow(clippy::suspicious_open_options)]
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(lock_path.as_path())
            .map_err(|e| Error::io(lock_path.as_str(), e))?;
        fs4::fs_std::FileExt::lock_exclusive(&file)
            .map_err(|e| Error::io(lock_path.as_str(), e))?;
        crate::fsx::fsync_dir(&self.root);
        Ok(HomeLock { _file: file })
    }
}
