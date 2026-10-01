# T32 M1审查反馈修复

实施Owner：Codex。基准`ca6d92f`，工作区原有本轮M1未提交精简已纳入本任务，未覆盖其他工作。独立Reviewer：`/root/m1_spec`、`/root/m1_standards`，均未修改本任务文件。

## 生产与合同

- 删除core_effects_to_ops/snapshot_data未消费参数、重复own_pending转发。
- 删除无反序列化caller的Deserialize，以及无真实caller的RequestIntent::work。
- 删除已校验managed输入的不可达外部观察fallback及observe_optional；显式@file首次读取保留。
- 持根锁的Work写操作直接传播事务CAS冲突，不自动重算，统一Store合同内部矛盾；持久格式、Serialize和请求摘要不变。
- C002 spec/design删除过时T01进度句，start名字意图明确保留原参数及省略状态。

## 真实caller回归

| 测试 | 独立oracle与正反例 | 原文 |
| --- | --- | --- |
| installed_manifest_accepts_exact_file_limit_and_rejects_one_more_byte | 有效TOML加注释填充，32MiB安装原bytes并verify ok，+1拒绝且无Store/pending | [green](manifest-limit-green.txt) |
| cleanup_rejects_each_invalid_delete_reference_before_removing_an_orphan | remove历史效果只改final/digest/版本分别拒绝；合法孤儿、全Store行、文件bytes/mode不变；恢复合法引用后清理成功 | [green](delete-reference-green.txt) |
| remote_update_accepts_exact_stream_limits_and_preserves_state_on_one_more_byte | 真实CLI/SelfManager，仅fake curl传输、不联网；stdout有效JSON32MiB与stderr1MiB，exact/+1分别核URL、binary/prev、Store与tmp | [green](stream-limits-green.txt) |

初稿编译签名错误与instruction32MiB假设违反64KiB合同均保留失败原文，只记测试设计错误，不当产品失败或红→绿修复证据。改为合法manifest后通过。

## 审查与验证

两位Reviewer最终静态增量结论通过：接口精简不改变持久字节/指纹、锁内事务和效果停止链；三条回归有独立边界值、真实caller与拒绝后停止oracle。Reviewer未重复执行测试，运行由Owner提供。全仓门禁原文见[gates](gates/gate-results.json)。旧变异静态分析分别保存在[Spec账目](spec-static-dispositions.json)与[Standards账目](standards-static-dispositions.json)，预计捕获不是caught，完整变异由M1固定本任务提交后执行。

Linux not_run；M1尚未通过；T16/T17 not_run。用户已恢复暂缓验证，仅实际平台拦截才逐项记录跳过，当前未观察到拦截。
