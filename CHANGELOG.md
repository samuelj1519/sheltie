# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [Unreleased]

### Changed

- English is the default for current documentation, source comments/API docs, CLI/status/editor messages, and new commits; retain explicitly selected Chinese documentation and methods.
- English sample Workbooks use 1.0.1 and spec-dev uses 0.2.3; localized Chinese IDs retain their prior versions. Frozen installed/running versions remain untouched.
- Preserve commit trees/identities/timestamps while translating local history messages, with an explicit old/new mapping. Published remote evidence retains its original identity.
- Use the standard root LICENSE filename with unchanged MIT terms and copyright; add contributor, security, support, conduct, issue, and PR entry points.

### Added

- Current-Attempt briefs, frozen inputs, and declared draft pointers; live status reads revision and pending effects together.
- Required terminal inputs/outputs may explicitly select final results; read-only `work result` returns complete references bound or sealed by that terminal Attempt.
- A minimal code-change method with fixed implementation, independent review, and delivery stages, preserving explicit rework edges.
- Explicit one-time executor replacement per Occurrence, preserving superseded history and inherited frozen inputs; actual-failure retry limits are counted independently.
- Read original bytes through the same result revision/key. The unreleased external sheltie-export verifies every artifact before creating a new editable copy, without overwriting objects and with accurate staging/published-but-unconfirmed states.

### Changed

- spec-dev 0.2.2 introduced shared approval-correction rules and delivery/reflection results through work result after final approval; existing Works retain their frozen method version.

### Breaking Changes

- The development candidate is `0.3.0-rc.1`, with Store schema 4 and public `cli-result/v4`. Attempt creation sequence is number; superseded records replacement reasons. Preserve and reject old schema 1/2/3 roots without automatic migration or clearing.

## [0.2.0]

### Breaking Changes

- Store schema 2 refuses reads/writes to old schema 1 roots and preserves them unchanged, without automatic migration or clearing.
- Use `cli-result/v2`, `workbook-digest/v2`, and the new Attempt layout; read historical v0.1.0 records with the matching old binary/root.

### Fixed

- Request identity binds the actual target; repeated requests return original commit-time responses. Historical artifact recovery verifies bytes/ownership without reverting the current card.
- Workbook/Work publication, sealing, deletion, and crash recovery enforce handle, transaction, and root boundaries; reject duplicate Flows, nested unknown fields, and untrusted persisted state.
- Successful writes safely maintain expired tmp with warning-only failures; pending follows ownership and recovery facts.
- Coordinator instructions cover input discovery, legal next actions, and human gates; spec-dev preserves original baselines, verified prefixes, full requirements, and actual replanning handoff.
- spec-dev 0.2.1 corrected reflection-report Attempt directories while preserving older frozen versions.

### Changed

- Standardize on MIT and package self-contained skills against matching-version contracts.

### Known Limits

- This version releases macOS aarch64 only; other platforms are excluded by the user until needed. Preserve original Linux CI failures.

- Retain historical M1 SK01/SK02 exceptions, 215 missing additional executions, and missing final Spec approval; these do not establish complete mutation/security validation.
- T16 is an actual local Codex scenario; usage is missing, manually supplied skill loading does not verify automatic discovery, and there is no cost/benefit control.
- See the release record for single-platform artifacts, same-SHA quality, and real remote update/rollback; historical local checks do not replace release evidence.

## [0.1.0] - 2026-09-26

### Bug Fixes

