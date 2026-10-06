# Implementation and test entry points

English | [简体中文](../../zh-CN/reference/implementation.md)

This page connects adopted capabilities to public entry points, implementation, and regression tests. The [specification index](../../../specs/README.md) alone defines current base versions and active changes. Validate changes against actual candidates and input closure.

The table lists existing complete paths. Functions/tests alone do not establish acceptance. See [acceptance](acceptance.md) and related changes for historical qualification/unexecuted boundaries. Revalidate changes against actual input closure.

| Adopted requirement | Implementation | Regression entry |
| --- | --- | --- |
| GF-01–GF-05: Business independence, explicit graphs, separate definition/execution | [Parsing/compilation](../../../crates/sheltie-core/src/flow), [State decisions](../../../crates/sheltie-core/src/work/decide.rs), [Workbook repository](../../../crates/sheltie-runtime/src/workbook_repo.rs) | [code-change](../../../crates/sheltie-cli/tests/scenario_code_change.rs), [article-review](../../../crates/sheltie-cli/tests/scenario_article_review.rs), core compilation tests |
| GF-06–GF-09: Briefs, frozen inputs, output sealing, bounded replies | [Input/output observation](../../../crates/sheltie-runtime/src/observe.rs), [File handles](../../../crates/sheltie-runtime/src/fsx.rs), [Rendering](../../../crates/sheltie-core/src/work/render.rs) | [Output paths](../../../crates/sheltie-cli/tests/output_paths.rs), [Results](../../../crates/sheltie-cli/tests/scenario_artifacts.rs), [File boundaries](../../../crates/sheltie-runtime/tests/fs_boundary.rs) |
| GF-10: Same-snapshot status/resume | WorkService::status_read, Store::read_work_bundle, StatusReadView | [Status queries](../../../crates/sheltie-cli/tests/work.rs), [Strict snapshots](../../../crates/sheltie-cli/tests/snapshot_qualification.rs) |
| GF-11–GF-14: Failure, revocation, gates, limits, terminal state, result selection | core decide/legal_next/result_view; runtime replace/result | [Replacement](../../../crates/sheltie-cli/tests/attempt_replace.rs), [Gates](../../../crates/sheltie-cli/tests/scenario_gated_release.rs), [Final results](../../../crates/sheltie-cli/tests/work_result.rs) |
| GF-15–GF-16, GF-30–GF-32: Request identity, old-format rejection, preflight, trusted recovery | [request](../../../crates/sheltie-runtime/src/request.rs), [load](../../../crates/sheltie-runtime/src/load.rs), [Store](../../../crates/sheltie-runtime/src/store), [recovery](../../../crates/sheltie-runtime/src/recovery.rs) | [Replay](../../../crates/sheltie-cli/tests/replay.rs), [Effects](../../../crates/sheltie-runtime/tests/effect_contracts.rs), [Crash recovery](../../../crates/sheltie-cli/tests/reliability_crash.rs), [Start preflight](../../../crates/sheltie-runtime/tests/start_preflight.rs) |
| GF-17, GF-28: Method versions, frozen copies, requires declarations | WorkbookRepo, workbook-digest/v2, WorkService::start; requires passes declarations only | [Method lifecycle](../../../crates/sheltie-cli/tests/scenario_workbook_lifecycle.rs), [Workbook](../../../crates/sheltie-cli/tests/workbook.rs) |
| GF-18–GF-19: Coordinator instructions and truthful delivery | [Sheltie skill](../../../skills/sheltie/SKILL.md), single-contract generation and human/engineering boundaries | [Skill delivery](../../../crates/sheltie-cli/tests/skill_delivery.rs), [skill](../../../crates/sheltie-cli/tests/skill.rs), Acceptance records |
| GF-27: self install/update/rollback/purge | [selfmgmt](../../../crates/sheltie-runtime/src/selfmgmt.rs) | [self](../../../crates/sheltie-cli/tests/self_cmd.rs), [Remote update](../../../crates/sheltie-cli/tests/remote_update.rs), runtime selfmgmt tests |
| GF-29: Fact statistics and Workbook reflection | [Stats rendering](../../../crates/sheltie-core/src/work/render.rs), [spec-dev](../../../workbooks/spec-dev) | [spec-dev scenario](../../../crates/sheltie-cli/tests/scenario_spec_dev.rs) and core rendering tests |
| GF-33: Raw-byte reads and new editable copies | WorkService::write_result_artifact, [CLI raw entry](../../../crates/sheltie-cli/src/commands/work.rs), [sheltie-export](../../../crates/sheltie-export/src) | [Raw results](../../../crates/sheltie-cli/tests/result_artifact.rs), [External export tests](../../../crates/sheltie-export/tests) |
| GF-34: Local visual authoring | [Tool](../../../tools/workbook-editor) validates drafts through public CLI and emits ZIP | [Tool tests](../../../tools/workbook-editor/test), C011 bounded usability and actual-user records |

## Unimplemented or unadopted directions

GF-20–GF-26 host installation/readiness, cross-version continuation, host isolation, multi-host MCP, automatic cost gates, parallel drafts, and dynamic expansion are not current capabilities. C008 completed declaration preflight; probes are unadopted. See the [roadmap](../../../specs/roadmap.md) for triggers. docs does not provide placeholder commands.

## Use this baseline

When maintaining behavior, connect requirements to public entry points, complete call chains, positive/negative oracles, and recovery windows before choosing validation. Record exact implementation/specification differences. Unexecuted, unknown, or excluded environments cannot become PASS. Tutorials/guides use existing public capabilities above; the current baseline uses fresh explicit roots without legacy migration.
