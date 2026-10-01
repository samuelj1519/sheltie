# M1 未变异候选：错误原响应未绑定真实业务身份

结论：需修改。候选 `06c3af3106e6669d788a2a718df01d12e0c51fb9`；核准非变异二进制 sha256 `7b31c2a4ea086ceed8608c1ae13a3bb9e1ab390b1d018b33d8ce2345b994ec34`。Reviewer `/root/m1_spec`，未参与实现。macOS 本地临时管理根，不改源码、冻结 clone、管线或真实管理根。

## 可重跑证据

`python3 /private/tmp/m1-original-binding-probe.py`。原始 argv、exit、stdout/stderr、首次成功响应、单字段 SQL、Store 全行与业务文件字节/权限前后 oracle 保存在 `/private/tmp/m1-original-binding-probe.json`。

1. 实际 CLI add `examples/two-step`。
2. 实际 CLI `--request-id remove-original workbook remove two-step@1.0.0`，捕获成功响应，其中 data.id=`two-step`、version=`1.0.0`。
3. 控制例不改变 Store，同 request-id 重放成功，业务字段正确。
4. 反例只改 `requests.reply_json` 的一个 data 字段：合法 id=`other-step`，或合法 version=`2.0.0`；audit、intent_hash、effects、request-id 不变。
5. 重放同一真实 CLI 操作。业务正确拒绝：`EFFECT_PENDING`、committed=true。但 `original.ok=true` 附上的 id/version 是篡改值，和捕获的首次成功响应不同。Store 全表及业务目录全字节/权限保持不变。

## 根因与合同

- `runtime/recovery.rs::before_write` 在 `access.load_effects` 核请求闭包之前，先调用 `original_snapshot`。
- `original_snapshot_parts` 对 Workbook 只用 `strict_workbook_data` 核形状和值域；没有核 `data.id/version` 与 audit 的 `RemoveWorkbook.target`。
- `WorkbookRepo::checked_effects_for` 随后核到 snapshot 与 audit 身份不同，正确返回 StoreCorrupt。`own_pending` 却保留先前构造的错误 `ok=true` 原响应。
- CLI `error_map::to_outcome` 只解析该 JSON 并透传 `original`；它不具备 Store/audit 绑定证据。

协议 §5（protocol.md:224–226、252）要求 original/pending_original 是该请求的提交时响应；storage.md §1.2/§3.2 将提交快照、请求、审计与效果定义为一个提交闭包。本例附上的 id/version 明确不是捕获的提交值。完整性拒绝不能同时把已确认不符的业务字段标为成功原响应。

## 现有真实 caller 补强

T33 `workbook_replay_rejects_audit_and_snapshot_single_field_drift` 已有同一反例，却只断 EffectPending 及 rows/files 不变，漏 original/pending_original 的业务事实。应在真实 CLI 投影上断 committed/request-id/cause，并对不能核实的坏 original 禁止作为已验证成功响应附上；合法 card/发布失败仍保留捕获的原始完整响应。旧 A 阻新 B 同义务适用于 detail.pending_original，不能将 A 的损坏快照作为 B 的成功结果。

原始字段合法不等于与提交绑定正确。修复应在 runtime 校验业务绑定之后决定可附原响应；CLI 不应猜业务或自己建立第二套 Store 权威。对已无法确认的快照应准确保留 committed 身份/损坏原因，不能静默修写 Store、伪造历史回复或声称完整原响应已确认。
