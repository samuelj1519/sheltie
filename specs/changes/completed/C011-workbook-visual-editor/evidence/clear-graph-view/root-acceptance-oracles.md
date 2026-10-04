# Root 独立验收预期

这些预期来自原Workbook和用户要求，不由生产布局函数生成。实际执行后另存结果；此文件不证明通过。

1. spec-dev默认主流程节点顺序：spec、plan、plan-review、scaffold、implement、verify、review、deliver、retro。可见边索引（从0计）：0、1、2、5、6、8、11、12；显示8/25，fix/escalate仍能选择。第二行方向从右到左，换行从上到下。
2. 选择plan时显示主骨架加plan-review→plan和escalate→plan，合计10/25。全部模式25条；左侧列表始终25条。选择隐藏边显该真实边、突出两端、保持节点位置；选中高亮d与普通路径完全相同，无额外尾线。
3. 实际鼠标路径、选中标签与列表均对应原Flow相同from/to/kind；所有25条核对。交汇处明确提示；最近线只在实际可见集合中比较。原生横/纵滚动、拖动、平移、缩放后点击仍对应，适应/整理清理原生滚动。
4. code-change默认2/3，spec-dev11节点/25边完整定义保持。仅布局、模式切换、聚焦后保存ZIP，spec-dev27个文件逐路径逐字节与原样例相同；不是只比文件数。
5. plan的直接前置为spec、plan-review、escalate。多选输出生成node.output，沿用前置input保留原from，不生成node.input。来自plan-review的plan/tasks与当前plan自引，应不可新增；原input不删除。合法下划线名称和中文自定义名称可新增，仅节点内唯一。
6. 前置optional输出生成required=false；required省略/true/false三态正确，不复制result/max_bytes。已有同name/from不重复添加，同名不同来源要求解决冲突。普通/未知/自引原值不在渲染时修复。
7. 从同一资料框添加绝对路径、相对路径与URL三个字符串，保存为三个真实resource，引用位置保持原文且不读host/访问URL。改名不改资源；修改位置创建新资源，原共享/原引用资源保留；失败时原Map和输入行不变。重新打开后能显示位置并继续编辑。
8. 常态已选资料仅名称/可读来源摘要/移除；精确语法在折叠高级。用户最终对易用性的判断与工程检查分开，未经回复不记human accepted。
