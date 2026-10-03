# C004-T07 实际执行与 M3 独立审阅

**T07：通过，仅已观察的实际执行范围。M3：需修改。** 审阅者 `/root/c004_execution_review` 未参与规程、任务或节点产物编写。仓库与专用 Home 全程只读；仅写本报告与同名 JSON。

## 候选与结论边界

准备提交为 `af830f5d188e7fc8a8fff8ff329bcbbe56a86464`，引擎源码候选为 `ccf7a0cc4d4c4a3a1b3c040473ab2cbc069a1982`。审阅覆盖当前 C004 文档差异、事前 freeze、完整运行记录、四 actor 原始工具记录、实际三份产物、只读 oracle 与下一消费者使用。T07 最终治理、staged scope、任务提交与提交读回此时尚未执行，留给 Root 完成；本报告不提前授予这些检查 PASS。

## 必须处理的 M3 项

**F-C004-M3-01：原始日志没有随目录移动进入版本控制。** af830f5 删除了旧 completed 路径中 21 份已跟踪的 mutation `.log`；新 active 路径的同字节文件受忽略规则影响，仍未跟踪。物理文件全部存在且与 ccf7 原 blob 逐字节一致，但新 checkout 无法取得完整 raw。`t06-preflight.tar.gz` 保存的是 27 份新准备材料，没有这些旧日志。依据是工程规范 §5 的完整证据要求，以及本轮保留全部旧记录的采用边界。修复只恢复精确 21 份原字节的 tracking，单任务提交；不修改原日志，不追改历史提交或扩大全局 ignore。Root 已安排独立 C004-T08，尚未执行，M3 待增量复核。

**F-C004-M3-02：逐查询观察要求有实际偏差。** plan 的 T07 写明每条 status/result 都比较五表与业务对象。handoff、review-running、final 三个窗口有完整前后快照；B 的两次直接 CLI status 只有原始成功输出、同一 Attempt/revision 与实际工具结束记录，没有各自前后对象快照。现有样本和同源工程证据支持已观察范围的只读行为，不能倒推本 run 每条查询均被逐条观测。该原义务保持未执行，不能补造快照。如果 M3 仍按原逐查询标准验收，需要先冻结包装观察器，再交实际执行者使用；若采用较窄窗口验收，必须明确保留原偏差，不称原标准全部闭合。

## 工程规范 §5 逐项核对

| 项 | 本次结论与依据 |
| --- | --- |
| 依据 | 本轮文档交付、A→B 同 Attempt 冷接续、独立 review 与三成果来自 T06/T07、protocol/task/project；未新增 Rust 行为。 |
| 不变式 | 未见 INV-1–INV-7 违反。Root 选合法 next；引擎只记执行与摘要；内容审查、代理接受、停写报告均为外部判断；无自然语言反推系统事实、宿主安装或第二状态权威。 |
| 正反例 | 已观察 running nonfinal/artifacts=[] 和终点 final/恰好三 refs；质量逐项保留旧 not_run/unknown。没有新规则或新 Rust 测试；逐 B 查询对象观察缺项见 F-02。 |
| 真实链 | 冻结 binary、真实 CLI、独立临时 Home；A 草稿经 B 接续、独立内容审查、deliver、提交封存及实际下一消费者均真实发生。 |
| 崩溃 | 本轮未新增写路径或协议，未重新故障注入。原工程证据按同源消费者引用；不能称本次执行了 crash/kill。 |
| 边界 | actor 仅写各自声明槽；记录中无管理状态写、后台命令或其他 agent 派发。成果实际为普通单链接文件、0444。停写范围仅限自启动命令与本轮 turn。 |
| 文档 | 开工包步骤/命令与现行 protocol/storage、C005 原件相符；旧义务完整。T07 最终治理检查尚待 Root，不以 T06 治理替代。 |
| 提交 | T06 事前提交在实际 Work 前，trailer 与任务对应。T07/T08 尚未提交；21 旧日志漏跟踪需 T08 闭合。 |
| 突变 | 没有本轮新突变执行。旧 13 项与剩余 105 未执行范围保持；原 raw 的 Git 保全见 F-01。 |
| 证据 | 实际 CLI 原 stdout/stderr/exit、四 actor tool 原件与 source_line、三 refs 与消费者原文可核；早期 deliver 部分调用只打印 r.output，未保存 shell exit 字段，不能补称逐条 exit0。 |

## 实际执行、质量与停写

A 在声明 change.md 写出 12631 B 的原义务与来源索引草稿。A 的 task_complete 为 17:45:03.248Z，B 的新 session 在 17:47:35.663Z 建立。B 的初始消息只给 Home/WorkId/binary/protocol，自行 CLI status→resume 取得 implement#1.0、revision 2、冻结 task/project 和草稿；同 Attempt 补齐 27476 B。最终 audit 没有 fail/replace/额外 begin；三个节点均只出现 #1.0。

