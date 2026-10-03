# C002-T49 独立审查

结论：**需修改**。新增 shape/lifecycle CLI 正反例与真实 owner 清理握手有有效证据；owner 接受场景缺独立返回内容和完整 SQL/原件判据，同窗口拒绝对照仍需补齐。G02未完成，不给队列或T49最终PASS。

Reviewer：`Codex /root/oracle_review`，未参与 T49 设计、oracle 或源码。只读审查，不改测试/生产文件、不重启变异或构建；仅写本报告。基准 `63b05ca8c4b6977677b01db957b550f83393926a`；审查文件 SHA256：snapshot_qualification `5f5c5a3811091f58bdb177bf8adf1d30b0b828b05c046012ee309ed509058fd3`，workbook_txn `01c8d12bde3526099987a69a67fe94c310d66b8adf838274ab6d50d48bcb9ed6`。两文件是基准原内容后的测试追加，未改生产行为。

## 需补项

F-T49-01（审查期间已静态修正）：owner race 新测试最初无 failpoint 门控，而 common::RendezvousWorker 仅在该 feature 下定义，runtime 无默认 feature。默认 feature 测试编译因此会找不到类型；all-features 基线掩盖此问题。当前新测试已加 `#[cfg(feature = "failpoint")]`，静态缺口关闭。实际 default-feature 编译尚未由本 Reviewer 执行，不冒称该配置已动态通过。

F-T49-02：owner race 的 stats 只检查 `stats()?` 成功，list 只检查 len=1；没有独立核返回的 Work 身份、已提交业务事实与 next。writer/release 后 SQL 只检查目标 published=true，没有五表变化集和 works/workbooks 原件 bytes/mode/inode 的独立快照。依据 engineering §3.3 的仅is_ok/内部计数不能充当验证，以及 T49 plan 的五表、全业务树原bytes/mode/inode要求。需记录已提交真实producer的独立事实；对 writer 明示允许 published 0→1 与完成元数据清理，核其余持久字段/业务原件不被改写。两个 CLI 拒例已核五表+树 bytes/mode，但 business_files 没有各对象 inode，lifecycle 只补了 installed 根 inode；不得外推为整树对象身份无变化。

F-T49-03：同一 owner 观察握手只测 unchanged metadata+published=true 接受，缺能排除“任意 owner 错误都接受”的拒绝对照。至少需要同窗口 owner 消失但请求仍 published=false 时拒绝，以及真实writer完成后仅一项immutable元数据改变时拒绝，并分别核原件与持久行无读取侧改写。已有 `pending_workbook_with_missing_owner_does_not_fall_back_to_another_version` 等一般owner拒例有价值，但没有当前 fallback reread 的完成/元数据单条件组合，不替代本次握手的负控制。immutable 字段逐项证明范围应精确列出，不把单字段样本扩展为任意外部SQLite并发保护。

## 当前有效部分与边界

shape：真实 add→合法历史重放后，分别改空/重复/非法 flows、requires 冒号两侧空、非法 id/version。非法身份同步改 effect final/owner，排除仅因 effect与snapshot不同早退；期望来自 Workbook身份/Flow/宿主声明合同、storage §3.2/§5.2 和 original资格。错误完整核 EFFECT_PENDING/STORE_CORRUPT、committed/request_id、无original，以及拒绝前后五表和业务树。真实baseline `586f3559-c180-447a-898c-8868f6707b4f` 1/1；其2项filter skip不算完整无skip。

lifecycle：真实 A→remove→B，B与A相同digest，控制组show及旧add/remove重放不改变B树和installed根inode。之后将requests/audit/workbooks登记时刻同时固定为同一个合法Timestamp，使错误旧publisher不能仅因时间不同被排除；这属于受控SQL夹具，不声称真实引擎修改历史时间。十种改变使当前B绑定不可信，查询必须拒绝且五表、树bytes/mode和installed根inode不变。corrected baseline `6a38e95f-2957-43c2-b897-0ef478ce567a` 1/1；初次错误remove CLI参数是fixture错误，保留原文，不是产品red。

lifecycle 的 audit_target 同时把 Command 的 intent 从add变成remove、换target，却未同步原request.intent_hash；load_checked_request可能先报audit/intent摘要不一致。该case可保留为一般拒绝，但不满足T49“不能靠前层摘要错误制造检测”的目标branch证明，不能用于归结 publication_for_row 的Remove ||、latest-remove等存活体。当前30Missed包含这些相关守卫，说明前层控制仍需按真实producer和精确consumer单独处置；不能按相似 || 批量等价。

owner handshake：真实Repo.add及Work.start产生当前state/Command/effects/owner。测试**人工把已完成published=1回写0**构造rename后/mark前相容状态；不是实际停在COMMIT窗口的writer，也不是持久崩溃资格。reader在regular_open_after_stat观察具体owner后进入reached；主线程等reached，再disarm全局配置，沿真实repo.add或service.start同id恢复并真实cleanup_pending，核owner消失后才release。disarm本身不会释放已捕获配置的reader；真正release文件握手使顺序成立，避免sleep碰运气。

RendezvousWorker 在父线程错误/断言unwind时 Drop 尝试写release并join，sync临时目录还活着；正常 finish 释放、join并disarm。已完成mutant的reader返回错误后在finish解包处断言，reader早已回收。该既有helper不提供release写失败时的严格有界回收或线程/进程树隔离保证，本报告不外推。baseline `7ecbd8d9-90f6-4a63-9c21-e66bd525b2aa` 1/1，含workbook/stats/list三个正场景；32项因filter未选。

## G02原队列核算

`mutation-g02-complete`选择47项，外层Python subprocess.run timeout=1200截断全组。当前完成42项：12 CaughtMutant、30 MissedMutant，0 TimedOutMutant，outcomes.end_time=null，无final result.json，不能写完整PASS。

12捕获来源：shape4项（validate_workbook_identity、flows两OR及requires OR），owner合法竞态8项（Work五个immutable equality反转，Workbook fallback false与两个equality反转）。实际原log均给出对应测试失败；它们是指定选中oracle下的局部捕获，不覆盖其余守卫或旧候选。

剩余5项拆开记录：

| 当前ID | 原执行边界 |
| --- | --- |
| workbook_repo.rs:786:13、:787:13 的 publication_for_row OR→AND | 已有log，cargo nextest --no-run分别结束18.42s/10.84s；无Test阶段、无outcome，attempted_unfinished/build_complete/test_not_run |
| workbook_repo.rs:784:13、:785:13 的 publication_for_row OR→AND；:834:52 load_row_from_directory OR→AND | 无对应log，not_dispatched |

1200s是全组外层watchdog，不是五个各自TimedOutMutant。脚本异常后无最终source_unchanged判据；审查期间feature门控改变了测试源码，原42只按原selection/scratch输入保存。新冻结闭包、负控制与尚缺执行须分组完成，不能改原Missed、timeout或初次fixture失败。

准确argv/选择/源码hash在 `mutation-g02-complete/selection.json` 与 `check-g02-complete.py`，完整原文在 `mutants.out/log/`。本审查只核未提交T49范围，不批准全部215、M2、旧candidate、未运行平台或无人真实trial。
