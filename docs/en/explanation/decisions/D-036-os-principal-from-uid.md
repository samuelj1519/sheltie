# D-036: Record the actual OS process principal

English | [简体中文](../../../zh-CN/explanation/decisions/D-036-os-principal-from-uid.md)

Status: `accepted`
Date: 2026-09-27
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## Context

v0.1.0 used USER/USERNAME environment values as audit and gate-approval principals (N04). Callers can set them arbitrarily: `USER=alice sheltie gate approve …` records any name. The constitution also promised human approval, although a single-account engine has no independent authentication. The promise exceeded capability.

## Decision

Use the initiating process's actual identity. On unix, `uzers::get_effective_uid()` gives the effective uid and `uzers::get_user_by_uid()` gives its account name. Missing entries or non-UTF-8 names become `uid:<number>`, without guesses. Never read USER/USERNAME. This is an accounting fact; host/user authorization determines whether it identifies a particular person. Documentation/reports say the account approved, without claiming independent human authentication (constitution §5). Do not build custom authentication.

API checks: [uzers get_effective_uid](https://docs.rs/uzers/latest/uzers/fn.get_effective_uid.html) and [get_user_by_uid](https://docs.rs/uzers/latest/uzers/fn.get_user_by_uid.html) are safe Rust interfaces internally calling geteuid/getpwuid_r. Project code retains unsafe_code = "forbid". Non-unix returns `unknown` and is outside release targets. T05 must verify platform/MSRV before adding the dependency; direct `libc` cannot bypass the prohibition.

## Rejected alternatives

- Environment variables: forgeable, making records meaningless.
- A child `id -un` process: needless PATH/locale failure modes per call; `getpwuid_r` suffices.
- Custom password/approver authentication: outside a local single-user tool, creating false confidence.

## C002-T05 implementation correction

`cargo deny check` rejected the original `users` 0.11.0 choice: RUSTSEC-2023-0040 (unmaintained), RUSTSEC-2023-0059 (unaligned pointer read), and other advisories. Engineering rules prohibit hiding warnings as passing checks. Use the maintained [uzers](https://crates.io/crates/uzers) fork with identical signatures. Decision semantics and verification remain unchanged.

## Consequences

Tests cannot forge principals by setting USER. Expectations use the actual effective-uid account name or uid:<n>. Approval means OS-account accounting; upstream constitution, specification, protocol, and skill reflect this boundary.

## Verification

Set `USER=someone-else`, approve a gate, and assert the record is not `someone-else` and equals the account for the current uid.
