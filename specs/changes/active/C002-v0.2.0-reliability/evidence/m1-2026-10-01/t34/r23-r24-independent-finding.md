# M1 快照语义漂移的原实现复现

独立 Reviewer `/root/m1_spec`。原产品候选 `06c3af3106e6669d788a2a718df01d12e0c51fb9`；核准冻结 clone binary SHA `90ca2d18fbcd912cd085e08792b7a70bab497d4e406ac56d86b9c947282ede0d`。本地临时 home，根源码、clone 和管线未改。

运行：`python3 /private/tmp/m1-snapshot-semantics-probe.py`。原始 argv/exit/stdout/stderr、首个提交回复、合法控制重放、单逻辑条件 SQL、所有业务表与目录 bytes/mode 前后 oracle 见 `/private/tmp/m1-snapshot-semantics-probe.json`。三个反例都在未变异 binary 上确认，且 Store/业务文件未发生额外变化。

## Fail 的不可能 Work status

真实 two-step start → outline begin → fail，先捕获成功响应，并确认同 rid 控制重放正确。随后只改这个 fail 请求的 `reply_json.data.work_status`：一例 `{"kind":"succeeded"}`，另一例 `{"kind":"cancelled"}`。同 rid 再 fail 返回 `ok=true,replayed=true` 与篡改状态。

这不是用当前状态误验历史快照：本回归没有任何后续状态推进。Command::Fail 根本不可能产生 Succeeded/Cancelled，按 protocol §3 attempt fail、architecture 状态转换与 core decide_fail，只能 Active 或 Blocked(RetriesExhausted)。

根因：`snapshot::check_data` 把 FailedData.work_status 解析后 `let _` 丢弃；`validate_command_owner` 只核原Attempt为Failed，不核该回复声称的Work状态。T34 metadata 阶段沿用这个checker，仍会把这种响应当成已核快照。

最小修复边界：对 Fail 可直接用原Attempt retry 与冻结Node max_retries 验证历史结果，不和当前Work status比较。为保持单一规则，可以复用/提取core现有失败后状态纯函数。不要重建所有历史状态或添第二套Work事实。

## Begin 的伪宿主资源

独立 Flow 声明：examples/two-step/flows/default.toml 的 outline 没有 requires，按合同为空列表；首次真实 begin 回复 requires=[]，合法控制重放也为空。只把同一语义字段在持久 Reply 与 data 的两个对应位置一致替换为一条合法 `mcp:extra-resource` 声明（可选version/digest/source均null）。这保持内部一致性；同 rid begin 重放仍成功并返回伪资源。

合同 protocol §3 attempt begin：requires 必须是本节点引用的 manifest 声明，按节点声明顺序。根因：check_data 只核 data.requires==Reply.requires；Begin 的 validate_command_owner 未与冻结Node.requires/Graph.requires核对。Start已有 Graph requires 比较，不会自动保护 Begin。

最小修复边界：Begin 回复requires与冻结Node引用解析后的HostRequire列表精确一致。沿用core begin的同一个选择函数/已编译图接口，不创建另一份声明来源。

## T34 原响应资格

T34 对 remove snapshot.id/version 的阶段分离方向正确，但 Add仍在构造original后才检查 effect/registered 的实际id/version/digest绑定：仅语法合法不是业务已核。Add audit只有source intent hash，没有实际装入Workbook身份。对无法独立确认的历史Add快照不能伪称checked；需要在原响应资格前使用可用的现有提交身份锚点，且仍保留合法Begin快照遇effect BLOB/坏路径时的原响应。

这些都是已存在的业务绑定事实校验。无需读取自然语言、重新执行任务、迁移Store、修改历史快照或装宿主资源。M1不能把结构化DTO解析成功或最终报错等同于原响应可信。
