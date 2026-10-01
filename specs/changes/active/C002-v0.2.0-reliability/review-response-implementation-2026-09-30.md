# C002 实现审查修复答复

2026-09-30 用户授权修复[实现审查](review-implementation-2026-09-30.md)全部七项并继续完成T31。实施者：Codex。原T31工作、历史失败及旧变异结果均保留；本答复不改变T31/M1进度，状态只看[plan.md](plan.md)。

## 七项处置

| 项 | 根因修复 | 真实回归与独立oracle |
| --- | --- | --- |
| CR-S01 必要同步 | PrepareAttempt逐目录核同对象并sync目录/父目录；匹配历史文件也sync原句柄/父目录，包括已published的显式补缺。路径/类型异常统一STORE_CORRUPT，真实I/O失败保留IO | runtime implementation_repairs三例：落位后连续两次sync失败不得成功；published历史补缺保持原snapshot/bytes/revision。全部红→绿；普通cargo test以Mutex/RAII隔离注入 |
| CR-S02 NextOp重复映射 | CLI与恢复共用core::work::render::next_item_json；CLI调用方传WorkId，删两处重复match | replay、状态卡、同形next及现有CLI场景真实入口；历史data/next与原快照比较 |
| CR-S03 快照校验重复 | snapshot.rs集中严格data类型和公共形状检查，业务归属用当前可信Work/Graph核；保留unknown拒绝及规范Attempt/WorkStatus，不改持久JSON | 原tampered snapshot、错误字段/归属/响应、历史重放与OriginalSnapshot回归；完整默认门禁 |
| CR-S04 重复装入 | Work读取仅定位其Start请求，避免解码无关历史effects大载荷；同写锁重放复用checked请求、响应与效果。maintenance仍先核全Store引用 | 原pending rename/mark/cleanup交错、坏引用停止、旧响应与跨类恢复回归。没有性能基准，不宣称SQL常数复杂度 |
| CR-P01 首次建库崩溃 | 内存SQLite同事务生成完整schema2并核结构，安全serialize API取得bytes；持锁同一个SafeFile独占写自有tmp，再NOREPLACE发布，sync两端父目录。删除最终空库初始化路径，不解释既有schema0/1 | CLI首建库精确SIGKILL→同rid重试→仅一请求→完整重放；旧schema原字节、staging换绑、并发add/install、purge等待者等原caller回归 |
| CR-P02 批准事实缺失 | 纯core结合冻结Graph核当前可离开和历史已离开的成功门槛；非gate不能为Gate受阻；runtime装入与list接入。损坏只拒绝、不补造事实 | CLI仅删除唯一approval，当前/历史两组status/stats/list/begin均STORE_CORRUPT、无新请求、原state保留；纯core非gate理由单改拒绝，未批准取消仍合法 |
| CR-P03 add重复源解析 | 源仅对象/目录/manifest存在/限额粗检；最终私有副本才parse/compile/digest。依据GF-17/GF-30纠正protocol泛称和原T24超范围断言 | 缺manifest拒绝不建Store/锁；内容非法无业务行/request/audit/final，来源修复后同rid成功及重放；原失败不覆盖 |

## 验证与保留记录

新证据在[evidence/repairs/t31/implementation-review-fixes](evidence/repairs/t31/implementation-review-fixes/README.md)。最终默认门禁全部exit0：fmt/check/Clippy、Nextest654/654、MSRV1.85.0 locked、离线deny、docs/specs/core-vocab/tests/skill、dist plan。Nextest使用隔离0.9.146，没有绕过版本检查。deny首次因只读公告缓存锁失败的原文保留，取得既有缓存锁权限后离线检查通过；不宣称公告库实时刷新。

第一次全仓回归暴露原T24的add内容不建库断言与上游GF-17冲突；原日志在gates-before-add-preflight-contract-correction。独立Spec裁定按GF-17/GF-30/GF-31勘误：源粗检失败不建控制对象；锁内副本内容失败允许控制Store/锁/自有未提交pending，但没有业务事实或最终Workbook。storage/architecture/protocol和任务卡已同步，不放宽Start的拒绝边界。

阶段653项通过后，独立Spec又发现已published历史补缺的二次sync遗漏，新增真实反例红→绿；此前653项结果归档到gates-before-published-history-resync，不作为最终654项运行。Standards指出的提交后分类及普通cargo test共享注入槽均已修复。独立审查者未参与任何实施。

## T31接续历史

旧普通clone候选da2bd的变异完成片、中断片、输入和旧审查全保存在[归档](evidence/repairs/t31/mutants/superseded-before-implementation-review-fixes/superseded.json)，不计新候选通过。新普通clone临时候选`3a9f6894d991aac609bacc3f3012046804acd811`仅固定验证输入，170项源码/fixture/配置/脚本在root/clone逐SHA核一致，治理输入由clone自己的Git提交冻结；新日志只写产品树evidence。

完整inventory、baseline、两阶段无漏无重、全部存活/timeout/unviable的准确分类及Reviewer处置尚待本次实际输出，完成前T31仍doing。Linux按用户授权not_run；本任务不替代独立M1、真实Host或发布门槛。

## 完整变异中的回归补强

3a9f输入完成core724与全部第一阶段未捕获项的完整workspace复验，发现批准比较AND→OR没有被先前“删空全部批准”回归捕获。新增真实双gate用例，只删second#1批准、保留first#1；原实现正确拒绝、精确反实现exit100，原源码SHA已核恢复。独立Spec核合法正例、单条件反例、独立oracle与零业务写均合格，见 [复核](evidence/repairs/t31/implementation-review-fixes/cross-gate-independent-review.md)。

测试改变后全部3a9f变异结果归档至superseded-before-cross-gate-oracle，不计新输入。新普通clone `target/t31-validation/repaired-source-v2` 临时提交 `c31dcb390c15496f6e9586a92b3b73fa2d199061`，170项输入一致，完整inventory与两阶段从头运行。新最终结果仍待实际完成，不从旧core结论或isolated负控制推断T31完成。

## 2026-10-01最终收尾

最终源码输入`49d3a191`固定167项源码、主测试、配置及fixture；全仓675项与Rust/治理/MSRV/deny/dist门禁通过，见[最终证据](evidence/repairs/t31/README.md)。原先654项及各个临时输入均是历史记录，不替代此运行。独立Spec/Standards认可七项修复和本次提交，见[最终审查](evidence/repairs/t31/final-review.md)。

用户明确要求跳过可能触发额外安全检查的任务，后续安全复现与混合runtime变异续跑已停止。完整2499项均有[逐ID处分](evidence/repairs/t31/mutants/final-dispositions.json)，其中269项为`deferred_by_user`，不当捕获、等价或安全PASS。T31在这一明确豁免范围内完成；M1、真实Host、发布及Linux运行不因此完成。
