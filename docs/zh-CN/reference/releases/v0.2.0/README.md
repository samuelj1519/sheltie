# v0.2.0 release record

[English](../../../../en/reference/releases/v0.2.0/README.md) | 简体中文

状态：`released`（按用户授权范围完成）
版本：`0.2.0`
Git tag：`v0.2.0`
Release commit：`5c7183a3644dd250069ed362dc1adeeb7fb71cc6`
原发布提交：`8455aed2bcd9ac1739852be3987091a0975e7ac3`
真实宿主与续接验收闭包：`ed73febf31bf039bf1e09b5e83ef22fd56a5b5a2`
发布时间：`2026-10-02T17:03:04Z`（UTC）
发布目标：`aarch64-apple-darwin`

发布 tag 现指向[英文信息迁移](../../../history/git-message-migration.md)后 `main` 历史上的对应提交，其源码树与原发布提交完全相同。历史工作流与发布资产保留原身份和校验和。调整 tag 不重跑历史验证，也不发布新的实物。

## 范围

纳入 [C002 可靠性修复](../../../history/changes/C002-v0.2.0-reliability/README.md)：请求身份与历史重放、文件归属、Workbook 摘要与冻结、事务发布与恢复、事实视图、协调者续接、Workbook 与 skill，以及发布治理。仍为三个 crate、单一 `sheltie` 二进制和本地 SQLite 工作流引擎。

本次正式发布仅提供 macOS aarch64 包。Linux 与 x86_64 目标按用户指示排除，本版没有这些平台的安装包；此前失败与未完成验证保留在 C002 的证据中，不记为跨平台通过。用户可感知变化见 [CHANGELOG](../../../../../CHANGELOG.md)。

## 验收

- [正式 Release](https://github.com/samuelj1519/sheltie/releases/tag/v0.2.0) 从上述原发布提交构建。[发布工作流 37037558064](https://github.com/samuelj1519/sheltie/actions/runs/37037558064) 实际完成 quality、构建、host 与 announce，结论 `success`。
- 同一源码 SHA 的 quality 实际通过工程、文档、规格、skill、测试治理、依赖与 MSRV 1.85 locked 门禁。Nextest 为 700/700、0 skipped、1 slow，测试阶段 156.541 秒；原始运行与范围见 [C002 历史原件](../../../how-to/maintain-docs.md#查阅历史原件)。
- T16 在本机实际完成样例、`spec-dev` 四任务、独立审查、真实关闭并重开会话后的 Work006 续接，以及获具体批准后的 Work003 最终 retro gate。闭包固定到上述 T16 提交，详见 [C002 历史审查](../../../how-to/maintain-docs.md#查阅历史原件)。
- 正式发布后重新下载并核验实际包、manifest 与二进制，不复用 PR 构建字节。使用默认 GitHub 来源在独立管理根执行指定 `0.2.0` 的远端 update 与 rollback；回退后二进制逐字相同、Store 不变、prev 移除。更新来源为 T16 的 C002/schema 2 构建，其显示版本为 `0.1.0`；这次验证不证明 legacy schema 1 迁移。

| 实物 | SHA256 |
| --- | --- |
| [macOS aarch64 包](https://github.com/samuelj1519/sheltie/releases/download/v0.2.0/sheltie-cli-aarch64-apple-darwin.tar.xz)，1779600 字节 | `fc63c9d0c7132796cdacd050be102ed5501f637f2db0f19b279da6b4744899e4` |
| 包内 `sheltie` 二进制 | `a220309ab437a5de52202acb8b8f933370a45969e2027c55964930818e315130` |
| [dist-manifest.json](https://github.com/samuelj1519/sheltie/releases/download/v0.2.0/dist-manifest.json) | `8540728b9db73a31a74e7f762128bebed90210ecfb2ec24845786db02a02354b` |

## 兼容性

v0.2.0 使用 schema 2、`workbook-digest/v2`、新 Work 目录布局与 `cli-result/v2`。旧 schema 1 管理根以 `STORE_SCHEMA_MISMATCH` 拒绝；不自动迁移、不清空、不按新格式解释旧记录。新版本使用新的显式管理根，旧管理根和旧二进制保留用于查询历史。

## 已知限制

- M1 按授权例外完成：SK01 缺最终 Spec 批准；SK02 的 215 个精确变异 ID 缺额外执行。完整变异验证、安全验证与最终 Spec 批准仍未通过，既有 FAIL、暂停与缺失原文保留。
- Linux 原生验收与跨平台验证没有因本次发布变为 PASS；Linux、x86_64 包不在本版范围。
- T16 使用手动提供的 skill，本机 Host 窗口访问限制保留。模型 usage 为 `null`；Work/Attempt 时长含人工、会话关闭和等待，不能换算为模型用量或成本。
- OS 主体与本地 gate 不构成独立真人认证。完整限制、证据入口与任务完成范围见 [C002 历史原件](../../../how-to/maintain-docs.md#查阅历史原件) 和 [历史审查入口](../../../how-to/maintain-docs.md#查阅历史原件)。
