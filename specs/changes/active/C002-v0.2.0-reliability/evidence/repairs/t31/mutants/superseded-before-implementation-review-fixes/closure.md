# 变异候选输入闭包

产品基线为 `b926789`。普通 Git clone 位于 `target/t31-validation/source`；临时提交 `da2bd13979df88dd987596d7ed726ef99bb89651` 只固定测试输入，不代表产品任务提交。`../candidate-input.txt` 记录 147 个源码、fixture、配置、锁文件与脚本 SHA；34 个改动文件另见 `copied-candidate.json`，均已逐字节核对。

完整清单共 2573 个变异：core 709、runtime 1864；只排除 `testkit.rs`。分片从零开始，`0/4` 至 `3/4`，最终核清单并集无漏无重。每个 package 先完成所有变异的完整 crate 测试，再把全部 missed 和 timeout 送完整 workspace，包括真实 CLI 调用；第二阶段逐项精确选择，不采样。caught、missed、timeout、unviable 分列；存活体处置完成前不记通过。

当前执行使用 cargo-mutants 27.1.0、Nextest 0.9.146、all-features、8 jobs、600 秒超时、离线依赖缓存，空 RUSTC_WRAPPER，并清除继承的 Git 定位环境。机器实测为 64 GiB 内存、16 CPU。`scripts/mutants.sh` 固定相对 target；每个变异副本使用自身 Cargo 注入的 CLI 二进制，不共享原始 binary，不在测试体构建 Cargo。

测试 profile 的执行配置为：

```text
CARGO_PROFILE_TEST_OPT_LEVEL=1
CARGO_PROFILE_TEST_DEBUG=0
CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true
CARGO_PROFILE_TEST_OVERFLOW_CHECKS=true
```

没有改仓库默认 profile 或发布 profile。先用同一批 642 原始 oracle 做独立 benchmark，再补公共合同测试并固定当前候选，加入 R20 计数一致性修复与正反 oracle 后，早期0aee普通clone全仓run `36cf2302-efc8-48f8-a1a4-1d5eb1770eb9` 647 passed；原文在 `unmutated-workspace-opt647.*`，但它不是本次持久clone输入的baseline。当前da2bd普通clone的完整647项baseline run `32ac9a76-3eeb-4e8e-a938-d7ed9b370680` 通过，647 passed、0 skipped；[原文](unmutated-workspace-durable.stdout.txt) 与 [元数据](unmutated-workspace-durable.metadata.json)。新inventory2573与旧候选名字集合一致，但旧执行结果不复用。默认 profile 的独立公共门禁另保留，不能被优化 profile 代替。

编译参数实测含 `-C opt-level=1`、`-C debug-assertions=on`、无 debug info。overflow-checks 环境显式开启；Cargo 在与 debug assertions 同值时省略独立 rustc 参数，rustc 此时仍开启溢出检查。见 `profile-benchmark/rustc-flags.stdout.txt`、[Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) 和 [rustc overflow checks](https://doc.rust-lang.org/rustc/codegen-options/index.html#overflow-checks)。独立 Reviewer 认可此执行优化，未放宽清单或 oracle。

cargo-mutants 自带 baseline 仅测选定 package，不能当完整 workspace。每次运行先移除已经归档的专用输出目录，再要求新 outcomes、成功 baseline 和完整处理数。完整执行脚本保留为 `pipeline.py`。

原 default profile 全 core 707 与 runtime 部分 380 的结果在 `superseded-default-profile-before-public-contract-oracles`：runtime 剩 86 项时主动 SIGINT，输出完整保留；它们不计本轮完整执行。2cfe 测试候选全 core 707、第二阶段3项及 runtime 83项保存在 `superseded-before-counter-integrity`。0aee测试候选全core709与跨workspace复验保存在 `superseded-ephemeral-input`，runtime第一片被环境清理中断；两者均不能计当前PASS。更早候选也全部标 aborted、superseded 或 INVALID，不复用。错误的 `4/4` 调用从未开始，旧结果计数已归零。

执行闭包只包含147个Rust源码、fixture、脚本、Cargo/Nextest配置与锁文件；`progress.md`、`validation.md` 和本目录证据在执行中记录新事实，不属于编译或变异测试输入。`copied-candidate.json`的34个文件SHA是da2bd克隆当刻的完整改动快照，不能把后来进度文档更新说成仍与工作树逐字节相等。最终产品提交后另核文档/治理门禁与候选SHA。

当前持久clone在`target/t31-validation/source`，被根`.gitignore`排除；验证进程临时副本仍在系统临时目录，若被清理，已完成片的raw归档以及本clone的`target/mutants.out`保留。`pipeline.py`未来重启采用独占运行锁、147项和工具/环境/自身SHA闭包、异常exit与未知outcome停机、完成片逐项对照counts/outcomes/inventory SHA及归档manifest、发现任何已有partial就保留并停止；运行中的首轮进程使用归档的`pipeline-at-launch.py`，最终逐片复核exit与真实测试FAIL后才接纳。中断片只对同闭包剩余精确ID续跑，不能把不完整结果计PASS。

`raw.tar.gz` 原样保存完整日志、diff 与 JSON；`raw-manifest.json` 给出每个原始文件 SHA。为便于审查，outcomes/inventory 同时保留在 raw 目录；需要日志时把 archive 解包到对应分片目录。压缩不修改日志字节。

当前全量执行和存活体处置仍未完成，最终结论以清单对账及独立审查为准。仅自由 reason 文本变化的 parse-input-source 变体已有独立合同语义等价证明；它不是字节等价或测试 PASS，仍保留完整 workspace 运行及 diff。
