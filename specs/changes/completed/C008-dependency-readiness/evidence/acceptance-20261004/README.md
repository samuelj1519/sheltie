# 剩余验收整理与本次自动工作

用户要求整理所有剩余工作、执行agent可以完成的内容并引导真人操作。本次沿用macOS aarch64范围；没有真实样本、参与者活动、撤销事件或必需宿主资源时不构造替代需求。全量交接见 [剩余验收清单](../../remaining-acceptance.md)。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 当前方案和剩余义务索引 | executed | C001–C010 plan/validation及v0.2.0发布记录，100a6f8当前源码 | 实际逐package核对，区分当前采用义务、真人输入、条件任务与历史限制 | 完成整理，不把历史缺项转PASS | [完整索引](../../remaining-acceptance.md) |
| 当前真实依赖前检 | executed | 同原7份物理副本的manifest/Flow字节，版本按当前源读取 | 独立TOML读取/计数/SHA | 7副本/7Flow/27Node/0requires；probe not_adopted，host ready unknown | [前检原件](current-preflight.json) |
| 当前原生资格与MSRV补验 | executed | 旧378项/369不变的源资格；新206项执行清单与工具/消费者补核 | 新1.85run ed11bd1a-42eb-4097-bb4d-c7bddd9ae864；另doctest | PASS：948/948、0skip、无LEAK；5doctest | [对应C010记录](../../../C010-local-simplification/evidence/acceptance-20261004/README.md) |
| 真人试用技术环境和模板 | executed | 新默认特性Cargo产物、固定方法原字节、独立新Home | Cargo JSON及7条实际CLI；无work start | 技术准备完成；正式准入/六run仍未开始 | [对应C007记录](../../../C007-pre-run-workbook-generation/evidence/human-acceptance-20261004/README.md) |
| 文档链接、规格路由与测试治理 | executed | 本次实际Markdown/JSON/TOML/脚本；记录中的206项只是编译输入子集，不能当文档完整闭包 | docs/specs/tests，各exit0；最终文档输入与原件另存final-* | PASS，限实际治理检查 | [首次docs](docs.json)、[specs](specs.json)、[tests](tests.json) |

真实任务/参与者/活动成本、宿主关闭重开、真人接受、实际撤销事件和C008采用前提仍未提供；异步问题已交任务提供者。模板中的空值保持未填，没有正式run ID、额外Work、虚构费用或提前开始计时。

现有原生工程和历史实验只按原候选及原run ID引用。没有重新在线刷新advisory，不声称缓存依赖检查为当前最新公告；没有发布v0.3.0、push/merge、替换安装或修改宿主配置。E01/E02真实载体缺失与旧取证/因果不能靠新测试追溯补造。
