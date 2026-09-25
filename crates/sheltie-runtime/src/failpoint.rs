//! 崩溃测试用的故障注入点。只在 `failpoint` 特性下生效。
//!
//! 三个命名点：`before_commit`、`after_commit_before_effects`、`update_between_renames`。
//! 调用处已由骨架放好；`maybe_exit` 原属 T23，因 `commit()` 一开始就调用它，
//! T13 起测试在 `--all-features` 下必经此函数，提前到 T13 填（见 plan.md T13 任务卡）。

/// 若开启特性且环境变量 `SHELTIE_FAILPOINT` 等于 `name`，以退出码 70 结束进程。否则什么都不做。
#[cfg(feature = "failpoint")]
pub fn maybe_exit(name: &str) {
    if std::env::var("SHELTIE_FAILPOINT").ok().as_deref() == Some(name) {
        std::process::exit(EXIT_CODE);
    }
}

/// 没开特性时是空函数，编译器会把它优化掉。
#[cfg(not(feature = "failpoint"))]
pub fn maybe_exit(_name: &str) {}

/// 故障注入点的退出码。
pub const EXIT_CODE: i32 = 70;
