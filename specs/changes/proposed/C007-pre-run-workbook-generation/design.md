# C007 候选设计

状态：`proposed`，实现 `not_run`。基准：`186ebd75d744fc2db83a72b715d36d05dd923883`。本文件是候选机制，不是当前架构；ADR 草案在采用时移入 `specs/decisions/` 并编号。

## 1. 分层

| 部分 | 知道什么 | 不知道什么 | 位置 |
| --- | --- | --- | --- |
| 规划者 | 任务目标、约束快照、编写规范 | 约束快照怎样生成；检查结果之外的授权状态 | 宿主 agent + authoring skill |
| 检查器 | 图结构、约束条目、约束映射 | 节点说明书写得好不好；被映射的节点是否真的在做审查 | 引擎之外的独立组件 |
| 引擎 | 与现在相同：装入校验、冻结、运行 | Workbook 是不是生成的 | 不变 |

引擎不新增任何概念。生成的 Workbook 装入后与手写的没有区别（WG-08）。

## 2. 约束快照

第一阶段用一份 TOML 表示，示意如下（字段名在采用时由 contracts 决定）：

```toml
schema = "constraints/v0"         # 实验格式，不是合同
version = "3"

[[require]]                       # 来自 C004 验收合同的一项条件
id = "tests"
kind = "machine_check"            # 闭集：machine_check | review | gate
after_candidate = true

[[require]]
id = "merge-approval"
kind = "gate"
after_candidate = true

[limits]
max_visits_per_node = 5
max_retries_per_node = 2
max_attempts_total = 40           # 结构上界，不是 token 或费用预算

[execution]                       # 引用 C005 的执行绑定版本，不在这里重写
binding = "binding-v2"
```

`kind` 只是检查器使用的标签，不进入引擎（GF-01）。项目默认要求和本次差异合成这份快照时，每一条都保留来源（项目默认 / 本次差异 / 执行政策）。

## 3. 检查器算法

所有规则都是有向图上的机械判定。设 Flow 编译后的图为 \(G\)，入口 \(s\)，终点集合 \(T\)（采用 C004 时为接收点），约束 \(c\) 映射的节点集合为 \(N_c\)，产生新候选的节点集合为 \(P\)。边包括全部四种类型（`main | back | branch | re_review`）。

| 规则 | 判定 |
| --- | --- |
| 路径覆盖 | 从 \(G\) 中删去 \(N_c\) 后，\(s\) 不能到达 \(T\) 中任何节点 |
| 候选之后 | 从 \(G\) 中删去 \(N_c\) 后，\(P\) 中任何节点都不能到达 \(T\) |
| 门槛存在 | `kind = "gate"` 的条件，\(N_c\) 中每个节点声明 `gate = true`，且满足上面两条 |
| 上限有界 | 每个节点 \(n\) 满足 `max_visits(n) ≤` 约束上限；且 \(\sum_n \text{max\_visits}(n) \times (1 + \text{max\_retries}(n)) \le\) `max_attempts_total` |

“候选之后”覆盖返工回环：实现节点经 `back` 边被重新到达、产生新候选后，如果存在一条不经过 \(N_c\) 就到终点的路径，判定不通过。

\(P\) 也属于约束映射的一部分，由规划者声明、用户确认。采用 C004 时，候选快照按 Attempt 记录，\(P\) 可以与运行记录对照。

尝试次数上界是纯结构量，可以静态计算；token、费用和墙钟时间的总量不能，列为“无法结构判定”，交给 GF-24 的运行时累计。

## 4. 锁定文件

检查通过并经用户确认后，检查器在草稿目录写入 `resources/plan.lock.toml`（路径待定，见 §7）：

```toml
schema = "plan-lock/v0"
constraints_version = "3"
constraints_digest = "sha256:…"
plan_digest = "sha256:…"          # 草稿目录去掉本文件后的摘要
checker_version = "0.1.0"
result = "pass"
unresolved = ["token-budget", "auth-semantics"]   # 无法结构判定的条目

[mapping]
tests = ["verify"]
merge-approval = ["approve"]
candidate_producers = ["implement"]
```

