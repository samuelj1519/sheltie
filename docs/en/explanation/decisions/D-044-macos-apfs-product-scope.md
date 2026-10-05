# D-044: Current product scope is local macOS/APFS use

English | [简体中文](../../../zh-CN/explanation/decisions/D-044-macos-apfs-product-scope.md)

Status: `accepted`
Date: 2026-10-05
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md), [C006](../../history/changes/C006-result-delivery/README.md), [C008](../../history/changes/C008-dependency-readiness/README.md)

## Context and decision

The user explicitly confirmed that the product targets the current macOS/APFS use case. Previous remaining-work lists treated physical non-UTF-8 names unconstructible on current APFS and dedicated external-physical-disk acceptance as missing environments, risking confusion with required delivery acceptance. The C008 adoption record in [fixed snapshots](../../how-to/maintain-docs.md#read-original-historical-records) preserves the original statement and original-evidence boundaries.

The [product specification](../../../../specs/spec.md#current-product-environment) is the sole environment authority. Existing macOS aarch64 scope remains. Non-APFS, other OS/architectures, and dedicated external-device certification are not required for current delivery. E01/E02 are not required under current scope; historical not_run/environment_blocked remain. Adopt new targets/media separately when actual use requires them.

## Consequences and verification

Current APFS rejects physical anomalous names at creation. Accurate engine rejection contracts for invalid names/parameters, existing byte interfaces, and real argv tests remain. Export integrity, permissions, atomic non-overwrite, sync, and error-state obligations still apply; the lack of a power-loss durability promise is unchanged.

This decision limits support commitments and acceptance scope only. It changes no code, CLI/Store/Workbook formats, historical Works, or tests, and adds no filesystem whitelist. Scope changes do not establish historical fairness, cost, 15% net benefit, or a new release. Independent factual review and documentation/specification gates verify updated lists.
