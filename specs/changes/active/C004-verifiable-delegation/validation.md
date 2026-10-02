# C004 验证

状态：`not_run`。方案检查不代表实现、用户收益或平台验证。本文件保存 oracle 与真实执行索引；任务状态只在 plan。

## 1. 机制矩阵

| 义务 | 正例 | 拒绝或反例 oracle | Owner |
| --- | --- | --- | --- |
| 可信读取原语 | 同一事务和合法归属真实读取 | 非法索引/audit/effects/Start 定位拒绝；期望独立 | T01/M1 |
| 交接可执行性 | 编译、原语绿、T02/T03 真实非零用例与预定红 | 未完成入口不激活；测试冻结与手册缺项不交初级实现者 | T01/M1 |
| 终点声明 | required input/output 明确选择 | 非终点、optional、重复 key 拒绝 | T02 |
| 具体版本 | 从成功终点已绑定引用取 | 不取上游后来产物/目录最新文件 | T02 |
| 完整性 | 合法成功终点/gate | 缺必需引用或身份矛盾报 STORE_CORRUPT，不能空结果 | T02 |
| 交付就绪 | succeeded 且已登记效果完成 | 当前 Work pending、未过 gate、取消不展示 final 成果 | T02 |
| 请求范围 | unrelated Work 可 pending | 本 Work 结果不能因另一 Work 被隐藏 | T02 |
| 一致快照 | state/revision/effects/audit/Start 同读事务 | 并发恢复不能拼接两版；request 工作索引被改、audit 关联缺请求或已发布非法 effects 均拒绝 | T02 |
| 接续 | 当前 brief/input/next 足够继续 | 草稿只列路径，不登记已完成或投递全历史 | T02/T03 |
| 卡片/实时事实 | refresh 后 mark_published，当前查询效果提示正确 | 磁盘卡不保存当时 pending=true；共有状态/resume 不产生另套事实 | T02 |
| 查询只读 | 独立 Home 与真实 Store 查询 | revision/request/audit/业务文件不变，不刷新卡或恢复 | T02 |
| 普通方法 | 返工与复核按显式边 | 摘要说通过不产生非法边或批准 | T03 |

期望来自手写图/状态、原始字节、独立 SQLite 和 CLI 观察。记录 COMMIT 后未完成效果的实际窗口，不能以文件存在或 is_ok 代替同版本引用。只读 SQLite 控制文件按 D-039 单列，不冒充零 I/O。

## 2. 用户结果

沿 [C007 指标](../../proposed/C007-pre-run-workbook-generation/validation.md)记录真实使用者、新任务、固定方法、完整输入闭包、准备/首次/复用活动、解释重做、最终独立质量、未完成和 usage。机制回归与新任务收益分开；共享指标不是共享 PASS。试用可以支持继续、缩小或停止，不能把小样本方向推广成普遍节省比例。

## 3. 范围与预算

T01 先验证原语和所有受影响既有消费者；M1 只核完整架构/测试与阶段就绪。T02 按冻结测试交付，T03 方法场景不得自行改 oracle，T04 按固定试用规程保存原文并由复杂模型分析。源码/协议/跨 crate 变化与最终 M2 跑完整工程门禁；阶段已审部分相同闭包可引用原 run，不能把 M1 当最终 PASS。定向突变先固定选择规则/拒绝路径 inventory、真实测试组、基线耗时、并发和停止条件。每个执行 ID 保留原结果；未知、timeout、未执行不转为 caught。用户实验与产品门禁分别报告。

## 4. 执行索引

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 阶段架构/原语/测试/M1 | not_run | 待 T01 骨架与真实用例 | — | not_run | — |
| result/status 真实链 | not_run | 待采用候选 | — | not_run | — |
| 完整工程与定向突变 | not_run | 待稳定候选 | — | not_run | — |
| 方法/skill 场景 | not_run | 待实现方法 | — | not_run | — |
| 新真实任务与重开 | not_run | 待真实使用者与新任务 | — | not_run | — |
| 平台与发布 | not_run | 只验证实际采用范围 | — | not_run | — |

执行后每行写退出码、非零测试数、输入闭包和原文链接。相同闭包证据复用要引用原 run ID；不同方法、模型、feature 或环境不混算。完整原文只保存一次，review 只链接本表。

