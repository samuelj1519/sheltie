# 清晰流程显示迭代

旧4311页仍显示三列数组布局、右出左入曲线，截图真实属旧版；新迭代不能只让用户换页，应直接让显示一目了然。完成后统一4311作为当前页，旧实例只在确认归属与保护草稿后停止；本次用户先前已明确同一spec-dev试点可恢复。

## 默认视图

按主流程阅读顺序紧凑折行（每行约3步、交替方向），换行段上进下出，同行段端口随前进方向左右选择；旁支放相关步骤侧边。不能再把原数组直接三列排造成次序混乱。小/中图尽量保持常用比例可读，不用9层横纵长条的极小缩略图冒充一目了然。主流程是从entry沿main边首次到达的显示骨架（循环不重复展开），只是视图，不能删改任何边或改变Engine next。其它节点仍可见，显示完整连线总数和主流程/全部切换，默认不把25条返回/旁支铺满画布。选节点显示它的入出邻接线，选边必显它的完整真实路径与两端，左侧全部连线列表始终覆盖全部25。查看完整连线是明确用户动作。

端口按几何方向选择上/下/左/右，不强制每条右出左入。返回边用独立外侧通道，方向箭头清楚；长跨越通道不穿卡片文字。未知kind原字面保留。常态不重复铺类型标签，只在hover/selected出现可读且避开节点的标签；同一时刻选中仅一个标签/路径。高亮直接复用普通路径同一d和同一marker，不能多一段尾巴、Bezier overshoot或旧命中stroke。不可避免交点仍按真实最邻近线选择、等距提示列表，继续保留原生scroll/worldBounds修复。

默认布局/展开聚焦/主流程-完整切换只改视图。已有缓存拖动位置保持，一键整理用新的紧凑折行布局；小图100%可读，大图概览不把小字体当质量通过，仍可pan/zoom与列表精确定位。新版本标识与单一地址避免截图再来自旧renderer。

## 统一资料选择

用户已澄清需要多选前置资料，而非仅输入名称建议。统一“选择或输入资料”可编辑多选框提供直接前置步骤输出、沿用前置资料来源两组候选，也接受多个自定义引用字符串；不设专门的路径或URL字段。已选资料名称可编辑；改名只更新当前row.name，不改row.from/required/result/max_bytes。没有前置步骤时仍可自定义。既有start.<key>、node.output、resource.<path>、engine.stats和未知原值保持，精确来源编辑收在高级设置，不自动迁移旧行。

## 验证

新几何/主流程显示/邻接聚焦/动态端口/唯一高亮d和label/名称建议使用生产共享函数加独立手写oracles，不静态镜像。code-change与spec-dev11/25都是真实UI场景：默认少线+明确总数；完整模式全25，路径/标签（shown whenselected）/列表真实鼠标选中准确；节点聚焦计数=实际邻接，隐藏边通过列表选中后必须显示。真实拖动、pan/zoom/native scroll后选择与显式fit不回退。完成切换/整理的全27ZIP同byte，改名只当前row.name且合法CLI通过。完整工具npm受影响一次，其后只新增问题消费者补核；Rust948不跑。独立审查后在同4311交人复测，当前humanaccepted仍false。

## 准备审查修订与用户澄清

渲染子集确定后，鼠标最近线只比较本次实际可见routes，禁止隐藏边抢选。默认fit只算可见节点/路径/选中标签，不用隐藏外侧通道压缩主流程；全部定义计数仍显示25。显示骨架按节点/edges原顺序，从entry队列首次到达的main边，visited不重入；坏入口/孤点仍显示不修定义。列表选隐藏边时显该edge并镜头fit到两端+其路径，保留positions不重新整理；清选/切换模式不洗用户缓存坐标。

hover只切现有label的显示class，不replaceChildren/rebuild正在命中的group，鼠标down/up身份稳定。选中路径和普通路径只用同一个d，highlight不额外draw同一尾巴。真实近线只在可见集合算，sourcepixel变换用actual worldRect/width，native scroll/zoom/pan修复保持。

用户澄清需要多选前置资料，而非仅name建议：输入可来自前置outputs或外部，还要求前置inputs列表多选。提供两组checkbox：前置步骤outputs生成node.output；前置步骤inputs明确“沿用原资料来源”，复制已声明from及required，不冒充冻结上一轮数据，也不发明node.input格式。仅直接入边前置节点，去重Node来源；无法在当前owner表达（如self引用/缺失原来源）明确不可选并保原定义。多选按明确“添加所选资料”提交，预览名称冲突并允许自定义name，不静默覆盖已有input/改变output/result/max_bytes；重复同名同from不重复加。

外部项支持多个自定义引用字符串，包括绝对/相对文件路径或URL。用户已选择固定写入方法，引用通过Workbook内真实文本resource文件保存，用现有resource.<path>绑定，并明确任务执行者读取引用。编辑器不自动读文件或访问URL，不在Flow或manifest塞未知默认值/第二个事实注册表，不把node.input当新增引擎能力。已存在的自定义来源原文仍可用。

最终采用：外部位置用真实Workbook文本resource逐项保存；任务执行者根据引用和方法说明读取。格式保持既有resource.<path>，生成路径使用固定前缀+安全UUID，不用用户名称当文件路径。更改位置总是创建新资源并仅改变当前输入的from，旧文件保留；同位置不创建新资源，更改名称只改row.name。已有共享resource不被猜为专用引用或覆盖。恶意或不能表示的原resource保留为普通文件来源。
前置outputs生成新input时保留required省略/false/true三态，尤其false不能默认true；不复制output.result/max_bytes等输出或终点属性。沿用前置inputs只复制其name/from及required三态，Node结果选择不自动复制；对当前owner自引用、缺声明或optional与来源组合不允许的新候选明确不可添加，原定义仍由CLI拒绝且不改写。名称冲突预览后自动建议后缀供改，不覆盖已有行；重复相同name/from不重复添加。

最终UI纠正（用户明确）：不做独立路径/URL字段或类型配置区。统一一个“选择或输入资料”的可编辑combobox：前置输出/沿用资料可在下拉中多选，直接键入自定义引用字符串亦为同一输入动作；已选项用短标签展示并可删除/名称改写。外部引用存储仍是上段安全Workbook resource机制，但UI不要求用户理解类型/from语法。保留既有start/resource/stats/unknown原值的可见摘要，精确编辑收在高级；渲染不自动迁移旧行或改写不支持的来源。没有额外网络/文件读取。

## 固定引用的文件格式与编辑边界

专用文本路径为resources/sheltie-editor-ref-<UUID>.txt。首行为Sheltie editor fixed reference v1，后接JSON，严格字段为schema、id、owner、location；schema为sheltie-editor-reference/v1，owner仅含flow与node。文件名UUID、正文id、当前Flow路径与节点ID、UTF-8和非空location须全部符合，才在界面显示引用位置；否则按普通resource处理。位置不作为脚本执行，界面不展示内部schema/id。

仅在明确添加或确认编辑时生成文件，不在渲染或每次按键时写入。先在候选WorkbookModel上构造完整Map与输入行并校验，再一次采用；失败保留原Map和原行，不能留下半成品。位置修改使用新文件，任何原资源都不覆盖或删除；删除输入的按钮只称“移除此输入”。改名不改UUID或资源正文。此规则仅约束作者工具生成与识别的参考文本，不增加引擎from类型或第二状态来源。
