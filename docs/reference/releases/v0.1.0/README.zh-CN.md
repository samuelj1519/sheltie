# v0.1.0 release record

[English](README.md) | 简体中文

状态：`released + accepted`
版本：`0.1.0`
Git tag：`v0.1.0`
Release commit：`6af9501b7bc12e2b18ac6017b6c894dadfc43626`
原发布提交：`9f87188f3b0ad2523abc7e975ae98e96db6f492b`
安装、发布与快速开始闭包：`4bf86f7be09c27bbee98a11825eb57704e4403b9`
真实宿主与 MVP 验收闭包：`7196697ea40c9c303d059279c02f9e3d2218efd9`

本地 tag 与记录中的提交 ID 已按[提交信息迁移](../../../history/git-message-migration.zh-CN.md)更新；远端发布 tag、工作流与资产仍绑定上面的原发布提交，未把历史验证改记为新身份上的重跑。

## 范围

MVP 实现了三个 crate、单一 `sheltie` 二进制、Workbook/Flow、Work 状态机、SQLite Store、CLI、self 管理、三份样例、`spec-dev` Workbook 与协调者 skill。

设计取舍见 [MVP 设计理由](../../../explanation/decisions/mvp.zh-CN.md)。完整任务、里程碑与历史操作材料从[固定 Git 快照](../../../how-to/maintain-docs.zh-CN.md#查阅历史原件)读取。用户可感知变化见 [CHANGELOG](../../../../CHANGELOG.zh-CN.md)。

## 验收

- MVP 的 T01–T26 与 M1–M3 已完成；完整计划见[历史原件入口](../../../how-to/maintain-docs.zh-CN.md#查阅历史原件)。
- 历史 T25/T26 手册记录 v0.1.0 四平台发布、install、update、rollback 与真实宿主步骤；这些事实不扩大后续版本支持范围。
- 首次真实运行在 Claude Code 中运行三个 Work，包含一次 article-review back 回环；原记录保存在上述历史快照。
- [CHANGELOG](../../../../CHANGELOG.zh-CN.md) 固定 v0.1.0 的用户可见变化；MVP 完成由项目在 `7196697` 接受。

## 已知限制与后续修复

T26 后复审发现的可靠性、目录可读性、skill 与 Workbook 问题已归入已采用的 [C002 change](../../../history/changes/C002-v0.2.0-reliability/README.zh-CN.md)。后续修复不改写 v0.1.0 的发布和 MVP 完成事实；当前支持与验收边界见[当前限制](../../limitations.zh-CN.md)。
