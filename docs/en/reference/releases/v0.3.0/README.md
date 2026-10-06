# v0.3.0 release record

English | [简体中文](../../../../zh-CN/reference/releases/v0.3.0/README.md)

Status: `released`
Version: `0.3.0`
Git tag: `v0.3.0`
Release commit: `063029a9d9a41ef97ee8be48c12187312e599d1c`
Engineering acceptance closure: `063029a9d9a41ef97ee8be48c12187312e599d1c`
Published: `2026-10-06T05:19:15Z`
Release target: `aarch64-apple-darwin`

## Scope

The first supported public baseline is the current local macOS Apple Silicon/APFS implementation. Older v0.1.0/v0.2.0 publications and their tags/assets are withdrawn under [D-046](../../../explanation/decisions/D-046-first-public-baseline.md). Their development history and original qualification remain verifiable by fixed commits. No legacy compatibility or data migration is supported; formats remain schema 4, cli-result/v4, work-result/v1, and workbook-digest/v2.

Ships the engine, self-contained coordinator skill, source archive, installer, manifest, and checksums. sheltie-export remains source-only and is excluded from the engine archive; the visual Workbook tool is used from the source tree. Workbooks retain their own versions. The source tag retains its pre-publication status; current publication facts are recorded here after actual acceptance.

## Acceptance

- [Release pipeline 37417410662](https://github.com/samuelj1519/sheltie/actions/runs/37417410662) passed quality, plan, native build, global build, host, and announce on the exact Release commit. CI Nextest: 961 passed, 0 skipped, 1 slow; locked MSRV 1.85, dependency and governance gates passed.
- Independent Standards and Spec reviews passed. [C013 validation and original output](../../../../../specs/changes/completed/C013-first-public-baseline/validation.md) retain initial setup failures, exact input closure, and actual publication evidence.
- Downloaded actual public assets, verified API sizes/SHA256 and distribution-manifest hashes, and checked Mach-O arm64. The public installer installed only in a temporary unmanaged directory; its binary exactly matched the extracted public package.
- A fresh independent management root completed installation and the two-step Work. Default GitHub-source pinned update from an unpublished 0.3.0-rc.1/schema-4 source binary to public 0.3.0 and rollback preserved exact original Store and binary bytes. This does not establish schema downgrade or old-release migration.
- The downloaded skill passed same-source self-contained delivery verification. Remote readback confirmed only v0.3.0 remains published/tagged.

| Artifact | SHA256 |
| --- | --- |
| [macOS aarch64 package](https://github.com/samuelj1519/sheltie/releases/download/v0.3.0/sheltie-cli-aarch64-apple-darwin.tar.xz), 1833540 bytes | `1d4e79b4bbf0ac4d0e563bb0dc0cd3dad8579a84a2bc2a865b1f9a37f77cc900` |
| Packaged `sheltie` binary | `219b884870d6a1aa824b3481df8065dee49753206284e0e2996e75586bc3cf79` |
| [dist-manifest.json](https://github.com/samuelj1519/sheltie/releases/download/v0.3.0/dist-manifest.json) | `96db972900ef4494a857d1f0f0fd920249c2f6f0570222edd3ebbe98bcc55281` |
| [Coordinator skill](https://github.com/samuelj1519/sheltie/releases/download/v0.3.0/sheltie-skill.tar.gz) | `6613c9655142d63d7abeecc47318098712a6e0de3846826ee56639c6eaa2eb89` |

## Known limits

Only macOS aarch64/APFS is supported. Linux/x86_64/non-APFS and external physical-device certification are excluded. Historical native mutation, security/Spec approval, LEAK attribution, usage, cost, and fair-comparison gaps remain as recorded; new release gates do not turn them into PASS. Gates identify the actual OS account without independent human authentication. Executor replacement does not stop/isolate processes. Content quality, host readiness, and measurable benefit remain separate from technical publication; see [limits](../../limitations.md) and [acceptance](../../acceptance.md).
