//! SQLite 存储：唯一状态权威。规则见 `specs/contracts/storage.md` §1、§2、§7。

pub mod commit;
pub mod read;
pub mod schema;

use sheltie_core::path::AbsPath;

use crate::error::Result;

pub use commit::{CommitInput, CommitOutcome};
pub use read::{WorkRow, WorkbookRow};
pub use schema::SCHEMA_VERSION;

/// 只读还是读写。只读打开不存在的库报 `NOT_FOUND`，不建库。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    ReadOnly,
    ReadWrite,
}

/// 存储句柄。不持久连接：每个方法开一个连接、用完关掉。
#[derive(Debug, Clone)]
pub struct Store {
    path: AbsPath,
    mode: OpenMode,
}

impl Store {
    /// 打开（必要时建库），并做结构校验。
    ///
    /// 顺序（存储合同 §1.1）：库不存在且 `ReadWrite` → 建库、建表、写 `user_version`；
    /// `ReadOnly` 且不存在 → `Error::NotFound`；`user_version != SCHEMA_VERSION` → `StoreSchemaMismatch`；
    /// 逐表比对 `sqlite_master.sql` 与 `schema::TABLES`（去掉全部空白后相等）→ 否则 `StoreSchemaMismatch`。
    #[allow(unused_variables)]
    pub fn open(path: &AbsPath, mode: OpenMode) -> Result<Self> {
        todo!("T13")
    }

    pub fn path(&self) -> &AbsPath {
        &self.path
    }

    /// 开一个连接并设 PRAGMA：WAL、`synchronous = FULL`、`foreign_keys = ON`、`busy_timeout = 5000`。
    #[allow(unused_variables)]
    pub(crate) fn connect(&self) -> Result<rusqlite::Connection> {
        todo!("T13")
    }

    /// 分配当日序号（存储合同 §7.1）。独立短事务；超过 999 报 `InvalidRequest`。
    #[allow(unused_variables)]
    pub fn allocate_seq(&self, day: &str) -> Result<u32> {
        todo!("T13")
    }
}
