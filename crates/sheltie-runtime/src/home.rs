//! 管理根 `~/.sheltie` 与路径约束。见 `specs/contracts/storage.md` §6、`specs/architecture.md` §5。

use sheltie_core::ids::WorkId;
use sheltie_core::path::AbsPath;

use crate::error::Result;

/// 管理根。只有 runtime 能在它下面写东西。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    root: AbsPath,
}

impl Home {
    /// 解析管理根：`cli` 参数 > 环境变量 `SHELTIE_HOME` > `$HOME/.sheltie`。
    /// 相对路径按当前目录转成绝对路径。不建目录。
    #[allow(unused_variables)]
    pub fn resolve(cli: Option<&str>) -> Result<Self> {
        todo!("T12")
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
    #[allow(unused_variables)]
    pub fn confine(base: &AbsPath, rel: &str) -> Result<AbsPath> {
        todo!("T12")
    }
}
