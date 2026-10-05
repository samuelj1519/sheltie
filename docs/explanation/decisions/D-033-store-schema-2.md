# D-033：Store schema 2 明确拒旧

状态：`accepted`
日期：2026-09-27
关联 change：[C002](../../history/changes/C002-v0.2.0-reliability/README.md)

当前适用范围：单一严格格式、不自动迁移或删除旧数据的原则继续适用。本文采用的 schema 2／cli-result/v2 先由 [D-040](D-040-result-resume-format.md) 的 schema 3／cli-result/v3 替代，再由 [D-041](D-041-attempt-number-and-replacement.md) 升为当前 schema 4／cli-result/v4。只读 SQLite 控制文件的允许变化由 [D-039](D-039-sqlite-read-control-files.md)限定，当前字段见[存储合同](../../../specs/contracts/storage.md)。

## 背景

v0.1.0 的 schema 1 存在多处持久化缺口（C002 findings O02–O05、N03）：请求指纹不含目标、历史响应由当前状态拼出、requests 表没有效果登记、建库 DDL 与 `user_version` 不在一个事务里。修复这些等于改 `requests` 表的字段集合与语义、改 WorkState 结构（累计受阻事实）、改目录摘要格式。旧 C002 曾考虑在同一 schema 里做双解释兼容（LegacyV1、默认缺字段），审查确认这会留下同一字段的两种含义，且不能解决摘要格式问题。

## 选择

`SCHEMA_VERSION` 从 1 升到 2，全部表结构、`workbook-digest/v2`、新 Work 目录布局与 `cli-result/v2` 响应属于**同一次格式切换**，由 C002-T07 一次接入。旧库（`user_version = 1`）在只读识别后被整体拒绝（`STORE_SCHEMA_MISMATCH`），拒绝之前对库文件零写入。不自动迁移、不清空、不降级解释；旧二进制同样拒绝 schema 2。v0.2.0 在新的显式管理根开工，旧管理根与旧二进制保留用于旧记录查询；出现真实的旧 Work 延续需求时另补有证据的迁移设计。

## 否决方案

- 自动迁移 schema 1 → 2：旧 `requests` 缺 `intent_hash` 与效果登记，无法 reconstruct 快照；强行迁移等于编造历史。
- 同 schema 双解释 + `serde(default)`：同一字段两种含义，重放语义不可判定。
- `reply_json` 里同时容纳新旧响应形状：调用者无法区分，违反「字段唯一合同」。

## 后果

用户升级 v0.2.0 后对旧管理根的写操作会被拒绝（读也一样拒绝，需要旧二进制查旧记录）。这是 0.x 下的显式破坏性变更，README 与更新说明必须写清。任何工具不得自动删除旧数据。

## 确认方式

测试放 schema 1 旧库作负例：断言 `STORE_SCHEMA_MISMATCH`，主库和已有 WAL 字节不变，不创建 Home、引擎锁或业务文件；SQLite 控制文件按 D-039 核对。
