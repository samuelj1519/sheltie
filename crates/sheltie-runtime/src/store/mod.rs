//! SQLite 存储：唯一状态权威。规则见 `specs/contracts/storage.md` §1、§2、§7。

pub mod commit;
pub mod read;
pub mod schema;

use rusqlite::OptionalExtension;
use sheltie_core::path::AbsPath;

use crate::error::{Error, Result};

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
    pub fn open(path: &AbsPath, mode: OpenMode) -> Result<Self> {
        let store = Self {
            path: path.clone(),
            mode,
        };
        if !path.as_path().exists() {
            if mode == OpenMode::ReadOnly {
                // 只读操作不建库、不建目录（GF-30）。
                return Err(Error::NotFound {
                    what: path.to_string(),
                });
            }
            // 合法写操作可以创建新管理根：先建父目录再建库（O06；建库 DDL 与
            // user_version 的同事务在 T07 的 schema 2 落地）。
            if let Some(parent) = path.as_path().parent() {
                std::fs::create_dir_all(parent).map_err(|e| Error::io(parent.to_string(), e))?;
            }
            let conn = store.connect()?;
            let mut script = String::new();
            for (_, sql) in schema::TABLES {
                script.push_str(sql);
                script.push_str(";\n");
            }
            script.push_str(&format!("PRAGMA user_version = {SCHEMA_VERSION};\n"));
            conn.execute_batch(&script)?;
            return Ok(store);
        }
        store.validate()?;
        Ok(store)
    }

    pub fn path(&self) -> &AbsPath {
        &self.path
    }

    /// 开一个连接并设 PRAGMA：WAL、`synchronous = FULL`、`foreign_keys = ON`、`busy_timeout = 5000`。
    pub(crate) fn connect(&self) -> Result<rusqlite::Connection> {
        let conn = match self.mode {
            OpenMode::ReadOnly => rusqlite::Connection::open_with_flags(
                self.path.as_str(),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ),
            OpenMode::ReadWrite => rusqlite::Connection::open(self.path.as_str()),
        }?;
        // WAL 与 synchronous 改的是库文件，只读连接上写不进去；本来就持久在库里。
        if self.mode == OpenMode::ReadWrite {
            conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")?;
        }
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        Ok(conn)
    }

    /// 已存在的库：`user_version` 与逐表建表语句比对（存储合同 §1.1）。
    fn validate(&self) -> Result<()> {
        let conn = self.connect()?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version != SCHEMA_VERSION {
            return Err(Error::StoreSchemaMismatch {
                detail: format!("user_version 是 {version}，期望 {SCHEMA_VERSION}"),
            });
        }
        for (name, sql) in schema::TABLES {
            let got: Option<String> = conn
                .query_row(
                    "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [name],
                    |r| r.get(0),
                )
                .optional()?;
            match got {
                None => {
                    return Err(Error::StoreSchemaMismatch {
                        detail: format!("缺表 {name}"),
                    });
                }
                Some(got) => {
                    if schema::normalize_sql(&got) != schema::normalize_sql(sql) {
                        return Err(Error::StoreSchemaMismatch {
                            detail: format!("表 {name} 的建表语句与 SCHEMA_VERSION 不符"),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// 分配当日序号（存储合同 §7.1）。独立短事务；超过 999 报 `InvalidRequest`。
    pub fn allocate_seq(&self, day: &str) -> Result<u32> {
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO work_sequence (day, last) VALUES (?1, 1)\n  \
             ON CONFLICT(day) DO UPDATE SET last = last + 1",
            [day],
        )?;
        let last: i64 = tx.query_row(
            "SELECT last FROM work_sequence WHERE day = ?1",
            [day],
            |r| r.get(0),
        )?;
        if last > 999 {
            // 返回 Err 会让事务回滚，当天最后一个已分配序号仍是 999。
            return Err(Error::InvalidRequest {
                reason: format!("{day} 的当日序号已达 {last}，上限 999"),
            });
        }
        tx.commit()?;
        Ok(last as u32)
    }
}
