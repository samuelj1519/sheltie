# C002-T47 失败回收修正后的增量审查

结论：**通过**。初审 F-T47-01 已修复，新的 child 夹具满足先释放、自有生产者有限退出、线程及 child 回收后再断言的要求。初审 `independent-t47-review.md` 原文保留，不反写初次需修改与错误 fixture 结果。此 PASS 限定于 T47 的 5 条 oracle 与 10 个指定存活体，不关闭 T43 其他义务、完整工程门禁、平台或 M2。

Reviewer：`Codex /root/oracle_review`。未参与实现、oracle 或此次修正，只读复核代码及原输出；仅写独立报告，没有构建或修改仓库、共享 target、mutation scratch。

HEAD：`06e5a9108be9afdfb9b5b68fc112ad08a600af8d`。当前 `selfmgmt.rs` SHA256 为 `b79de47dd76d1e40e54cbb489942e46ff3da0702b47cc40096c8f2a0e292b08a`；`store/tests.rs` 仍为 `f490e8b9b26a1da059d71d99e905ae4433df0d8d3e5ae6b155d1a77804f28d99`。selfmgmt 的 cfg(test) 前生产前缀与 HEAD 逐字相同；Store 测试和 Store commit/mod 生产文件哈希与首轮 selection 完全相同，资产名单测试内容也未改。改动只影响 child 夹具及其辨别力。

## F-T47-01 的修复依据

`std::thread::scope` 为 scoped worker 提供离开作用域前 join 保证。`ReleaseOnDrop` 在 scope body 中声明，body unwinding 会先执行守卫的 release 尝试，随后 scope 回收 worker；临时目录在 scope 外，回收完成前保留。spawn 前的 fixture 构造失败没有子进程需要释放。

正常路径先保存显式 release 写结果，再取得 `worker.join()` 结果；返回 scope 后才 unwrap/assert。显式 release 写失败没有提前 panic；shell 在 300 次有界轮询后 exit 73，`run_child_bounded` 观察退出并 wait/reap，join 完成后才报告原写错误。删除了原 channel 与接收超时 unwrap，worker panic 也以 join 返回值保存，父线程在 scope 返回后报告。

stdout/stderr 先显式关闭，随后写 reached 并等待 release；测试继续保持“恰好 4 bytes、pipe EOF、child 尚未退出”的真实条件。`worker.is_finished()` 观察提前返回，250ms 检查后明确释放。实际失败断言均在 join 后，四个突变产生 SIGKILL 状态并被准确拒绝，没有把杀掉的 child 当成功。

有界的是生产者 300 次轮询，不是严格 3 秒墙钟上限；sleep 调度和系统调用可能增加总耗时。3 秒仍为 reached 就绪观察预算。此修正没有引入生产 deadline、全局观察配置或进程树隔离承诺。

## 修后动态原文

- `t47-cleanup-baseline.txt`：Nextest run `2f13721b-0c6f-4ee3-be6f-17463e11c1f0`，5/5，52 项因过滤未选；不是完整套件零 skip。
- `mutation-cleanup-check/result.json`：exit 0，70.583s，Unmutated baseline Success 1、CaughtMutant 4、源码无漂移。outcomes 完成时点 `2026-10-03T03:43:22.001272Z`；0 missed、0 timeout。
- 四条指定 mutant 原 log 均在新测试的 join 后断言报告 `the child was terminated before explicit release`，含 SIGKILL wait status 9 和相应手写 stdout/stderr bytes。两个 `>`→`<` 通过空的另一条 pipe 提前 kill；两个 `>`→`>=` 在恰好限额 pipe EOF 后提前 kill。
- Store/资产名六项的源码、oracle、目标行为不变，保留首轮已完成原执行，未假称它们在修后重新执行。新的五项 baseline 已确认该合法闭包仍通过；增量重验准确针对四个改变的 child 消费者。

准确 argv、选择过滤与源码清单见 `mutation-cleanup-check/result.json`、`selection.json`，完整原文见 `mutants.out/log/`；首轮十项原文继续位于 `mutation-oracle-check/`。本 Reviewer 独立审计完成原文，没有重新运行构建或测试。

无剩余 T47 审查 finding。受影响 runtime/CLI、稳定候选完整工程门禁、文档治理、任务提交范围和最终 M2 由任务执行者按计划收尾，本报告不预判这些尚未结束的义务。
