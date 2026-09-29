# C002-T30：重规划的可追溯交接

基线 `a4f1968`。macOS arm64；Linux 按用户指示为 `not_run`。最终源码闭包见 [candidate-input.txt](candidate-input.txt)。

## V29/V30

plan-review 必须提交被审 plan/tasks 的原字节镜像，分别限 65536 字节；plan 从不同节点绑定镜像及最新 verify/change/fix。实际安装发现 `reviewed_plan` 不符合既有 OutputName 合同，改为 `reviewed-plan`，未放宽 core。首次方案记录整体原始基线；重规划递归核引用链、累计前缀、每条历史审批、Git Task/白名单与原始门禁证据，不重设基线、不从聊天补事实。

真实 CLI/Git 回归涵盖首次空输入、required 镜像遗漏、冷读恢复、基线重设/字段遗漏/漏行/缺原始证据拒绝、继续后继承最新报告、较旧整体修复的 Git 祖先判定、深层漏行/失败报告/拒绝审批/循环停止。失败 verify 的必需字段同样核验；合法失败报告仅删本轮变更字段即停止。人工更正通过自己的升级任务书取得 spec/approval，显式记录原审批与两个摘要；后续报告引用实际更正及原审批，普通继续不能代替批准。

每次 fixture 门禁实际执行 `sh gate.sh`，保存 argv/cwd/退出状态、stdout/stderr 与摘要。task stdout 的 SPEC_DEV_RAW_GATE JSON 保留这些原始字节及 Git 范围。gate.sh 是离线 fixture 命令；模拟 worker 只验证任务书可达性、记录来源及 Git 范围，不代表 T16 的真实 agent 内容质量。

## 验收中暴露的构建竞态

[nested-build-race.stdout.txt](nested-build-race.stdout.txt) 保留全仓失败：runtime 测试内 cargo build 会重链共享 CLI，另一个 CLI 测试 spawn 报 ENOENT。提前完成 T31 原定迁移步骤：7 条子进程崩溃用例迁 CLI/crash，2 条直接 API 用例留 runtime/crash；OS 参数、身份子进程用例迁 CLI/os_process。原函数体、Owner、独立 oracle 保留；使用 CARGO_BIN_EXE_sheltie，全 crates 不再有测试体 Cargo 构建入口。依赖同步点的 marker API 例仅 failpoint 启用，普通状态卡 API 例仍默认启用。

## 门禁与独立审查

最终 Rust 四门禁、MSRV 1.85 locked、离线 deny、docs/specs/core-vocab/tests/skill 均为本目录原始输出与 gate-results.json。最终全仓 Nextest run `2aa8b4a7-65f2-4953-8222-4894e81ff4b2`：625 passed、0 skipped、1 slow、无 LEAK。Task run 见 task.stdout.txt；按 Task 筛选之外的测试属于 task-filtered，不是能力跳过。

临时目录使用官方 Nextest 0.9.146，压缩包 SHA256 为 `39785160b3c2f6ed9a765049cf4fa79f3b39aa02eb7598a5a0e2a1a0b9ffb9a8`；未替换全局 0.9.140。上游 0.9.145 修复 macOS 并发 spawn 继承兄弟捕获管道导致误报 LEAK，见 [官方 changelog](https://docs.rs/crate/cargo-nextest/latest/source/CHANGELOG.md)。历史 LEAK 原文保留；不能仅以这次无 LEAK 推断所有子进程回收路径都已证明，T31继续核验。

独立 Spec/Standards 均通过，含后续完整迁移复核；Reviewer只读，未自行运行测试。见 spec-review.md 与 standards-review.md。失败日志的 raw.gz 保留逐字节输出，文本仅去尾随空白；早期缺镜像、错误输出名、旧审批入口失败均未覆盖。

T30关闭 R16/V29/V30；T31全窗口与完整突变、独立 M1、T16真实宿主及 T17发布各按原门槛继续。
