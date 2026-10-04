建议交付

独立 Spec 增量审查关闭 S1：候选按 key 复用 checkbox 和标签，名称/使用状态刷新只原位更新；点击处理读取实时资料，未再依赖旧 used 闭包。Root 对同一原生步骤复测，名称保存后首次直接点击即勾选并产生一项待添加资料；A 转为已用后 B 保留，取消 B 不影响 A。输入/输出紧凑视图及资料数据代码其余保持，未发现新增必修项。结论限工程 Spec 轴；最终完整副本、窄窗和编辑核验由 Root 完成，真人接受仍未知。

## 候选与责任

- Reviewer：`/root/c011_clear_spec`，未参与本轮准备、实现或测试编写；不写正式 Home review 草稿。
- 前版 tree：`85aef8b30a8c0d86798736ded18cf57b81e1c527`，首轮报告及 Root 失败原件保持。
- 新 tree：`011ec67375aed79e6bf17aab60c9607471982d57`。
- 累计 patch：189457 字节，SHA-256 `6ee6412735a35515fc33439341827dcd1a318e920259c55ee462f43b54ff1aa6`。
- 既有工具文件唯一差异为 `public/app.mjs`。实际 blob 与新 tree 均为 `e09363717f2a5961f59399c4436798bbb182f17c`，文件 SHA-256 为 `42115ed9be488c43bb1881457d7da7fddad8bb5d93955323c5485ba503a5e628`。
- 已读 review#2.0 brief、新 change/checks；task/project 沿用前轮冻结输入及已读依据，范围未变。

## S1 关闭证据

静审 `materialChooser.renderOptions`：候选分组和 key 对应 DOM 在当前面板内缓存，数量与顺序不变时 `placeOptions` 不 detach 既有 checkbox；checked、disabled、名称及 usedNames 只原位更新。查询和实际增删仍局部调整展示。checkbox 处理器以同 key 获取实时 `materials()`，已有或不可选项恢复实际状态，未用项才加入 staged，且不会重复加入同 key。没有延时、模拟第二次点击、改动数据事务或新模型副本。

Root 原件 `direct-candidate-click-final.json` 标记同 app SHA：与旧失败完全相同的原生操作——将 `tasks_tpl` 改为“任务模板”，不提前 blur，首次直接点击 request。实际 checked=true、chips 只有 request 一项，编辑名称仍为“任务模板”。这关闭首轮 S1；本轴读取真实原件，不冒充浏览器执行者。

Root `pending-prune-final.json` 另记录：暂选 request/checklist 后，现有 row 来源改为 request；request 转为已有勾选只读，checklist 仍为待添加。取消 checklist 后 chips=0，request 仍勾选只读，checklist=false。新 DOM 复用没有回退前轮 staged-prune 与别名状态规则。

## 资格与边界

前轮本轴的 plan12/2、精确 from 别名、原 row/index、27 文件纯投影同字节、逐个/最后别名移除及 stale A 不能删 B 消费者，其 sources/model 代码未变，原结论继续有效。48 数据/CLI 消费者按精确依赖保持；本轮未重复它们。新 syntax/diff/静态 HTTP 1 是工程窄检查资格，不能替代上面的原生 DOM 红绿。

Root 最终 tabs/search/fold 的 27 文件 ZIP、窄窗可达、实际编辑与 CLI 检查由 Root 后续证据核验。本轴未跑完整 npm、Rust、图75或浏览器，未改源、服务、CLI、索引、旧报告及封存输出。旧 held-stream 失败与隔离成功限定继续保留，不称工程单跑全绿，不推断旧失败原因；人工时间、费用及整体真人接受未知。
