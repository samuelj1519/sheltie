# C004 独立审查

结论：`PASS`（本轮实际采用的实现闭包；授权延期/未知项不在PASS范围）
Candidate: `aa26d2c4f4396302b4b4f00394ada57a7b0e84b8`

最终独立Reviewer：`/root/independent_review`。本结论不含真实用户收益、原版本环境门禁、在线fresh更新、全mutation或未执行平台；具体限制见validation独立缺项表。

方案审查：`通过`（首次读者实施规划、产品边界与工具静态接线）。
历史审查时 package 为 `proposed`；架构准备、产品代码、测试资产、真实试用、平台和实施里程碑均为 `not_run`。

## 历史方案审查（采用前）

源码基线：`5256aa5e86614bd09eb9553076a04d2522ff38a4`。
独立 Reviewer：`/root/review_product_docs`，未参与本次实施计划、契约、oracle 或资产的编写。
未解决的必改项与阻断项：无。

已核首次读者导航、复杂作者与简单执行者分工、全部真实接口/消费者、准备原语与冻结测试、实际命令及参数/期望、白名单、停止交回、阶段入口/出口和完成边界。并直接核 task.sh 的非零/ignore 行为、check-task 的基准与 mixed/整文件/快照保护、check-tests 的任务标题和真实测试名归属。测试支撑、冻结基准和实际调用位置均与计划相符。

48 文件方案正文闭包：`53b84534c979016aa822e671a723edc0b479bf31e94780bd46956b43a4187eea`，不含五份 review.md。范围为五包其余七份文件及共同入口、权威、指南和来源。域为 `sheltie-proposal-docs/v1\0`，按路径排序，逐项 BE64 路径长度/UTF-8 路径/BE64 内容长度/原字节，再求 sha256。复算一致，记录不改变正文。

## 历史方案中的阶段审查判据（当时 not_run）

| 范围 | 独立复杂模型的判据 | 状态 |
| --- | --- | --- |
| C004-M1 | 总体契约、可信读取原语、可编译窄骨架、真实阶段测试与初级实现交接 | not_run |
| C004-M2 | 完整 status/result、已审方法、Rust 工程与真实质量/成本/续接 | not_run |

每任务短语义复核保留；阶段对同闭包已经核准的工作引用原候选与 run，不重复全套门禁。M1 是准备就绪，不是公开功能、用户价值或 M2 PASS。Reviewer 不编写被审修复，问题交复杂作者补接口/测试并重固定基准。

文档、规格、测试声明/归属、五份任务 TOML/plan 对齐与 diff 静态检查通过；没有执行未来 Rust 用例或真实实验。实际命令与原文见 [validation](validation.md)，任务状态只见 [plan](plan.md)。全部实际采用义务完成后才能 completed；发布、推送、合并和外部安装不随方案通过发生。

## C004-T00 采用与实施入口

开工候选：`18e043f2ff7680683ca2d9b7aca2b82fbf4cdbeb`。独立 Reviewer：`/root/independent_review`。首轮发现 change 顶部与迁移 package 的旧状态残留；作者已修正采用状态与历史审查标题。授权、一次 active、迁移链接、逐项 not_run、无虚假产品/价值/发布 PASS 已核。最终短复核：通过。Reviewer 独立重跑 docs/specs/diff，未修改文件；产品与用户价值不在 T00 结论内。

## C004-T01 / M1 首轮实现审查

独立 Reviewer：`/root/independent_review`，其 Standards/Spec 辅助审查者未参与实现。结论：需修改。三项实现必改：same-version schema fixture 和 CLI 初始化断言未同步 3；Resource 持久输入只核路径，篡改单个摘要或大小仍可 final=true；完整非 Start 请求载荷严格解码晚于冻结 Workbook 业务读取。另补 T02 实际 runtime 3 个测试名，过滤应为 11 例。

独立核心 4/4 通过，原文见 [独立核心 run](evidence/20261003-M1-independent-core.txt)。独立 schema 合法控制 run `19745b44-0cec-49e5-9d45-c44caff9be19` 为 1/1 FAIL，形状/CLI 初始化 run `5f56e0eb-bb68-4931-9144-586d5adf9744` 为 2/2 FAIL。Resource 单字段反例见 [独立脚本](evidence/20261003-M1-resource-counterexample.rs.txt)。作者修复，Reviewer 不代写。

