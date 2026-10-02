# D-041 创建顺序号与行政替换

状态：`accepted`
日期：2026-10-03
关联 change：[C005](../changes/completed/C005-executor-continuity/README.md)

## 背景与选择

撤销正式资格需要一个事务内结束旧并开始新Attempt，不能拆fail+begin。失败额度描述业务失败，行政撤销不消耗它。Attempt后缀改number，只有一套字段；failed/superseded计数直接从事实序列派生，每Occurrence固定最多一次替换。

schema升4，公开合同cli-result/v4；replacement_reason字段必须存在且nullable，serde缺失不默解None。旧1/2/3原件保留并整体拒绝，不迁移，不同时维护retry/number。Workbook、result与目录摘要格式不改。

新Attempt继承旧冻结非stats引用与来源，runtime同句柄核path/sha/bytes；stats取poststate，历史精确字节登记。历史failed响应必须核原Failed及截至它的失败前缀，不能看当前总数或number。Replace审计保存物化后的完整有界reason，严格绑定旧Attempt.replacement_reason；意图仍保存字面参数或文件源路径。该审计核验不新增状态库，其他命令的审计摘要政策不变。

## 后果与确认

替换只撤销正式接口资格，不停止进程/隔离宿主/认证人。真实使用的旧执行者处置归操作者，不升级为engine核事实。合法/拒绝、一次额度、max_retries=0/1、迟到、同意图重放、并发单胜及COMMIT前后原字节恢复的真实caller核全部闭包；独立Reviewer不写修复。
