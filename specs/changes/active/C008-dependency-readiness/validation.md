# C008 验证：机制与真实净成本

Candidate: `none`

状态：`active`；采用决定尚无真实对象，探针与独立里程碑均为 `not_run`。

## 1. 开工条件与冻结

没有真实声明、作者确认、重复摩擦、固定宿主规则和预算时，采用结论是 not_adopted，机制与价值不执行。不能为满足计划而添加 requires、扫描所有已装资源或造一项依赖。

T01 固定 Workbook/Flow/Node 及已校验证据、原始资源声明、宿主实际版本、允许读取范围、完整/部分搜索依据、选择与身份规则、Python 版本、人工对照、顺序、预算与停止。未知 version/digest/source 的解释必须在开发前明确，不以脚本当前能算什么倒推合同。

预算覆盖规则调查、脚本制作、必要测试、真实观察、unknown 复核、误报和维护。任务提供者预先给活动分钟/费用上限、近期预期复用次数和有意义的改善；值未填不开发。超预算或真实对象消失时停止并保留已有成本。

T01 复杂模型完整制作 probe、手写 fixtures/tests、runbook 并完成临时机制验证，M1 核可执行性/权限和首次读者步骤，不宣称真实资源当下可用。T02 按冻结手册做正式真实对照，不修改资产；T03 分析，M2 核真实范围与净成本。同闭包的短审与 M 审引用原 run ID，不重复跑无关机制测试。

## 2. 机制 oracle

| 规则 | 正例 | 反例 / 无法确认 | 预期 |
| --- | --- | --- | --- |
| 真实明确声明 | Node 引用 manifest 同一真实必需 kind:name | 只有自然语言或 resource.* | 不进入实验，不推断依赖 |
| 存在与选择 | 完整规则证明选中目标 | 同名候选存在但优先级不明 | presence 可 matches，selection unknown |
| 缺失范围 | 完整搜索闭包均获准且无目标 | 仅在部分获准位置未发现 | 前者 mismatch，后者 unknown |
| 版本 | 声明解释和实际来源可靠且相符 | 无版本元数据或范围解释 | unknown，不用字符串猜范围 |
| 摘要 | 明确完整闭包的已知字节摘要相符 | 脚本/关联内容未覆盖或读取漂移 | unknown；同一已定义闭包不符才 mismatch |
| 来源 | 明确来源标识按冻结规则相符 | 只有目录或显示名 | unknown |
| source 数据 | 仅读取声明与元数据 | source 有程序或远程地址 | 不执行、不下载、不安装 |
| 只读 | fixture 调用前后字节/目录无变化 | probe 区间发生写入 | 机制失败并停止；有限快照只证范围内 |
| 非法配置 | 参数定位明确，退出 2 | 未知字段或目标不存在 | 拒绝，不输出有效观察 |
| 有效未知 | JSON 明确保留缺失依据，退出 0 | 把退出 0 写成资源可用 | 拒绝业务结论 |

临时夹具可以人为缺失、遮蔽或改变内容，只核机制。真实宿主不能被人为删除、移动或安装以制造收益。no-write 核 probe 调用；外部保存证据、正常 Work 推进或用户修改任务仓库均另列。

## 3. 真实观察与价值

人工核对与脚本核对使用同一声明、宿主环境、搜索资料和范围，记录实际顺序及熟悉效应。声明或宿主改变后的观察单列；不覆盖第一次结果，不主张持续有效。

| 指标 | 口径 |
| --- | --- |
| 必需性与重复性 | 实际任务为什么离不开资源，已有人工核对事件与频率 |
| 准备与维护 | 规则调查、脚本、测试和版本变化适配的实际活动时间 |
| 人工核对 | 同样范围内核声明所用活动时间及前次观察影响 |
| 脚本核对 | 运行、解释 unknown、复核和错误提醒处理的全部时间 |
| 实际发现 | 有完整证据的明确缺失或不符，不把 unknown 算准确识别 |
| 运行后问题 | 原记录可证来自依赖的问题，来源不明单列 |
| 可观察覆盖 | presence、selection 及实际声明身份字段各自的结果与范围 |
| 净成本 | 包含准备维护的累计成本及近期复用回收估计，未知不当零 |

