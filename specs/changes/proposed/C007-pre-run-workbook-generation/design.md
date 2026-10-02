# C007 设计：一份方法、六次完整运行

状态：`proposed`；资产和运行 `not_run`。下面的路径与字段是实验记录约定，不是产品格式。

## 1. 最小资产

```text
experiments/
  protocol.md                    组别、使用者历史、顺序、预算、停止、质量判据
  confirmed-input/               一份当前格式的固定代码 Workbook
  shared-method.md               两组共用的完整方法说明
  first-use.md                   首次读者操作入口
  samples/<sample>/              冻结目标、标准、初始仓库标识和检查命令
  oracle/                        质量审阅标准；不向执行者提供隐藏答案
  preparation/                   公共方法与各组配置的实际投入记录
  runs/<sample>/<arm>/            原始命令、活动记录、最终交付和盲审
  report.md                      方向性结论、总成本及下一最小范围
```

T01 的复杂模型或有经验作者完整制作全部准备资产、必要脚本与独立测试，M1 核可执行性后才进入真实运行。T02 只按 first-use.md 保存 runs/，不改方法、协议、oracle、脚本或测试；T03 复杂模型分析，M2 独立核结果。三份任务输入与六个运行已经足够。不默认制作第二种方法、authoring checker、统一执行器、数据平台或脚本骨架。只有人工记录确有重复且脚本比记录更省事时，T01 作者才制作完整的 `scripts/collect_run.py` 和必要测试；它只调用现有入口，不解析 Flow 算法、不复刻摘要、不读取或修改 Store。

## 2. 固定代码方法

方法从现有 `workbooks/spec-dev/` 的检查与有限返工思想收窄，不照搬其全部规划流程。

| 节点 | 输入与动作 | 输出 |
| --- | --- | --- |
| implement | 本次任务、项目位置与约束；实际修改代码并执行既有本地检查；重访时读取可选审查反馈 | `change.md`、`checks.md`；代码保留在每组独立仓库 |
| review | 相同任务标准、实现说明和检查记录；审查真实候选，决定返工或交付 | `review.md`，含明确缺陷和依据 |
| deliver | 同一候选、检查记录与审查；整理实际 patch 和最终使用说明 | `delivery.md`、`change.patch`；默认 `gate = false` |

Flow 入口为 implement；边为 implement→review 的 `main`、review→implement 的 `back`、review→deliver 的 `main`。implement/review 的 `max_visits = 3`，deliver 的 `max_visits = 1`；implement 的 `max_retries = 1`。这是本次小任务方法的有界策略，不作为所有任务的产品默认。超过上限停下并记录，不放宽方法取得成功。

本次输入固定为 `task` 与 `project`：`start.task` 承载任务目标与验收标准，`start.project` 承载项目位置、初始候选、约束与现有检查命令。A 组获得完全相同的 task/project 内容和方法文字，B 组通过当前起始输入冻结。公共检查说明可放 `resources/` 并用 `resource.*` 引用；只有实际宿主机制必需时才声明 `requires`。每个输出的路径与 `max_bytes` 在 T01 根据交付对象固定，不给无限上限。

A 组获得上述全部方法内容、返工策略和标准，用宿主已有办法保存阶段、审查和交付。B 组把相同内容编码成当前 Workbook。只有真实不可替代的授权边界才在 T01 固定 gate，并为 A 组固定等价人工动作，不能因实验观察就插入门槛。最终用户接受与实验独立盲审在流程之外分别执行、记录和计时；日常 review 不代替实验最终盲审，gate 不代替接受或质量。不能把方法本身的质量优势算成引擎收益。

## 3. 当前 CLI 的实际路径

T01 固定真实 binary 的版本、绝对路径及字节标识。可使用已存在的发布二进制；需要构建时用隔离 `CARGO_TARGET_DIR` 和 `RUSTC_WRAPPER=`，从 `cargo build -p sheltie-cli --message-format=json` 的 `executable` 取路径，不猜 target 目录。

