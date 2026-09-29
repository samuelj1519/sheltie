# T28 独立审查问题处置

Reviewer：`/root/spec_review`、`/root/standards_review`。两位 Reviewer 未参与实现，只读复核源码、真实 caller、合同和测试 oracle。首轮及第二轮结论为「需修改」，以下修订完成后，两轴给出 task-local PASS；最终门禁另见本目录 README。Linux 始终为 `not_run`。

| 审查问题 | 修订与独立反例 |
| --- | --- |
| 合法孤儿含 0555/0444 冻结副本时无法删除 | 对同一 ManagedTree 核归属后放开内部权限，再删除该句柄绑定的私有树；`cleanup_removes_a_valid_orphan_with_a_frozen_readonly_workbook` |
| 索引跳过其他效果路径，可把已引用树误判为孤儿 | 所有效果路径都校验规范和 requests.work_id；PrepareAttempt 只允许该 Work 的目录及必要祖先，WriteFile/SealOutputs 不得进入 pending；`cleanup_index_rejects_a_decodable_effect_path_that_points_into_an_orphan` |
| published=1 的 final 缺对应成功请求证明 | Work 核唯一 Start、审计、快照精确字段、effect、状态身份和内容；Workbook 按 audit.seq 选择当前生命周期，核 Add/请求/快照/effect/登记行和内容；完成后缺 audit、错 snapshot.request_id、未知 Start.data 字段均拒绝 |
| 索引后发生合法 mark/cleanup 被误报损坏 | owner 不再存在时只接受同请求其他字段未变且 published 已完成的重新装入；`workbook_reader_accepts_publication_and_cleanup_after_its_reference_index` |
| 空容器检查后重开路径并递归删除，可能删除新内容 | 持有同一 ManagedTree，完成残片只作空目录删除；删除前再次枚举，内核 REMOVEDIR 拒绝新内容；`completed_empty_container_that_changes_before_unlink_is_preserved` |
| Work 装入把真正 rename 的 NotFound 包成损坏，完成态缺失又被重试成 IO | pending 根移走才有限重读；稳定完成态缺失保持 Workbook verify Missing / Work STORE_CORRUPT；`real_work_status_retries_after_start_publish_renames_post_location` 及原有缺失回归 |
| 完成 final 仍依赖已清或异常的历史 pending | 完成分支只读当前 final 及成功发布闭包；`completed_final_views_ignore_preserved_abnormal_pending_metadata` |
| UUID basename 的异常 container 被忽略，owner 被删 | 记录所有合法 UUID 叶类型，异常类型保留 owner、链接和目标；`cleanup_preserves_owner_for_a_valid_id_with_a_symlink_container` |
| 预筛正确 audit.work_id/revision 或忽略另一变体可退回旧生命周期 | 读取全部 audit；按 Workbook 命令或生命周期引用强制纳入，再严格校验；同秒 old add→remove→new add 中单改最新 revision、work_id 或合法他者 remove 命令均拒绝 |
| 告警缺请求上下文或声称维护未执行 | stderr 保留 request_id/object/reason，未知归属明确 unknown；坏旧 completed JSON 不改变新 Add 的成功 JSON/exit0；`successful_new_add_keeps_json_success_when_old_completed_effects_break_maintenance` |
| Cargo.lock 未列白名单 | 精确加入 Cargo.lock，理由是 CLI SQLite 集成测试显式声明已有 workspace dev-dependency |

两个既有 remove 拒绝测试原用 `list().len()` 证明登记行保留。T28 list 现在核当前发布证明和内容，不能用损坏对象的成功 list 作为行保留 oracle；这两处改为直接 SQL COUNT，原拒绝条件、错误码和行保留断言未改变。

非阻断建议：单次 Workbook list 可复用本次审计索引，减少重复查询；本任务未引入长期缓存或第二事实来源。
