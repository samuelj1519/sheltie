# C002-T29：stats 与 next 同一次装入

基线：`4526b7e`。平台：macOS arm64。Linux 按用户授权保留 `not_run`。

## 行为与 V28

`WorkService::stats` 从一次 Loaded 返回文本、StatsJson 和协议形状的 next。三个结果都使用同一 state/graph；next 复用 core 的 legal_next 与 next_item_json。CLI 只渲染这一组结果，已删除第二次 status 查询；未新增持久视图或响应字段。

真实 CLI 测试 `stats_and_next_keep_one_snapshot_when_a_writer_begins_after_reader_load` 在装入后暂停读者，等待另一 CLI 的 attempt begin 完成，再释放读者。手写 oracle 要求旧响应 attempts=0、visits=1、next=begin/cancel；随后新查询才返回 attempts=1、next=submit/fail/cancel。tier=standard 来自 Workbook 合同的默认值，期望没有调用生产渲染函数生成。

`red.stdout.txt` 的 run `62afea1c-9e02-47ba-896c-bad43181426e` 保留旧实现失败：旧 stats 与新 next 混在同一响应。最初期望遗漏默认 tier，已按合同补齐，旧/新 stats 与 next 的关系断言保持不变。最终绿色运行见 `task.stdout.txt`。

## 停止路线与测试边界

writer 有 5 秒 timeout；reader 使用本测试局部 Drop guard，提前 panic 或 release 写入失败时尝试 release、kill 并 wait。正常路径先回收所有子进程，再断言结果。`stop-route-mutation.stdout.txt` 临时给 stats 增加跨同步点 HomeLock，测试约 5.74 秒失败、exit100；总耗时 13.761 秒含编译。`stop-route-mutation.metadata.txt` 记录逐字节恢复的源码 SHA。这只验证 T29 的禁止读锁反实现，不替代 T31 完整突变清单。

rendezvous 仅在 failpoint feature 下可由 name+id+dir 配置，默认构建不启用。CLI feature 显式映射 runtime/failpoint，供真实子进程测试使用。目录由测试创建并在测试结束后回收；writer 不继承 reader 的配置。T29 白名单精确扩展 failpoint.rs 和 CLI Cargo.toml，提前提供了 T31 需要的 feature 映射。

## 门禁、审查与后续

Rust 四门禁、MSRV1.85 locked、离线 deny 以及 docs/specs/core-vocab/tests/skill 的命令和原始输出保存在本目录。复用 T28 的 Cargo 编译缓存，测试仍使用独立临时管理根和当前源码；源码闭包见 candidate-input.txt。原始 deny 压缩保存，文本副本只去尾随空白。

两个失败日志的 `.raw.gz` 保存逐字节输出，文本副本仅去掉 Nextest 的尾随空白，以通过 whitespace gate。

最终定向 run `ce4bf2ae-5c76-4f19-ab7e-3627391a13aa`：1 passed、615 task-filtered。最终全仓 run `85201df7-8fc1-456c-9620-a9b345dc14e3`：616 passed、0 skipped、1 slow、1 leaky，exit0。LEAK 项是 `release_workflow_gates_announce_on_quality_job`；原始日志保留，不将其原因记为已关闭。

Spec 与 Standards 两位未参与实现的 Reviewer 最终 PASS，均只读复核，未自行运行代码。T29 只关闭 R15/V28 的同次事实装入；T30、T31、M1、真实 Host 与发布继续按各自门槛执行。全仓运行出现的 Nextest LEAK 保留原文，原因未确认，交 T31 综合验证，不用后续 PASS 关闭原因。

重跑命令与 T28 相同，任务命令换为 `scripts/task.sh C002-T29`，范围门禁换为 `scripts/check-task.sh C002-T29 --staged`。Cargo 使用 `/private/tmp/sheltie-c002-t28-target`，`RUSTC_WRAPPER=`；MSRV 使用 `/private/tmp/sheltie-c002-t28-msrv-target`；deny 使用本机缓存的临时 Cargo home，并显式 `cargo deny --offline check`。
