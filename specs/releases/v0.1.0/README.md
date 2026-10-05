# v0.1.0 release record

状态：`released + accepted`
版本：`0.1.0`
Git tag：`v0.1.0`
Release commit：`9f87188f3b0ad2523abc7e975ae98e96db6f492b`
安装、发布与快速开始闭包：`4177b57738ef5f56a6699bbb5310342d32dadd68`
真实宿主与 MVP 验收闭包：`31d7ddee18921b4066c7433a752c2000e5110869`

## 范围

MVP 实现了三个 crate、单一 `sheltie` 二进制、Workbook/Flow、Work 状态机、SQLite Store、CLI、self 管理、三份样例、`spec-dev` Workbook 与协调者 skill。

设计取舍见 [MVP 设计理由](decisions.md)。完整任务、里程碑与历史操作材料从[固定 Git 快照](../../guides/documentation.md#查阅历史原件)读取。用户可感知变化见 [CHANGELOG](../../../CHANGELOG.md)。

## 验收

- MVP 的 T01–T26 与 M1–M3 已完成；完整计划见[历史原件入口](../../guides/documentation.md#查阅历史原件)。
- 历史 T25/T26 手册记录 v0.1.0 四平台发布、install、update、rollback 与真实宿主步骤；这些事实不扩大后续版本支持范围。
- 首次真实运行在 Claude Code 中运行三个 Work，包含一次 article-review back 回环；原记录保存在上述历史快照。
- [CHANGELOG](../../../CHANGELOG.md) 固定 v0.1.0 的用户可见变化；MVP 完成由项目在 `31d7dde` 接受。

## 已知限制与后续修复

T26 后复审发现的可靠性、目录可读性、skill 与 Workbook 问题已归入已采用的 [C002 change](../../changes/completed/C002-v0.2.0-reliability/README.md)。后续修复不改写 v0.1.0 的发布和 MVP 完成事实；当前支持与验收边界见[当前限制](../../limitations.md)。
