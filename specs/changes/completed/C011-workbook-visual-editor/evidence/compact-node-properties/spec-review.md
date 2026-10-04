需修改

输入/输出页签、紧凑分组与单项编辑、精确 from 派生已有勾选及别名、逐项移除和跨页签保留待添加项符合采用范围；纯投影不改原声明，COW 与图代码未变。但编辑资料名称后直接点击未用候选，首次点击被候选列表重建吞掉：名称已保存，勾选及待添加项却为空。Root 已用最终候选原生操作复现。此为当前简单操作闭环的 S1 必修缺口，修复并补真实直接点击消费者前不建议交付。真人接受仍未执行。

## 候选与身份

- Reviewer：`/root/c011_clear_spec`；未参与本轮准备、实现或测试编写。只写本报告，不写正式 Home review 草稿。
- Baseline HEAD：`602d8ea1dbf136a5e962bfaef48bf7493646eb2c`；baseline tree：`6e6c25c46e6382797865d9a36810899417e987b0`。
- 候选 tree：`85aef8b30a8c0d86798736ded18cf57b81e1c527`。
- 完整增量 patch：154823 字节，SHA-256 `4277a78bdc07eda59579f1334e88a45b7d6ce6438db60d471f6125b7ca47971b`。
- 实际 app/sources blob 与候选 tree 对应，分别为 `d7af94677c33297b7d6c2281cff03dcdaf6735a9`、`8df5ea3f39df2210f6d017f7c9376161c1029632`。
- 已读 review#1.0 brief 四份冻结输入及实际生产/测试差异，依据 Root GF-34、Workbook 作者工具紧凑段、当前 plan、本目录 design/preparation-review/browser-oracles。

## S1：名称编辑后直接点击候选未生效

位置：`tools/workbook-editor/public/app.mjs:54` 调用 `refreshMaterialOptions`，`:283` 无 prune 时仍调用 `renderOptions`，`:260` 对整个 dropdown 执行 `replaceChildren`。已有紧凑资料行虽然原位复用，候选 checkbox 仍被整体替换。

触发：先展开添加区及下拉，再编辑单项资料名称，不用 Tab 或其它操作提前 blur，直接点击尚未使用的候选。名称提交引发候选重绘，正在点击的 checkbox 身份被替换。

实际：Root 原生将 `tasks_tpl` 改为“任务模板”，随后直接点击“写需求规格 · request”。名称已保存，但 checkbox 为 false，chips 为 `[]`。原件为本目录 `direct-candidate-click-observation.json`，固定候选为上述 85a；本轴读取该原件，不冒充浏览器执行者。

期望：一次直接点击选中该候选并形成待添加项，名称同步成功；候选状态更新不能破坏当前鼠标操作，不能要求用户额外 Tab 或重复点击。此问题对应 GF-34 简单资料操作与本目录 browser-oracles 的单项编辑/多选状态闭环，严重性 P2。

## 本轴实际窄消费者

在独立 source repo 调用生产函数，退出 0，未重复全部测试：

- 真实 spec-dev plan 保持 12 输入/2 输出，13 个候选有使用状态；`spec.spec` 对应 `spec`，`plan-review.reviewed-plan` 对应 `previous_plan`。
- 搜索 `PREVIOUS_PLAN` 返回原 row 及索引 7；输入/输出投影后全部 27 文件逐路径逐字节保持。
- 同来源新增别名后使用名有两项；移除原项仍保留另一别名及 required=false，移除末项才清空该来源使用状态。
- prune 已用 A 后，调用旧 A 的移除回调不会删除仍待添加的 B。

静审确认页签只隐藏原添加 DOM，现用 checkbox 只读并显示实际名称；单项移除针对实际 row，不隐式批量删除同 from 别名。搜索分组保原 row/index，不排序底层数组；精确配置与说明仍可编辑，原未知值/三态保持。新增 sources 仅投影与暂选辅助，不添加引擎格式、宿主访问或新事实注册表。

## 验证限制

48/48 是实施者的新及受影响数据/CLI 消费者资格，不覆盖本次原生候选点击失败，也不是完整工程单跑全绿。旧 held-stream 失败及其限定保留，费用和人工分钟未知。本轴未执行全 npm、Rust、图 75 次或浏览器，未修改源、服务、CLI 状态、索引、旧报告或封存输出。Root 最终完整副本与其它真实 UI 核验、人工接受按各自证据记录，不能关闭 S1。
