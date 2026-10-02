# C004 设计

状态：`proposed`；拟新增符号必须在实施任务中创建，不能当作已经存在的接口。

## 1. 职责与入口

保持三个 crate 和 `cli → runtime → core`。core 从已校验状态和冻结图选择引用、派生投影；runtime 严格装入 Store/Workbook、取得文件效果事实；CLI 解析和渲染。无额外 crate、执行器、trait 平台或第二状态来源。

| 真实入口 | 本方案的工作 |
| --- | --- |
| `core/flow/{def,parse,compile,graph}.rs` / InputDecl、OutputDecl、Graph | result 声明和终点/required/key 校验 |
| `core/work/state.rs` / WorkState、Attempt | 读取输入输出归属；不得缓存第二份最终选择 |
| `core/work/render.rs` / StatusView、status_view、render_status_card | 同源紧凑接续指针 |
| 拟 `core/work/result.rs` | 纯最终选择与 ResultView，不做 I/O |
| `runtime/service.rs` / WorkService::status_with_publication、拟 WorkService::result | service 内取得可信上下文，再传明确参数给投影 helper，不跨模块访问私有 store/home/Loaded |
| `runtime/load.rs` / validate_work_paths；WorkState::validate_gate_facts | 完整身份、路径和 gate 事实校验 |
| `runtime/store/read.rs` / requests 的 published/effects 读取 | 同一 SQLite 读快照核本 Work 未完成效果，不因其他 Work 阻断结果 |
| 拟 `runtime/result.rs` 的 crate 内 helper；`cli/commands/work.rs`、`commands/mod.rs`、`cli.rs`、`output.rs` | 结果投影、只读分派/request-id 拒绝和文本/JSON |

表中 core/runtime/cli 均指对应 `crates/sheltie-*/src/`，实际完整路径以 tasks.toml 为准。方法资产位于拟 `examples/code-change/` 与 `skills/sheltie/`。

## 2. 结果声明与选择算法

InputDecl/OutputDecl 增加一个私有 bool，构造时校验，默认 false。parser 严格拒绝未知字段；compiler 校验标记只在无出边节点出现、选中项 required、所选逻辑名跨 input/output 唯一。最多选择该节点声明允许的项，不增加另一套 include/exclude 规则。

拟 Store::read_work_bundle 在 read.rs 的单一只读事务中取得 WorkRow、本 Work 关联的完整请求/效果/published、audit 和 Start 发布定位。按 requests 与 audit 两边关联取并核 work/request/revision 归属；缺请求、缺审计、索引冲突或 published=1 但 effects 非法均报 STORE_CORRUPT，不按未完成标记预筛或只信 requests.work_id。service 消费该 bundle，不能再调各自连接的 load_work/inspect_request/audit_rows 拼字段；需要的可信加载 helper 接受已取得记录，不能回查另一快照。不增加 reader trait 或外部状态源。

service 在该上下文核冻结 Workbook，再把明确的 state、graph、revision 与 effects_pending 交 core 的 result_view；revision 不是 WorkState 可自推字段。status 的共有投影只接 state/graph，实时 revision/效果事实在 runtime 的具体 StatusReadView 中与该投影组合，再渲染文本/JSON；不重新计算 next 或增加可推进状态。runtime/result.rs 的 helper 只接实际参数，不访问 service 私有字段。core 校验成功终点和门槛事实，并从该具体终点 Attempt.inputs/outputs 取引用。source.attempt 表示终点绑定/封存的 Attempt，source.kind/name 明确所选槽；ArtifactRef.path 是实际原件路径。输入可以来自 start/resource/engine.stats 或其他 Attempt，不能把绑定者说成生产者。

不能用全局最新成功 Attempt、当前仓库 HEAD 或目录扫描求结果。状态 succeeded 但无法找到对应终点成功 Attempt、必需选中引用缺失或归属不符，返回 STORE_CORRUPT；不是空选择。有效图没有选择则正常返回空数组。

## 3. 只读结果协议

拟命令：`sheltie [--json] work result <work>`。前缀解析按公开只读合同；不接受 request-id。data 格式如下：

