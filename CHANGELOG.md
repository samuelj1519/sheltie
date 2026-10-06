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

- **(ci)** Remove duplicate release job and use cargo-dist only - ([0ba0ea6](https://github.com/samuelj1519/sheltie/commit/0ba0ea6fd18ca4933a31d7d1ed126eaf939dc98c)) - Samuel-J
- **(cli)** Include at in gate-approval data - ([06d0460](https://github.com/samuelj1519/sheltie/commit/06d04603a49fe2c62f5e070bec1fe3e21a1a78f3)) - Samuel-J
- **(cli)** Remove next_ops_of JSON roundtrip and silent fallback - ([0691e0b](https://github.com/samuelj1519/sheltie/commit/0691e0b7b5574c1dfed7a49f02635789982891e9)) - Samuel-J
- **(cli)** Follow CARGO_PKG_VERSION and register T25/T26 scope - ([8e46759](https://github.com/samuelj1519/sheltie/commit/8e467591db2fca0e98d7cf66293e0ab207847c49)) - Samuel-J
- **(core)** Reject noncanonical numbers, complete Han ranges, correct error fields - ([448f650](https://github.com/samuelj1519/sheltie/commit/448f6509d0fd2833f49a27e9599cd28c776b73cf)) - Samuel-J
- **(core)** Reject required references to optional outputs in rule 5 - ([e6f41c2](https://github.com/samuelj1519/sheltie/commit/e6f41c2b85296e0bc612198be3b77685a9546132)) - Samuel-J
- **(core)** Carry all requires and resolve declared versions - ([0ca633f](https://github.com/samuelj1519/sheltie/commit/0ca633fd0b30ca470f37dd6391e7950476cb8642)) - Samuel-J
- **(core)** Identify upstream node for unbound inputs - ([d126736](https://github.com/samuelj1519/sheltie/commit/d1267362e33e79abcbc0e82329eede169c4ee86a)) - Samuel-J
- **(core)** Enforce readback bounds/normalization and prevent Graph decoding - ([65e85ad](https://github.com/samuelj1519/sheltie/commit/65e85ad9abb3bac3a0949939623156278c0484af)) - Samuel-J
- **(core)** Report stats serialization errors and compute once - ([dc1cc43](https://github.com/samuelj1519/sheltie/commit/dc1cc437715cba04b3694ef10a12c73d8f00c0f9)) - Samuel-J
- **(core)** Render resource-kind notes and calculate durations via unix_secs - ([d199b2c](https://github.com/samuelj1519/sheltie/commit/d199b2c3d7a076fd039677b697b6b84c6d9e83f0)) - Samuel-J
- **(core)** Include current Attempt in engine.stats - ([c767989](https://github.com/samuelj1519/sheltie/commit/c767989a24d821eec504662b980c4f744c4d3f31)) - Samuel-J
- **(core)** Add deny_unknown_fields to persistent structures - ([eb4ca85](https://github.com/samuelj1519/sheltie/commit/eb4ca85540888daebbef1eeb0bd7fc8db835038c)) - Samuel-J
- **(release)** Use actual samuelj1519/sheltie repository everywhere - ([4f5912f](https://github.com/samuelj1519/sheltie/commit/4f5912f25d0ad8d2a8e3ac6e19f09cbc52d0e4a7)) - Samuel-J
- **(runtime)** Rebuild replay stats, correct failpoint, add M2 tests - ([f2d6323](https://github.com/samuelj1519/sheltie/commit/f2d63231b5688acea5ae77b16f0be0c7ebe29525)) - Samuel-J
- **(runtime)** Create Store before install's idempotent shortcut - ([1a55b0b](https://github.com/samuelj1519/sheltie/commit/1a55b0b7bb1bcf189cbf70bf99bc618154e8906f)) - Samuel-J
- **(runtime)** Delete discarded rollback binary with remove_file - ([8a259c2](https://github.com/samuelj1519/sheltie/commit/8a259c2aa606281ba91df55ecab4efee4df30627)) - Samuel-J
- **(runtime)** Adapt actual cargo-dist manifest and archive layout - ([a0c1396](https://github.com/samuelj1519/sheltie/commit/a0c139601b43fa8a46d591f807f8829e1528c7e0)) - Samuel-J
- **(scripts)** Support macOS bundled bash 3.2 in check-task.sh - ([4ca7e55](https://github.com/samuelj1519/sheltie/commit/4ca7e55961263836c2101367b1fbe406e50a446f)) - Samuel-J
- **(scripts)** Correct check-task mixed files, shared placeholders, and timing - ([9027006](https://github.com/samuelj1519/sheltie/commit/9027006cd013f93bb050c5d49b0af4122696c4b6)) - Samuel-J
- **(scripts)** Union check-task scope, fix fullwidth colon and empty arrays - ([3bcec2f](https://github.com/samuelj1519/sheltie/commit/3bcec2f70e5e581f41ad3feb28e9e494281ca9cf)) - Samuel-J
- **(scripts)** Put mutation output under target to keep worktree clean - ([d908f85](https://github.com/samuelj1519/sheltie/commit/d908f859986c5190026c928c0c9a37ef7760f5d9)) - Samuel-J
- **(scripts)** Limit check 2 file entries to .rs too - ([502882b](https://github.com/samuelj1519/sheltie/commit/502882bd14c6d290f311795c1ade88fb3f88be9a)) - Samuel-J

### Documentation

- **(cli)** Correct self_cmd header: install creates store.db - ([edc097b](https://github.com/samuelj1519/sheltie/commit/edc097bd1f08a732738795407330d60bfdf6192d)) - Samuel-J
- **(readme)** Make install.sh quick start executable end to end - ([c92dd4d](https://github.com/samuelj1519/sheltie/commit/c92dd4d162e6d5ab6e943b4ec4f8c70cfa16c449)) - Samuel-J
- **(readme)** Address three observed onboarding obstacles - ([d2f8903](https://github.com/samuelj1519/sheltie/commit/d2f8903e8217926c1e407035fda3f4e7080b9c75)) - Samuel-J
- **(specs)** Establish Sheltie specifications, contracts, plan, and spec-dev Workbook - ([24a2dfc](https://github.com/samuelj1519/sheltie/commit/24a2dfc903aa817f53c979b3dcb6a68e417e4258)) - Samuel-J
- **(specs)** Define work stats blocked/approval accounting - ([80e576c](https://github.com/samuelj1519/sheltie/commit/80e576cc5fe77402c44b34d99333766a839496cb)) - Samuel-J
- **(specs)** Correct D-28, document year 9999 saturation, restore D-25 facts - ([5efdde7](https://github.com/samuelj1519/sheltie/commit/5efdde78e20c6586e4da9afe0c97ad4de460041e)) - Samuel-J
- **(specs)** Record M2 review, D-29, and lessons - ([9fd7bbe](https://github.com/samuelj1519/sheltie/commit/9fd7bbe4b092d27cccd715c61f9a5ce642942b40)) - Samuel-J
- **(specs)** Record M3 review and twelve end-to-end scenarios - ([7586042](https://github.com/samuelj1519/sheltie/commit/7586042b69d6bf308350e02d90812d2f825fc54b)) - Samuel-J
- **(specs)** Record M3 re-review, D-31, and consistent install semantics - ([e3aa914](https://github.com/samuelj1519/sheltie/commit/e3aa91452bc95c4c9e7fc1f5f698de0df4ff7bec)) - Samuel-J
- **(workspace)** Replace literal translations with natural Simplified Chinese - ([0767b84](https://github.com/samuelj1519/sheltie/commit/0767b842269172720798ddf1feaad30fe3509f0b)) - Samuel-J
- **(workspace)** Replace stiff translated nouns with natural Chinese - ([90deda9](https://github.com/samuelj1519/sheltie/commit/90deda99c7d554dba4e7ac8cdbd9c3096a2a0a9b)) - Samuel-J
- **(workspace)** Standardize response envelope, sole state authority, reference checks - ([b89653b](https://github.com/samuelj1519/sheltie/commit/b89653b8e01995821ae66b5cdc66376649fa0c69)) - Samuel-J

### Features

- **(cli)** workbook add/list/show/remove/verify - ([b5cf46f](https://github.com/samuelj1519/sheltie/commit/b5cf46f7dc26afc52dbf2dcc80ca9c33aba2e3b0)) - Samuel-J
- **(cli)** work start/list/status/cancel - ([f2db5b6](https://github.com/samuelj1519/sheltie/commit/f2db5b62c6879f5bab649d8e75073ab2c7518a2e)) - Samuel-J
- **(cli)** Add attempt begin/submit/fail and gate approve - ([9d3bdd7](https://github.com/samuelj1519/sheltie/commit/9d3bdd790175d96f184179682e441a631af2781f)) - Samuel-J
- **(cli)** Add self commands and cargo-dist release chain - ([0bc7e6e](https://github.com/samuelj1519/sheltie/commit/0bc7e6eefe9e1757647fd1cfe2b365bd141c2f11)) - Samuel-J
- **(core)** Add typed IDs, relative paths, bounded text, and digests - ([56f48c9](https://github.com/samuelj1519/sheltie/commit/56f48c9da17172a438fd96fd618d9035f026ef0f)) - Samuel-J
- **(core)** Parse workbook.toml - ([bb6c8dd](https://github.com/samuelj1519/sheltie/commit/bb6c8ddf798f529458bcc8e071a8764b70d706b4)) - Samuel-J
- **(core)** Parse flow/v1 nodes, edges, and input sources - ([6919528](https://github.com/samuelj1519/sheltie/commit/69195282b970941f48ff04147820ddb5085d90c9)) - Samuel-J
- **(core)** Compile Flow into a validated graph - ([776e6f7](https://github.com/samuelj1519/sheltie/commit/776e6f7ab82bb3b30895a1200bcb1f1896549860)) - Samuel-J
- **(core)** Add Work state, Start command, and initial legal next - ([ba05cb7](https://github.com/samuelj1519/sheltie/commit/ba05cb7534cbfa8f651968d393563fca5383f03b)) - Samuel-J
- **(core)** Add BeginAttempt edge choice, counting, and frozen inputs - ([dae16cb](https://github.com/samuelj1519/sheltie/commit/dae16cb559b645e1866b3d8ad6dcba60184074c5)) - Samuel-J
- **(core)** Add SubmitAttempt/FailAttempt and output-contract validation - ([78e9a12](https://github.com/samuelj1519/sheltie/commit/78e9a126e0e373766122d72b6b4685bdce4c4f03)) - Samuel-J
- **(core)** Add gate approval, cancellation, and terminal guard - ([7694854](https://github.com/samuelj1519/sheltie/commit/76948544e1588cdf33467e93617a2407fea66c9b)) - Samuel-J
- **(core)** Render briefs, cards, and next projections - ([aa80ca1](https://github.com/samuelj1519/sheltie/commit/aa80ca1909ad263ed014675ee52e1437286875e8)) - Samuel-J
- **(retro)** Add reflection node, engine.stats input, and work stats facts - ([8ccaae0](https://github.com/samuelj1519/sheltie/commit/8ccaae0e51a24d6917964f82308aa5fb954f7d48)) - Samuel-J
- **(runtime)** Resolve management roots, confine paths, observe files - ([fce89af](https://github.com/samuelj1519/sheltie/commit/fce89af53a42516f136596de0a7b94ba1a2f2aac)) - Samuel-J
- **(runtime)** Add SQLite validation, deduplication, revision CAS, sequences - ([ce400eb](https://github.com/samuelj1519/sheltie/commit/ce400eba05dfd7907ff0d56565ca07c272f57574)) - Samuel-J
- **(runtime)** Add Workbook repository and atomic staging registration - ([391f6ef](https://github.com/samuelj1519/sheltie/commit/391f6ef2db32085d283ed73550716d5ba54a0efe)) - Samuel-J
- **(runtime)** Add Workbook removal reference checks and verify - ([debade8](https://github.com/samuelj1519/sheltie/commit/debade8c583a01b6b2298bc45d469e7a8c519539)) - Samuel-J
- **(runtime)** Complete Work commands and read-only views - ([60abbc6](https://github.com/samuelj1519/sheltie/commit/60abbc610095a9c24de52106184f423494b3e375)) - Samuel-J
- **(skill)** Deliver sheltie skill and command allowlist checks - ([ec94090](https://github.com/samuelj1519/sheltie/commit/ec94090e5917191e262a5d208a614ff1bf30f4d1)) - Samuel-J

### Miscellaneous Chores

- **(release)** Move dist config, fix install path, regenerate workflow - ([7e8673d](https://github.com/samuelj1519/sheltie/commit/7e8673d921890d69c8dd4b1ba9ebd20cff81ed63)) - Samuel-J
- **(review)** Recheck T02, add three boundary tests, refine body-filling workflow - ([07e9e10](https://github.com/samuelj1519/sheltie/commit/07e9e10ec072b2d31db45d2b463069a6b3370039)) - Samuel-J
- **(review)** Accept T02 rebuttals and fix scaffold residue/tool blind spots - ([a9812bc](https://github.com/samuelj1519/sheltie/commit/a9812bcc07dc7c8ab4982a73a8d51c0d166efb0b)) - Samuel-J
- **(review)** Correct T05 whitespace-test replacement - ([5672127](https://github.com/samuelj1519/sheltie/commit/567212726d841c504a93591ec90acdfd8bb7f453)) - Samuel-J
- **(review)** Correct example/fixture input names violating ID rules - ([3e136a2](https://github.com/samuelj1519/sheltie/commit/3e136a2364617d99bf3439795e01d269a7ed64f0)) - Samuel-J
- **(review)** Pin T05 rule 9 rejection - ([6beaf9d](https://github.com/samuelj1519/sheltie/commit/6beaf9d1b3d23392d75518482dcdd2ce40eb20a6)) - Samuel-J
- **(review)** Complete spec-dev files and repair review-loop fixture - ([b296d99](https://github.com/samuelj1519/sheltie/commit/b296d9959aa713190b22f2127182d0b2d5fa0622)) - Samuel-J
- **(review)** Validate input-name uniqueness only, revert kebab renames - ([d4f75c6](https://github.com/samuelj1519/sheltie/commit/d4f75c622725364799221f901b92c90dfd254ab2)) - Samuel-J
- **(workspace)** Scaffold three crates, disabled tests, and task tools - ([2af9e7d](https://github.com/samuelj1519/sheltie/commit/2af9e7dfb66860d34ad38237e5d6888a93b719ba)) - Samuel-J

### Refactoring

- **(core)** Remove duplicate summary checks and early return - ([a02c166](https://github.com/samuelj1519/sheltie/commit/a02c16648098b19c3162ff2373fcd9f422297c01)) - Samuel-J

### Tests

- **(cli)** Cover review back edges, visit limits, and reference inputs - ([ee001cd](https://github.com/samuelj1519/sheltie/commit/ee001cd7f39be42ab6f3615636997dabab89f1c0)) - Samuel-J
- **(cli)** Cover gates, integrity, and Workbook lifecycle - ([53b3c9a](https://github.com/samuelj1519/sheltie/commit/53b3c9ab3c11b5d76841d18190e4a21fa5a1dabf)) - Samuel-J
- **(core)** Add M1 tests, remove dead code, record workflow lessons - ([d585b6f](https://github.com/samuelj1519/sheltie/commit/d585b6f2c6ac3d8ef39fa9d4f2e782081425179e)) - Samuel-J
- **(core)** Add M1 round 2 oracles, full requires, strict Timestamp - ([e260462](https://github.com/samuelj1519/sheltie/commit/e2604629aec72d5cb9a47ede63d552ab22ec1062)) - Samuel-J
- **(core)** Close M1 after cross-declaration and distant-date tests - ([0c20e7c](https://github.com/samuelj1519/sheltie/commit/0c20e7c801553402d2d4a171ef45673390902c19)) - Samuel-J
- **(core)** Compile example Workbooks under contracts - ([70ad3d2](https://github.com/samuelj1519/sheltie/commit/70ad3d2995f868f04970ef1b3261c396e0aa6712)) - Samuel-J
- **(runtime)** Keep directories traversable in T15 missing-directory test - ([8b6654b](https://github.com/samuelj1519/sheltie/commit/8b6654bad41e86db7bc75907b2862ddbdb788e42)) - Samuel-J
- **(runtime)** Cover crashes, request replay, and upgrade recovery - ([24d098e](https://github.com/samuelj1519/sheltie/commit/24d098e99213aadc7244577879f4c8b39e8212e9)) - Samuel-J
- **(runtime)** Add M3 release/install oracles - ([81a3970](https://github.com/samuelj1519/sheltie/commit/81a3970c5126e9148e66cd6fee5b62c5a865fa66)) - Samuel-J
- **(runtime)** Cover .tgz archive suffix - ([53f1049](https://github.com/samuelj1519/sheltie/commit/53f104908ec5b3a30a27777874852c3019dbcd20)) - Samuel-J
- Use behavior names and // Task ownership markers - ([c5193dd](https://github.com/samuelj1519/sheltie/commit/c5193dd6ce213c8107b62771ba4fd8a9d7ecff8e)) - Samuel-J

Commit links above point to the corresponding commits in `main` history. Release records retain original publication identities and artifact checksums.

<!-- generated by git-cliff -->
