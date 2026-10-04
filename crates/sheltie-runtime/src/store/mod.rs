//! SQLite 存储：唯一状态权威。规则见 `specs/contracts/storage.md` §1、§2、§7。

pub mod commit;
pub mod read;
pub mod schema;

#[cfg(test)]
mod tests;

use rusqlite::OptionalExtension;
use sheltie_core::path::AbsPath;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::home::{Home, HomeLock};

pub use commit::{CommitInput, CommitOutcome};
pub use read::WorkbookRow;
pub use schema::SCHEMA_VERSION;

/// 只读还是读写。只读打开不存在的库报 `NOT_FOUND`，不建库。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenMode {
    ReadOnly,
    ReadWrite,
}

/// 用 `READ_ONLY | NOFOLLOW` 核对 `user_version` 与逐表建表语句。识别连接在旧库
/// 拒绝前不改 journal mode、不建表或checkpoint；`NO_CKPT_ON_CLOSE` 防止关闭时写主库。
/// 只允许 SQLite 在已存在管理根内维护合同认可的 `store.db-shm` 控制文件。
fn validate_readonly(path: &AbsPath) -> Result<()> {
    let conn = rusqlite::Connection::open_with_flags(
        path.as_str(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    prevent_checkpoint_on_close(&conn, path)?;
    // 识别连接也要等待本地写者（首次建库窗口）。
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    check_schema(&conn)
}

fn prevent_checkpoint_on_close(conn: &rusqlite::Connection, path: &AbsPath) -> Result<()> {
    if !conn.set_db_config(
        rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,
        true,
    )? {
        return Err(Error::io(
            path.as_str(),
            std::io::Error::other("SQLite未启用NO_CKPT_ON_CLOSE"),
        ));
    }
    Ok(())
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
pub(crate) struct Store {
    path: AbsPath,
    mode: OpenMode,
    owner_home: Option<Home>,
    owner_lock: Option<Arc<HomeLock>>,
}

impl Store {
    /// 单元测试的低层Store夹具；产品代码必须经Home与WriteSession打开。
    ///
    /// 顺序（存储合同 §1.1）：库不存在且 `ReadWrite` → 一个事务内建表并写
    /// `user_version`（无半结构库）；`ReadOnly` 且不存在 → `Error::NotFound`，
    /// 不建库不建目录（GF-30）。库已存在时**先以只读连接**识别 `user_version` 与
    /// 建表语句：`user_version ≠ SCHEMA_VERSION` 报 `StoreSchemaMismatch`，
    /// 拒绝之前对库文件没有任何写入——不改 journal mode、不写 PRAGMA、不建表。
    #[cfg(test)]
    pub(crate) fn open(path: &AbsPath, mode: OpenMode) -> Result<Self> {
        if let Err(error) = std::fs::symlink_metadata(path.as_path()) {
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(Error::io(path.as_str(), error));
            }
            if mode == OpenMode::ReadOnly {
                return Err(Error::NotFound {
                    what: path.to_string(),
                });
            }
            // 合法写操作可以创建新管理根：先建父目录再建库（O06）。
            if let Some(parent) = path.as_path().parent() {
                std::fs::create_dir_all(parent).map_err(|e| Error::io(parent.to_string(), e))?;
            }
            let conn = rusqlite::Connection::open_with_flags(
                path.as_str(),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                    | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
                    | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )?;
            conn.execute_batch(&schema::create_script())?;
            drop(conn);
            return Self::open_existing(path, mode);
        }
        Self::open_existing(path, mode)
    }

    pub(crate) fn deferred_for_home(home: &Home, mode: OpenMode) -> Self {
        Self {
            path: home.store_path(),
            mode,
            owner_home: Some(home.clone()),
            owner_lock: None,
        }
    }

    pub(crate) fn open_for_home(home: &Home, mode: OpenMode) -> Result<Self> {
        const RETRY_LIMIT: usize = 8;
        let mut last_mismatch = None;
        for _ in 0..RETRY_LIMIT {
            let opened = crate::fsx::validate_store_files(home).and_then(|()| {
                Self::open_existing_inner(&home.store_path(), mode, Some(home.clone()), None)
            });
            match opened {
                Err(error @ Error::StoreSchemaMismatch { .. }) if mode == OpenMode::ReadOnly => {
                    match home.acquire_existing_lock()? {
                        Some(lock) => drop(lock),
                        None => return Err(error),
                    }
                    last_mismatch = Some(error);
                }
                result => return result,
            }
        }
        Err(last_mismatch.unwrap_or_else(|| Error::StoreCorrupt {
            detail: "锁内重验Store schema时没有保留原始错误".into(),
        }))
    }

    #[cfg(test)]
    pub(crate) fn open_existing(path: &AbsPath, mode: OpenMode) -> Result<Self> {
        Self::open_existing_inner(path, mode, None, None)
    }

    pub(crate) fn open_existing_locked(
        home: &Home,
        lock: Arc<HomeLock>,
        mode: OpenMode,
    ) -> Result<Self> {
        crate::fsx::validate_store_files_locked(home, &lock)?;
        Self::open_existing_inner(&home.store_path(), mode, Some(home.clone()), Some(lock))
    }

    fn open_existing_inner(
        path: &AbsPath,
        mode: OpenMode,
        owner_home: Option<Home>,
        owner_lock: Option<Arc<HomeLock>>,
    ) -> Result<Self> {
        let metadata = std::fs::symlink_metadata(path.as_path()).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound {
                    what: path.to_string(),
                }
            } else {
                Error::io(path.as_str(), error)
            }
        })?;
        if !metadata.file_type().is_file() {
            return Err(Error::InvalidRequest {
                reason: format!("Store {} 必须是普通文件", path),
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            if metadata.nlink() != 1 {
                return Err(Error::InvalidRequest {
                    reason: format!("Store {} 必须是单链接文件", path),
                });
            }
        }
        // 只读识别：旧库拒绝前无写；有 Home 时还绑定根与主库inode前后身份。
        if let Some(home) = &owner_home {
            let readonly = Self::deferred_for_home(home, OpenMode::ReadOnly);
            drop(readonly.connect()?);
        } else {
            validate_readonly(path)?;
        }
        let store = Self {
            path: path.clone(),
            mode,
            owner_home,
            owner_lock,
        };
        store.validate()?;
        Ok(store)
    }

    pub(crate) fn empty_database_bytes() -> Result<Vec<u8>> {
        let conn = rusqlite::Connection::open_in_memory()?;
        conn.execute_batch(&schema::create_script())?;
        check_schema(&conn)?;
        let bytes = conn.serialize(rusqlite::MAIN_DB)?.to_vec();
        Ok(bytes)
    }

    #[cfg(test)]
    pub fn path(&self) -> &AbsPath {
        &self.path
    }

    /// 开一个连接并设 PRAGMA：WAL、`synchronous = FULL`、`foreign_keys = ON`、`busy_timeout = 5000`。
    pub(crate) fn connect(&self) -> Result<rusqlite::Connection> {
        if let Some(home) = &self.owner_home {
            if let Some(lock) = &self.owner_lock {
                crate::fsx::validate_store_files_locked(home, lock)?;
            } else {
                crate::fsx::validate_store_files(home)?;
            }
        }
        crate::failpoint::rendezvous("store_before_metadata", self.path.as_str())
            .map_err(|error| Error::io(self.path.as_str(), error))?;
        let metadata = match std::fs::symlink_metadata(self.path.as_path()) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::NotFound {
                    what: self.path.to_string(),
                });
            }
            Err(error) => return Err(Error::io(self.path.as_str(), error)),
        };
        if !metadata.file_type().is_file() {
            return Err(Error::InvalidRequest {
                reason: format!("Store {} 必须是普通文件", self.path),
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            if metadata.nlink() != 1 {
                return Err(Error::InvalidRequest {
                    reason: format!("Store {} 必须是单链接文件", self.path),
                });
            }
        }
        let root_identity = self.owner_home.as_ref().map(root_identity).transpose()?;
        self.verify_owner_binding(root_identity, file_identity(&metadata))?;
        crate::failpoint::rendezvous("store_before_sqlite_open", self.path.as_str())
            .map_err(|error| Error::io(self.path.as_str(), error))?;
        let conn = match self.mode {
            OpenMode::ReadOnly => rusqlite::Connection::open_with_flags(
                self.path.as_str(),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                    | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
            ),
            OpenMode::ReadWrite => rusqlite::Connection::open_with_flags(
                self.path.as_str(),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                    | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
            ),
        }?;
        self.verify_owner_binding(root_identity, file_identity(&metadata))?;
        // 等待先于任何可能取写锁的 PRAGMA（journal_mode 在建库后的首次设置要等）。
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        if self.mode == OpenMode::ReadOnly {
            prevent_checkpoint_on_close(&conn, &self.path)?;
        }
        check_schema(&conn)?;
        // WAL 与 synchronous 改的是库文件，只读连接上写不进去；本来就持久在库里。
        if self.mode == OpenMode::ReadWrite {
            conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")?;
        }
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        self.verify_owner_binding(root_identity, file_identity(&metadata))?;
        Ok(conn)
    }

    fn verify_owner_binding(
        &self,
        expected_root: Option<(u64, u64)>,
        expected_store: (u64, u64),
    ) -> Result<()> {
        let Some(home) = &self.owner_home else {
            return Ok(());
        };
        if let Some(lock) = &self.owner_lock {
            if !lock.identity_still_valid() {
                return Err(Error::InvalidRequest {
                    reason: format!("{} 的HomeLock身份已改变", home.root()),
                });
            }
            crate::fsx::validate_store_files_locked(home, lock)?;
        } else {
            crate::fsx::validate_store_files(home)?;
        }
        let actual_root = root_identity(home)?;
        let actual_store = std::fs::symlink_metadata(self.path.as_path())
            .map_err(|error| Error::io(self.path.as_str(), error))?;
        if Some(actual_root) != expected_root || file_identity(&actual_store) != expected_store {
            return Err(Error::InvalidRequest {
                reason: format!("{} 的根或Store对象在SQLite连接期间被替换", home.root()),
            });
        }
        Ok(())
    }

    /// 已存在的库：`user_version` 与逐表建表语句比对（存储合同 §1.1）。
    /// 结构校验通过的读写连接才设 WAL。
    fn validate(&self) -> Result<()> {
        self.connect().map(|_| ())
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

fn root_identity(home: &Home) -> Result<(u64, u64)> {
    let metadata = std::fs::symlink_metadata(home.root().as_path())
        .map_err(|error| Error::io(home.root().as_str(), error))?;
    if !metadata.file_type().is_dir() {
        return Err(Error::InvalidRequest {
            reason: format!("管理根 {} 必须是目录", home.root()),
        });
    }
    Ok(file_identity(&metadata))
}

#[cfg(unix)]
fn file_identity(metadata: &std::fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt as _;
    (metadata.dev(), metadata.ino())
}

#[cfg(not(unix))]
fn file_identity(metadata: &std::fs::Metadata) -> (u64, u64) {
    (0, metadata.len())
}
