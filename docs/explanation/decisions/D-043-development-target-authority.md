# D-043: Development-target authority for unreleased candidates

English | [简体中文](D-043-development-target-authority.zh-CN.md)

Status: `accepted`
Date: 2026-10-03
Related change: [C006](../../history/changes/C006-result-delivery/README.md)

An unreleased RC may remain after product implementation, followed by experiments outside the product release. Before this decision, check-specs derived the base version only from an active target and wrongly rejected the same RC after product-package archival.

One Development target: field on the first screen of specs/README specifies the numeric base version. It describes the source candidate's target; active plans describe implementation progress. Numeric active product targets must match. With non-product experiments or no active change, Cargo base versions are still checked against that target, allowing the target itself or its RC. Unknown, missing, duplicate, or invalid authority fields fail closed.

Released Cargo versions remain individually checked against release records, tags, history, and CHANGELOG; development targets do not replace them. Completed product packages, passing checks, or adopted experiments neither establish release nor create tags/uploads. C006-T05 independently verified this governance repair. Read the specific target only from the [specification index](../../../specs/README.md), without inferring it from completed version numbers.
