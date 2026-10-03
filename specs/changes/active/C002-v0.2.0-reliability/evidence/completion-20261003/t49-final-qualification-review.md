# C002-T49 / G02 最终独立验收

结论：**通过，限定当前T49/G02候选闭包。** 47个精确当前ID无漏无重：35项实际Caught、12项限定静态处分。35项中的11项只检测准确诊断/资格来源差异，12静态中的4项保留错误文本差异；不是47项都动态捕获或完整错误响应等价，不批准旧e3eea89、其他SK02组、全部215或C002-M2。

Reviewer：`Codex /root/oracle_review`，未参与oracle或实现编写。只读核源码、caller/producer、优先级、每项已完成原log、输入闭包、门禁和派生账本，未改源码、oracle、旧mutation标签或重新构建。基准 `63b05ca8c4b6977677b01db957b550f83393926a`；最终workbook_txn SHA256 `b514c1411f632f1ec09efdb5c024c31d51ecd5c0e70ebef6e49f0a9cf90b37b4`，snapshot_qualification `0f78ee613a5677a855a3d84dd993ffa463462e11cc0ee8be5db459335fc3d268`。

完整逐ID判据、原失败片段、log/diff SHA256、来源、真值表和闭包核验在同名JSON；oracle与阶段正确性延续 `t49-all-oracles-independent-review.md`，本报告补实际最终执行验收。

## 12个精确静态处分

| 当前位置 | 个数 | 独立核验的限定依据 |
| --- | --- | --- |
| effects.rs:514:57 | 1 | 唯一caller先核同一audit局部值；len错误仍被下一纯exact-slice target检查拒，原响应资格/效果停止不变，detail不同 |
| pending.rs:173:66 | 1 | 唯一consumer用load_work_start，SQL固定work/revision1；from_rows拒Workbook/Work归属交叉，所有目录ref恒为同Work publish，&&/||成员和顺序相同 |
| pending.rs:185:58 | 1 | row规范路径与完整index producer使Workbook owner/final双向对应，Work命名域互斥；&&/||成员相同，不能套给删除函数或==→!= |
| service.rs:922:18 | 1 | 同一解码Vec后紧接完整pure单Publish/count/owner/final/digest/root守卫，扩大find_map不会扩大成功或文件读取集合，错误detail不同 |
| workbook_repo.rs:740:46 | 1 | 同一Returned row.reply_json完整资格已核request_id/replayed，metadata与cleanup fallback逐字锁定它；复解析两谓词恒假，不把新audit查询当同值 |
| workbook_repo.rs:749:30、:749:61 | 2 | 同一data与CheckedEffects严格绑定；弱化data-vs-row后纯effect-vs-row仍支配身份/摘要拒绝，未进业务文件读取；拒绝detail不同 |
| workbook_repo.rs:769:13、:770:13、:771:13 | 3 | 单mutation下未改749与strict CheckedEffects/root/row规范条件，因此四谓词恒假；局部owned值不被后续SQLite修改 |
| workbook_repo.rs:785:13、:786:13 | 2 | 原始internal_id不正规化，严格pending三段与raw map lookup使B恒假；规范owner/final使C↔D。Rust分别解析A∨(B∧C)∨D∨E∨F、A∨B∨(C∧D)∨E∨F，约束16组合独立枚举零差异 |

其中8项在当前真实producer/caller闭包内观察相同，4项（514、922、749两项）仅拒绝集合及副作用边界相同，有真实可观察错误文本差异。全部保留single-mutant、完整producer/其他guard未变、当前caller限制；不称任意手工Index/未来caller/安全或跨平台等价。

785/786的代数不覆盖787/788：旧index与fresh CheckedEffects之间digest独立，A=B=C=D=F=false/E=true可被787/788吞掉，二者由真实restored-index消费者动态捕获。补充JSON中F=true的另一个代数反例是过近似；当前Workbook两root恒空，真实可达依据是E=true/F=false，不影响已采用12项证明。

## 35项实际动态结果

| 批次 | 实际结束 | 秒数 | 完成结果 |
| --- | --- | --- | --- |
| A | 12个精确mutant | 259.252 | baseline10/10；12Caught；exit0 |
| B | 12个精确mutant | 179.984 | baseline10/10；12Caught；exit0 |
| C | 11个精确mutant | 162.935 | baseline10/10；11Caught；exit0 |

每批outcomes有实际end_time，0Missed、0Timeout、0Unviable，source_unchanged=true。独立逐log核全部35为Build Success、Test Failure100，原panic来自目标测试/断言；没有编译失败、空filter、准备阶段误失败或外层预算伪捕获。

| 检测类型 | 个数 | 实际原文与scope |
| --- | --- | --- |
| 坏状态被实际接受 | 16 | shape/identity4、Start effect/path4、实体manifest1、二次audit4、stale-index digest2、older-publisher request/index桥1；unwrap_err/err或CLI意外成功，确实命中目标拒绝判据 |
| 合法publication被错误拒绝 | 8 | Work五个immutable equality反转、Workbook fallback false和两个equality反转；真实writer/cleanup后合法metadata不变，本应接受却报owner缺失；不是setup错误 |
| 准确诊断/资格来源检测 | 11 | lifecycle桥pending3和WB656/692/698、latest-remove708、copy273、compile NotFound843三个变体；仍拒绝，但错误来源/阶段不准确。保留诊断分类，不能宣称它们接受坏对象或完成非法写 |

35项实际Caught是新运行事实，不覆盖原stage1/普通回归Missed、全组外层1200s截断、13项短样本11Caught/2Missed或初次fixture失败。派生账本47项均保留 old_stage1_formal_summary=MissedMutant；24 behavior_or_qualification等于本审查16坏接受+8合法被拒，11 diagnostic一致。

## 唯一集合与输入闭包

独立核目录catalog47唯一ID；A/B/C为35唯一、静态12唯一，交集空，union与catalog完全相同。catalog的47个old_id也与归档 `e54dcd41d8f1e186007b62b47583063cb19a4b66` 的原G02缺项集合完全一致，无新增样本补数、无漏ID。

三个批次190份source/fixture/skill/Cargo/config输入映射逐字相同，当前各SHA全部一致；含45份examples/workbooks输入。最终工程登记输入与该闭包共同文件SHA逐项相同，无current drift；static关键source SHA一致。全65份src Rust文件独立与基准逐字比较相同，既有两测试文件前缀不变，只有T49新测试及其归属注释发生变化。

## 最终工程与治理

最终完整门禁fmt/check/clippy/nextest/doctest全exit0；五份raw输出SHA与结构化记录一致。Nextest run `d9c118ef-a227-4725-bd47-259128d33306`：863/863、0skip、0LEAK、3slow，test阶段185.113s（整命令187.326s）。Workspace doc命令中core5个compile-fail通过，其他crate0 doctest不冒称新增覆盖。

MSRV Rust1.85与默认feature Cargo check原文完成，执行者记录exit0；本Reviewer读取原文，没有另行重跑，默认配置既有unused warnings保留。docs/specs/tests/core-vocab/skill原文均OK；最终check-tests为863测试/204任务卡。旧 helper归属注释868/863 FAIL和precomment 863测试PASS属于各自历史输入；当前helper归属注释已修，最终完整门禁重跑在新冻结SHA上，不借用旧PASS。

本报告通过当前T49/G02关闭所需的oracle、47精确处分、工程及治理闭包；任务状态/最后文档和提交范围仍由执行者按计划收尾。342原件档案后续追加本review/更新内外账本的字节保真由执行者另核，不以本报告预判更新。macOS aarch64限定范围不变，其他平台、无人trial和后续M2不转PASS。
