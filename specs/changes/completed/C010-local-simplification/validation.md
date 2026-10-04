# C010 验证

Candidate: `d744bd77180e8baeb6b31d07d844b62b9810e002`

## T01 采用范围

基线 53b51e7。独立范围审查 PASS，见 review。docs/specs/tests 均 exit 0；静态测试归属 948 项、任务卡 196 条，不是 Rust 执行结果。

- [文档检查](evidence/t01-docs.json)
- [规格治理](evidence/t01-specs.json)
- [测试治理](evidence/t01-tests.json)

原文按确定性 gzip 保存，metadata 记录每条 argv、退出码与解压字节 SHA256。历史 C009 绿测不当本轮执行。

## T02 固定输入与实际执行

实现基线为 `3fd442ffa7e7f590f1094f47c800f5e1d17d454d`。最终执行闭包 [input-a50864348de2c6361d3e0aac542717de3886f47eb2ee0f4fdb82f275e35f3a5d.json](evidence/input-a50864348de2c6361d3e0aac542717de3886f47eb2ee0f4fdb82f275e35f3a5d.json) 保存 378 项路径、字节摘要与权限；下表完整门禁全部绑定同一闭包，执行期间 input_drift=false。Reviewer 另逐项核当前原件。随后只更新本 package 进度/报告与归档路由，源码、方法、测试及构建输入保持；治理消费的这些文档变化另跑对应检查。

工具为 Rust/Cargo 1.98.1、nextest 0.9.145、MSRV 1.85.0、cargo-deny 0.20.2，见 [工具原件](evidence/t02-tools.json)。RUSTC_WRAPPER 为空，CARGO_TARGET_DIR=/private/tmp/sheltie-c010-target；Cargo 离线，deny 使用现有 advisory 缓存，不声称最新在线刷新。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 格式、全目标编译与 Clippy | executed | a50864348de2，当前源码/方法/配置 | t02-fmt / t02-check / t02-clippy；各 exit 0 | PASS | [fmt](evidence/t02-fmt.json)、[check](evidence/t02-check.json)、[clippy](evidence/t02-clippy.json) |
| 完整回归及故障窗口 | executed | 同一 378 项；all features、nextest 0.9.145 | 244c3674-e087-431d-a796-a63338ac7cd8；948/948、0 skip、1 slow、无 LEAK | PASS | [nextest](evidence/t02-nextest.json) |
| 公开类型约束与 MSRV | executed | 同一闭包；真实 1.85 locked-check | 5 compile-fail doctest；MSRV exit 0，不授 MSRV 实际执行测试结论 | PASS | [doctest](evidence/t02-doctest.json)、[MSRV](evidence/t02-msrv.json) |
| 依赖政策 | executed | 相同 deny.toml / Cargo.lock；已有离线缓存 | t02-deny-cache-lock；advisories/bans/licenses/sources ok，exit 0 | PASS | [deny](evidence/t02-deny-cache-lock.json) |
| 文档/规格/skill/测试归属/纯内核 | executed | 同一闭包；948 静态测试、197 卡条目 | t02-docs/specs/skill/tests/vocab；各 exit 0 | PASS | [docs](evidence/t02-docs.json)、[specs](evidence/t02-specs.json)、[skill](evidence/t02-skill.json)、[tests](evidence/t02-tests.json)、[vocab](evidence/t02-vocab.json) |
| C010 新方法实际 CLI oracle | executed | 相同 source/方法；真实临时管理根 | scripts/task.sh C010-T02；1/1 pass，947 筛选跳过单列 | PASS | [task](evidence/t02-task.json) |
| 任务范围与独立内容审查 | executed | T02 完整 staged diff；报告变化另记录闭包 | check-task C010-T02 3fd442f --staged；exit 0；Reviewer 独立核 82 个允许路径 | PASS | [scope](evidence/t02-scope.json)、[review](review.md) |

受影响短反馈 [t02-consumers-fixed](evidence/t02-consumers-fixed.json) 实际 528/528、0 skip、1 slow；它发生在正式闭包登记前，只作为短反馈记录，不复用成上表的完整回归结果。四个未通过记录如下，原文未覆盖：

| 运行 | 实际结果 | 原因与处置 |
| --- | --- | --- |
| [t02-method-red](evidence/t02-method-red.json) | 1 实际行为失败，948 筛选跳过 | 原方法批准后 0 refs，独立期望为 2；[原 0.2.1 文件摘要](evidence/t02-method-red-input.json)保存 |
| [t02-method-green](evidence/t02-method-green.json) | 1 失败，947 筛选跳过 | 成果/raw 已通过，resource 路径预期用了 /var，实际规范路径为 /private；独立 OS canonicalize 修正预期，未改产品实现 |
| [t02-consumers](evidence/t02-consumers.json) | 36/528 实际运行；35 pass、1 fail，492 未运行 | core 静态文件表漏新 resource；追加 include_str 项，保留原断言；此运行不称 528 执行 |
| [t02-deny](evidence/t02-deny.json) | exit 1 | advisory cache 锁在只读路径；取得同一缓存锁后离线同政策执行，未改配置或隐藏告警 |

## 完成范围

本轮只证明采用的实现收敛、完整测试后继和方法机械交付。新方法用例使用模拟阶段产物，不能证明真实 agent 遵从、真人接受、净收益或发布。磁盘卡、删除引用保护、原方法强度及替换语义保持。原 C002–C008 的未执行、未知与历史因果边界不变。

## T03 提交与归档读回

T02 本地提交为 Candidate 所列完整 SHA，正常提交钩子通过，未推送或安装。其 [完整原件](evidence/t02-commit.json) 另记录提交输入与 input_drift=false；[原文和实现读回](evidence/t03-original-readback.json) 核每份 stdout/stderr 的解压 SHA，并确认实现输入仍与正式运行一致。归档只移动本 package、更新进度/报告和两个当前路由；实际治理与真实 release_governance 消费者另记录，不复用旧文档输入。

归档后 [docs](evidence/t03-docs.json)、[specs](evidence/t03-specs.json)、[tests](evidence/t03-tests.json) 均 exit 0；实际 10 个 completed、0 active，948 静态测试与 196 条历史卡。真实 [release_governance](evidence/t03-governance.json) run 97c47f41-e913-49ab-a5d0-87addeb49edb 执行 30/30、0 skip；新执行闭包均无漂移。Reviewer 增量核 75 份旧 evidence 与 Git 原提交逐字一致，最终独审 PASS。最终范围门禁原件为 [t03-scope](evidence/t03-scope.json)。