| 字段 | 精确含义 |
| --- | --- |
| `format` | 固定 `work-result/v1` |
| `work_id`、`revision` | 同一次 SQLite 读快照的完整身份和整数 revision |
| `workbook` | `{id, version, digest}`，本 Work 冻结方法的身份 |
| `flow`、`status` | 冻结 Flow ID 和 WorkStatus 的合同表示 |
| `effects_pending` | 本 Work 存在已登记而未完成的文件效果；来源为严格校验的 Store 请求记录 |
| `final` | succeeded、有效成功终点/gate、effects_pending=false；其他情形 false |
| `artifacts` | final=false 时空；否则按 key 排序的明确选择 |

Artifact 项为 `{key, path, sha256, bytes, source:{attempt, kind, name}}`。kind 只为 input/output；attempt 是选择发生的具体终点 Attempt，name 是该槽逻辑名。path/sha256/bytes 完整使用被冻结引用，不重算出另一份结果身份。协议文案称“由终点绑定/封存”，不称所有输入都由终点生产。

不含 candidate、check_records、snapshot_digest 或自然语言通过位。引用没有覆盖整个代码仓库的含义。文本与 JSON 从一个 ResultView 渲染；CLI 不能再读 Store 或文件拼接字段。普通封装的 next 仍来自同一状态；只读结果不增加推进操作。

本接口列举已封存引用，不声称查询时已再次验证所有源字节。C006 需要可信字节时，在自己的采用范围中扩展此只读入口、核 revision/归属/同句柄字节并完成真实消费者测试。

## 4. 紧凑接续协议

StatusView 增加纯状态的 resume；实时 `work status` 的完整读取视图增加 revision 和 effects_pending，不建立独立 handoff 命令或第二套业务判定。resume 无 Attempt 时为 null，否则为 `{attempt, brief_path, inputs, draft_outputs}`。inputs 按当前 Attempt.inputs 列 ArtifactRef 或 null；draft_outputs 按当前 Node 输出声明列绝对路径，仅 running 时提供，否则为空。最近摘要/失败原因仍由 last_attempt 承载，next 由 legal_next 提供。

默认不列其他 Attempt 的输入、草稿、报告或完整历史。done/visits 维持摘要，不重复输出尝试正文。brief_path 与 draft_outputs 从 WorkLayout 和当前身份派生；未完成 prepare/write/seal 等效果统一提示，不能把路径存在等同发布完成。

磁盘 status-card 是已提交状态的投影，承载状态与 resume，不保存实时 effects_pending。效果链在 refresh_status_card 后才 mark_published，不能在写卡时假装本请求已完成，或把当时的 pending=true 当永久就绪事实。实时 revision/效果提示以 Store 查询为准；公开 status 的文本与 JSON 使用同一个完整读取视图，磁盘卡与查询的共有状态/接续字段保持同源。T01 将这个字段范围同步到 protocol/storage，不能承诺磁盘卡含所有实时读取元数据。

查询不自动恢复或刷卡。effects_pending 从本 Work 相关记录投影；与其他 Work 无关。SQLite 事务读一致性、可信图加载与文件效果核验必须一起通过，错误不降级成可继续提示。只读控制文件例外按 D-039。

## 5. Rust 实现与验证

字段私有、受校验 newtype、闭集 enum、严格 serde；函数默认私有，新增 pub 只为实际跨 crate caller。ResultView 与 StatusView 采用具体类型，不传自由 JSON 在层间做决定。依赖从参数进入纯函数，不为测试增加 trait；临时 Home、SQLite 和真实 CLI 验证持久消费者。

总体接口、关键读取原语和本阶段真实测试由复杂模型先准备，M1 核可实施性；简单模型只填已确定主体和接线。T01 对耦合接口完整处理 parser/compiler/state loader/runtime/CLI/fixtures/说明，不能把未定架构留给初级实现者；不留接到用户入口的占位、禁用测试、假成功或双实现。格式版本在 T01 从实际采用起点固定，改字段即同步完整读写合同；不变的摘要算法无需因新增命令重写。

T01 的 stage-1.md 在交接时必须写出真实符号、实际测试名、精确命令、允许主体和骨架完整提交；实现者不能根据拟新增路径自行选接口。未完成行为不接正常命令入口。M2 核最终公开链与用户结果，不将 M1 原语/骨架通过当功能完成。

正反期望使用手写图/状态、明确原件字节和独立 SQLite 观察。重点覆盖终点选择、同名冲突、绑定旧版本、gate、取消、COMMIT 后效果未完成、源文件被改和文本/JSON 一致性。价值实验与机制验证分别报告。
