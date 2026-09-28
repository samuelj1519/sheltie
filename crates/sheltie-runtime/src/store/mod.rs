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

/// 核对 `user_version` 与逐表建表语句。识别连接**不执行任何会写库的操作**
///（不改 journal mode、不写 PRAGMA、不建表）；普通打开而非 `READ_ONLY` 旗标，
/// 因为 WAL 库的只读连接在写者活动或 `-shm` 缺席时会直接失败。库文件不存在由
/// 调用方先行判断。
fn validate_readonly(path: &AbsPath) -> Result<()> {
    let conn = rusqlite::Connection::open(path.as_str())?;
    // 识别连接也要等待本地写者（首次建库窗口）。
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    check_schema(&conn)
}

fn check_schema(conn: &rusqlite::Connection) -> Result<()> {
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

/// 存储句柄。不持久连接：每个方法开一个连接、用完关掉。
#[derive(Debug, Clone)]
pub struct Store {
    path: AbsPath,
    mode: OpenMode,
}

impl Store {
    /// 打开（必要时建库），并做结构校验。
    ///
    /// 顺序（存储合同 §1.1）：库不存在且 `ReadWrite` → 一个事务内建表并写
    /// `user_version`（无半结构库）；`ReadOnly` 且不存在 → `Error::NotFound`，
    /// 不建库不建目录（GF-30）。库已存在时**先以只读连接**识别 `user_version` 与
    /// 建表语句：`user_version ≠ 2`（含 schema 1 旧库）报 `StoreSchemaMismatch`，
    /// 拒绝之前对库文件没有任何写入——不改 journal mode、不写 PRAGMA、不建表。
    pub fn open(path: &AbsPath, mode: OpenMode) -> Result<Self> {
        if !path.as_path().exists() {
            if mode == OpenMode::ReadOnly {
                return Err(Error::NotFound {
                    what: path.to_string(),
                });
            }
            // 合法写操作可以创建新管理根：先建父目录再建库（O06）。
            if let Some(parent) = path.as_path().parent() {
                std::fs::create_dir_all(parent).map_err(|e| Error::io(parent.to_string(), e))?;
            }
            let store = Self {
                path: path.clone(),
                mode,
            };
            let conn = store.connect()?;
            conn.execute_batch(&schema::create_script())?;
            return Ok(store);
        }
        // 只读识别：旧库拒绝前无写。
        validate_readonly(path)?;
        let store = Self {
            path: path.clone(),
            mode,
        };
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
        // 等待先于任何可能取写锁的 PRAGMA（journal_mode 在建库后的首次设置要等）。
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        // WAL 与 synchronous 改的是库文件，只读连接上写不进去；本来就持久在库里。
        if self.mode == OpenMode::ReadWrite {
            conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")?;
        }
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(conn)
    }

    /// 已存在的库：`user_version` 与逐表建表语句比对（存储合同 §1.1）。
    /// 结构校验通过的读写连接才设 WAL。
    fn validate(&self) -> Result<()> {
        let conn = self.connect()?;
        check_schema(&conn)
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