## C004-T00 入口检查

输入闭包：T00 采用与链接变更；Rust/方法/实验未改变。2026-10-03 执行 `scripts/check-docs.sh`（131 文件）与 `scripts/check-specs.sh`（8 change、1 active），退出 0。独立 Reviewer 只读核授权、状态/历史与完成边界；状态残留修复后重核。产品实现/用户价值仍为 not_run。

原文索引：[C004-T00-20261003](evidence/20261003-T00.txt)。独立复核通过，仅覆盖采用与文档入口。

## C004-T01 准备过程原文

首次统一 check/clippy 曾因 self 提示访问私有 store 常量而退出 101，随后漏改一个调用参数也退出 101；已用既有公开 `selfmgmt::version_info` 同源取得 schema，完整调用者改齐。保留 [初次编译](evidence/20261003-T01-check.txt)、[参数修订编译](evidence/20261003-T01-check-repaired.txt) 和 [首次 clippy](evidence/20261003-T01-clippy.txt)，均不是行为红。

修订后 [check](evidence/20261003-T01-check-final.txt)、[clippy](evidence/20261003-T01-clippy-final.txt) 退出 0。输入闭包 `cb2304529df7fc6766160e310cb9dd9754d336b197bd7bc975acf209a74751da`（263 文件；源码/测试/fixture/config/协议与消费者，排证据及交接/审查/验证记录）。后续独立发现产生了新修订，以上不覆盖修订后完整闭包。

- [原 task 命令](evidence/20261003-T01-required-nextest.txt)：nextest 0.9.140 < 配置要求 0.9.145，退出 92，测试 not_run；按用户环境授权保留，不安装或更新宿主工具。
- [原语补测](evidence/20261003-T01-primitives.txt)：run `6685caed-230f-44f5-b255-c9f30bb3e853`，25/25 PASS、713 不在过滤范围；实际版本 override 命令，不称 0.9.145 门禁已执行。
- [真实 caller 红](evidence/20261003-T01-public-callers-red.txt)：2 执行、2 预定失败、退出 100；status revision 缺失与 CLI result 不可调用，非编译失败或占位 panic。
- [schema fixture 修复](evidence/20261003-T01-schema-fixtures-repaired.txt)：run `45237845-ca71-49a4-af3b-aa9398d1816b`，合法打开控制、坏列准确定位、CLI 初始化恢复 3/3 PASS，退出 0；其余形状和全回归待完整冻结后运行。

独立首轮需修改项见 [review](review.md)。最终候选、门禁和 M1 结论在实际完成后登记；当前不把准备过程通过当产品能力完成。

## C004-T01 修订候选

修订源码/消费者闭包：`1cc9b7f972ff3a746511b4ee583b5c899572d20f23f3de430bf7c88731c3885f`（263 文件，排 evidence/progress/review/validation）。资源真值与读取顺序的 [行为 red](evidence/20261003-T01-runtime-repair-red.txt)：2/2 预定失败、退出 100、run `a6aebe89-a712-4834-a2b7-40da435c57d2`；[修复 targeted green](evidence/20261003-T01-runtime-repair-green.txt)：16/16 PASS、退出 0、run `f3ac5fa2-005c-4099-9c9b-8b4a0b29082f`，包括实际 pending/原响应消费者。

root 修订后 [check](evidence/20261003-T01-revised-check.txt)、[clippy](evidence/20261003-T01-revised-clippy.txt) 退出 0；[T01 同归属补测](evidence/20261003-T01-revised-primitives.txt) run `32c19836-f360-400a-b8f2-d546f8f867ce` 为 28/28 PASS，退出 0，713 不在本过滤；工具仍为 nextest 0.9.140 override，0.9.145 原环境门禁 not_run。

fmt、docs/specs/tests/skill/corevocab/diff 通过；测试登记 741 个、任务卡 246 个，T01/T02/T03 真实归属为 28/11/2。未启用的公开接口/方法场景保持所属 ignore，不能算本阶段成功。完整现行回归原文写 [此 run](evidence/20261003-T01-regression.txt)，实际结果在结束后登记。

## C004-T01 最终基础闭包

