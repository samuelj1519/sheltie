# C002-T27：删除同对象与完成证明

候选平台：macOS arm64。Linux原生运行按用户授权保留`not_run`。本任务依赖T25统一恢复和T26受管目录句柄；不改存储格式与`delete-complete/v1`字段。

## 实现

- `workbook remove`在COMMIT前要求登记行、合法非空摘要和受管final目录都存在，摘要必须与登记相同；缺目录或空摘要不再略过完整性检查。
- `delete_dir`从已校验的`pending/<id>/payload`相对路径读取internal id，并再次核对`.owner`的`format/id/request_id/op`。final与pending状态使用管理根目录句柄和no-follow观察；两者同时存在、两者缺失且无marker、已有marker却又出现对象，都会停止，不写成功证明。
- final只有登记的Workbook manifest身份和摘要匹配时才移入本请求payload。pending恢复也重新核manifest/摘要；部分删除留下的内容与原摘要不同就停止，保留剩余对象，不猜测并继续删除。目录权限变化、NOREPLACE移动、根目录句柄绑定、删除与父目录sync均传播错误。
- `delete-complete/v1` marker严格核普通单链接文件、结尾换行、拒绝未知字段、规范JSON字节和本请求internal id。读取或独占创建返回的同一`SafeFile`句柄贯穿sync；sync前后复核证明字节、文件句柄类型/链接数和当前路径绑定。marker同步期间final或pending重现时不mark。
- 删除完成后退出、最后一个子对象删除后退出、marker校验后替换、root unlink前空目录换绑均有命名故障点和真实恢复入口。partial delete失败时已改目录先尝试sync；原始删除错与sync错同时保留在恢复错误里。

## V21–V23证据

- `task.stdout.txt`：`scripts/task.sh C002-T27`，Nextest run `e17ccf93-e371-4e9b-87b4-d4e29358cfc6`，8 passed、585 task-filtered。V21覆盖删除后marker前exit70后结果不明、本请求`committed=true`原响应保留、新B以`committed=false`被阻断；两个不同internal id互借marker及坏format marker均拒绝。marker校验后以rendezvous替换叶节点时，同fd路径检查拒绝并保持`published=0`。
- V21还在第一个payload子对象删除后exit70。重放发现部分树摘要已变化，保留余下payload，不写marker、不标published。
- V22分别把final和pending替换成同身份但不同摘要的目录。两处均保留新对象字节和inode，不写marker、不标published。
- root unlink窗口持有原ManagedTree与同一个parent fd；在最后一次身份检查前把已清空root换成另一个**空**目录，恢复停止且保留替代目录的inode/mode。该空目录oracle可识别移除身份检查后`REMOVEDIR`会误删替代对象。
- V23重装同id/version但不同摘要的新生命周期，重放旧remove与旧add后比较新目录inode、文件字节、Store digest行及两份旧reply_json原字节，全部不变。

删除前后的根目录entry检查与`unlinkat(REMOVEDIR)`使用同一已打开parent fd，并在unlink前核同inode。这个方案沿用`HomeLock`协作边界；不宣称一个stat与随后unlink的系统调用能阻止任意不遵守锁的外部写者。

## 门禁与审查

- `workspace-nextest.stdout.txt`：macOS全仓 Nextest run `7e61974e-fbfb-486b-82ad-0bc5ca7e37cb`，593 passed、0 skipped、1 slow、1 leaky；命令exit 0。`leak-isolation-cli-effect-pending.stdout.txt`隔离重跑该CLI测试1 passed、无leaky；并行run的leaky原因未确认，不把隔离PASS当作关闭原因。
- `fmt.stdout.txt`、`cargo-check.stdout.txt`、`clippy.stdout.txt`、`msrv-check.stdout.txt`、`deny.stdout.txt`和`check-docs/specs/core-vocab/tests/skill`原始输出均通过。`deny`使用本机缓存advisory DB离线执行；`deny.stdout.raw.gz`保留逐字节输出，文本副本去掉box-drawing空白行的尾随空格以通过`git diff --check`。
- `check-task.stdout.txt`记录暂存候选的`check-task C002-T27 --staged`；最终提交后同一任务门禁再次运行。
- 独立Spec与Standards Reviewer复核通过，实施者未参与审查。

## 边界

T27只关闭Workbook remove的同对象删除与marker完成证明。Linux保持`not_run`；T28 pending清理/只读发现及V25维护诊断、T29一致快照、T30 spec-dev交接、T31完整故障窗口/突变处置与C002-M1仍由各自门槛决定。T16真实宿主回归和T17发布不提前关闭。
