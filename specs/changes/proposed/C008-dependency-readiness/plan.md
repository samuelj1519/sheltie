# C008 实验计划

状态：`proposed`；未采用，不执行任务。全部 `todo`，探针和真实任务 `not_run`。共用规则见[提案实施指南](../../../guides/proposal-implementation.md)，工程与提交规范见[engineering.md](../../../engineering.md)。

## 1. 实施方式与任务表

强模型先确定实验架构、输入输出和关键验收 oracle，再搭建本阶段骨架。简单模型按固定边界实现或执行，不猜宿主规则、不改期望、不扩大样本。每个里程碑由未参与本阶段设计和实现的强模型 review。M2 是实际实验阶段，骨架是冻结输入、人工步骤和证据表，测试是独立写定的验收 oracle，不为这个阶段再生成一套程序。

当前 package 不改 core、runtime、CLI、Workbook 合同或宿主配置。拟新增实验路径全部由 T01 采用后创建。Python 测试不受 `scripts/task.sh` 的 Rust 归属过滤；没有 Rust 用例的任务不强行运行 `task.sh`。填空任务 T02 显式使用 T01 骨架 commit 作为 `check-task.sh` 基准。

| ID | 状态 | Owner | 依赖 | 交付 |
| --- | --- | --- | --- | --- |
| C008-T01 | todo | 强模型 / 架构与 M1 骨架 Owner | 采用授权与真实必需资源 | 单宿主边界、输入输出、探针骨架与机制 oracle |
| C008-T02 | todo | 简单模型 / 探针实现 Owner | T01 | 只读外部探针与机制测试结果 |
| C008-M1 | todo | 独立强模型 | T02 | 单宿主规则、无写入与 unknown 审查 |
| C008-T03 | todo | 强模型 / M2 实验设计 Owner | M1 通过 | 真实对照、证据骨架和判据冻结 |
| C008-T04 | todo | 简单模型 / 实验执行者 | T03 | 真实任务证据、成本与收益记录 |
| C008-M2 | todo | 独立强模型 | T04 | 机制/价值分开结论与后续建议 |

里程碑的 review 文档提交用审查开始候选作为 `check-task.sh` 显式基准。里程碑发现错误先回对应任务 Owner，复核后才关闭；review 不代写实现。任务状态只在本表更新。

## 2. M1：有依据地进行一次只读检查

### C008-T01：总体边界、单宿主骨架与机制 oracle

**Owner / 依赖。** 强模型；人采用只读实验，已有真实 Workbook/Node 的 requires 和作者确认的必需资源。没有这些输入时先记录停止理由，不写脚本。

**第一次阅读。** `CONTEXT.md` 的 resource/host resource 区分、`specs/contracts/workbook.md` 的 requires 字段与 Node 对应规则、`crates/sheltie-core/src/workbook/manifest.rs::HostRequire/parse_manifest`、`flow/parse.rs`、`work/render.rs::render_brief`。只读盘点 examples 和 workbooks 的显式声明，不从说明书自动推断。

**输入 → 输出。** 输入为真实显式声明、一个实际宿主及固定版本、允许只读观察的位置；输出为拟新增 `experiment/{probe.py,test_probe.py,probe.toml,README.md,fixtures/}`、本次精确规则与测试期望。所有文件都在 adopted package 目录内，无新 crate 或产品写路径。

**步骤。**

1. 固定作者已按正常流程确认合法的 Workbook/Node、宿主版本、工作区、需检查的真实资源和其声明。记录为什么任务离不开此资源。
2. 查固定版本官方规则或用无副作用信息命令观察解析行为；记录来源与日期。决定哪些项能够确认，其他项直接 unknown。
3. 明确 source/version/digest 的匹配语义和摘要闭包。缺少解释依据时不创造算法；把对应 oracle 定为 unknown。
4. 固定 probe.toml 的允许字段、路径范围和 JSON 输出。输入未知字段拒绝；枚举/读取不能越出明确允许范围，source URL 不执行。
5. 创建只读探针骨架、临时 fixtures 和 test_probe.py。未实现边界抛带 `C008-T02` 的 NotImplementedError，不返回假成功。未来阶段不预写产品准入测试。
6. 用手写 JSON、独立摘要和文件前后快照固定正反例；写启动命令与 Python 版本，冻结实际非零用例清单与数量。

**验收用例。** 拟新增 Python 用例：`test_declared_resource_matches_observed_identity`、`test_missing_resource_reports_mismatch`、`test_unobserved_search_location_keeps_presence_unknown`、`test_unconfirmed_host_precedence_stays_unknown`、`test_digest_does_not_cover_scripts_reports_unknown`、`test_source_is_data_and_never_executed`、`test_probe_does_not_write_host_work_or_store`、`test_unknown_config_field_is_rejected`。明确声明一对合法/拒绝例；fixture 注入只发生临时目录。Python 归属在测试文件注明 T02，阶段提交后登记实际名字。

**停止 / 验证 / 交接。** 无真实必需资源、不确定宿主选择规则或没有无副作用观察途径时保留 unknown/停止，不偷偷加入更多宿主。运行语法检查、独立 oracle 和新行为预期失败，编译/导入错误不算正确红。交接完整配置、宿主规则证据、固定测试、T01 commit 与 T02 白名单。

### C008-T02：实现小型外部探针

**Owner / 依赖。** 简单模型；T01 已固定配置、接口、机制 oracle 与骨架 commit。

