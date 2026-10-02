# C006 设计

状态：`proposed`。只以 C004 的 `work-result/v1` 为结果来源；实现 `not_run`。

## 1. 最小模块与真实 caller

| 入口 / 位置 | 职责 |
| --- | --- |
| `core/work/result.rs::{ResultView,result_view}`、`runtime/service.rs::WorkService::result` | 同一次读状态提供资格、revision 与精确 ArtifactRef |
| `crates/sheltie-runtime/src/result.rs`、`service.rs::load_row`、`load.rs`、`fsx.rs::ManagedFs::open_regular` | 按结果 key 可信读源；同句柄核摘要/大小，不直接信路径 |
| `crates/sheltie-cli/src/cli.rs::WorkCmd`、`commands/work.rs`、`commands/mod.rs::dispatch` | `--artifact/--revision` 严格参数、只读分派、原字节 stdout |
| `crates/sheltie-cli/src/output.rs::Outcome`、`print` | 现有文本/JSON 路径；原字节模式直接写 stdout，错误只写 stderr；不经过 String 打印、不转码或补换行 |
| 拟新增 `crates/sheltie-export/src/{main,source,target,export,error,output}.rs` | 参数、可信子进程、受限目标、整份复制、明确错误与输出 |

导出器不依赖 sheltie-runtime，不直接开 Store；只调用公开 CLI。可以复用稳定纯类型，但不为一个消费者抽通用文件系统 crate。管理根 `ManagedFs` 依赖 HomeLock，不能为了导出让它写任意宿主位置。

表中的 core/runtime 路径分别指 `crates/sheltie-core/src/` 和 `crates/sheltie-runtime/src/`。采用时核对 C004 完成候选的实际结果入口，同步精确符号与任务白名单。可信源读取与 raw 路由原语在 C006-T01 完整实现并测试；C006-T02 接正式参数与编排，之后才激活正常 raw/export 入口。不要求 C004 预先建立未使用的接口。

## 2. 结果与源字节

导出器先以绝对二进制和参数数组读取 JSON 结果，stdin 关闭，stderr 与 stdout 分离。payload 必须严格匹配 C004：`format/work_id/revision/workbook/flow/status/final/effects_pending/artifacts`；每项为 `key/path/sha256/bytes/source`，来源字段为 `attempt/kind/name`。`source.attempt` 是终点绑定或封存该槽的 Attempt，`source.kind/name` 是终点的 input/output 槽；不把终点说成所有输入的生产者。`path` 才是实际原件位置。C004 的排序、选择和空结果规则不在导出器重算。

JSON 接收上限为 1 MiB；超限拒绝。单个文件上限与管理根读取合同一致，为 32 MiB，选定成果合计上限 256 MiB；使用检查加法，声明超限在创建暂存前拒绝，实际字节超限立即停止源进程并拒绝发布。这是本工具的资源边界，不能通过 stdout 无限输出绕过。

每项调用 spec §2 的公开字节模式。runtime 按同一次 SQLite 读快照与冻结图复用结果投影，先核 revision、final、effects_pending 和 key。选定引用只允许管理根内的正式原件，路径经现有 `load` 与受限打开校验；没有来自导出器的任意路径入口。

源打开后，使用同一句柄检查普通文件、元数据、大小并流式读取；读取时计算 sha256 和实际字节，读毕复核句柄与路径身份，准确报告替换、增长、截断或内容不符。stdout 可能已经含字节，最终退出码才表示源核验通过。终态查询不自动完成 effects；待完成效果必须先按既有写链恢复，不能在复制中偷做恢复。

导出器把字节写入自己独占的目标文件句柄，独立计算 size/sha256，关闭 stdout 后 wait 子进程，只有零退出且两项符合才能继续。stderr 直接送诊断流，不按内容分类；无 stderr 管道积压，也不无界缓存。源和目标错误均不得发布目录。取消时终止并回收直接源子进程，不保证停止任意宿主进程树。

## 3. 副本目录与来源清单

```text
<authorized-parent>/
  .sheltie-export-<random>/          # 私有暂存；不是完成结果
    artifacts/0001/<original-leaf>
    artifacts/0002/<original-leaf>
    manifest.json
  <full-work-id>-<random>/          # 仅整份发布时出现
```

随机名用于避免冲突和独占创建，不是授权令牌、幂等 ID 或持久请求。最终名在本次执行内固定；冲突不覆盖，重跑选择新名。每个排序 Artifact 分配从 0001 开始的索引目录；原 leaf 必须是一个非空安全路径段，拒绝 `. / ..`、NUL、分隔符、特殊文件或名称碰撞。相同 leaf 位于不同索引目录，不需要复杂编码规则。

`manifest.json` 的格式为 `work-export-manifest/v1`，包含完整 C004 结果 payload 和按 key 排序的 `files` 映射，每项为 `key/path/sha256/bytes`；path 为上述副本中的相对路径。清单确定性序列化，不含临时目录名或时间，不加入 commit 的独立核实声明。不生成第二份重复的结果报告；查看说明由 CLI 输出完成。