首轮查阅真实 CLI 的两个预定行为失败；编译失败另记，不能冒充行为红。版本门禁与实际 nextest 0.9.140 补测分开；M1 未通过，尚不能接 T02。修复后重新固定输入闭包并增量复核。

## C004-T01 / M1 修复增量复核

独立 Reviewer：`/root/independent_review`。四项缺口增量短审通过。独立 6/6 用例 run `2b6acdf6-28ba-4352-8d98-9b957ca2c768`，原文见 [修复独立验证](evidence/20261003-M1-independent-repairs.txt)。未改独立 Resource 脚本重新链接修订候选后，合法控制成功、单摘要与单大小篡改均准确 STORE_CORRUPT；见 [原文](evidence/20261003-M1-resource-repaired-independent.txt)。首次临时链接错误只作诊断，不作行为红。

同闭包未改变的 core 选择/gate/resume、真实 CLI 预定红沿前次结论；正式完整候选 SHA 与回归尚待固定，M1 未作为公开功能或价值结论。

## C004-T01 完整现行回归与投影职责

第一次全回归 run `f81e1b5e-8f96-4fdb-b380-189b2166f3d3` 为 499 PASS/1 FAIL/228 未执行（fail-fast）；失败是旧手写恢复卡期望漏 resume。作者按新合同和独立输入摘要补齐，保持逐字节判据，Reviewer 独立 1/1 通过，见 [卡片 oracle](evidence/20261003-M1-independent-card-fixture.txt)。

第二次 no-fail-fast 全回归执行 728：727 PASS/1 FAIL；真实 publication consumer 暴露 status() 返回卡 JSON 却把实时 metadata 放进配对文本。作者修复私有 status_projection 共享一次严格读取：card API 输出纯共有投影，status_read 渲染完整实时 DTO。Reviewer 核职责且独立 3/3 通过（run `de36c8c7-7ac3-4f5a-8f40-dd268f213d63`），见 [投影独立复核](evidence/20261003-M1-independent-projection.txt)。未降低旧 oracle，不使用字符串剥离或第二事实路径。

最终候选完整现行回归 run `6acddb6a-86d1-44d9-a278-f785c715cf6a`：728/728 PASS、2 slow、13 后续阶段 ignore，退出 0；check/clippy/fmt 与治理通过。nextest 实际版本 override 的授权边界保持，0.9.145 原环境门禁 not_run。T01 可提交基础候选；M1 完整 SHA 在提交后记录。公开 result、最终方法与真实价值尚未验收。

## C004-M1 最终阶段准备结论

通过。独立 Reviewer `/root/independent_review` 未参与合同、原语、oracle 或实现；最终固定 `0cedb8c1f3ee7ba8d4041151a5382d3b9e8abb4e`。已核总体合同、全部 caller、同快照读取、完整资源真值、解码顺序、纯卡/实时视图、真实行为 red、M1 手册及白名单，无剩余必改。提交后独立 check-task/docs/specs/tests/diff 通过。

T01 完整回归实际 728/728 通过；T02 11、T03 2 仍 ignore。自有临时副本中，仅移除 ignore 控制通过，revision 断言 3→4 准确被冻结门禁拒绝；见 [冻结守卫](evidence/20261003-M1-freeze-guard.txt)。第一次临时 fixture 缺 completed 目录只作 [环境诊断](evidence/20261003-M1-freeze-invalid-fixture.txt)，不作反例或行为红。真实候选从未修改。

本结论仅授权进入已采用的 T02/T03；nextest 0.9.145 原环境门禁按用户授权 not_run，实际 0.9.140 override 范围已执行。公开功能、真实用户质量/收益和 M2 尚未验收。

## C004-T05 独立测试期望修复

通过。Reviewer `/root/independent_review` 未编写该期望。首次接口 draft 11 实际为 10 PASS/1 FAIL，fixture 别名错误，生产规范根正确。作者仅在 expected Work 目录上添加 std canonicalize/unwrap，再拼固定子路径；不改摘要、大小、来源、只读与业务断言，不从结果生成期望。单独提交保留 T02 ignore，公开接线仍归 T02。最新完整测试修复 SHA 在提交后固定。

