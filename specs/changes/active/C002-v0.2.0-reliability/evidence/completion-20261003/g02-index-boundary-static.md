G02 索引边界补充独立复核：**:785、:786限定证明成立；:787、:788仍open。** 不是动态PASS，不更改原mutation标签。精确旧/current ID、diff SHA、源码SHA与真值表反例见同名JSON。当前partial outcomes中:785–787没有完成终态，:788为实际Missed；静态复核不补造执行。

原拒绝式定义：

- A：reference.request_id != request_id。
- B：reference.pending != checked.pending。
- C：reference.final_path != checked.final。
- D：reference.owner != checked.owner。
- E：reference.digest != checked.digest。
- F：reference.digest_root != checked.root。

原式 `A || B || C || D || E || F`。这里不能称整个reference与checked是同一不可变快照；Index可以较早捕获，CheckedRequest后查，但两侧分别有结构producer保证。

B恒假：`pending_internal_id`不把UUID文本正规化成另一个字符串，而是原样返回第二段；前后pending都严格为三个分段、prefix=pending、leaf=payload。Index key来自同一个raw第二段，`references.get(internal_id)`命中后reference.pending与current pending逐字相同。UUID大小写/格式不会因helper隐式正规化破坏证明。未命中或重复key在此前就拒绝。

C当且仅当D：Index producer对Workbook publish/delete严格核合法id/version与owner=`workbook:id@version`、final=`workbooks/id/version`；CheckedEffects对当前Add同样严格绑定，current row也先validate。合法id/version不含分隔符使两种表示一一对应。Work ref的owner/final属于互斥命名域，对当前Workbook两者同时不等。这个关系跨较早Index与较新当前值仍成立，不需要它们metadata相同。它不约束digest，E仍独立。

| ID | Rust实际优先级 | 限定判断 |
| --- | --- | --- |
| :785 | `A || (B && C) || D || E || F` | B=false、C↔D时化为A∨C∨E∨F，与原式相同；同一pure错误分支/返回PublishLocation，无附加I/O或可达诊断差异。 |
| :786 | `A || B || (C && D) || E || F` | B=false、C↔D时同样化为A∨C∨E∨F；成立边界相同。 |
| :787 | `A || B || C || (D && E) || F` | C覆盖D&&E，但**原本独立的E也丢失**。化为A∨C∨F，与原式不同；不能关闭。 |
| :788 | `A || B || C || D || (E && F)` | E&&F不能代替独立E；当前Workbook root相同时，digest错配可被吞，仍open。 |

反例 A=B=C=D=F=false、E=true：原式拒，:787与:788都不拒。独立枚举满足B=false/C↔D的16种布尔组合，:785/:786零差异，:787/:788各有这条反例。枚举只核谓词代数，不是产品oracle执行。

真实可达方向是已经采用的inverse digest race：第一次捕获Index前只让effect.digest=Y（合法SHA）；row与snapshot一直X，Index结构producer允许Y，因为它核格式/owner/final，不核snapshot/row摘要。现有pending_after_reference_index后把effect.digest恢复X，后续checked请求与row完全合格。匹配同internal-id/owner/final/根的ref保留Y，current checked为X，于是只有E=true。原guard应在业务目录读取前报STORE_CORRUPT；787/788都需要此实际正反消费者。健康Index/current都X作成功对照，保持完整row/bytes与准确停止点。

656/692/698的生命周期来源桥、708的最新remove闭包、727–730的新audit查询不套此证明。此证明也依赖单mutation及完整producer不变；不扩大到未校验手工Index、未来新owner域或其它平台。
