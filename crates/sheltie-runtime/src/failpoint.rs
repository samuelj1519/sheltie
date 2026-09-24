//! 崩溃测试用的失败点。只在 `failpoint` 特性下生效。
//!
//! 三个命名点：`before_commit`、`after_commit_before_effects`、`update_between_renames`。
//! 调用处已由骨架放好，T23 填 `maybe_exit` 的实现。

/// 若开启特性且环境变量 `SHELTIE_FAILPOINT` 等于 `name`，以退出码 70 结束进程。否则什么都不做。
#[cfg(feature = "failpoint")]
#[allow(unused_variables)]
pub fn maybe_exit(name: &str) {
    todo!("T23")
}

/// 没开特性时是空函数，编译器会把它优化掉。
#[cfg(not(feature = "failpoint"))]
pub fn maybe_exit(_name: &str) {}

/// 失败点退出码。
pub const EXIT_CODE: i32 = 70;