## C004-T02 公开接线独立结论

通过。Reviewer `/root/independent_review` 核 result/status 的 typed 数据与 next 同一上下文、文本/JSON 同源、request-id 在 Store 前拒绝、只读不进入维护、无文件/Store 回查或 Null fallback；独立关键 CLI 4/4 PASS（run `930dd78e-1cb8-4452-8a01-19b85886f8f0`），见 [原文](evidence/20261003-T02-independent-cli.txt)。版本/Unreleased 与冻结测试仅删除 11 ignore 符合范围。首次手册仍指旧 M1，已统一最新修订基准 `2914e8564047809659c4610d151f9ea6ceef7476`，Reviewer 增量通过。

完整适用门禁实际 739/739 PASS、2 T03 ignore；nextest 标记一条旧初始化 SIGKILL 测试为 LEAK。只读诊断与一次独立局部执行未确认代码缺陷，原因 unknown；不通过重复直到变绿、加宽等待或配置隐藏它。原 suite 的 LEAK 事实及历史 PID/FD 缺项保留；它不被局部无复现覆盖。T02 功能与范围通过，未知历史泄漏列为限制，M2 再核整条链与证据边界。

## C004-T03 方法与协调者说明

通过。Reviewer `/root/independent_review` 未编写方法或 oracle。独立真实 CLI 返工/终点绑定与摘要不能跳审查 2/2 PASS（run `bd4d326f-7c31-49e2-a107-3f4e59653c60`）；见 [原文](evidence/20261003-T03-independent-method.txt)。方法图/上限/断言保持基准2914，测试仅删除2 ignore。首次 reader 发现 skill 对已有 Work 的入口路由不清、裸命令与 JSON 铁律冲突，作者已加新建/接续/只读分支并统一示例，Reviewer 增量通过。

首次使用、复用、专用 Home、历史/当前、草稿/冻结、终点绑定者与生产者、默认无gate、外部权限和质量边界准确。受控 trace 18 次 CLI、3 个独立读取文件真实，限定为机制；真人首次使用和成本/质量收益仍 not_run。

## C004-T04 真实试用前提与延期记录

记录通过。独立 Reviewer `/root/independent_review` 核授权未制造实际用户行为，actor/配对材料/盲审接受/usage缺项与机制证据分离，真实试用和净收益未记PASS。初稿误把未建立会话规程写成工具无能力，作者已收窄为实际缺少原宿主标识、保存/关闭/重开规程与接续观察；不必尝试关闭当前持续执行会话。补验标准、预算与停止规则可用。

本轮Task完成仅覆盖前提盘点与交接，原真实增量义务not_run并按用户环境跳过授权延期，不能从T03演练、M1/M2代码审查或预先批准推断真实接受/质量收益。详见 [前提记录](experiments/readiness.md)。

## C004-M2 最终完整实现结论

实现闭包通过，按用户已授权例外限定。候选 `aa26d2c4f4396302b4b4f00394ada57a7b0e84b8`。Reviewer引用前阶段具体候选/run并补组合来源、INV、严格完整装入、终点/gate、state/card/实时/历史、只读全链、method/skill路由与Rust工程；无新增生产必改。

Reviewer独立核完整741/741无skip及check/clippy/fmt/governance；13selected mutation的raw76files每BuildSuccess/TestFailure101及非零失败断言，baseline32/8/11，未执行其余105；原result0-mutant baseline失败保留。deny政策只变temp db-path，缓存117edb3快照检查通过，在线未完成清楚。生产/测试/fixture/config未漂移。

真实trial/用户收益和nextest原0.9.145授权not_run；在线fresh更新执行后终止未完成；历史LEAK causeunknown保留。本轮按实现范围归档，原真实义务与环境缺项留同一package等待后期补验；不是完整用户价值验收、发布或全安全结论。阶段M1通过从未用来替代M2。

收口独立复核通过：Candidate与实现限定PASS表一致，所有授权延期/未完成/unknown项独立完整保留，未把not_run改PASS；日志原字节保全后可按已有授权归档并采用C005。生产闭包未变，未发布。
