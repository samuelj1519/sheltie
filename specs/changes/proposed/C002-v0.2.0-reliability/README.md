# C002：v0.2.0 可靠性修复

状态：`proposed`
目标版本：`v0.2.0`
兼容性：`minor`
基线：`31d7ddee18921b4066c7433a752c2000e5110869`
Owner：待采用时指定
权威性：未采用；不得据此修改产品代码

## 要解决的问题

MVP 与 T26 已完成。后续复审确认了请求重放、输出路径、Workbook identity、start 副作用、Work 目录可读性、sample Workbook、skill 与 `spec-dev` 的一组可靠性问题。

## 成功判据

- [findings.md](findings.md) 中的确认问题全部有合法例、单条件拒绝例和真实 caller 验证。
- v0.1.0 完整 Work 无迁移可读；新 Work 使用版本化可读目录。
- `INPUT_MISSING` 不分配序号、不创建最终 Work 目录。
- 同 request-id 重放返回提交时响应与引擎效果。
- rc 完成独立全链审查和真实宿主回归。

## 不做什么

本 change 不增加 MCP、宿主资源安装器、多人认证、并行节点或动态模板。

## 文档入口

- [findings.md](findings.md)：复审结论与证据。
- [spec.md](spec.md)：产品 delta、成功判据和不做项。
- [design.md](design.md)：候选 interface、顺序和兼容策略。
- [plan.md](plan.md)：任务、依赖和完成判据。
- [validation.md](validation.md)：当前验证状态和采用前验证要求。

## 采用条件

人确认范围、兼容策略和目标版本后，移动到 `changes/active/`，指定 Owner，更新根规格、架构、合同与 ADR，再开始实现。仅有 review 或 plan 不构成采用。
