//! I/O layer of the Sheltie engine.
//!
//! Responsibilities: management root, file observation, SQLite storage, Workbook repository, Work service, and binary management.
//! No business judgments: sheltie-core holds all rules; read state, observe files, call core, commit one transaction, execute effects.
//!
// Tests may unwrap; library code may not (workspace lints).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod effects;
pub mod error;
pub mod failpoint;
pub mod fsx;
pub mod home;
mod load;
pub mod observe;
mod pending;
mod recovery;
pub mod request;
mod result;
pub mod selfmgmt;
pub mod service;
mod session;
mod snapshot;
mod store;
pub mod workbook_digest;
pub mod workbook_repo;

pub use error::{Error, Result};
pub use home::Home;
pub use result::StatusReadView;
pub use service::{Response, StartArgs, WorkService, WorkSummary};
pub use store::WorkbookRow;
pub use workbook_repo::{
    Added, LoadedWorkbook, Removed, VerifyRow, VerifyStatus, WorkbookListItem, WorkbookRepo,
};
