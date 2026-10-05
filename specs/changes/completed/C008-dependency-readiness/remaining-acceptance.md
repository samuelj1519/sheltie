# 剩余验收与当前完成范围

当前 release 为 v0.2.0，开发目标为 v0.3.0；没有 active change。当前产品环境以 [根规格](../../../spec.md#当前产品环境) 为准：已采用的macOS aarch64／APFS本地场景；Linux与Intel macOS保持 `excluded_by_user`。本清单提供义务、证据和执行入口，各 package 的 plan 与原件仍是实施状态来源。

2026-10-05，用户明确采用 C011、翻译和 DSH 为本轮正式对照实验的三项任务，并确认「当前全部完成」「对照试验可以按3项任务完成来继续推进」。当前按这三项实际完成材料进行对照与收尾，不要求另交三项代码任务。三任务属于混合样本，原三代码任务、六组 Native/Sheltie 配对与15%净收益协议保留为历史要求；不存在的过去执行、计时和费用不会由本次采用生成。采用原话、时点和范围见 [当前采用记录](../C007-pre-run-workbook-generation/evidence/acceptance-20261005-engine-readback/current-user-adoption.json)。

## 1. 当前已完成的正式三任务

| 样本 | 明确终点 | 当前材料与实际接受 |
| --- | --- | --- |
| C011 Workbook作者工具 | `2026-10-04-007-workbook-compact-properties` | 工具测试72/72、独立审查、完整patch、真人接受与同Attempt宿主重开；已归档并按任务提交 |
| 英文Markdown简体中文翻译 | `2026-10-04-008-agents-markdown-translation` | 73份译文已同级交付，完整覆盖/语义/命令保真及独立LIVE核验；用户本次报告任务已完成，逐份阅读活动没有单独记录 |
| DSH客户端 | `2026-10-05-001-dsh-desktop-client` | macOS arm64真实App、模型/工具/权限允许与拒绝/取消继续/重开恢复、完整源码和实物、独审、真人实际接受 |

三个终点均 `succeeded`、revision11、`final=true`，15份明确成果的实际字节与摘要已通过公开CLI读回。专用Home共有9个实际Work，6个早期C011迭代按原交接保留历史指针，不能把它们改成成功或冒充Native对照运行；个人方法使用历史不按任务slot或Work总数倒填。见 [公开状态及15份成果读回](../C007-pre-run-workbook-generation/evidence/acceptance-20261005-engine-readback/proof.json) 和 [本轮对照与历史](../C007-pre-run-workbook-generation/evidence/acceptance-20261005/README.md)。

## 2. 当前补验、未来条件与历史限制

| ID / 归属 | 当前实际处置 | 证据与继续条件 |
| --- | --- | --- |
| H01 / C007 | 已完成：用户实际阅读并接受三份指南，接受绑定当前文件SHA | [接受原件](../C007-pre-run-workbook-generation/evidence/acceptance-20261005-engine-readback/guide-human-acceptance.json)；冻结交接卡按其声明的历史时点接受 |
| H02 / C007、C004 | 用户指定的正式混合三任务已完成，按完成材料进行对照；原严格六次代码配对未被执行原件证明 | [三任务记录](../C007-pre-run-workbook-generation/evidence/acceptance-20261005/README.md)；不重做已解决任务来补造过去Native运行 |
| H03 / C004、C007 | 当前C011真正宿主关闭重开、同running Attempt接续已完成；DSH App重开另有实物证据 | [C011接续记录](../C011-workbook-visual-editor/resume-card.md)；原两组配对的真人宿主观察没有新增原件，二者不能互相替代 |
| H04 / C007、C004 | 当前真实独立review/自然back/明确成果、C011及DSH真人接受已核，用户报告三项全部完成 | [当前历史与资格](../C007-pre-run-workbook-generation/evidence/acceptance-20261005/history-and-qualification.md)；流程外盲审和未写方法首次读者身份若未记录，保持未知/未执行 |
| H05 / C004、C006、C007 | 当前完成情况对照可作结论，历史活动、usage、费用与净收益仍未知，不能计算15%改善 | [对照记录](../C007-pre-run-workbook-generation/evidence/acceptance-20261005/current-record.json)；以后取得真实计费/活动原件可追加，不能把未知填0或从Attempt时长推算 |
| H06 / C006 | 已实际完成三份工具导出、15份字节/源状态核验，审计注记与再次导出保留旧编辑，提供持久报告副本供审阅 | [实际报告副本](../C006-result-delivery/evidence/acceptance-20261005/README.md)；操作者为agent，原真人手工同质量对照、人工计时/接受不因此变为已执行 |
| C01 / C005 | 条件前检完成；没有提供需撤销的真实执行者、普通resume不足的理由及处置闭包，不执行replace | [当前前检](../C005-executor-continuity/evidence/acceptance-20261005/current-revocation-preflight.json)；真实事件成立后按 [原手册](../C005-executor-continuity/experiments/runbook.md#2-真实接续的前提) 执行 |
| C02 / C008 | 同7份声明/27Node的当前原字节核对完成，requires仍0；probe未采用，host ready未知 | [当前声明前检](evidence/remaining-20261005/current-declaration-preflight.json)；实际kind:name、作者确认、重复摩擦、单host规则/获准路径/预算齐全后才采用probe，不添加假requires |
| E01 / C002 | 当前范围无需执行：APFS无法创建的物理非UTF-8名称现场测试属于扩展覆盖，不是当前验收欠项 | [当前范围采用](evidence/scope-macos-apfs-20261005/user-scope-adoption.json)；原not_run/environment_blocked、APFS errno及准确拒绝测试保持，以后采用对应实际载体才补验 |
| E02 / C006 | 当前范围无需执行：外置物理设备专项认证属于未来扩展，本地APFS导出与读回已验收 | [当前范围采用](evidence/scope-macos-apfs-20261005/user-scope-adoption.json)；原physical not_run与载体元数据保持，不改成PASS；以后有真实移动介质需求再采用 |
| L01 / C002、C004、C005、C006 | 旧LEAK/查询取证/旧native及SK01/SK02标签和未知保留；当前215项处置已闭合，原输入不能由新操作追溯生成 | [原处置交接](evidence/resume-20261004/completion-report.md)；有原运行原件才能讨论旧因果，不重复215项来伪补历史 |

H01及三任务当前完成是已记录事实。E01/E02按用户当前产品定位不再列为交付欠项；原执行未发生的标签仍保留。C005/C008属于需求触发后的未来操作，条件前检完成不等于实际触发；历史不可追溯不记PASS。当前实施、声明和导出工具均未新增产品机制；Rust/构建/Workbook输入与可信准备基线相同，原合格测试按各自run引用，见 [输入范围核对](evidence/remaining-20261005/engine-source-scope-readback.json)。

## 3. 人与实际环境的后续入口

三份指南的实际阅读接受已收到，不再要求重复接受同字节。它们仍可直接使用：[源码快速开始](../../../guides/source-quick-start.md)、[接续方式选择](../../../guides/continuity-choices.md)、[冻结时点交接卡](../../../guides/current-acceptance-handoff.md)。

- 扩展载体：当前无需准备。以后出现移动介质或非APFS应用需求，先采用对应支持范围，再提供实际载体及授权新目录，执行对应身份/完整性/错误路径验收。
- 真撤销：提供真实Work/Attempt、同目标/输入/质量/gate、普通resume为何不足、旧进程停止或隔离、新执行者与成本记录。缺项时保持普通接续或停止，不注入事故。
- 宿主依赖：作者提供真正必需的kind:name、对应Node、重复摩擦，以及单host版本/身份规则、获准读取范围和预算。只在采用后做极小只读probe，不扫描全宿主寻求样本。
- 过去成本/原生配对/盲审：取得真实原件后追加独立核验。无法取得的过去事实继续未知，不用本轮完成率推省时、省钱或严格公平。

v0.3.0发布、用户安装替换、push/merge/deploy不在本次剩余验收的自动动作中。旧费用、取证、平台和对照限制不由本轮关闭。2026-10-04清单原字节保存在 [旧索引快照](evidence/remaining-20261005/remaining-acceptance-before-update.txt)，本轮证据在对应package的新目录内，旧实验原件不改。

当前macOS／APFS采用范围的实施与验收已完成收尾，没有需继续实施的产品任务。真撤销/宿主资源预检保留触发入口，历史成本/公平原件保留未知，发布仍单独治理。范围确认与最新审查见 [本次范围记录](evidence/scope-macos-apfs-20261005/README.md)。
