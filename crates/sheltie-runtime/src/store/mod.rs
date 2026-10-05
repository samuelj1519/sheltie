//! SQLite storage: sole state authority; storage contract §1, §2, and §7.

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

/// Read-only/read-write mode; read-only missing databases yield NOT_FOUND without creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpenMode {
    ReadOnly,
    ReadWrite,
}

/// Use READ_ONLY | NOFOLLOW to verify user_version and table DDL. Recognition connections
/// do not change journal mode, create tables, or checkpoint before rejecting old databases; NO_CKPT_ON_CLOSE prevents writes on closure.
/// SQLite may maintain only contract-approved store.db-shm control files within the existing management root.
fn validate_readonly(path: &AbsPath) -> Result<()> {
    let conn = rusqlite::Connection::open_with_flags(
        path.as_str(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    prevent_checkpoint_on_close(&conn, path)?;
    // Recognition connections also wait for local writers during initial creation.
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
            std::io::Error::other("SQLite NO_CKPT_ON_CLOSE is not enabled"),
        ));
    }
    Ok(())
}

fn check_schema(conn: &rusqlite::Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version != SCHEMA_VERSION {
        return Err(Error::StoreSchemaMismatch {
            detail: format!("user_version is {version}; expected {SCHEMA_VERSION}"),
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
                    detail: format!("Missing table {name}"),
                });
            }
            Some(got) => {
                if schema::normalize_sql(&got) != schema::normalize_sql(sql) {
                    return Err(Error::StoreSchemaMismatch {
                        detail: format!("Table {name} DDL does not match SCHEMA_VERSION"),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Storage handle without persistent connections; open and close per method.
#[derive(Debug, Clone)]
pub(crate) struct Store {
    path: AbsPath,
    mode: OpenMode,
    owner_home: Option<Home>,
    owner_lock: Option<Arc<HomeLock>>,
}

impl Store {
    /// Low-level Store fixture for unit tests; production must open through Home and WriteSession.
    ///
    /// Storage §1.1 order: missing ReadWrite database -> create tables and write
    /// user_version in one transaction; missing ReadOnly database -> NotFound,
    /// without database/directory creation (GF-30). Existing databases are first identified read-only by user_version and
    /// DDL; version mismatch yields StoreSchemaMismatch,
    /// with no writes before rejection: no journal-mode changes, PRAGMA writes, or table creation.
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
            // Valid writes may create a root: parent directory before database (O06).
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
            detail: "Store schema recheck under the lock lost the original error".into(),
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
                reason: format!("Store {} must be a regular file", path),
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            if metadata.nlink() != 1 {
                return Err(Error::InvalidRequest {
                    reason: format!("Store {} must be singly linked", path),
                });
            }
        }
        // Read-only recognition performs no writes before rejecting old storage; Home also binds root/main-database inode identities.
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

    /// Open a connection and configure WAL, synchronous = FULL, foreign_keys = ON, busy_timeout = 5000.
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
                reason: format!("Store {} must be a regular file", self.path),
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            if metadata.nlink() != 1 {
                return Err(Error::InvalidRequest {
                    reason: format!("Store {} must be singly linked", self.path),
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
        // Wait before PRAGMAs that may take a write lock, including initial journal_mode configuration.
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        if self.mode == OpenMode::ReadOnly {
            prevent_checkpoint_on_close(&conn, &self.path)?;
        }
        check_schema(&conn)?;
        // WAL/synchronous changes require writes; read-only connections retain existing persistent configuration.
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
                    reason: format!("{} HomeLock identity changed", home.root()),
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
                reason: format!(
                    "{} root or Store object was replaced while opening the SQLite connection",
                    home.root()
                ),
            });
        }
        Ok(())
    }

    /// Existing databases: compare user_version and per-table DDL (storage §1.1).
    /// Configure WAL only on structurally verified read-write connections.
    fn validate(&self) -> Result<()> {
        self.connect().map(|_| ())
    }

    /// Allocate a daily sequence in an independent short transaction (storage §7.1); above 999 yields InvalidRequest.
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
            // Err rolls back the transaction, retaining the day's final allocated sequence at 999.
            return Err(Error::InvalidRequest {
                reason: format!("Daily sequence for {day} is {last}; limit 999"),
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
            reason: format!("Management root {} must be a directory", home.root()),
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