明确缺失时不强制运行制造失败。「提前发现」可以报告，「避免了几次失败或多少损失」没有真实对照时保持无法估算。一个匹配样本、多个 fixture 或一个完整任务成功都不能证明长期净收益。

允许结论：保持外部小工具、改善声明/说明、停止、或针对一个无法由外部工具承担的明确义务另提范围。C007 样本发现摩擦只是触发材料，不是 C008 独立收益 PASS。

## 4. 执行记录

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 真实必需资源与重复摩擦 | not_run | 待实际声明与事件 | 未执行 | not_run | 无 |
| 宿主规则与授权范围 | not_run | 待单宿主固定版本 | 未执行 | not_run | 无 |
| 单目标探针与机制 fixtures | not_run | 待 T01 固定配置 | 未执行 | not_run | 无 |
| probe 调用 no-write | not_run | 待脚本与可观察范围 | 未执行 | not_run | 无 |
| 人工 / 脚本真实核对 | not_run | 待重复真实事件与顺序 | 未执行 | not_run | 无 |
| 累计净成本与保留建议 | not_run | 待全部准备、观察与维护原件 | 未执行 | not_run | 无 |
| C008-M1 准备与机制 review | not_run | 待 T01 脚本/规则/fixture/runbook 闭包 | 未执行 | not_run | 无 |
| C008-M2 真实价值 review | not_run | 待固定分析与真实原件闭包 | 未执行 | not_run | 无 |

保存实际命令、raw run ID、stdout/stderr/退出码、配置/脚本/fixture 标识及宿主环境。未知、中断、未采用或无样本不能改写成 PASS；复用必须保留同输入闭包和原 run ID。静态文档检查不进入上表机制或价值结果。

本轮采用前提盘点/授权延期交接，原probe未采用与全部机制/真实义务not_run。C007十二份preparation/evidence原文保真；不修改原正式数据门槛。

## C008-T01 前提证据与原规则边界

基准 `fd11f96d98eb987a89546f81fed9dc76c509b65d`。只读[evidence/target](experiment/evidence/target/README.md)盘点6manifest/6Flow/24Node：manifest/node requires均0，resource.*15绑定/13文件是冻结参考材料。29当前声明/Flow/resource/私有方法源文件SHA读后复核一致；两个历史/tmp清单原字节保留、旧path/HEAD明确历史。没有扫描host/Store/env或新增依赖。

[preflight](experiment/evidence/target/preflight.json)实际kind:name/作者必需确认、host/version/规则/roots、重复摩擦、预算/benefit/cost均null；probe not_adopted，原8机制/实际观察/value义务not_run。不把0声明解释成host资源不存在或mismatch，也不以resource.*推requires。

[protocol](experiment/protocol.md)/[host-rule](experiment/host-rule.md)/[runbook](experiment/runbook.md)保原完整采用/机制/三态/身份/no-write/真实对照条件；真实目标缺失不创建probe/config/fixture/test或运行占位命令。技术盘点时间不当人工实验投入；原准备/维护/观察成本unknown，费用/usage不估算。无Rust/机制测试实体，test_files为空，不跑零task.sh或以import错误当拒绝。

C008-M1本轮前检限定PASS，candidate77f09112完整SHA；原probe未采用与机制准入not_run，不将未来三态规则当实际观察。无输入变化，无scanner/引擎重跑。

## C008-T02 原义务的授权未执行记录

开工49b48799c7180cbf1ececbd5aba588344d9ab351。[8项延期](experiment/evidence/real/README.md)逐项记target必要性/摩擦、host规则、probe/config/fixture、no-write测试、原机制M1、实际人工/script、全成本/复用及原M2 not_run。没有实际观测、matches/mismatch、ready、正式run ID或活动数据；人工/费用/净改善null。不以0decl证明host不存在，不制造host缺失、不以import失败当机制拒绝。声明与规则原件保持，无产品/host变化。
