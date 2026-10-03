# C005 独立审查

方案审查：`通过`（首次读者实施规划、产品边界与工具静态接线）。
历史方案审查时 package 为 `proposed`；架构准备、产品代码、测试资产、真实试用、平台和实施里程碑均为 `not_run`。

## 历史方案审查（采用前）

源码基线：`5256aa5e86614bd09eb9553076a04d2522ff38a4`。
独立 Reviewer：`/root/review_product_docs`，未参与本次实施计划、契约、oracle 或资产的编写。
未解决的必改项与阻断项：无。

已核首次读者导航、复杂作者与简单执行者分工、全部真实接口/消费者、准备原语与冻结测试、实际命令及参数/期望、白名单、停止交回、阶段入口/出口和完成边界。并直接核 task.sh 的非零/ignore 行为、check-task 的基准与 mixed/整文件/快照保护、check-tests 的任务标题和真实测试名归属。测试支撑、冻结基准和实际调用位置均与计划相符。

48 文件方案正文闭包：`53b84534c979016aa822e671a723edc0b479bf31e94780bd46956b43a4187eea`，不含五份 review.md。范围为五包其余七份文件及共同入口、权威、指南和来源。域为 `sheltie-proposal-docs/v1\0`，按路径排序，逐项 BE64 路径长度/UTF-8 路径/BE64 内容长度/原字节，再求 sha256。复算一致，记录不改变正文。

## 两次实施审查

| 范围 | 独立复杂模型的判据 | 状态 |
| --- | --- | --- |
| C005-M1 | 数值/历史前缀、严格载荷/输入/恢复原语、接口与冻结测试 | not_run |
| C005-M2 | 原子替换公开链、历史响应/恢复、Rust 工程与真实接续 | not_run |

每任务短语义复核保留；阶段对同闭包已经核准的工作引用原候选与 run，不重复全套门禁。M1 是准备就绪，不是公开功能、用户价值或 M2 PASS。Reviewer 不编写被审修复，问题交复杂作者补接口/测试并重固定基准。

文档、规格、测试声明/归属、五份任务 TOML/plan 对齐与 diff 静态检查通过；没有执行未来 Rust 用例或真实实验。实际命令与原文见 [validation](validation.md)，任务状态只见 [plan](plan.md)。全部实际采用义务完成后才能 completed；发布、推送、合并和外部安装不随方案通过发生。

## C005-T00 采用与C004归档

通过。独立 Reviewer `/root/independent_review` 未参与采用记录编写；首轮清理C004 plan/新active README/spec/采用条件/根当前合同等残留文字，作者修后增量通过。明确开发需求直接采用与真实使用前提分开，不主张已发生事故、停止或隔离；撤销正式资格不代表停止进程。唯一active、scope完成边界和未发布准确。Reviewer独立逐字节核C004迁移132份evidence：无缺失、无变化，授权缺项保持。docs/specs/tests/diff通过；无Rust变化不重跑。

## C005-M1 独立实现准备审查

阶段结论：`PASS`。Reviewer 为 `/root/independent_review` 及其只读辅助，未参与该设计、实现、测试或 oracle 编写。完整骨架 `8d00a29e10810c79bb5b44bdc006dc021e26637b`。该结论只表示 T02 准备就绪，不表示公开能力、真实价值或发布完成。

已审 core 编号、Superseded 组合与连续顺序、固定一次资格、失败前缀、完整输入观察、poststate stats；runtime 意图、完整有界 reason 审计与旧状态绑定、相邻身份/时刻/输入/来源/paths/requires、完整 payload 先解码再 I/O、同 FD 观察、事务、历史重放及精确字节恢复。零输入进入来源修复和旧 Store 诊断顺序修复均经独立增量复核。没有剩余生产必改。

上游 pipe、next 需调用者补内容、编号语义、schema 目标、错误说明、nullable 字段范围均已收口。三个既有 next 场景保留真实场景，预制未来 oracle 并准确迁移归属；阶段 skip 不计 PASS。T02 交接已改为只开放三个生产文件的 CLI/next，不重写冻结基础。