- 放在 `resources/` 下，是因为 `workbook add` 复制整个目录，Workbook 的目录摘要自然覆盖它；节点也能用 `resource.resources/plan.lock.toml` 把它绑定为输入；
- `plan_digest` 排除锁定文件自身，避免自引用；
- 安装前检查器重新计算 `plan_digest` 和 `constraints_digest`，任一不一致就拒绝继续。

第一阶段中，用户可以跳过检查器直接 `workbook add`，所以这一阶段的约束只是流程约定。是否让 `work start` 要求一份有效的锁定文件，是第二阶段的决定（§7 第 3 问）。

## 5. 第一阶段实验路径

不改产品代码：

1. 用户手工整理约束快照（参照 C004、C005 的候选字段）；
2. 规划者在宿主中按 authoring skill 起草 Workbook 与约束映射；
3. 外部脚本实现 §3 的检查器，输出三种结果；
4. 用户查看摘要、映射、差异和检查结果后确认；脚本写锁定文件；
5. 现有 `sheltie workbook add` 安装，`sheltie work start` 开始。

## 6. ADR-E 草案：检查器与约束快照的位置

状态：`proposed`
日期：2026-09-27
关联 change：C007

### 背景

规划 agent 生成的流程需要对照一份用户授权的约束检查。INV-1 要求引擎不含业务判断，GF-01 要求引擎代码不出现业务词汇；GF-17 规定 `workbook add` 的校验不执行脚本、不调模型、不联网。

### 候选

| 方案 | 做法 | 优点 | 缺点 |
| --- | --- | --- | --- |
| E1 引擎装入校验 | `workbook add` 读取约束快照并执行路径覆盖检查 | 装入即强制 | 装入校验依赖外部约束文件；约束的来源与版本管理进入引擎；修改 GF-17 |
| E2 引擎之外的独立检查器 | 第一阶段是外部脚本，第二阶段作为独立组件随 Sheltie 发布；引擎不变 | 引擎零改动；检查规则可以独立演进；与 C006 导出组件同一模式 | 第一阶段可以被跳过，只是流程约定 |
| E3 只靠 skill 让规划者自检 | 在 authoring skill 里写检查清单 | 最简单 | 规划者既生成计划又判定合规，等于没有检查 |

### 建议

采用 E2。第二阶段如果实验显示“跳过检查器”是真实风险，再评估由 `work start` 核对锁定文件（只核对摘要与签发来源，不在引擎里重跑业务规则）。

### 否决方案

- **E3。** 违背“约束来自规划者之外”。
- **E1 作为首版。** 在还不知道生成流程是否有用之前，就修改装入合同和引擎边界，代价过高。

### 后果

- 检查器需要自己的失败路径测试（validation.md §5）；
- 锁定文件的位置与格式在采用时写入 contracts/workbook.md。

### 确认方式

- 草案中存在绕过必需条件的路径时，检查器判定不通过并指出路径；
- 返工回环后可以不经必需条件到达终点时，判定不通过；
- 约束中有预算类条目时，结果列为“无法结构判定”，不是“通过”；
- 规划者修改锁定文件或约束快照后，安装前的重新计算拒绝继续。

## 7. 待定问题

1. 项目默认要求放在哪里：仓库内文件便于共享，但 agent 能改，需要与上次确认的摘要对比；用户目录更难被 agent 改动，但不随仓库分享。
2. 锁定文件的固定路径与格式；是否需要让 `contracts/workbook.md` 为它保留名字。
3. 第二阶段是否让 `work start` 核对锁定文件；核对什么（摘要、检查器版本、约束版本）。
4. `kind` 闭集是否只需 `machine_check | review | gate` 三种。
5. 编写规范（示例加规则）以 skill 还是 Workbook 的 `resources/` 提供；两者都不能含状态（GF-18）。
6. 用户确认在第二阶段是否像 GF-12 一样由 CLI 以当前 OS 用户身份记录。