**入口 / 范围。** 只填 `experiment/probe.py` 的 T02 占位；test_probe.py 和 fixtures 不改。读取 probe.toml，并按获准范围读取真实 Workbook 与宿主元数据；不调用 Sheltie 写命令。

**输入 → 输出。** 明确 requires 与单宿主只读资料进入固定函数，输出 JSON 中逐项 matches/mismatch/unknown 和证据。不能用整体 true 代替未知字段。

**步骤。** 严格读取配置与选择 Node 声明；按已冻结规则枚举/解释宿主；比较有依据的身份字段；输出完整观察。用本任务 fixtures 检查结果与无写入快照。不会解释自然语言 instruction 或自动执行 skill。

**验收用例。** T01 建立的 test_probe.py。骨架提交后卡中登记实际存在的测试名；缺失与同名遮蔽保持分开，版本/摘要不明必须 unknown。不得修改 fixture 把未知改为 matches。

**停止 / 验证 / 交接。** 遇到契约之外的宿主输出、版本或内容闭包，返回 T01 Owner；不写兼容适配层。运行已固定 Python 版本的 `python3 -B -m unittest discover -s specs/changes/active/C008-dependency-readiness/experiment -p 'test_*.py'`、文档门禁、工程规定的提交检查和 `scripts/check-task.sh C008-T02 <T01骨架commit> --staged`。测试发现数必须与冻结非零清单一致，零用例即失败；交接实际测试输出、围绕probe调用的no-write证据和所有unknown，正常Work推进的合法Store写入单列；一个任务一个提交。

### C008-M1：独立强模型 review

固定脚本候选、Python 版本、宿主版本、声明与 fixtures。检查 source 不执行、token 不读取、unknown 不改写、无 Store/宿主/工作区写入。按原始宿主依据重新核解析顺序和摘要闭包；fixture 结果只证明机制。接口或 oracle 有缺口交回 T01，代码问题交回 T02。findings 与证据写 review/validation，未解决阻断项不进 M2。

## 3. M2：判断这个工具是否值得保留

### C008-T03：真实任务、对照与证据骨架

**Owner / 依赖。** 强模型；M1 通过，有真实必需资源的任务仍然存在。

**入口。** spec §1/4、design 的对照边界、validation 的真实价值指标、已通过的 probe.py 和真实 Workbook。没有真实任务时不把 fixtures 升格为样本。

**输入 → 输出。** 输入为实际任务、显式 requires 与宿主环境；输出为冻结的 probe.toml、experiment/README 操作步骤、evidence 输入闭包表和人工验收 oracle。此任务只准备真实实验，不生成第二个执行系统。

**步骤。** 冻结人工核对与脚本核对共用的声明和环境；记录顺序、熟悉度及任务标准；规定如何计时间、误报、未知与执行中失败。预先定义成功/停止条件，并对原始证据字段填一个手写示例。配对任务不足时注明个案限制，不改标准或人为增加宿主依赖。

**验收用例。** 正例是实际任务确需资源，双方读取相同声明，脚本提供可复核观察；反例是只有 resource.* 或没有 requires，应停止，不进入真实样本。明确缺失时可以停止任务，不能强行失败来凑「避免失败」指标。

**停止 / 验证 / 交接。** 无真实任务保留价值 not_run，交人决定停止方向。核对输入摘要、记录字段和完整可执行步骤，运行 probe 机制回归与文档门禁；交接 T03 commit、冻结数据口径、原命令和 T04 允许操作。无 Rust 用例，不运行 task.sh C008-T03。

### C008-T04：执行真实对照并整理原始证据

**Owner / 依赖。** 简单模型或人工执行者；T03 配置、步骤、判据已经冻结。

**入口 / 范围。** experiment/README 的固定命令、probe.toml 与 package/evidence；只保存观察和记录，不改 probe.py、测试、宿主配置、requires 或任务标准。

**输入 → 输出。** 固定环境与实际任务 → 人工核对记录、脚本 JSON、时间、unknown/误报、执行结果和成本。每个 raw run ID 关联输入闭包。

**步骤。** 按固定顺序做人工与脚本核对；脚本 stdout 保存至证据目录；协调者依据报告决定正常推进，过程遵守现行 Work/Attempt 合同。记录检查后发生的变化，不回写第一次观察。运行时失败作为原执行事实，来源不明时不归因于宿主依赖。

**验收用例。** T03 固定的实际样本与人工 oracle。正例包括成功识别真实声明；反例包括无从确认的字段保持 unknown、结果没有写 Store。机制 fixtures 与自然失败分别计数。

**停止 / 验证 / 交接。** 不匹配或工具输出超出预期就停本条样本，返回 T01/T02；不改配置重跑到绿。缺样本保留 not_run。核对证据完整、敏感值不进入日志、仅probe区间的no-write快照（实际Work推进写入单列）与实际实验命令结果；运行文档门禁、工程规定提交检查与 `scripts/check-task.sh C008-T04 <T03证据骨架commit> --staged`。无 Rust 用例，不运行 task.sh。

### C008-M2：独立强模型 review 与产品建议

从原始声明、宿主解析依据、命令输出到用户实际决策核整链。分别给机制结论和真实价值结论，列净成本、误报、未知和样本限制。只允许提出「保持外部」「改善任务说明」「停止」或证据充分后的新采用建议；不因 probe PASS 自动修改核心合同。人决定是否产品化，M2 不实施安装、TTL、执行绑定或准入。
