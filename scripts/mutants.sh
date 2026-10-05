#!/usr/bin/env bash
# Milestone mutation tests. Usage: scripts/mutants.sh sheltie-core [args]; timeout defaults to 120s.
# Keep CARGO_TARGET_DIR within each mutation copy. A global target-dir would otherwise
# make parallel copies share unmutated binaries and invalidate results (M1: 175 false survivors).
# Exclude testkit fixtures.
# Write output under ignored target/mutants.out; the default ./mutants.out dirties the tree
# and silently skips check-task.sh trailer checks, which require a clean worktree.
set -euo pipefail

crate="${1:?Usage: scripts/mutants.sh <crate> [cargo-mutants arguments]}"
shift

cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR=target
exec cargo mutants -p "$crate" --timeout "${MUTANTS_TIMEOUT:-120}" --test-tool nextest \
    --output target --exclude 'crates/*/src/testkit.rs' "$@"
