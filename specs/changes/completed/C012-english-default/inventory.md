# Language inventory

The baseline is 206a944491acfa9fa4843ced06103427e13054aa. Counts refer to tracked baseline files, excluding binary content. The original default branch history has 230 commits; all local refs, including archives and stash, reach 260 commits, 256 with Chinese messages. No reachable commits are signed. Existing release and review tags are lightweight commit refs.

| Content | Baseline files containing Han text | Decision |
| --- | --- | --- |
| Root README, vocabulary, agent instructions, changelog | 5 | Keep explicit Chinese copies and translate defaults |
| docs | 68 | Keep .zh-CN.md copies and translate defaults; preserve raw historical evidence |
| specs | 18 | Keep .zh-CN.md copies and translate defaults; migrate governance parsers |
| crates | 135 | Translate comments and human messages; retain meaningful Unicode data |
| scripts | 30 | Translate comments/diagnostics and update language-sensitive consumers |
| tools | 23 | Translate editor UI, comments, and harmless test data |
| examples | 19 | Preserve Chinese methods and translate defaults with explicit language selection |
| workbooks | 27 | Preserve Chinese spec-dev and translate the default method with a new version |
| skills | 1 | Preserve Chinese instructions and use English by default |

## Data and evidence decisions

- Retain Han-range/name normalization fixtures: they test the adopted work-name contract rather than English prose.
- Retain Chinese paths, keys, and shell-metacharacter combinations when they independently test Unicode, literal argv, confinement, or integrity behavior.
- Retain byte-limit fixtures whose multibyte size is the independent oracle; translating them would change the test condition.
- Translate ordinary demonstration topics, summaries, harmless content fixtures, assertions, and debug diagnostics where their language has no behavioral purpose.
- Preserve original historical logs, frozen snapshot file trees, hashes, platform exclusions, waivers, failures, and not_run outcomes. Translate explanatory prose without reclassifying results.
- There is no authorized user-data migration under ~/.sheltie. Existing Work/Workbook frozen copies and artifacts remain untouched.
- No capability-bearing tests or original evidence are deleted merely because they contain Chinese.

A final path-level residual inventory will record actual retained Han text after migration. Bilingual translations are maintained content, not prohibited residuals.

## Retained Rust input and persistence text

| File | Current lines | Purpose |
| --- | --- | --- |
| `crates/sheltie-cli/tests/result_artifact.rs` | 31, 120 | Unicode and shell-sensitive artifact keys travel literally through argv and result retrieval. |
| `crates/sheltie-cli/tests/language_default.rs` | 84, 95 | Independent end-to-end proof that English briefs/status preserve original Chinese user input bytes and summaries, including emoji. |
| `crates/sheltie-cli/tests/work.rs` | 158, 163 | Han Work-name whitespace normalization and real CLI WorkId roundtrip. |
| `crates/sheltie-core/src/ids.rs` | 406, 407, 408, 414, 421, 554, 563, 567, 568 | Han WorkName normalization, canonical deserialization, WorkId roundtrip, and a 51-byte UTF-8 rejection boundary. |
| `crates/sheltie-core/src/text.rs` | 73, 74 | Two Han characters are exactly six UTF-8 bytes: reject limit5, accept limit6. |
| `crates/sheltie-core/src/flow/parse.rs` | 625 | Non-ASCII output-path rejection, paired with ASCII/full-width/separator rejection controls. |
| `crates/sheltie-export/tests/export.rs` | 36 | Byte-exact copy/provenance comparison with original Unicode user content, shared by the completed fixture. |
| `crates/sheltie-export/tests/crash.rs` | 125, 204 | Byte-exact crash/export comparison with original Unicode user content, shared by the completed fixture. |
| `crates/sheltie-export/tests/source.rs` | 134, 156, 329 | Unicode source artifact leaf acceptance, non-ASCII Workbook-version rejection, and literal shell-sensitive Unicode keys. |
| `crates/sheltie-export/tests/target.rs` | 110, 147 | Unicode result keys and duplicate artifact filenames remain distinguishable in exported manifest mappings. |
| `crates/sheltie-export/src/output.rs` | 223, 234, 239 | Unicode user destination path is preserved in structured and readable English export responses. |
| `crates/sheltie-export/tests/common/mod.rs` | 125 | Original Unicode user task becomes a selected frozen result input and must export without translation. |
| `crates/sheltie-runtime/tests/crash.rs` | 18, 19, 43 | Unicode Work name and original 23-byte topic occur in an independently handwritten complete status-card byte oracle. |
| `crates/sheltie-runtime/tests/service.rs` | 30, 463, 1321, 1967, 2070, 2072, 2151, 2173, 2319, 2358, 2404, 2412, 2477 | Original 23-byte Unicode topic and one-character same-length mutation preserve independently handwritten digest/size, integrity/recovery and exact-byte oracles; the exact length-unit literal asserts the persisted audit marker. |
| `crates/sheltie-runtime/tests/start_preflight.rs` | 94, 98 | Han Work-name normalization after failed preflight preserves the daily sequence and exact normalized WorkId suffix. |
| `crates/sheltie-runtime/tests/result.rs` | 49 | Original Unicode start-input byte size matches the result metadata reference. |
| `crates/sheltie-runtime/src/service.rs` | 2119 | Exact <n 字节> persisted audit.command_json marker; changing it would break historical Command/audit equality without a persistence-format migration. |
| `crates/sheltie-runtime/src/selfmgmt.rs` | 964 | Valid Unicode asset filename acceptance; only unsafe path segments are rejected. |
| `crates/sheltie-runtime/tests/common/mod.rs` | 183 | Shared original 23-byte Unicode topic supports independently handwritten digest/size and same-length integrity mutation oracles. |

Exact literals/actions are in [retained-rust-text.json](retained-rust-text.json): 52 retained source lines across 19 files. These are Unicode/user-data/persistence oracles, not untranslated comments. No tests or original evidence were deleted.

## Retained editor inputs

`tools/workbook-editor/test/clear-graph.test.mjs` retains the input names `中文_方案`, `计划_模板`, `计划_模板-2`, and `中文新名称` to test node-local Unicode/underscore names, alias conflicts, renaming and unchanged reference bytes. Other editor fixture prose/labels are English. Exact locations are in [retained-editor-text.json](retained-editor-text.json).

## Other language-bearing literals

- `.zh-CN.md` files, explicitly localized `*-zh-CN` method trees, and `简体中文` selectors are maintained translations/navigation.
- `specs/contracts/storage.md` retains `<n 字节>` because it is the audit machine-format marker.
- `scripts/check-specs.sh` retains three Chinese field patterns to validate unchanged original completion snapshots.
- `scripts/check-docs.sh` retains Chinese prohibited-word patterns alongside English patterns to govern both documentation languages.
- Original published commit fields, upstream GitHub URLs, the TSV original-ID column, and historical source trees are preserved evidence/identity.
- Existing user data under management roots is untouched; historical replay responses and artifacts retain their original language/bytes.