1. 方法作者把完整方法复制到一个固定 `confirmed-input/`，暂停编辑。首次读者和质量审阅者直接读取这个目录的 `workbook.toml`、Flow、说明和资源。`workbook show` 只展示 manifest、节点/边、起始输入等摘要，不能代替完整材料审阅。
2. 为实验创建显式的专用 Home。运行 `<binary> --json --home <experiment-home> workbook add <confirmed-input>`，保存原始 stdout/stderr/退出码及 `data.id/version/digest`。再用同一 Home 执行 `workbook show <id>@<version>`、`workbook verify <id>@<version>`。结构拒绝即停止准备，不继续运行。
3. 每个 B 组任务运行 `work start --workbook <id>@<version> --flow <flow> --input task=@<task-file> --input project=@<project-file>`。全局 `--json --home <experiment-home>` 始终显式传入，保存响应的 `work_id/work_dir/workbook.digest`。
4. 按当前响应的 `next` 调用 `attempt begin <work_id> --node <node>`。使用返回的 `brief_path`、`inputs`、`outputs`、`output_dir`；把声明输出写到这些真实路径。完成用 `attempt submit`，执行失败用 `attempt fail`。审查内容由协调者判断选边；引擎只限定合法操作。
5. 重开会话后先读 `work status <work_id>`、`next` 和该 run 保存的 begin 响应，按其真实路径找任务书、输入与输出；必要时读 Work 目录中的当前格式文件。记录每一次查找和核对，不能假定存在 C004 handoff 接口。
6. 默认 deliver 提交后流程结束。只有 T01 已固定真实授权门槛时，用户授权后才调用 `gate approve <work_id> --node deliver`，A 组执行等价人工动作。保存完整最终交付及外部检查原始输出，分别记录流程之外的用户接受和独立盲审。未接受如实记录，不由 agent 或 gate 推断接受。

命令块中的值由 T01 替换为实际路径、ID 和上限并写入 first-use.md。手册逐步列命令、参数文件、stdout/退出码、产物位置、唯一证据路径和停止交接；T01 先用真实 CLI 核合法与拒绝 oracle，M1 独立走查。不让第一次使用者猜接口或从提案推断能力。所有正常 Work 写入发生在专用 Home；任务仓库修改与实验记录由实验 Owner 在用户授权范围内完成，不归为引擎的宿主写入。

## 4. 记录格式

每个 run 是一次正式求解，`run.json` 保存以下字段。字段名固定，T01 可以补充说明，不在结果出现后删减成本项。

| 字段 | 类型与含义 |
| --- | --- |
| `run_id` / `sample_id` / `arm` | 非空字符串；arm 为 `native` 或 `sheltie` |
| `actor_id` / `assistant_ids` | 本次任务使用者的稳定匿名 ID、实际参与助手 ID 数组；换人须用新 ID |
| `prior_uses` / `actor_arm_use_index` | 开始前该 actor 使用本方法的次数，记录 `{native: n, sheltie: n}`；本组实际序号为 `prior_uses[arm] + 1`，含正式实验之外的实际使用 |
| `protocol_id` / `order_index` / `exposure` | 冻结协议标识、全体运行正整数顺序；本组 prior_uses 为 0 时 first_use，大于 0 时 reuse，不由 sample_id 决定 |
| `input_refs` | 目标、标准、初始仓库、完整方法、模型/宿主/环境及 binary 标识的证据引用 |
| `started_at` / `ended_at` / `wall_seconds` | UTC 时间和实际墙钟；未结束为 `null` |
| `activity` | 人工活动事件数组，每项为 actor_id、phase、cost_bucket、start/end、minutes、evidence；phase 为 preparation、task_input、execution、verification、rework、resume、delivery、acceptance、final_review 或 maintenance；cost_bucket 为 shared_method、arm_setup、arm_maintenance、run 或 experiment_overhead |
| `interruptions` / `rework` | 实际中断与返工证据数组；返工标 `natural` 或 `injected`，原因未知明确写 unknown |
| `usage` | 宿主原始 input/output/cache token 与 cost 的可观察值和来源；未知值为 `null` |
| `delivery_refs` / `acceptance_ref` / `quality_ref` | 最终 patch、候选、说明、检查原始输出、流程外用户接受和独立审阅记录引用 |
| `outcome` / `stop_reason` | `completed`、`failed`、`stopped` 或 `not_run`；停止理由或 `null` |

T01 在 protocol.md 固定 actor × arm 的初始使用历史和三项任务分配，至少同一使用者分别完成两组各三项任务。每次开始前更新实际 prior_uses；换人、助手代办、同方法练习、跨组熟悉度与顺序偏差单列，不能据样本序号声称复用。助手的活动用其自己的 actor_id 记录，代办关键步骤的影响不能藏在主操作者的成功记录中。

人工活动事件区间不得重叠重复计同一人的时间；所有参与者、作者与助手活动按 cost_bucket 唯一归类再求和。模型或工具等待不算人工活动，原始模型耗时、墙钟与 usage 单列。`work stats` 不替代人工计时。公共方法准备、每组专用配置/维护、每次 run 及实验测量/分析开销分开保存；完整公式见 [验证 §3](validation.md#3-分析口径)，不能同时把同一准备活动计入公共成本和 run 成本。

盲审使用中性编号的最终 patch、候选代码、任务标准和必要检查材料，移除仅提示组别的路径标签；原件保留在 run 中，编号映射仅给分析者。不能为遮蔽而改动代码或检查内容；无法完全盲审时记录原因和可能偏差。