最终生产/消费者闭包 `f30b96b62d973a8017a0a997d4263be19e138815a9c7b2501551f039cb5b3227`（263 文件，排记录）。[最终 check](evidence/20261003-T01-projection-check.txt)、[最终 clippy](evidence/20261003-T01-projection-clippy.txt) 退出 0；fmt 与文档/规格/测试/skill/corevocab/diff 通过。没有新增依赖，deny 留到最终里程碑按实际闭包执行。

[首次全回归](evidence/20261003-T01-regression.txt) 退出 100、499 PASS/1 FAIL/228 未执行；[修订卡片后的全回归](evidence/20261003-T01-regression-repaired.txt) 退出 100、727 PASS/1 FAIL。失败与修复见 review，不删除原 run，不把未执行转换成通过。

[卡片修复](evidence/20261003-T01-card-oracle-repaired.txt) 和 [投影修复](evidence/20261003-T01-projection-repaired.txt) 分别通过；[最终全回归](evidence/20261003-T01-regression-final.txt) run `6acddb6a-86d1-44d9-a278-f785c715cf6a`，命令 `CARGO_TARGET_DIR=/private/tmp/sheltie-c004-target RUSTC_WRAPPER= cargo nextest run --override-version-check --all-features --no-tests=pass --no-fail-fast`，退出 0，728/728 PASS，2 slow（60.342s/69.842s，通过且非 timeout），13 T02/T03 阶段 ignore。工具为 nextest 0.9.140，Rust 1.98.1 / aarch64-apple-darwin；0.9.145 原版本门禁仍 not_run。

本阶段基础满足；不把 13 后续阶段 ignore、公开 caller 的预定红、缺真实用户/平台/发布替代为完整产品或价值 PASS。T01 提交前范围门禁与 M1 完整候选随后登记。

