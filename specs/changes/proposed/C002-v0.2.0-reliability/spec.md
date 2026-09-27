# C002 产品 delta

状态：方案完成，产品修复 `not_run`
基准：`31d7ddee18921b4066c7433a752c2000e5110869`
来源：[T26 同步提交与 Work 目录复审](findings.md)

本文定义 C002 要改变的产品行为和验收边界。设计见 [design.md](design.md)，任务见 [plan.md](plan.md)。

本文是修复提案，不是进度权威。采用方案时，先执行 C002-T01，更新上游权威和本 package 的任务白名单，再改产品代码。

## 修复目标

`v0.2.0` 必须达到以下结果：

1. `work start` 的所有确定性校验都发生在分配序号和文件写入之前。`INPUT_MISSING`、多余输入、非法名字、缺 Workbook 或缺 Flow 均无序号、目录、数据库和请求副作用。
2. 已通过 preflight 的 start 使用 staging 与请求恢复。最终 `works/<work-id>/` 只在状态提交成功后出现。
3. Work 目录的 Occurrence 与 Attempt 两个维度保留，但目录名可以自解释；引擎文件与 worker 输出分开。
4. Workbook 安装副本与 Work 冻结副本的根目录均只读；已装版本按数据库登记摘要加载。
5. `workbook show` 公开每张 Flow 的起始输入；skill 在 start 前补齐输入，不用失败命令探测。
6. 同一个 `request_id` 绑定一个用户意图，重放返回提交时的响应与引擎效果，不受当前状态或文件变化影响。
7. 上一轮 13 项 finding 与本轮新增问题全部有真实 caller 测试。
8. MVP 与 T26 保持完成；`v0.2.0` 发布前另做宿主回归，覆盖 human 节点与 token 观测。

本轮不增加 MCP、宿主资源安装器、多人认证、并行节点或动态模板。