所有目录 mode 为 0700，文件 mode 为 0600，副本归当前操作者且可编辑。权限只设在当前进程持有并核验的对象上，不能按不明名称 chmod。

## 4. 受限写入与整目录发布

1. 在任何目标写入前，逐段 NOFOLLOW 打开父目录，记录句柄身份，并与管理根路径及祖先对象核对不重叠。父目录不存在、路径含链接或无法核身份时拒绝。
2. 在父句柄下独占创建随机私有目录，打开并核同一 inode。后续相对目标操作全部锚定这个目录；不重新用绝对字符串路径写文件。
3. 独占创建索引子目录和普通文件；每项从源读取到同一目标句柄，核 size/sha256/退出码。每次检查文件身份与单硬链接，sync_all 后保留文件供最后读回。
4. 逐项通过受限句柄读回实际字节并与源期望核对。按已确定 schema 生成清单并核映射，同步所有文件、子目录和暂存根目录。
5. 核父目录与暂存身份，用当前锁定安全 Rust API 的 `renameat_with(..., RenameFlags::NOREPLACE)` 或实测等价操作，把整个暂存目录移到同一父句柄下的最终名。
6. 核最终名指向原暂存对象，同步父目录，确认后输出 complete。移动已经发生而后续失败时返回 publication_unconfirmed；保留对象、不自动补偿删除。

平台操作参考当前 `fsx.rs::{verify_tree_at,rename_tree_new,sync_managed_tree}`，但由导出器私有 target 模块承担授权父目录规则。T01 完整实现并实测锁定 API 的目录原子不替换、同步和原语竞态；T02 通过完整双二进制链复核；不能用“先检查不存在”替代 NOREPLACE。

异常保留暂存，不自动递归清理、不重新接管同名旧目录。当前进程持有句柄与身份时可以准确报告残留；kill 后工具没有持久恢复承诺。用户需要回收空间时按实际目录核查处理，工具不增加清理平台。

## 5. 原子性的边界

发布前，用户没有被称为完成的最终目录；暂存中可能有部分文件。原子移动后，全体已核验文件和清单同时出现在最终名下。同步与进程响应不属于目录可见性的同一个原子动作：移动后 kill 或 sync 失败时，目录可能已完整出现，但工具没有确认 durable complete。

并发两次独立导出可以得到两份完整新副本；无需锁文件或全局去重。目标竞争不会覆盖对方；父路径替换不把句柄操作导向替代对象。恶意同权限进程可以改任何对象，不承诺隔离或永久完整性；测试核拒绝路径、实际输出、身份、哨兵字节和权限。

## 6. Rust 与交付

新增外围 Cargo 包使用 workspace 依赖与 lints，禁止 unsafe；每 crate 一个 Error 枚举，路径、结果 key、摘要、revision 与目标名称以校验类型跨模块。状态使用 enum 表达；不为测试注入另造成功结果，不用 `anyhow` 穿过组件边界。

source 只管理公开子进程和严格解码，target 封装 OS 能力，export 管一次完整复制。不为未来同步、恢复或云送达增加 trait。真实测试从两份 Cargo 构建的二进制入口运行，构建路径取 `--message-format=json` 的 executable，不猜 target 目录。

新包 Cargo.toml 明确 publish=false，并在 [package.metadata.dist] 设 dist=false。锁定 cargo-dist 0.32.0 支持该字段；采用时仍核实际配置没有 packages override 把它纳入发布集合。T01 固定并验证本地 dist plan 的 exporter 排除；T02/T03 在该配置未变时引用原闭包和 run ID，不构建或上传 release。这项排除是开发范围约束，不是另一套发布流程。

T03 提供开发构建、临时目录使用和首次读者指南。工具 release 版本可以独立变化；只支持声明的结果格式和实际读取模式。首版不修改 self 安装/更新，不建立双归档、自动版本配对或发布流水线；确需对外分发时另按发布权限制定范围。

## 7. 阶段接口与实现责任

复杂模型在 T01 完整实现 source 的 strict DTO、字节/退出码/上限和可信读原语，以及 target 的目录句柄、身份、链接、独占写、读回、NOREPLACE、sync 与未确认错误。raw stdout/stderr 原语也由 T01 实测。export/main 只保留可编译的编排骨架，正常复制和 raw 参数不启用；骨架不能提供假成功。安全工作量主要在 T01 是有意安排，不把它化为空壳。

T01 创建所有 source/target/CLI/crash tests 和 fixtures：原语测试归 T01 且 green，新完整行为归 T02 先 ignore，并从真实未来 caller 跑有意义 red。M1 核平台、原语、接口、测试和实现准备。T02 只用固定 source/target 接口编排、接用户入口并删本任务 ignore，不改协议、OS、资源、恢复政策或 oracle。缺口交复杂作者的明确修复任务处理。

T03 按冻结 runbook 和现成工具执行真实副本使用，复杂模型解释不明失败和产品结论。M2 核全链和实际结果；M1 后没有变动的安全合同/原语以原输入闭包和 run ID 引用，新增用户链仍须新证据。
