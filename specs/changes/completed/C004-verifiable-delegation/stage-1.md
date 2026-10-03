# C004 阶段交接

采用完整范围和同等能力负责人见 [adoption](adoption.md)。本阶段完成真实纯类型、读取原语和测试；T02 接通公开命令。当前冻结测试基准只见 [plan](plan.md) 末尾的“最新冻结测试修订”；M1 原始候选仅作历史准备证据，不能替代已独立批准的后续测试修订。

## 实际接口

| 文件与符号 | 已确定输入与输出 | T02 责任 |
| --- | --- | --- |
| `crates/sheltie-core/src/flow/{def,parse,compile}.rs` | input/output `result` 默认 false；规则 10 固定终点/required/key | 不再修改字段、编译政策或 oracle |
| `crates/sheltie-core/src/work/result.rs` / `result_view` | `state, graph, revision, effects_pending` → `Result<ResultView, String>` | 采用已完成纯实现，不另选引用 |
| `crates/sheltie-core/src/work/render.rs` / `status_view` | 共有状态与当前 `resume`；磁盘卡不含实时效果 | 使用同源 card，不从文件重建状态 |
| `crates/sheltie-runtime/src/store/read.rs` / `Store::read_work_bundle` | 单一 SQLite 事务的完整 Work/request/audit 闭包 | 不新增分离查询 |
| `crates/sheltie-runtime/src/service.rs` / `WorkService::status_read` | `(String, StatusReadView)`；flatten card 加整数 revision 和始终 bool effects_pending/pending_publish | 切换 CLI status 到此具体读取视图 |
| `crates/sheltie-runtime/src/service.rs` / `WorkService::result` | `(ResultView, Vec<NextOp>)`；错误准确转 STORE_CORRUPT | 接只读 `work result` |
| `crates/sheltie-cli/src/cli.rs` / `WorkCmd` | 新枚举项固定为 `Result { work: String }`，只读、拒绝 request-id | T02 创建并接到实际 service；T01 不公开未完成命令 |
| `crates/sheltie-cli/src/commands/{work,mod}.rs` | service 读取 → 同一 view 的文本/JSON 与 next | 不回查 Store 或文件拼数据 |

ResultView 完整字段和最终来源见 [protocol](../../../contracts/protocol.md#work-result)。草稿 map 为 `{name: absolute_path}`，非 running 为 `{}`；inputs 为当前 Attempt 的完整冻结 ArtifactRef 或 null。core String 错误由 runtime 准确映射，不新增空结果 fallback。

## 独立用例与命令

T01/T02/T03 的真实测试名单只在 [plan](plan.md) 对应任务卡维护。T01 为 28 个新增纯原语/持久读取测试；T02 为 11 个公开 CLI 与 runtime 结果行为测试，T03 为最小方法场景。已完成原语全绿；T02 接线前应在真实 CLI 观察 result 不可调用、实时 status 缺 revision 的预定失败。不是编译失败或占位 panic。

在仓库根执行，使用同一隔离构建目录：

```bash
CARGO_TARGET_DIR=/private/tmp/sheltie-c004-target RUSTC_WRAPPER= scripts/task.sh C004-T01
CARGO_TARGET_DIR=/private/tmp/sheltie-c004-target RUSTC_WRAPPER= scripts/task.sh C004-T02
CARGO_TARGET_DIR=/private/tmp/sheltie-c004-target RUSTC_WRAPPER= scripts/task.sh C004-T03
```

本机 nextest 为 0.9.140，配置要求 0.9.145；原命令会以 92 拒绝且不执行测试。按用户授权保留此环境缺项；补充验证使用安装版本提供的 `--override-version-check` 和 task.sh 相同归属过滤，不称 0.9.145 门禁已运行：

```bash
task_filter="$(scripts/test-owners.sh | awk '$1 == "C004-T02" { printf "%stest(/(^|::)%s$/)", (n++ ? " | " : ""), $2 }')"
CARGO_TARGET_DIR=/private/tmp/sheltie-c004-target RUSTC_WRAPPER= cargo nextest run --override-version-check --all-features --no-tests=fail --run-ignored all -E "$task_filter"
```

T01/T03 只把该过滤中的任务号改成相应 ID。非零测试数和完整输出写 [validation](validation.md)；退出 0 才是该实际执行范围通过。源码/fixture/config/工具或环境改变后重新核输入闭包，不只比较 SHA。

## 允许改动与停止入口

T02 只改 tasks 白名单中的生产接线，并移除本任务 ignore；签名、字段、断言、快照和准备的接口不得静默变更。需要修订时由同等能力负责人新增明确修复任务，独立 Reviewer 复核后重新固定测试基准。范围命令传 plan 中最新独立测试修订完整 SHA：`scripts/check-task.sh C004-T02 2914e8564047809659c4610d151f9ea6ceef7476 --staged`；提交后同基准再核。

方法已在 `examples/code-change/` 准备：implement/review 各最多到达 3 次、业务重试 1 次，deliver 到达 1 次、默认无 gate。task 带目标与验收，project 带位置/检查前提；终点选 change/review 的冻结输入和 delivery 的封存输出。T03 完成首次使用说明、协调者指引和实际场景；方法政策如需改变先修订 oracle。

独立 Reviewer 为 `/root/independent_review`；其不参与合同、测试或代码编写。发现请求归属、格式、文件事实或权限不足时保存实际错误交负责人；环境缺项按采用授权逐项 not_run，不构造成功。

## 真实增量试用规程

T04 使用 [C007 指标](../../active/C007-pre-run-workbook-generation/validation.md)，在正式运行前确认新的未解决真实任务、实际使用者、独立质量标准、两组等价初始输入和顺序，分别计方法准备、配置、首次/复用、解释、重做、核对与未完成。每 run 人工活动最大 60 分钟；质量下降、预算到顶、输入漂移或外部前提缺失就停止对应 run，保留原件和全部已发生成本。未知 usage 为 null。

至少实际关闭并重开原宿主会话一次；普通继续消息或 agent 夹具不替代该动作。缺真实使用者/新任务/独立质量或会话控制时记录具体 not_run，交后续真人按此入口补验。机制场景不报告成本净收益，完成状态不代表用户接受。