- **(ci)** Remove duplicate release job and use cargo-dist only - ([9d60711](https://github.com/samuelj1519/sheltie/commit/9d60711373bba2870059e15b82758385b937e0bd)) - Samuel-J
- **(cli)** Include at in gate-approval data - ([dc05b1d](https://github.com/samuelj1519/sheltie/commit/dc05b1d9cd2c17e72824bd2df52bf7eadd20d8a9)) - Samuel-J
- **(cli)** Remove next_ops_of JSON roundtrip and silent fallback - ([36e8577](https://github.com/samuelj1519/sheltie/commit/36e8577856a4f7f7d6d20a96c6cb56f5bf240e61)) - Samuel-J
- **(cli)** Follow CARGO_PKG_VERSION and register T25/T26 scope - ([7e4243a](https://github.com/samuelj1519/sheltie/commit/7e4243adb76ea9a31ba127c98fc14077dbb00a66)) - Samuel-J
- **(core)** Reject noncanonical numbers, complete Han ranges, correct error fields - ([e55effd](https://github.com/samuelj1519/sheltie/commit/e55effddfbdf94914935b27c76ee2d9f962b0618)) - Samuel-J
- **(core)** Reject required references to optional outputs in rule 5 - ([5f4b6d6](https://github.com/samuelj1519/sheltie/commit/5f4b6d6ece4b8dfa1bb8f646187bae2d4226b006)) - Samuel-J
- **(core)** Carry all requires and resolve declared versions - ([51eda62](https://github.com/samuelj1519/sheltie/commit/51eda626d1dfbf22788451a4d2bad07b27973c15)) - Samuel-J
- **(core)** Identify upstream node for unbound inputs - ([496a182](https://github.com/samuelj1519/sheltie/commit/496a182834bf5ac06dcd2218f8d2e220360d3de2)) - Samuel-J
- **(core)** Enforce readback bounds/normalization and prevent Graph decoding - ([f47af48](https://github.com/samuelj1519/sheltie/commit/f47af4834ec55bc9e8741d3ad8eefdca74030f95)) - Samuel-J
- **(core)** Report stats serialization errors and compute once - ([03a9be9](https://github.com/samuelj1519/sheltie/commit/03a9be9bfe1642fc30ba11170de738ab8e36fdc7)) - Samuel-J
- **(core)** Render resource-kind notes and calculate durations via unix_secs - ([bc14d36](https://github.com/samuelj1519/sheltie/commit/bc14d36c0383ddb506f30479bdaa4a395c2e5a3a)) - Samuel-J
- **(core)** Include current Attempt in engine.stats - ([00eaeeb](https://github.com/samuelj1519/sheltie/commit/00eaeeb9eab574068f2e8e15f8e4c0818cbecf54)) - Samuel-J
- **(core)** Add deny_unknown_fields to persistent structures - ([c5d567c](https://github.com/samuelj1519/sheltie/commit/c5d567c51284fc08a171cdc311acf54ec2db9f8c)) - Samuel-J
- **(release)** Use actual samuelj1519/sheltie repository everywhere - ([7dae6d0](https://github.com/samuelj1519/sheltie/commit/7dae6d01364570085406d2dff698acc61a7d6a2f)) - Samuel-J
- **(runtime)** Rebuild replay stats, correct failpoint, add M2 tests - ([1a5d7a7](https://github.com/samuelj1519/sheltie/commit/1a5d7a76f1a7bada8f198f3218765bc94d298a7b)) - Samuel-J
- **(runtime)** Create Store before install's idempotent shortcut - ([67332b6](https://github.com/samuelj1519/sheltie/commit/67332b6dbe16efcac40cf7bb416b62c0c5a19372)) - Samuel-J
- **(runtime)** Delete discarded rollback binary with remove_file - ([8c5c7cf](https://github.com/samuelj1519/sheltie/commit/8c5c7cfdc7871dbc3fed4bf90e146a322c7faade)) - Samuel-J
- **(runtime)** Adapt actual cargo-dist manifest and archive layout - ([d5d2008](https://github.com/samuelj1519/sheltie/commit/d5d200827e1d86b5b82f00a3d2674522001d1643)) - Samuel-J
- **(scripts)** Support macOS bundled bash 3.2 in check-task.sh - ([8b33c48](https://github.com/samuelj1519/sheltie/commit/8b33c48f1fb5bdfee7d9eac9d6864d94139a4cdf)) - Samuel-J
- **(scripts)** Correct check-task mixed files, shared placeholders, and timing - ([deb31ed](https://github.com/samuelj1519/sheltie/commit/deb31ed8a6c6ce5126954069e3d20cd7fc7926e1)) - Samuel-J
- **(scripts)** Union check-task scope, fix fullwidth colon and empty arrays - ([904a54c](https://github.com/samuelj1519/sheltie/commit/904a54c69ee9eb1fd61e91cb062064fa6b8bcaec)) - Samuel-J
- **(scripts)** Put mutation output under target to keep worktree clean - ([82d870b](https://github.com/samuelj1519/sheltie/commit/82d870ba7192ceff6e7a40fccfb79466830fb912)) - Samuel-J
- **(scripts)** Limit check 2 file entries to .rs too - ([767084b](https://github.com/samuelj1519/sheltie/commit/767084b5b805677316dfe25e3cf82490123a1599)) - Samuel-J

### Documentation

- **(cli)** Correct self_cmd header: install creates store.db - ([dbebdcd](https://github.com/samuelj1519/sheltie/commit/dbebdcd90331a41e8fdfd0862932f64ea7ca0574)) - Samuel-J
- **(readme)** Make install.sh quick start executable end to end - ([e502a92](https://github.com/samuelj1519/sheltie/commit/e502a92a0a25c38fa4493d0d9e4f2904f74254a1)) - Samuel-J
- **(readme)** Address three observed onboarding obstacles - ([022764c](https://github.com/samuelj1519/sheltie/commit/022764c66d100e2f235d7fa2c64a61589b12e13e)) - Samuel-J
- **(specs)** Establish Sheltie specifications, contracts, plan, and spec-dev Workbook - ([82099e1](https://github.com/samuelj1519/sheltie/commit/82099e1870c185e14085e9698947167d33785f6e)) - Samuel-J
- **(specs)** Define work stats blocked/approval accounting - ([ac17547](https://github.com/samuelj1519/sheltie/commit/ac17547e1709db1b2021553ae6b8f2a543f32591)) - Samuel-J
- **(specs)** Correct D-28, document year 9999 saturation, restore D-25 facts - ([64f4647](https://github.com/samuelj1519/sheltie/commit/64f4647dcfcba8890d45dc818aeb4cafdfd31148)) - Samuel-J
- **(specs)** Record M2 review, D-29, and lessons - ([268ec6d](https://github.com/samuelj1519/sheltie/commit/268ec6d7904a7ce9ea40bc7b641395fcad6f5ef8)) - Samuel-J
- **(specs)** Record M3 review and twelve end-to-end scenarios - ([458e947](https://github.com/samuelj1519/sheltie/commit/458e947b021012b67ce8082f90b636d7361c1f58)) - Samuel-J
- **(specs)** Record M3 re-review, D-31, and consistent install semantics - ([a3b63f7](https://github.com/samuelj1519/sheltie/commit/a3b63f78dd49d4d141d5e6257bba8da3a659c0be)) - Samuel-J
- **(workspace)** Replace literal translations with natural Simplified Chinese - ([dc44577](https://github.com/samuelj1519/sheltie/commit/dc4457728352fdec105e0e25128717f09d4d75ac)) - Samuel-J
- **(workspace)** Replace stiff translated nouns with natural Chinese - ([18c65fa](https://github.com/samuelj1519/sheltie/commit/18c65fad1874c6403cf20e24596515b960991bea)) - Samuel-J
- **(workspace)** Standardize response envelope, sole state authority, reference checks - ([fa4710b](https://github.com/samuelj1519/sheltie/commit/fa4710bbe92a3a194313000bba0c1bc6a88964c7)) - Samuel-J

### Features

- **(cli)** workbook add/list/show/remove/verify - ([71930b6](https://github.com/samuelj1519/sheltie/commit/71930b6b5caa88bb4808e5932270a84b54e75f60)) - Samuel-J
- **(cli)** work start/list/status/cancel - ([63dea79](https://github.com/samuelj1519/sheltie/commit/63dea790c2bc8a9a0255b29e57a6e748d3f1938b)) - Samuel-J
- **(cli)** Add attempt begin/submit/fail and gate approve - ([72794ea](https://github.com/samuelj1519/sheltie/commit/72794eaf6dbcd728d77bedbb9e142b636c40d080)) - Samuel-J
- **(cli)** Add self commands and cargo-dist release chain - ([420157e](https://github.com/samuelj1519/sheltie/commit/420157e229e0ac998e66e2796a8de22f3bb53b57)) - Samuel-J
- **(core)** Add typed IDs, relative paths, bounded text, and digests - ([ef584ab](https://github.com/samuelj1519/sheltie/commit/ef584ab0098104db2f73aefacd16b7a106693d95)) - Samuel-J
- **(core)** Parse workbook.toml - ([b292a38](https://github.com/samuelj1519/sheltie/commit/b292a38663aaeb2c6b91f52305132366cc6d0d99)) - Samuel-J
- **(core)** Parse flow/v1 nodes, edges, and input sources - ([7c45ed8](https://github.com/samuelj1519/sheltie/commit/7c45ed80d4158245b99192897760e9df14a07d18)) - Samuel-J
- **(core)** Compile Flow into a validated graph - ([7f23d81](https://github.com/samuelj1519/sheltie/commit/7f23d8167b4cecfb99d76b15f479dfc3812d3e8c)) - Samuel-J
- **(core)** Add Work state, Start command, and initial legal next - ([12a6ade](https://github.com/samuelj1519/sheltie/commit/12a6ade437baedd4e1c0800267f6ed16ae1bc8a7)) - Samuel-J
- **(core)** Add BeginAttempt edge choice, counting, and frozen inputs - ([9044612](https://github.com/samuelj1519/sheltie/commit/9044612f4ab329ca27367a3f1e4cda99c93fb060)) - Samuel-J
- **(core)** Add SubmitAttempt/FailAttempt and output-contract validation - ([4334925](https://github.com/samuelj1519/sheltie/commit/4334925b67ba61a4b340385fe410f351fed727c1)) - Samuel-J
- **(core)** Add gate approval, cancellation, and terminal guard - ([810730f](https://github.com/samuelj1519/sheltie/commit/810730fabf20d02b7e42649c2228752226eaebb1)) - Samuel-J
- **(core)** Render briefs, cards, and next projections - ([8af5f94](https://github.com/samuelj1519/sheltie/commit/8af5f9491d5fe8467b9f99df12a5bd6525cde513)) - Samuel-J
- **(retro)** Add reflection node, engine.stats input, and work stats facts - ([6a26fc6](https://github.com/samuelj1519/sheltie/commit/6a26fc60d0e97329f74549c077b6224aca2cdfd7)) - Samuel-J
- **(runtime)** Resolve management roots, confine paths, observe files - ([9b2d00f](https://github.com/samuelj1519/sheltie/commit/9b2d00f4a39742f7cb6adaaaef70d4c224d4346a)) - Samuel-J
- **(runtime)** Add SQLite validation, deduplication, revision CAS, sequences - ([a860311](https://github.com/samuelj1519/sheltie/commit/a860311a94c042ef726bcd1146e5d9d74cab6199)) - Samuel-J
- **(runtime)** Add Workbook repository and atomic staging registration - ([00bd835](https://github.com/samuelj1519/sheltie/commit/00bd835f52e5147afcbfc8a88708ac6e0af1db20)) - Samuel-J
- **(runtime)** Add Workbook removal reference checks and verify - ([fc4f9f9](https://github.com/samuelj1519/sheltie/commit/fc4f9f97312affa62c4c69255a04c68abac74ce0)) - Samuel-J
- **(runtime)** Complete Work commands and read-only views - ([2344894](https://github.com/samuelj1519/sheltie/commit/2344894fd4bae0aafe63f3200811b8c089282dd2)) - Samuel-J
- **(skill)** Deliver sheltie skill and command allowlist checks - ([86ef062](https://github.com/samuelj1519/sheltie/commit/86ef062e5bccbf1966cd63444776f8601b714a46)) - Samuel-J

### Miscellaneous Chores

- **(release)** Move dist config, fix install path, regenerate workflow - ([5286e97](https://github.com/samuelj1519/sheltie/commit/5286e97d4392ede3a46dc2b4ca4306a58fef893d)) - Samuel-J
- **(review)** Recheck T02, add three boundary tests, refine body-filling workflow - ([9d73175](https://github.com/samuelj1519/sheltie/commit/9d731753721113077db040dd4da7db78247f6673)) - Samuel-J
- **(review)** Accept T02 rebuttals and fix scaffold residue/tool blind spots - ([a9ed84c](https://github.com/samuelj1519/sheltie/commit/a9ed84c742546b8f3f92b7f06514a3d23ff88a5e)) - Samuel-J
- **(review)** Correct T05 whitespace-test replacement - ([ac5eb9c](https://github.com/samuelj1519/sheltie/commit/ac5eb9cfdd19b0c8d07ab8db4bed373d20cdfff9)) - Samuel-J
- **(review)** Correct example/fixture input names violating ID rules - ([00ca15a](https://github.com/samuelj1519/sheltie/commit/00ca15adc07f8aeb471f8e88b7ebf41c17a59ace)) - Samuel-J
- **(review)** Pin T05 rule 9 rejection - ([8aad2e9](https://github.com/samuelj1519/sheltie/commit/8aad2e95cefd11bd3f9cb28271a947643a47e50d)) - Samuel-J
- **(review)** Complete spec-dev files and repair review-loop fixture - ([43ec6bf](https://github.com/samuelj1519/sheltie/commit/43ec6bf3bd95e6097b6bbf0e71b3c58fad9818a7)) - Samuel-J
- **(review)** Validate input-name uniqueness only, revert kebab renames - ([685d4f2](https://github.com/samuelj1519/sheltie/commit/685d4f2040b1a85d4693d96c7eb8c955bdac288d)) - Samuel-J
- **(workspace)** Scaffold three crates, disabled tests, and task tools - ([461f5c7](https://github.com/samuelj1519/sheltie/commit/461f5c7a3ec4db47ee6042922da5eac2cf0bcb51)) - Samuel-J

### Refactoring

- **(core)** Remove duplicate summary checks and early return - ([6290c61](https://github.com/samuelj1519/sheltie/commit/6290c613fc07354aab06f25a1e10dffaba60cc0f)) - Samuel-J

### Tests

- **(cli)** Cover review back edges, visit limits, and reference inputs - ([96c79f9](https://github.com/samuelj1519/sheltie/commit/96c79f9307e2e89925faa63c12d21c80017aafed)) - Samuel-J
- **(cli)** Cover gates, integrity, and Workbook lifecycle - ([3c62ead](https://github.com/samuelj1519/sheltie/commit/3c62eadd060fb5188f925e97bbc5975348578752)) - Samuel-J
- **(core)** Add M1 tests, remove dead code, record workflow lessons - ([6ace5a3](https://github.com/samuelj1519/sheltie/commit/6ace5a30dc74a8bf62bcd0ee394deb02cfc6d7cd)) - Samuel-J
- **(core)** Add M1 round 2 oracles, full requires, strict Timestamp - ([caa9d23](https://github.com/samuelj1519/sheltie/commit/caa9d2365474f789b65065a837efe2637e48e50c)) - Samuel-J
- **(core)** Close M1 after cross-declaration and distant-date tests - ([94519d7](https://github.com/samuelj1519/sheltie/commit/94519d7b59c2c6268c7581641894b90632fd48ed)) - Samuel-J
- **(core)** Compile example Workbooks under contracts - ([9ca2526](https://github.com/samuelj1519/sheltie/commit/9ca25261cf85a297723d631a5c9c9c1b1a792ef3)) - Samuel-J
- **(runtime)** Keep directories traversable in T15 missing-directory test - ([fe7f5fe](https://github.com/samuelj1519/sheltie/commit/fe7f5feb056fdf77cc81d8a10c06cf9d80edbb2c)) - Samuel-J
- **(runtime)** Cover crashes, request replay, and upgrade recovery - ([64c34ad](https://github.com/samuelj1519/sheltie/commit/64c34ad2cc0acbf90ef0b7944b9861a7b4697793)) - Samuel-J
- **(runtime)** Add M3 release/install oracles - ([fe91704](https://github.com/samuelj1519/sheltie/commit/fe91704601948d8a9c302501b875212fad3e60d7)) - Samuel-J
- **(runtime)** Cover .tgz archive suffix - ([7008803](https://github.com/samuelj1519/sheltie/commit/700880308c03deea8eaa4611f657e39bf46c304d)) - Samuel-J
- Use behavior names and // Task ownership markers - ([f1a8715](https://github.com/samuelj1519/sheltie/commit/f1a871553b7992f044e2f83f70ae5fb8576c14da)) - Samuel-J

Historical GitHub commit links above identify original published history. Mapped local identities are recorded in [the message-migration map](docs/en/history/git-message-migration.tsv).

<!-- generated by git-cliff -->