四份摘录与其原 session 每个 source_line 精确相等，且覆盖其全部工具调用与响应。A/B/content-review/deliver 分别为 7/13/9/9 调用，均有返回与 task_complete。实际脚本只有只读材料检查及各自声明产物写；未见后台、长驻、未完成 shell session 或再次派发 agent。B 未获得 A 会话正文。该结论支持本轮 actor 停写，不能称全宿主进程树终止、真人关闭或模型历史首次。

独立内容审查逐项查 C005 原材料，未编写 change。本人另核六项原义务、CLI 命令、资格/一次额度/输入/门槛边界、版本、LEAK run/case、首动作与正反判据，未发现必须修改的内容问题。实际没有自然内容返工。delivery 初始可写路径措辞歧义导致一次澄清；执行者先停写，澄清未改 input/Flow/成果标准，往返计入成本。

## 最终成果与实际消费者

本人现场通过同 binary/Home 查询 status 与 result。前后五表完整行和 47 个业务对象的 dev/ino/mode/nlink/hash 相同；SQLite DB/WAL/SHM/锁载体不冒充业务原件。Work 为 succeeded、revision 7、effects_pending=false、next=[]。

| key | bytes | sha256 | source |
| --- | ---: | --- | --- |
| change | 27476 | e465b6cd1f45cd30fc2b88e798a58d0e0be9c62eff6e517c03b1eb725be93332 | deliver#1.0/input/change |
| delivery | 11870 | 7825e55ef8924d6af1513c08ade61c786eade0c52f71d113c09cf787720e26e2 | deliver#1.0/output/delivery |
| review | 9259 | 3edd3209f279dc5cd72806dd8501718676846bbd1bbee02c0157063a26b22b93 | deliver#1.0/input/review |

三条引用逐实际 byte/size/hash、brief 中的绑定路径和 evidence 副本核一致；每个实际对象 mode=0444、nlink=1。Root 已读取三文档并按交付说明执行 validation143–160 与 C005 preflight 的只读首动作，两个原调用 exit0，整理真实准备判断并代理接受本文档包。真实 C005 撤销七前提仍 null，C004 冷接续不能替代撤销场景。

## 时间成本与工程复用

prepare 从 17:23:21.309533Z 算，run 在首条 add 前于 17:41:37.324097Z 启动，硬截止 18:41:37.324097Z。记录/等待、路径澄清、两次观察 oracle 修正、补 actor 原件、独立审阅及最终治理均计入。到本报告时间仍在预算内；最终总墙钟尚待 Root 结束记录，不能在 17:59 消费者时点截断。usage、付费、人工活动与原真人接受保持 null，墙钟不折算净收益。submit-implement/begin-review 只有 at_utc，早期 deliver 的部分 shell exit 也未打印，保留实际粒度。

本人独立重算 freeze 的 6 方法、18 source、3 task/project/protocol 与 62 C005 原件 SHA，无漂移；各清单重叠，不相加。当前 192 个 tracked 消费输入与 ccf7 blob 逐字节相同，符号链接按 Git literal 核。C002 原 gates 134 输入为其同值子集，binary SHA 同 7c961；原 951 run 424c53bc-4a8a-4bf1-8c26-f4b600335a83、5 doctest 与原环境归属可引用。本轮未重跑 Rust、空 task.sh、deny 或旧突变。192-file 比对不观察仓库外的构建环境，不能冒充新的全工程运行。

原真人首次/复用/宿主重开、人类 paired 净收益、原实际接受、旧 LEAK 原因、旧工具版本/fetch 失败、105 未采用突变与排除平台均保持原限定，不转为本次 PASS。M3 暂不通过；修复和重新限定后只增量核发生变化的义务。

## F-01 的 21 份原日志

旧目录前缀为 `specs/changes/completed/C004-verifiable-delegation/`，新目录前缀为 `specs/changes/active/C004-verifiable-delegation/`。下表路径拼接同一 suffix；JSON 保存完整旧/新路径及旧/新 SHA。所有 old_sha256 与 new_sha256 相同，仅 tracking 缺失。

