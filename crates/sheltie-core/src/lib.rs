//! Sheltie 引擎的纯规则层。
//!
//! 这个 crate 不做任何 I/O：没有文件、没有时钟、没有随机数、没有数据库。
//! 时间、ID、对文件的观察都由调用方（`sheltie-runtime`）作为参数传进来。
//! 因此每条规则都能用手写输入与期望输出做单元测试。
//!
//! 模块对应 `specs/architecture.md` §2 的类型骨架：
//! - [`ids`]、[`path`]、[`text`]、[`digest`]：构造即校验的基础类型。
//! - [`workbook`]：`workbook.toml` 的解析。
//! - [`flow`]：Flow 的解析与编译成图。
//! - [`work`]：Work 状态机、合法下一步、任务书与状态卡的渲染。
//!
//! 骨架阶段（T01 到 T10）允许 `dead_code`，M1 里程碑删除这一行。
#![allow(dead_code)]
// 测试代码允许 unwrap；库代码不允许（workspace lints）。
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
