# C009 验证

Candidate: `2ce3967f4932bc5f64adccc76198d827679703dc`

源码基线：`620a9b13858eba5afa68039cd3c2ce9a91e66c4f`。36 份变更 Rust 文件按路径、NUL、原字节 SHA256 排序哈希的闭包为 `798a869b674eda683d3661ce4bf3a1cce6a78bd9849eb7fa57c7807c13850068`，与独立审查一致。实际编译/fixture/工具输入见 [输入闭包](evidence/input-afb36b180c8ec9b3.json)，工具版本见 [tools](evidence/t02-tools.json)。当前平台 macOS aarch64，stable 1.98.1；MSRV 单独使用 1.85.0 和独立 target。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 格式 | executed | 最终源码闭包 | cargo fmt --all -- --check | PASS | [原文](evidence/t02-fmt-final.log.gz) |
| 全目标/特性编译 | executed | 最终源码闭包 | cargo check --all-targets --all-features | PASS | [修订后原文](evidence/t02-check-fixed-import.log.gz) |
| Clippy | executed | 最终源码闭包 | cargo clippy --all-targets --all-features -- -D warnings | PASS | [原文](evidence/t02-clippy.log.gz) |
| T02 所属消费者 | executed | 最终源码闭包，7项/941筛选跳过 | 43d908cc-5fd5-4f1e-a8a7-785319d7462a | PASS | [7/7原文](evidence/t02-task.log.gz) |
| 全量行为回归 | executed | 最终源码闭包，全部特性 | f87b6553-591f-412c-a90f-be30182415f9 | PASS | [948/948、0 skip、无 LEAK；1 slow](evidence/t02-nextest.log.gz) |
| 文档代码示例 | executed | 同源码，独立 doc target | cargo test --doc --workspace --all-features | PASS | [5/5原文](evidence/t02-doctest.log.gz) |
| MSRV | executed | 同源码，Rust 1.85.0，locked | cargo +1.85.0 check --workspace --all-targets --all-features --locked | PASS | [原文](evidence/t02-msrv.log.gz) |
| 依赖策略 | executed | Cargo/lock/deny未改，现有离线advisory库 | cargo deny --offline check | PASS | [原文](evidence/t02-deny-cache-lock.log.gz) |
| 文档与规格 | executed | 当前文档与package入口 | check-docs / check-specs | PASS | [docs](evidence/t02-docs.log.gz)、[specs](evidence/t02-specs.log.gz) |
| skill与测试治理/纯内核词汇 | executed | 当前skill/归属/源码 | check-skill / check-tests / check-core-vocab | PASS | [skill](evidence/t02-skill.log.gz)、[tests](evidence/t02-tests.log.gz)、[vocab](evidence/t02-vocab.log.gz) |
| 行为与测试后继独审 | covered | 完整T02 diff及已核源码闭包 | /root/product_evidence | PASS | [review](review.md) |

## 失败与修订

- 首次 check 因 fsx 测试模块仍使用 BTreeMap 而缺 import 失败；[原文](evidence/t02-check.log.gz)保留。恢复该测试 import 后重新编译，修订源码才用于后续测试；编译失败不算行为红。
- 首次离线 deny 因沙箱不能获取既有缓存的锁失败；[原文](evidence/t02-deny.log.gz)保留。获工具批准后在同一离线库上执行，未联网、未改依赖。
- 独审 F-C009-01 发现共享初始化夹具的线程与同步目录退出顺序不闭合，修为 scoped threads：scope内先释放，join后才回收外层目录。新增 Err/panic 两变体验证；实际任务与全量均通过。

## 处置与范围

[测试处置](test-disposition.md)列确切后继；[盘点](evidence/t02-disposition.json)记录四项退休、一项新增和名称/入口迁移。Rust源码含空白/注释从62,387行到62,195行，增加两份共享测试支持文件；计数不代表收益或性能。全部snapshot原件未修改。

批量生命周期索引、跨请求缓存、filesystem大重构没有采用；本次只复用已核载荷，保留不同时间的Store/文件复核。没有新的产品能力或发布。原真人净收益、费用、物理宿主与其他平台缺项保持原验收记录；本轮不授这些结论。

## 归档与原文

T02提交的pre-commit钩子通过，原文见 [提交记录](evidence/t03-t02-commit.log)。工具日志为无损 `.log.gz`，用 `gzip -cd <path>` 读取；[归档清单](evidence/t03-log-archives.json)给原字节长度和SHA256，全部压缩后读回相同。保留原有终端空白和EOF，不为格式门禁修改执行原文。

归档后的实际文档消费者为39/39，见 [运行原文](evidence/t03-governance-consumers.log)与 [命令/闭包](evidence/t03-governance-consumers.json)；只执行 release_governance、skill_delivery、skill 组，不当作再次执行全部Rust。最终 docs/specs/tests、diff及任务范围原文见 [收尾检查](evidence/t03-checks.json)，全部exit0；204份编译/fixture输入和16份压缩原文的读回见 [readback](evidence/t03-archive-readback.json)。