独立 core 3 项、辅助 core 11 项、runtime 3 项、history/strict 3 项和顺序修复 2 项均实际通过，原 run ID 与输出保留。最终门禁 764/764 PASS（2 slow、1 LEAK unknown、10 phase ignore），10 项有效 future-red，T01 提交前后 scope PASS。原 nextest 0.9.145 与在线 fresh advisory 仍 not_run；无零残留或真实收益声明。

所有 source/test/fixture/合同与 T01 骨架一致；M1 工作区只有计划和记录变化。可进入已授权 T02。

## C005-T04 测试修订短审

独立 Reviewer 结论：`PASS`（只批准预期修订）。旧恢复场景的合法 next 按公开合同确实包含 replace。保留原场景和全部字节 oracle，只手写新增行、迁移 T02 归属并保留 ignore；没有用 renderer/返回值生成期望，也没有修改生产来迁就测试。foundation 实际 1 FAIL 的 raw 与 byte diff 保留，禁用场景尚未通过。新 SHA 再作为 T02 allow_test_changes=false 基准。

Reviewer 确认 T04 可提交。全 cached whitespace 检查的两份原始输出尾空格单列，作者文件通过；不裁剪 raw，不声称全 cached 检查通过。

## C005-T02 短语义审查

独立 Reviewer：`PASS`。一次额度 next、参数身份/理由源、已有 WorkService 调用、原 Response 输出均符合合同，不宣称停止/隔离。c9492f8 测试基准正确；六份测试文件仅删除 11 ignore，oracle/fixture不变；14项草稿字节恢复可核。11 feature与774完整回归通过，原失败保留。此结论不替T03/M2或发布。

## C005-T03 短审

独立 Reviewer：限定范围 `PASS`。已完成机制与真实not_run分开，接手前操作者确认停止/隔离、不升级引擎保证；真实字段全null且授权延期，原义务仍保留。修后9技能消费者和治理检查通过，新LEAK unknown另记；代码/oracle未变。范围只覆盖说明与交接，不覆盖真实撤销价值。

## C005-M2 最终结论

结论：`PASS`（限定实现、说明、前提盘点与授权延期交接）。候选 `d8d8c20fc71e3bd8bb813afecc6a4aa1af4a63f4`，独立Reviewer没有编写该代码、设计或oracle。完整EX01–08和caller、number/failed/quota、历史/输入/并发/事务/恢复/严格载荷及首次读者说明已审，未发现剩余必改。

同消费者闭包的T02代码774/774原run、当前T03技能9/9和治理证据分别引用，未声称whole input相同；M1未变基础不重复长跑。真实撤销与收益not_run，原工具版本/onlinefresh/其他平台及LEAK unknown保留；原语、静态PASS或gate不能代替价值，不推导发布或永久无泄漏。补充Rust1.85 locked全targets/features编译已执行通过，测试工具链边界保持。

## C005-T05 当前补验事前审

Reviewer /root/c005_preflight_review未参与准备/实现/runner，当前准备scoped PASS，原件见[evidence](evidence/resume-20261004/t05-preflight-review.md)与JSON。核134/192及C004275原件同SHA、37用例原951实际PASS行、旧四LEAK真实run/case、七前提null及不造撤销样本。单一timeout处置finding在MSRV实际执行前修复；只授T05收尾提交，不授T06/MSRV/M3/原真实价值或发布。

## C005-T06 当前Rust1.85及行为独审

Reviewer /root/c005_preflight_review实际只读审通过，无必修项；[报告](evidence/resume-20261004/t06-independent-review.md)是原session最终消息精确内容，工具原件与source_line可核。六实际步骤、951/951、37用例、5doc、160.473s、134/192资格与EX01–08完整CLI-runtime-core/原子/计数/历史/恢复/门槛已核。M3技术/负前检范围通过，最终治理、任务提交和全blob读回仍待Root，不提前授完成。独立frozen1.85产物同SHA，target/debug重建后的变化另记录，不冒称原路径仍冻结。
