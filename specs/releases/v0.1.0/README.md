# v0.1.0 release record

状态：`released + accepted`
版本：`0.1.0`
Git tag：`v0.1.0`
Release commit：`9f87188f3b0ad2523abc7e975ae98e96db6f492b`
安装、发布与快速开始闭包：`4177b57738ef5f56a6699bbb5310342d32dadd68`
真实宿主与 MVP 验收闭包：`31d7ddee18921b4066c7433a752c2000e5110869`

## 范围

MVP 实现了三个 crate、单一 `sheltie` 二进制、Workbook/Flow、Work 状态机、SQLite Store、CLI、self 管理、三份样例、`spec-dev` Workbook 与协调者 skill。

完整任务与里程碑历史见 [MVP plan](plan.md)。设计与复核历史见 [legacy decision log](decisions.md)。T25/T26 操作方法见 [archived runbook](runbook.md)。用户可感知变化见 [CHANGELOG](../../../CHANGELOG.md)。

## 验收

- [MVP plan](plan.md) 中 T01–T26 与 M1–M3 状态为 `done`。
- [T25/T26 archived runbook](runbook.md) 记录 v0.1.0 四平台发布、install、update、rollback 与真实宿主步骤。
- [首次真实运行](decisions.md#首次真实运行) 在 Claude Code 中运行 three Works，包含一次 article-review back 回环。
- [CHANGELOG](../../../CHANGELOG.md) 固定 v0.1.0 的用户可见变化；MVP 完成由项目在 `31d7dde` 接受。

## 已知限制与后续修复

T26 后复审发现的可靠性、目录可读性、skill 与 Workbook 问题已归入已采用的 [C002 change](../../changes/completed/C002-v0.2.0-reliability/README.md)。修复进度只看 C002 的 [plan.md](../../changes/completed/C002-v0.2.0-reliability/plan.md)；这些 finding 不改写 v0.1.0 的发布和 MVP 完成事实。
