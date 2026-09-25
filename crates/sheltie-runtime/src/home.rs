//! 管理根 `~/.sheltie` 与路径约束。见 `specs/contracts/storage.md` §6、`specs/architecture.md` §5。

use sheltie_core::ids::WorkId;
use sheltie_core::path::{AbsPath, RelPath};

use crate::error::{Error, Result};

/// 管理根。只有 runtime 能在它下面写东西。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    root: AbsPath,
}

impl Home {
    /// 解析管理根：`cli` 参数 > 环境变量 `SHELTIE_HOME` > `$HOME/.sheltie`。
    /// 相对路径按当前目录转成绝对路径。不建目录。
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
            AbsPath::new(given)?
        } else {
            let cwd = std::env::current_dir().map_err(|e| Error::io(".", e))?;
            AbsPath::new(cwd.join(&given).to_string_lossy().into_owned())?
        };
        Ok(Self { root })
    }

    /// 直接用一个绝对路径当根（测试用）。
    pub fn at(root: AbsPath) -> Self {
        Self { root }
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
