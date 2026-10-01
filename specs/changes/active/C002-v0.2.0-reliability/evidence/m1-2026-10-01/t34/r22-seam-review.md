# 原响应绑定修复 seam 复核

Reviewer：`/root/m1_spec`。只读；候选 `06c3af3`。

## 结论

不建议把 `original` 的资格直接绑定到完整 `access.load_effects` 成功。合法快照与坏效果载荷是独立事实；前者必须继续交接，后者必须阻止任何效果 I/O。应把现有请求校验分成「快照元数据绑定」和「效果载荷/完成证明」两个阶段，保留同一套 validator。

## 必须保留的测试边界

| 条件 | 原响应资格 | 现有真实 caller |
| --- | --- | --- |
| begin 快照及 audit/Work 绑定合法，effects_json 是 BLOB | 保留原响应或旧 A 的 pending_original；禁止执行、登记 B | `service.rs::malformed_old_effect_row_is_attributed_to_the_blocking_request`，`common::assert_effect_pending` 会 unwrap 原响应 |
| 合法 begin 快照，后续效果 path 指 ../外部，前面的 brief 已缺 | 保留原响应；整个批次停止，brief 不补、外部 bytes 不变 | `service.rs::invalid_later_effect_is_rejected_before_any_effect_runs` |
| 合法元数据，seal/card/sync/mark 失败 | 保留完整已核原响应、committed/rid/Work revision | `post_commit_card_failure_returns_committed_response`、`start_replay_sync_failure_preserves_own_commit_snapshot`、`mark_published_zero_rows_returns_committed_recovery_error` |
| work_id 列是 BLOB | committed/rid 保留；original=None | `malformed_current_owner_column_keeps_committed_request_identity` 明确断言 owner 不能可信绑定 next |
| reply_json 是 BLOB、request-id/replayed/快照业务归属错误 | 保留已知提交身份；不得附未经核实成功 original | `malformed_current_reply_keeps_commit_identity_and_request_conflict_priority`；T33 Workbook 单字段漂移以及本轮真实 CLI probe |
| 缺/重复/不符的 audit 使绑定不可核实 | 原始 JSON 的形状合法不足以取得资格 | `duplicate_audit_owner_is_rejected_before_recovery_io`、`begin_replay_rejects_audit_and_snapshot_metadata_drift_without_business_changes`；须区分损坏的是元数据还是效果，不能用全局 helper 强制所有错误均有 original |

## 最小现有代码 seam

1. **Store 读取层**：增加窄的请求快照元数据读取（`intent_hash, reply_json, work_id, at`），不解码 `effects_json` 和 `published`。request-id 是 SELECT 参数。hash lookup 保持冲突优先级及「已查到行，因此 committed=true」事实。effects BLOB 或坏 published 不应阻碍独立核合法快照；坏 owner/reply 则让这个阶段准确失败。效果读取随后取这两个字段；在同一个 HomeLock 下不会有协作 writer 改写已提交元数据。
2. **WorkService**：从 `load_checked_request` 提取 `decode_effects` **之前**的既有校验：hash 语法、唯一 audit、work_id/at、audit revision、PersistedResponse身份/revision/replayed、加载 Work/frozen graph、`validate_command_owner`。产出一个私有 checked context，保留 snapshot、command、Loaded，后续效果检查复用它；不要先校验一次原响应，再完整 load 同一 Work 第二次。
3. **WorkbookRepo**：从 `checked_effects_for` 提取 `decode_effects` **之前**的 audit / request / snapshot 校验和 identity。关键是合法但错 id/version 的 `RemovedSnapshotData` 必须在这阶段与 audit target 核对；add 的 id/version/digest/flows/requires 合法性也属于快照阶段，不能等 `check_workbook_effects` 才核。将 effect helper 内目前独占的 id/version/digest 语法验证移动到这唯一阶段，而非再复制一套。
4. **投影**：只接受该 checked context / opaque checked snapshot 的纯原响应转换。删除 `original_snapshot_parts` 对任意 raw JSON 的可信升级路径，尤其不能凭 `strict_workbook_data` 的合法值域就构造 `ok=true`。CLI 继续只做协议渲染，不读 audit/Store 或猜业务。
5. **恢复编排**：metadata 校验失败 -> committed/rid/cause 保留、original=None；metadata 成功而效果读取/解码/路径/owner/完成证明失败 -> 附 checked original；执行/sync/mark 失败也附同一个 checked original。旧 A 阻 B 保持 B committed=false，checked A snapshot 只进 pending_original。不要把已经包装的 EffectPending 再包装成 cause=EFFECT_PENDING；编排及 trait 的错误责任需在一处固定。

可以调整现有 `RecoveryAccess` 让它一次返回 checked metadata 与效果结果，或让服务方法在 metadata 成功后对后续失败附已核原响应；核心要求是调用者能区分两阶段、复用已装上下文。无需新增通用重试框架、业务状态或第二套校验规则。

## 注意无法独立核实的情况

Work metadata 装入目前经 Start publisher effect 定位冻结副本。若损坏的正是 Start 自己的定位信息，合法原快照可能客观上无法独立核实。这时不能为了凑 original 回退 installed Workbook、枚举猜 pending 或假装合法；保留已知提交身份与准确损坏原因，并明确原响应不可核实。这与 begin 的自身 effects BLOB（Start 定位仍合法）不是同一个条件。

已采用的错误原响应规则需明确这个资格边界；不可只删除原始断言让门禁变绿。正常发布失败仍必须完整 original，effects BLOB/坏路径也保留能独立验证的 snapshot；元数据损坏则不伪造成功交接。

## 增补 oracle

扩展 T33 真实 Workbook replay 漂移用例以及 CLI：合法版本控制返回捕获快照；metadata 单字段错时 original/pending_original 不含坏目标；纯效果 BLOB/坏路径时原响应逐业务字段仍等于首次捕获成功回复；rows/业务 bytes/mode 不变。不同错误阶段不能共用一个总是 unwrap(original) 的 helper。所有期望来自捕获的未破坏提交响应、手写变异字段和合同，不能调用新 validator 生成 expected。
