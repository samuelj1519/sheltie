# 管理根与文件布局

管理根按 `--home` → `SHELTIE_HOME` → `~/.sheltie` 选择。相对参数按调用时工作目录解析，受管祖先、目录和文件须符合身份与路径约束。精确权限、对象归属及恢复规则见[存储合同](../../specs/contracts/storage.md)。

## 目录树

```text
<management-root>/
  .lock
  store.db
  store.db-wal
  store.db-shm
  bin/
    sheltie
    sheltie.prev
  workbooks/<id>/<version>/
  works/<work_id>/
    workbook/
    start-inputs/<key>
    status-card.md
    attempts/<node>/
      occurrence-<NNN>/attempt-<NNN>/
        brief.md
        engine/stats.json
        outputs/<declared-path>
  pending/<internal-id>/payload/
  tmp/
```

该图展示用途，不保证每次运行都存在所有条目。`NNN` 是至少三位零补齐的目录标签，Attempt ID 仍写 `node#occurrence.number`。实际路径从本次响应、resume 与 ArtifactRef 读取，不能从标签推测来源。

## 状态、投影与产物

| 对象 | 用途与维护边界 |
| --- | --- |
| `store.db` | Work、请求、审计与效果登记的唯一状态权威；不得手改来推进流程 |
| `store.db-wal`、`store.db-shm` | SQLite 的事务日志与共享内存控制文件；不能单独删除来处理锁或损坏 |
| `.lock` | 合法写操作使用的根锁；查询不创建或取得它，purge 保留原锁 |
| `workbooks/<id>/<version>/` | 已装方法的只读目录；同版本不覆盖 |
| Work 的 `workbook/` | 本次运行自己的冻结方法，后续操作不改读已装目录 |
| `start-inputs/` | 起始文本或文件内容物化后的冻结输入 |
| `brief.md` | 指定 Attempt 的任务书，含已绑定输入和输出要求 |
| `engine/stats.json` | 引擎生成的冻结统计输入；仅声明该来源时用于绑定 |
| `outputs/` | running 时供执行者写声明输出，submit 后按摘要封存 |
| `status-card.md` | 最新写操作生成的状态投影；查询不重写，文件存在不授予资格 |
| `pending/` 及 `.owner`、`.deleted` 侧车 | 发布或删除原件与归属证明；按数据库登记和对象身份处理 |
| `tmp/` | 下载、建库等自有暂存；不能与受保护的 pending 原件混为一类 |
| `bin/sheltie.prev` | 一级二进制回滚实物，不是 Store 备份 |

只读操作不创建 Home、不刷新状态卡、不恢复效果。SQLite 控制文件有 [D-039](../explanation/decisions/D-039-sqlite-read-control-files.md) 规定的例外；“只读业务查询”不等于文件系统中绝无控制文件维护。

## 写入与副本边界

工作 agent 只向本次任务书声明的输出位置写报告；业务仓库修改须有任务授权。冻结输入、已装方法、Work 方法副本、封存输出和 Store 元数据不能编辑。

`self uninstall` 默认只删二进制；确认 purge 后数据无法继续查询，不能把它作为排障步骤。异常 pending 或未知目录先核响应、原命令和归属，不按目录名认领、删除或覆盖。处理路径见[接续](../how-to/resume-work.md)和[排查错误](../how-to/troubleshoot.md)。

外围工具写入范围不同：作者工具创建自有临时根并下载 ZIP；导出器只通过 CLI 取得源字节，在显式授权的目标父目录建立新副本。它们的副本不成为第二状态来源，也不写回 Work。
