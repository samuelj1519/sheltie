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
