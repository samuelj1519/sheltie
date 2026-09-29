//! Sheltie 引擎的 I/O 层。
//!
//! 职责：管理根目录、文件观察、SQLite 存储、Workbook 仓库、Work 服务、二进制自管理。
//! 不做业务判断：所有规则在 `sheltie-core`，这里只是「读状态 → 观察文件 → 调 core → 一个事务写回 → 执行效果」。
//!
// 测试代码允许 unwrap；库代码不允许（workspace lints）。
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
pub mod selfmgmt;
pub mod service;
mod session;
mod store;
pub mod workbook_digest;
pub mod workbook_repo;

pub use error::{Error, Result};
pub use home::Home;
pub use service::{Response, StartArgs, WorkService, WorkSummary};
pub use store::WorkbookRow;
pub use workbook_repo::{
    Added, LoadedWorkbook, Removed, VerifyRow, VerifyStatus, WorkbookListItem, WorkbookRepo,
};
