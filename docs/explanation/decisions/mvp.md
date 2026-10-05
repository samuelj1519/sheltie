# MVP design reasons

English | [简体中文](mvp.zh-CN.md)

This summarizes v0.1.0 D-01–D-31, retaining motivations and successor boundaries. It is historical explanation; current specifications/contracts govern behavior. Original reviews, repair logs, and first runs are in [fixed Git snapshots](../../how-to/maintain-docs.md#read-original-historical-records).

## D-01 Explicit edges define legal next actions

Method authors declare edges; coordinators choose only from next. Do not infer flow from input references: dependencies and execution order each have explicit sources.

## D-02 Coordinators choose edges; the engine does not read content

Coordinators interpret natural-language conclusions; the engine checks only legality. Content review is an ordinary Node, without business judgments in the kernel.

## D-03 Small CLI and next in every response

MVP used a few explicit verbs and returned legal next actions with business responses. Historical verb counts do not limit later interfaces; the protocol contract defines current commands.

## D-04 Gate and executor are orthogonal

executor determines who executes; gate requires approval after success before leaving. Human execution does not automatically constitute authorized approval.

## D-05 State JSON and optimistic concurrency

Store whole Work state as JSON; revision supports CAS. SQLite transactions ensure consistency instead of scattering state transitions across business tables.

## D-06 Work IDs contain date, sequence, and name

UTC date and daily sequence aid reading. Allocate sequence numbers in short transactions without reclaiming failed gaps. Complete identity comes from Store; directory names are projections.

## D-07 Host writes are the INV-3 boundary

Workbook installation and self management write only within the management root. Authorized independent tools install host configuration, skills, and named subagents.

## D-08 One binary and self management

The engine binary handles install/update/rollback. D-30 superseded the initial axoupdater idea, avoiding host-configuration writes in the installation chain.

## D-09 Every Work holds a frozen Workbook

start copies the method; subsequent execution reads that copy. Deleting installed versions or manually editing source directories does not change old Work definitions.

## D-10 Separate reference files from host mechanisms

Place ordinary files in resources and bind them as inputs. Only host mechanisms use kind:name requires. Host installation/deduplication remains future tooling, not current engine capability.

## D-11 Execution failure differs from negative content verdicts

A completed review with a negative verdict succeeds operationally; explicit edges route rework. Crashes or missing deliveries are failed. C005 later added superseded; current contracts define states.

## D-12 Separate arrival and retry limits

max_visits limits loop arrivals; max_retries limits execution-failure retries within one arrival. Avoid a third semantically overlapping revision count.

## D-13 Start with simple workflows

Method graphs are fixed; agents act autonomously within nodes. Complex routing, dynamic expansion, and platform capabilities require separately adopted real needs. See [design sources](../design-sources.md).

## D-14 Three Rust layers and synchronous execution

core makes pure decisions, runtime handles I/O, and cli exposes entry points. Synchronous CLI/SQLite WAL suit local use. Do not add I/O traits for mocks or an async platform in advance.

## D-15 CLI before MCP for MVP

Hosts can invoke CLI; the skill teaches the operation loop. Multiple-host experience and independent human identity require separate adoption.

## D-16 Optional inputs from upstream outputs

Review feedback may be absent on first arrival, so upstream output inputs may be required=false. start/resource sources do not become optional.

## D-17 spec-dev models development stages as a method

Planning, implementation, review, and delivery live in a Workbook. D-24 and later versions superseded the initial eight-node form. Read workbooks/spec-dev for the current graph.

## D-18 CI and delivery authorization are separate

CI stays outside method execution; methods explicitly declare authorization gates. The original deliver gate moved to retro under D-24.

## D-19 Human-review branches and entry source

Use a human Node and explicit branch when a person's conclusion is needed. Briefs retain entered_from to explain node arrival.

## D-20 Prepare interfaces and independent expectations first

MVP prepared interfaces and disabled tests before implementation. That protocol is closed; later work prepares complete behavior according to actual risk rather than repeating a repository-wide fill-in plan.

## D-21 Tier is a coordinator label

Nodes may carry strong/standard. Coordinators select models; the engine does not schedule from tier. Current spec-dev follows its actual method version.

## D-22 Stable entry points and traceable commits

Entry points retain lasting rules; task commits retain ownership. D-032 partially superseded current-stage content in AGENTS.md and old trailer rules.

## D-23 Exact IDs and error fields

Reject invalid IDs and preserve parse round trips. Shared macro errors must name actual domain fields rather than substitute type names.

## D-24 Reflection lives in the Workbook; the engine supplies facts

retro reads statistics, summarizes method lessons, and proposes evidence-backed improvements with locations. People decide new versions. The engine does not automatically schedule, change graphs, or learn from natural language.

## D-25 Review conclusions require complete evidence

Verify rebuttals individually; complete diffs and raw output precede summaries. Partial observations do not establish that all files, checks, or inputs were verified.

## D-26 Input display names are not path IDs

inputs.name requires uniqueness within a node only, without output-ID character restrictions. Do not rename legal fixtures to satisfy extra implementation limits.

## D-27 Test names describe behavior; comments identify ownership

Task comments support grouping; names omit schedule IDs. Explicit snapshot names prevent function renames from changing byte oracles.

## D-28 Preserve requires declarations and second-precision timestamps

Replies retain resource fields. Timestamp is validated with fixed UTC seconds, avoiding identity loss or parse errors defaulting to zero.

## D-29 Statistics include the current Attempt

begin advances state before generating stats. Recovery publishes exact historical bytes recorded at commit rather than recomputing from current state. Reverse-engineering pre-commit time or retaining duplicate stats breaks frozen bindings.

## D-30 Read release manifests directly for controlled updates

Avoid axoupdater paths invoking installation scripts without full checksum verification. self management uses curl for manifests, verifies bytes, and performs controlled replacement without host shell-configuration effects.

## D-31 Install idempotence means identical bytes

The reliable binary comparison is content. Identical installed bytes remain untouched; the same version with different bytes is a different artifact.
