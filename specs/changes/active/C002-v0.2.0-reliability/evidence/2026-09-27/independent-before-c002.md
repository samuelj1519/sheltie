候选 a664e75；独立阅读阶段尚未打开 C002。
已复现：跨 Work cancel request_id 串用返回成功但目标仍 active；submit 删除输出后同请求误报 REQUEST_CONFLICT；submit 重放把历史 revision/next 和当前 cancelled 状态混合；start 删除仓库后无法重放；缺输入 start 遗留 Work 目录；历史 engine.stats 重建字节变成当前状态。
静态：Store init 非事务；Workbook add 先提交记录再 rename且无恢复；load 不核登记摘要；digest double hash；managed filesystem confine未贯通；整目录读多次/观察全文后才限额；单用户 CLI 无独立人工认证；公开类型可变不等于非法状态不可表示。
产品：有价值的确定性本地流程记录/边界层；主优势跨会话续接与产物来源；没有实证不能承诺省token与提质；门槛是协作协议不是同用户agent的权限隔离；保留三个crate/同步/sqlite，先修数据与请求协议，再真实宿主成本质量比较，延后adapter/DSL/并行调度/安装器。
