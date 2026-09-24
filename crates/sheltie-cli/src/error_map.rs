//! runtime 错误到协议错误码、`detail` 与退出码的映射。退出码：成功 0；`ok = false` 1；参数错误 2（clap 自带）。

use sheltie_runtime::Error;

use crate::output::Outcome;

/// 一张 `match`，一次填全。`detail` 按协议 §7 每行的 `detail.*` 字段构造。
#[allow(unused_variables)]
pub fn to_outcome(err: &Error) -> Outcome {
    todo!("T17")
}