| suffix | bytes | old_sha256 = new_sha256 |
| --- | ---: | --- |
| `evidence/20261003-M2-mutation/compile/mutants.out/debug.log` | 27070 | `518dc2557ab574d7fdafab0e5ec2d7bfa21f4a58c523a32b56c03e8e500e7459` |
| `evidence/20261003-M2-mutation/compile/mutants.out/log/baseline.log` | 111883 | `314f582bca7aa29157a661ddff5c79bf9ee48dd2df278c9d31060f88ab7cb9db` |
| `evidence/20261003-M2-mutation/compile/mutants.out/log/crates__sheltie-core__src__flow__compile.rs_line_326_col_5.log` | 23435 | `aaeda38595b890dd97cecda639515bbec6bd6accaaf994b520aec87937ede952` |
| `evidence/20261003-M2-mutation/compile/mutants.out/log/crates__sheltie-core__src__flow__compile.rs_line_350_col_16.log` | 22131 | `a337aa53d8d88ef5a93b45327acb37f0066903713448070719d9feb392ca2c25` |
| `evidence/20261003-M2-mutation/compile/mutants.out/log/crates__sheltie-core__src__flow__compile.rs_line_353_col_16.log` | 19856 | `e84329c5bd9c14d71a443a53bf7a77a46662f4fec0ef7f84b1fe9885c5c82423` |
| `evidence/20261003-M2-mutation/compile/mutants.out/log/crates__sheltie-core__src__flow__compile.rs_line_356_col_16.log` | 19880 | `1b7f3b34460adc9bee5f7e6b71e1346beaa12628187b9e11e5bf49297b42c1ae` |
| `evidence/20261003-M2-mutation/compile/mutants.out/log/crates__sheltie-core__src__flow__compile.rs_line_359_col_16.log` | 18608 | `04c1dc79e407785d4c28fca714e77dac9c935a49f0d98f06251b06a981bef6bb` |
| `evidence/20261003-M2-mutation/resource/mutants.out/debug.log` | 18156 | `57c46bfa0943243bbeb42cd134b5cb478a63b986ed6e335fcf4a0f06fe6c9dde` |
| `evidence/20261003-M2-mutation/resource/mutants.out/log/baseline.log` | 69564 | `8fe838f969686cfd693fa15438b16dc8177196d49561a7e1223161991129dfa8` |
| `evidence/20261003-M2-mutation/resource/mutants.out/log/crates__sheltie-runtime__src__load.rs_line_164_col_45.log` | 50470 | `d4ac3f673c23c89f3361d49138a4ae071109988e5cf03e6471898c5986529d63` |
| `evidence/20261003-M2-mutation/resource/mutants.out/log/crates__sheltie-runtime__src__load.rs_line_164_col_64.log` | 50112 | `43abfe8ee279e99b57f2566d68f67092c9243726afc233484a232032ce6f333a` |
| `evidence/20261003-M2-mutation/resource/mutants.out/log/crates__sheltie-runtime__src__load.rs_line_164_col_83.log` | 50470 | `9270bce619c589eeddb2c2d16c66e7b2bf3b91b56aad79c4250c7d5b74148af1` |
| `evidence/20261003-M2-mutation/result/mutants.out/debug.log` | 3923 | `2e7deca0b10a0df984d2967ade7cf881394928c73acc35620d56411a3d9ed1df` |
| `evidence/20261003-M2-mutation/result/mutants.out/log/baseline.log` | 12290 | `986dd9f1645fc07752cf12cbddab4be7688cf8463f2cd597016cfbeb101c2ed9` |
| `evidence/20261003-M2-mutation/result_with_cli_baseline/mutants.out/debug.log` | 27416 | `15f03d34be0e5fc00e01212f017d4b185abf8402900f72035548a4c53bf3b860` |
| `evidence/20261003-M2-mutation/result_with_cli_baseline/mutants.out/log/baseline.log` | 142835 | `ccc64a68706d0c298121aa380e5633cd786c806178651e71e4ce9d65500773b5` |
| `evidence/20261003-M2-mutation/result_with_cli_baseline/mutants.out/log/crates__sheltie-core__src__work__result.rs_line_116_col_8.log` | 68897 | `6c7a51d2984b7ac84613685c6a38cc216f4a1fab0b25fc802212018b3b1dde55` |
| `evidence/20261003-M2-mutation/result_with_cli_baseline/mutants.out/log/crates__sheltie-core__src__work__result.rs_line_67_col_21.log` | 80266 | `fe698ad461fb09f129b4bd9ebff0629b784db387cf5b9f4d7f792a0b0acb6906` |
| `evidence/20261003-M2-mutation/result_with_cli_baseline/mutants.out/log/crates__sheltie-core__src__work__result.rs_line_73_col_8.log` | 70778 | `9afd6aadf487cab2bbcddf0165b54d1be5b33bd1eabdb4f03ff4ed4204752912` |
| `evidence/20261003-M2-mutation/result_with_cli_baseline/mutants.out/log/crates__sheltie-core__src__work__result.rs_line_81_col_51.log` | 70905 | `285255f24836712bbaa8c5d76f275b5ee54b21b22c1f6c7221de3a62a5672b01` |
| `evidence/20261003-M2-mutation/result_with_cli_baseline/mutants.out/log/crates__sheltie-core__src__work__result.rs_line_83_col_24.log` | 70827 | `c11054fa6262ff7f7ab20971451545883f2d3181c415852c0d1843b716ba2c5a` |

审阅完成记录时点：`2026-10-03T18:04:59.134024+00:00`；截至此点 run 墙钟 `1401.810s`，并非最终总成本。
