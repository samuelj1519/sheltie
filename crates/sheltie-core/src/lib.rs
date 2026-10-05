//! Pure rule layer of the Sheltie engine.
//!
//! This crate performs no I/O: no filesystem, clocks, randomness, or databases.
//! Callers (sheltie-runtime) pass timestamps, IDs, and file observations as arguments.
//! Every rule can therefore be unit-tested with handwritten inputs and expectations.
//!
//! Modules follow the type skeleton in specs/architecture.md §2:
//! - [`ids`], [`path`], [`text`], [`digest`]: primitive types validated on construction.
//! - [`workbook`]: workbook.toml parsing.
//! - [`flow`]: Flow parsing and graph compilation.
//! - [`work`]: Work state machine, legal next actions, brief and status-card rendering.
// Tests may unwrap; library code may not (workspace lints).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod digest;
pub mod error;
pub mod flow;
pub mod ids;
pub mod path;
pub mod text;
pub mod work;
pub mod workbook;

#[cfg(feature = "testkit")]
pub mod testkit;

pub use error::{Error, ErrorCode};