C004-M1 最终固定基础 SHA：`0cedb8c1f3ee7ba8d4041151a5382d3b9e8abb4e`；独立阶段准备通过，冻结守卫及提交后治理见 [review](review.md#c004-m1-最终阶段准备结论)。M1 只新增审查/证据/交接，未改编译或运行输入，不重跑未受影响 Rust。

M1 初次范围门禁退出 1：两份独立冻结守卫原文未列入 M1 files。初次本地记录提交在该失败后误执行；已补两个精确证据路径，重新核范围并修正同一个本地 M1 提交。生产/测试/fixture 和 T01 基准没有变化。

## C004-T05 规范根 oracle 修复

[真实 alias 失败](evidence/20261003-T05-alias-oracle-red.txt)：T02 draft、binary 0.3.0-rc.1，run `8b6c3faf-a864-4b72-9b85-c200db135395` 为 10 PASS/1 FAIL、退出 100。生产返回规范路径，期望使用了系统别名；不记环境豁免。

[修复消费者验证](evidence/20261003-T05-alias-oracle-green.txt)：同 draft 公开接口与修正 oracle，19/19 PASS、退出 0，含 11 T02 与 8 已实现读取原语。实际 nextest 0.9.140 override，原 0.9.145 环境门禁不变。该 run 只证明修正期望与真实消费者吻合，不把尚未提交/完整验收的 T02 当产品完成。T05 提交只含 oracle 与记录，原 T02 ignore 保留。

隔离过程：初始工作区全为本会话 11 个已明确归属的 T02 草稿，按内容 sha256 记录并保留在本地 stash `13538700ac45b3ea18280e32b7ccdb1c3b77f2cc`；T05 提交后重新应用这些草稿，不删除或覆盖他人修改。

## C004-T02 公开行为与适用门禁

冻结测试基准：`2914e8564047809659c4610d151f9ea6ceef7476`。开发 binary `0.3.0-rc.1`，只改 workspace 三个包版本，没有依赖变化。接口/消费者闭包 `5e2cdec810e72d27005bae8e489e1d53c89f468b48fbe5cd5dc4af584b42fb80`（263 文件，排记录；后续仅手册基准修正与计划状态记录，独立治理另核，不称全部字节相同）。

- [冻结 red](evidence/20261003-T02-frozen-red.txt)：run `370f6057-41f3-412a-bdf7-cb1f48497ac9`，11 执行＝3 runtime PASS＋8 CLI 预定失败，退出 100，保留原命令/路径。
- [最新冻结 green](evidence/20261003-T02-frozen-green.txt)：run `7e12c178-59a7-4dd0-8a0d-0529053d8959`，11/11 PASS、退出 0；oracle 修复见 T05，T02 测试差异只有 ignore 删除。
- [check](evidence/20261003-T02-check.txt)、[clippy](evidence/20261003-T02-clippy.txt) 退出 0；fmt/docs/specs/tests/skill/corevocab/diff 通过。
- [完整工程回归](evidence/20261003-T02-regression.txt)：run `0f862f61-d69c-4a65-a317-a7bb8236f5f0`，739/739 PASS、2 slow、1 LEAK、2 T03 ignore，退出 0；命令为隔离 target/RUSTC_WRAPPER= 下 `cargo nextest run --override-version-check --all-features --no-tests=pass --no-fail-fast`。实际 nextest 0.9.140，要求的 0.9.145 环境门禁仍 not_run。

LEAK 出现在 `implementation_repairs::killed_first_store_initializer_allows_the_same_add_request_to_retry`。nextest 0.9.140 的 [官方语义](https://nexte.st/docs/features/leaky-tests/)是测试退出后输出句柄未在等待期内观察到闭合，不能据此断言永久孤儿。历史 run 没有 PID/FD 观测，cause 保持 unknown。[一次独立诊断](evidence/20261003-T02-leak-diagnostic.json)观察到 exit 后 stderr/stdout EOF 相差 0.040/0.056ms、最终进程组空；很短的一次 CLI 未被采样，不声称全程零遗漏，不溯及证明历史 run。可复现 [脚本](evidence/20261003-T02-leak-diagnose.py)和实际 Cargo/nextest [binary 元数据](evidence/20261003-T02-leak-binaries.json)保留；未改 source/config，未确认责任修复。728/739 等原执行数不因诊断改变。

首轮 schema 独立失败原文由当时工具输出补保存，不重新执行或重命名旧 run：[合法控制](evidence/20261003-M1-schema-original-control-fail.txt)、[形状与初始化](evidence/20261003-M1-schema-original-shape-and-init-fail.txt)。此处只归档迟到原文，历史结论仍在 T01/M1。

## C004-T03 实际方法消费者

最新测试基准 `2914e8564047809659c4610d151f9ea6ceef7476`。图、声明/上限、任何 oracle 未改，只有2方法ignore删除。采用 writing-for-agents 与中文技术文档规则完善首次使用/复用、接续与最终引用说明；技能只教公开 CLI，不存状态、不推断推进。

[方法场景](evidence/20261003-T03-method.txt) run `a7c04f29-471d-403c-a855-aefded5becef` 为 2/2 PASS、退出0；[技能/样例消费者](evidence/20261003-T03-consumers.txt) run `b76ee226-c05a-442b-ab62-d02ceb40b2c7` 为23/23 PASS、退出0。实际 nextest0.9.140 override，原0.9.145环境门禁仍not_run。fmt/docs/specs/tests/skill/corevocab/diff通过；无新Rust生产代码或格式，不重跑未受影响工程链。M2稳定完整候选按其要求验证。

[受控CLI trace](evidence/20261003-T03-cli-trace.json)从Cargo compiler-artifact.executable定位实际binary，18次实际CLI完成implement→review→back→implement→review→deliver，三项selected文件由stdlib独立重读hash/bytes与引用一致。初次工具println写19，实际记录数组及cli_call_count均为18；按真实数组纠正，未产生第19次调用。home及输入都是自有临时夹具，报告明确机制演练；human_first_use和quality_and_cost_benefit均not_run，不能换算用户净收益。

## C004-T04 本轮真实试用边界

[前提盘点](experiments/readiness.md)按授权记录已确认缺项与补验入口；没有正式用户run。任务输入/使用历史/两组配置及活动/独立盲审/实际接受/原宿主会话动作的可核规程与原文均未建立；usage null。用户已批准执行，不等于这些行为已发生。独立事实复核通过，仅覆盖记录。docs/specs/tests/diff退出0；纯记录不重跑Rust，不虚构task.sh零测试PASS。当前真实trial/result-quality/cost-benefit/真正重开为not_run，后续按stage-1与C007指标补验。
