# C002 已实现任务与 T31 代码审查

结论：**需修改**。当前候选存在 3 个正确性问题，以及 4 个合同偏离或精简项。不能据此关闭 T31 或 M1。审查没有修改实现、删除 T31、改任务状态或提交代码。

## 范围与证据

- Reviewer：Codex 主 agent，以及未参与实现的 Standards、Spec 两个独立审查 agent。
- 基线：C002 声明的 `a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。
- 候选：`b926789093c51c62ee742bfbe1cb0c3c43044035` 加本轮开始时全部未提交改动；包括 `reliability_crash.rs`、`persisted_contracts.rs`、`owned_tempdir.rs`，不是只审 HEAD。
- 依据：根 spec、constitution、architecture、contracts、engineering，以及 C002 的 spec/design/plan、repair-design/repair-plan/repair-validation。当前工作区合同包含 T31 的累计受阻事实勘误。
- 平台：`aarch64-apple-darwin`，Rust/Cargo `1.98.1`，全 features。Linux 仍为 `not_run`；本轮未执行 MSRV、完整 mutation、真实宿主或发布验证。

原始证据保存在 [evidence/implementation-review-2026-09-30](evidence/implementation-review-2026-09-30/README.md)。`input-closure.json` 记录 273 个源文件、fixture、合同及配置的摘要，并单列治理测试会复制的 1681 个文件（含 evidence）；该 manifest 的 SHA256 为 `5e2e49f938d5d88d2a3ddada17c8387677a616abb79327cfd8dab3e27b3f0b23`。测试输入与后续新增的本报告分开记录，本轮不复用历史运行。结束时 273 个源码及合同输入均未变化，但一份 T31 mutation stdout 在测试期间被其他运行追加，详见 `input-audit.json`；因此不声称治理夹具拥有完全固定的全量输入闭包。

| 审查链 | 对应任务 | 检查重点 |
| --- | --- | --- |
| core 定义、路径、状态机与视图 | T02、T03、T06、T09、T14、T31 增量 | 已校验构造、持久装入、门槛记录、计数、布局、摘要和 next |
| Work 写入、查询与恢复 | T04、T05、T07、T19–T22、T24–T26、T28、T29 | 预检、请求身份、锁内 Store、效果闭包、文件同步、历史响应和同次装入 |
| Workbook 与 self 生命周期 | T08、T15、T21、T23、T26–T28 | staging、版本身份、同对象发布/删除、更新/rollback、purge 锁生命周期 |
| CLI、Workbook、skill 与测试调用链 | T10–T13、T15、T30、T31 | 响应映射、重规划输入、Cargo binary、实际故障窗口与现有回归覆盖 |

该表说明审查范围，不是逐任务重新验收。T01/T18 的文档用于判定合同；T16/T17 未实施范围不作为缺失实现上报。

## Standards

本轴独立判断工程要求与简洁性。代码味道是精简建议，不自动算行为错误。

### CR-S01 P1 同步失败后的恢复会提前标记完成

位置：`crates/sheltie-runtime/src/effects.rs:655`，`fsx.rs:403`、`:939`，`recovery.rs:313`。涉及 T19、T25、T26 和当前 T31 的历史文件发布修改。

`write_atomic_unlocked` 完成 rename 后再同步父目录；父目录 fsync 失败时，新文件已经落位，但请求仍未发布。重试进入 WriteFile 的已有文件分支，只核摘要，不补文件或父目录同步，之后可能标记 `published=1`。`ensure_dir_unlocked` 也存在 mkdir 成功、父目录同步失败后，重试仅接受已有目录的问题。尤其是 `outputs/a/b/` 之类嵌套目录，其父目录不会因后续 brief 或状态卡写入而自动获得同步。

依据：[storage §3.2](../../../contracts/storage.md)：“所有必需文件/目录 sync 成功后才置 published=1”。这是静态完整调用链确认的错误；本轮没有动态注入这两个 fsync 失败窗口，不把它写成运行复现。

修复方向：区分“已经存在”与“持久化义务已经完成”。未发布请求的恢复应核同一对象并补必要文件、父目录及目录链同步；已完成请求的普通重放继续遵守不重做效果的合同。增加真实 caller 反例：首次 rename/mkdir 后同步失败，第二次仍同步失败时必须继续保持 pending；同步成功后才 mark。

### CR-S02 重复的 NextOp 协议映射可以直接删除

位置：`crates/sheltie-cli/src/output.rs:103`、`crates/sheltie-runtime/src/recovery.rs:620`；已有共同实现为 `crates/sheltie-core/src/work/render.rs:432::next_item_json`。

三处逐变体构造同一种 JSON，协议变动需要同步修改三处。属于 Duplicated Code / Shotgun Surgery；依据 engineering §1.4“一个事实只在一处定义”。当前没有确认输出差异。

直接复用已有 helper，调用方传 `WorkId`，删除重复 match；不需要新增抽象、trait 或另一份协议类型。

### CR-S03 快照结构校验重复且混合不同职责

位置：`crates/sheltie-runtime/src/service.rs:740`、`:1227`，`recovery.rs:496`。

Start 的 data 字段、Reply 与归属关系在三处展开；后两处还重复其他 Reply 的匹配。raw `Value` 使字段存在性、形状与 Work/Graph 归属检查交织，读者难以看清每处多核了什么。属于 Repeated Switches / Primitive Obsession，当前没有据此断言运行错误。

将严格的 snapshot data 解码与共同结构校验集中一处；依赖可信 Work/Graph 的归属检查保留在明确的调用点。保持已有持久 JSON 和历史响应字节，不增加第二套事实来源，不通过删除检查来缩短代码。

### CR-S04 普通装入和重放反复扫描整个历史

位置：`crates/sheltie-runtime/src/service.rs:641`、`:1178`、`:1190`、`:1191`，`pending.rs:58`。

每次 Work 装入读取并解码全库 `effects_json`，其中含完整历史 brief/stats content；同一次重放又连续调用数次完整装入。单个 Work 的操作成本随整个管理根历史增长，且大量工作发生在写锁内。这是可以从调用链确认的重复工作；本轮未做规模基准，不给出性能倍数或延迟承诺。

先在同一锁内复用已校验的请求、效果和装入结果。清理继续执行合同要求的全库引用检查；普通定位用目标请求查询，并保留唯一归属验证。不要为了优化增加持久索引或另一套状态。

本轴 4 项：1 个硬错误、3 个精简建议。最严重项为 CR-S01 的同步完成证明缺口。

## Spec

本轴独立判断 C002 当前方案与合同，不以 Standards 结论替代规格结论。

### CR-P01 P1 首次建库被杀后同请求不能恢复

位置：`crates/sheltie-runtime/src/session.rs:42`；提前拒绝入口为该文件 `:33` 与 `store/mod.rs` 的 schema 检查。涉及 T24 及其并发初始化 follow-up。

新管理根先创建并持久化零字节最终 `store.db`，再初始化 schema。进程在两步之间被杀，清理分支不会执行。重试看到已有 schema 0 文件，在进入初始化分支前即返回 `STORE_SCHEMA_MISMATCH`。

本轮用真实全 features CLI 在 `write_session_after_store_create` 同步点 SIGKILL，随后以相同 request-id 重试 add：退出码 1、错误 `STORE_SCHEMA_MISMATCH`、文件仍为 0 字节。原始输出见 `initialization-evidence.json` 和 `probes-evidence.json`。

依据：[storage §3.1](../../../contracts/storage.md)：“进程可能在任何时刻被杀”；COMMIT 前允许“同 request-id 重试……从预检重走”。T24 已修复的并发等待只覆盖初始化者继续运行的情况，没有闭合初始化者死亡的窗口。

修复方向：在可证明归属的暂存库完成初始化后再发布最终 Store，或提供可验证的初始化恢复归属，并闭合 SQLite 控制文件的生命周期。不能把任意 schema 0 文件当成自己留下的文件，更不能放宽旧库拒绝或自动清空规则。增加 add/install 首次初始化 SIGKILL 与同请求重试的真实回归。

### CR-P02 P1 缺失 gate 批准记录的持久状态仍能推进

位置：`crates/sheltie-core/src/work/state.rs:371`、`:388`；`crates/sheltie-runtime/src/load.rs:76`；推进经过 `core/work/next.rs` 的 Succeeded 分支与 `decide_begin`。涉及 T14/T20；T31 当前新增的计数必要界不能检出此问题。

现有验证检查“已有 approval 必须对应成功 Attempt”，以及“Blocked(Gate) 不能已经批准”，却没有核实一个已可离开门槛的成功 Occurrence 必须保留批准记录。runtime 的冻结图校验主要核路径和输入，也不补这个状态关系。

真实反例：gated-release 合法 notes submit→approve 后，只将该 Work 的 `state_json.approvals` 改成空数组；SQL status、revision、其他状态字段均不改。`work status` 仍成功给出 archive，`attempt begin archive` 也成功。原始响应见 `gate-evidence.json` 和 `probes-evidence.json`。

依据：[GF-12](../../../spec.md)：“批准前任何出边都不合法”；[constitution §4 第 3 条](../../../constitution.md) 要求批准记录；[plan 的 T14](plan.md) 要求校验关键状态组合并将损坏映射为 `STORE_CORRUPT`。

修复方向：集中执行需要冻结 Graph 的状态一致性检查，核对当前可离开及历史已离开的 gate Occurrence 与批准记录。合法状态继续接受；删除唯一批准记录的单字段反例必须在装入时拒绝，后续写操作不得登记新请求。保留原坏数据，不能通过补造 approval 或重算状态来“修复”。这是损坏数据检查缺口，不是同一 OS 用户之间的安全隔离问题。

### CR-P03 P3 add 的预检偏离粗检方案并重复完整扫描

位置：`crates/sheltie-runtime/src/workbook_repo.rs:191`、`:213`。

锁前调用完整 `load_tree`，已经 parse manifest、parse/compile Flow 并计算摘要；复制到 payload 后再次完整装入。代码旁“只对最终副本 parse/compile/digest”的注释因而不符合实际调用链。

依据：[storage §5.2](../../../contracts/storage.md)：源仅做“结构粗检”，并“只对最终副本 parse manifest、parse+compile 每个 Flow、算 workbook-digest/v2”。

按合同将源预检限于目录结构、对象类型和限额，最终副本作为内容解析、编译和摘要的唯一输入。保留预检拒绝与复制期间变化的反例，不引入基于源扫描结果的缓存身份。

本轴 3 项：2 个正确性问题、1 个合同偏离项。最严重项为 CR-P01 的初始化恢复缺口与 CR-P02 的门槛事实不一致。

## 验证与实施边界

`fmt --check`、`check --all-targets --all-features`、Clippy `-D warnings` 均通过。正式 nextest 命令因本机 `0.9.140` 低于当前配置要求的 `0.9.145` 而退出 92；跳过版本检查的补充运行退出 0：647/647 通过、0 skipped，另有 24 slow、1 leaky（`release_governance::check_specs_accepts_active_target_without_tag`）。不能改称正式门禁通过；leaky 警告保留，未据此推定原因。两个独立反例的脚本退出 0 表示错误成功复现，不代表产品 PASS。CR-S01 的故障注入仍为 `not_run`。

T31 当前加入的 checked 计数增量、历史文件 NOREPLACE、精确故障同步点、测试临时目录归属清理与同请求未提交残留处理均应保留。本轮没有发现必须删除 T31 才能审查的障碍；这些修改也不能代替上述缺口的修复或完整 mutation 处置。

建议先完成三个正确性问题的真实 caller 回归，再进行上述定向精简，保持原始失败证据与历史响应。每个修复按 package 任务与门禁归属交独立复核；本报告不新增或启动任务，不改变 T31 `doing`、M1/T16/T17 `not_run`。
