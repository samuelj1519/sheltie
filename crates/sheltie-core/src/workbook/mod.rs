//! `workbook.toml` 的解析。格式见 `specs/contracts/workbook.md` §2。

pub mod manifest;

pub use manifest::{HostRequire, Manifest, RequireKind, parse_manifest};
