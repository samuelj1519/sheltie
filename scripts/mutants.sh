#!/usr/bin/env bash
# 里程碑用的突变测试。用法：scripts/mutants.sh sheltie-core [额外参数]；超时默认 120 秒，MUTANTS_TIMEOUT=300 覆盖。
# CARGO_TARGET_DIR 固定到仓库内：全局 ~/.cargo/config.toml 若设了 target-dir，
# 并行的突变副本会共用一个目标目录，测试跑的是没突变的二进制，结果全部作废（M1 实测 175 个假存活）。
# testkit 是测试夹具，不计入。
# 输出放 target/mutants.out（已 gitignore）：默认的 ./mutants.out 会让工作树变脏，
# check-task.sh 的提交信息检查（工作树干净才查）因此静默跳过。
set -euo pipefail

crate="${1:?用法: scripts/mutants.sh <crate> [cargo-mutants 参数]}"
shift

cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR=target
exec cargo mutants -p "$crate" --timeout "${MUTANTS_TIMEOUT:-120}" --test-tool nextest \
    --output target --exclude 'crates/*/src/testkit.rs' "$@"
