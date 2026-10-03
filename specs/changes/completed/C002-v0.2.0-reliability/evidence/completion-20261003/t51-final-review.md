# C002-T51 / G04 独立最终审查

结论：**PASS，限定当前 macOS aarch64 的 T51 源码/oracle、G04 精确27项及当前工程闭包。** 未发现剩余必须修正项。Reviewer `/root/oracle_review` 未参与 G04 方向、实现或 oracle 编写；只读逐项核原日志、diff、映射及产物，未修改源码、plan、历史结果，也未并发构建共享 target。

承接 `t51-final-source-review.md/json` 的14条测试审查，源码 SHA 未变。基准 `b5226d07a8abca1c2a1b7eaee4de71eb19cf770d`；`fsx.rs` SHA256 `241b16b73ec2f8288966b25cf9e2d236b99ee436b8914d2ee4bfb4242ae06989`，`failpoint.rs` `5a9268044be37aa05441aa81b66bc2ce6d4b1c708b19dd48b954615922e6889d`。相对基准仅这两个 crate 源文件变化；Core及生产public接口不变。新增准确窗口均通过既有failpoint机制，默认无feature没有新增文件/SQL/注入环境读取；路径字符串构造并非零成本。私有实际tmp路径payload复用原匹配/发布/释放，没有公开FD getter、通用observer trait或第二个随机名来源。

独立核27个 current mutation 的 function、准确column、相邻flag及锁定 libc 0.2.189/rustix 1.1.4 的 Darwin 位值与访问mode。最终集合与原 `e54dcd41…/SK02-missing-execution-map.json` 的G04完整27个旧ID一一对应、无漏重；原stage1全部 `MissedMutant` 保留。每个最终diff/log SHA及目标panic见同名JSON。

| 实际检测 | 数量 | 目标事实及边界 |
|---|---:|---|
| 真实exec FD继承 | 6 | child只枚举/fstat `/dev/fd`；变体实见 `[6]`、`[8]`或`[11]`，原控制空。remove/writable/mode-directory同log的另一个alias失败没有代替exec事实。 |
| 内核FD标志/访问mode | 6 | 真实fcntl测CLOEXEC、NONBLOCK、access `0 != 2`；不是setup提前失败。 |
| 排他创建/真实generatedtmp碰撞 | 2 | 变体不再拒绝占用；实际本次tmp路径来自before-open记录；控制保留原对象和外sentinel。 |
| 同inode alias实际接受 | 3 | external read及mode-file两项返回成功；alias指回真实旧A，dev/ino不变不能证明名称未变。 |
| 首次受限open诊断差异 | 8 | ensure-dir3项已有错误但类别变为后层拒绝；另5项未保留原NOTDIR错误类别/cause。不是8项最终非法接受。 |
| FIFO实际opened机制 | 2 | after-open reached真实存在；晚read_dir的NOTDIR不能掩盖先open FIFO。无timeout捕获。 |

FIFO预先救援FD在release前建立并持到operation结束，after-open release预置；无writer线程或Drop join unwrap。scope/release和OwnedFd覆盖普通panic/join清理，救援建立失败先还原再panic。marker三支路保留真实非目录错误和精确写入bytes：post-file-sync报RecoveryRequired保留对象，before-open报Io无创建。这不证明parent sync/断电持久或SQLite COMMIT，亦不等于完整CLI五表资格。

最终 A/B/C 分别12/12、12/12、3/3 Caught，elapsed 133.919/77.990/25.217秒；每批原候选14/14真实非零基线。27项均Build Success后Test exit100且命中上述目标，零Missed、Timeout、编译失败捕获；静态处分0。三个selection的190项输入完全相同，当前逐SHA匹配；完整gate的134项输入均包含其中且匹配。source_unchanged标记与独立回读一致。

完整原文门禁SHA回读匹配：nextest **894/894、0skip、0LEAK**，run `720db473-4ee4-40e6-b620-866743570264`，含2 slow；5 compile-fail doctest；fmt/check/clippy、Rust1.85、默认feature check均exit0。Task14/14通过。docs/specs/tests/core-vocab/skill的实际OK报告已核。完整gate使用显式固定engine/exporter env；Cargo `--message-format=json` 返回的真实executable路径、fresh=true产物及当前/frozen SHA全一致（engine `3db78ac2…80f0485e`，exporter `5eae70f2…efe4967`），不凭约定路径认构建产物。

早期sample Missed及初次compile101均保留，不被最终Caught反写。此前 `check-task` exit1是doing状态的准确门禁，**尚需Owner标done后复查、归档回读与提交**；本报告不声称已完成这些动作。限定当前exec路径/原语消费者，不扩大为全部并行spawn、所有平台、G05/其他组、C002-M2或C001–C008全部完成。
