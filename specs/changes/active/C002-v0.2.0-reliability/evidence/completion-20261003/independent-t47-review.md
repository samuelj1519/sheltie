# C002-T47 独立审查

结论：**需修改**。5 条新增测试的合同期望和 10 个指定突变的辨别力通过；子进程夹具失败回收尚不满足任务要求。此结论只针对 T47，不关闭 T43 的其余 215-ID 义务、完整工程门禁、最终 Spec、平台或真实产品价值验收。

Reviewer：`Codex /root/oracle_review`。未参与当前生产实现、新增测试或 G10 反例方向编写；只读复核仓库，仅在本报告路径写审查记录。未在被测 target 或 mutation scratch 构建、修改源码。

审查时 HEAD：`06e5a9108be9afdfb9b5b68fc112ad08a600af8d`。实际测试源码：

| 文件 | SHA256 |
| --- | --- |
| `crates/sheltie-runtime/src/selfmgmt.rs` | `0113f93ab4331034b8fa86f0ea1371f2c75328864b68c5856ec2000fd25b367e` |
| `crates/sheltie-runtime/src/store/tests.rs` | `f490e8b9b26a1da059d71d99e905ae4433df0d8d3e5ae6b155d1a77804f28d99` |

## 需修改项

F-T47-01：`selfmgmt.rs:1007–1010` 在子进程释放和 worker 回收完成前仍有 `unwrap()`。`write(release).unwrap()` 失败时未建立 release，测试线程立即 panic，shell 继续等文件；随后 TempDir 删除同步目录，shell 的等待条件持续为真。`recv_timeout(...).unwrap()` 失败时也跳过 `worker.join()`。这两个路径不能证明自有 child 已回收。依据 active plan T47：“超时3秒保存失败并停止，任何panic前先释放自有子进程。”

最小修正是在测试专用夹具中先保存全部错误、执行释放/回收，再断言；为 release 写失败和后续超时提供可确认的独立终止/回收路径，且不得添加生产观察配置。保留当前明确关闭 stdout/stderr、等待 release 的场景。修正后跑实际非零基线与 4 个对应 child 突变；旧原文保留。

## 已通过部分

- 生产范围：`selfmgmt.rs` 的 `#[cfg(test)] mod tests` 前缀与 HEAD 逐字一致，比 AST 相同更强；`store/tests.rs` 只追加测试 helper/测试，且由 store 的 `#[cfg(test)] mod tests` 引入。无生产行为扩展，INV-1–INV-7 未受影响。
- WAL：storage §1、§1.1 要求通过结构校验后读写打开设置 WAL。测试从手写有效 schema、DELETE 模式开始，调用真实 `WriteSession::open_existing`，随后用独立 READ_ONLY SQLite 连接读 `PRAGMA journal_mode`，没有调用 Store getter 补做 connect。五张持久表完整行前后相等。真实调用链为 WriteSession → open_existing_locked → open_existing_inner → Store.validate → connect → PRAGMA；validate 不是无副作用的重复 schema 检查，删除它会漏掉初次 WAL 设置。
- revision：合法 start 得 1、合法更新得 2，再只改已有 `works.revision=0`。新 request 未命中重放，expected=2，准确期望为在 CAS 冲突前报告 STORE_CORRUPT；五张表全行拒绝后相等。期望源于原 SK02 的 O09 明文义务和当前计划，没有镜像 `.filter()` 表达式。
- INSERT：合法 Workbook 插入、真实重复 ConstraintViolation→WorkbookExists 对照均保留。新增 BEFORE INSERT trigger 引用不存在的 SQLite 函数，使真实 `tx.execute(INSERT INTO workbooks)` 在 prepare 阶段返回非 ConstraintViolation 的 SQLite 错误；不是伪造 Error。更早 works 更新必须随事务回滚，五张表完整行相等。新名字 other 不触发合法重复条件；错误 detail 保留 `missing_fixture_function`。不声称触发器正文已执行。
- 资产名：手写合法普通名、带版本的发布名、Unicode 名逐字返回；空、点、双点、正反分隔符及 NUL 拒绝为 UpdateUnavailable。对应采用的单段安全路径限制与 T47，不调用被测 helper 算期望。
- child 正常/突变路径：分别手写 stdout/stderr 恰好 4 bytes，shell 显式关闭两个 pipe 后创建 reached，随后仍等待 release；250ms 内提前结束视为失败，释放后检查 success 与完整字节。该场景区别于“已退出 child 恰好限额”，已有 exact/+1 测试继续保留。当前实际 baseline 和四个突变的正常执行分支都先写 release、取得返回、join 后断言；失败清理普遍保证仍见 F-T47-01。
- helper 只用独立 SQLite 查询快照五张业务表，不生成业务期望、不调用被测 Store 读取路径。它不证明文件字节不变；WAL 初始化本来会改变 SQLite 控制/模式，报告不把行相等外推为物理字节无变化。

## 原证据与动态辨别力

逐字读取归档 `e54dcd41d8f1e186007b62b47583063cb19a4b66` 中 `SK02-missing-execution-map.json`：原 215 项、G10/G11/G12 分别 6/3/4 项。只核 T47 对应 10 个存活体；另外 schema 原有捕获、其余组与历史 Missed 不由本报告重记。

初次 baseline `2de50515-66d6-49af-a385-5e60326988eb` 的 WAL `is_empty` 期望不适用于含合法手写行的 fixture，属于测试期望错误，原文保留，不是产品 red。修正为开会话前后全行相等后 baseline `f9e556d8-ccd4-48c8-b101-0b6186d30a6b` 为 5/5，52 项因 filter 未选，不是完整测试零 skip。

`mutation-oracle-check/result.json` 最终 exit 0，111.096s，baseline Success 1、CaughtMutant 10，源码前后相同；outcomes 完成时点 `2026-10-03T03:40:41.07009Z`。独立读取原 log 的真实失败：

| 变异 | 新测试的实际失败 |
| --- | --- |
| Store.validate→Ok | WAL 为 delete，期望 wal |
| revision >→>= | 返回 RevisionConflict(expected=2, actual=0)，期望 StoreCorrupt |
| Constraint guard→true | 返回 Workbook other@1.0.0 已装，期望 StoreCorrupt |
| 3 个资产名 OR→AND | 单段拒绝用例失败 |
| stdout/stderr 的 >→<、>→>=，共 4 个 | child 在 release 前被终止，或返回不成功 |

复现入口：`/private/tmp/sheltie-completion-20261003/check-new-oracles.py`；准确 argv、选择过滤与源码 SHA 分别在 `mutation-oracle-check/result.json`、`selection.json`，完整原文在 `mutants.out/log/`。本 Reviewer 审计这些已完成原文，没有重启同一测试或构建任务。

独立审查覆盖代码/合同/真实 SQLite 事务、child 同步模型、归档来源及指定突变原文；完整 runtime/CLI、四条稳定工程门禁、docs/specs、提交范围与 M2 验收仍由任务执行者按新冻结输入完成，不预判 PASS。
