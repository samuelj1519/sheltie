//! workbook.toml parsing; format in specs/contracts/workbook.md §2.

pub mod manifest;

pub use manifest::{HostRequire, Manifest, RequireKind, parse_manifest};
