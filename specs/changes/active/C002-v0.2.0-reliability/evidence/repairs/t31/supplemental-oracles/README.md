# 存活项的补充反例

这些测试针对冻结生产候选`49d3a191aa4c918aab279617fa2bf7d9b3b36a0e`，在独立普通clone执行。每轮先逐SHA核167项主输入不变，再保存额外测试源码及其摘要，单列输入闭包。主候选的675项测试和正在进行的完整workspace复验均未修改。补充捕获不是对正式run的追溯改写，也不替代全部381项复验。

| round | 能力与真实入口 | 独立oracle | 实际结果 |
| --- | --- | --- | --- |
| 1 | WorkbookRepo完成add重放；WorkService同RID Start；真实CLI更新 | 三份一致但非法version必须拒绝；两份一致的requires必须与冻结manifest绑定；非法版本在任何curl调用前拒绝 | 未变异runtime2项、CLI1项通过；3个精确变异捕获 |
| 2 | CLI首次self install；CLI purge；CLI rollback | 新Home确实不存在；独立只读SQLite核schema2及WAL；外部hardlink导致整批删除前停止，先前owned文件、binary、Store与外部原件原字节保全 | 未变异CLI4项通过；3个变异捕获；静态rollback负控未捕获，原文保留 |
| 3 | 公共ManagedFs::rename_new的真实文件API | 单链接正例移动成功；新增外部hardlink后拒绝，源与alias原字节保全，目标缺失 | 未变异runtime3项通过；捕获round2仍漏的rename guard；不是CLI rollback资格 |
| 4 | WorkService/status、WorkbookRepo/list/add | schema2下新增列拒绝，只有SQL空白变化接受；合法表上的SQLite整数溢出报STORE_CORRUPT而非WORKBOOK_EXISTS，业务事务回滚 | 未变异runtime6项通过；4个精确变异捕获 |
| 5 | 公共ManagedFs::set_readonly | 根字符串相同、epoch不同的旧句柄不得先chmod；旧根/新根文件原字节及0600权限保全 | 未变异runtime7项通过；1个变异捕获，最终同为Err但原始副作用不同 |
| 6 | CLI add的before_commit精确同步点 | 已完成Store识别后加入外部WAL硬链接，必须在SQLite连接前拒绝；旧Store/外部原件不变，业务行零 | 未变异CLI5项通过；1个变异捕获，原拒绝、变异返回业务成功；失败stdout未执行后续bytes断言，不称已观察外部改写 |
| 7 | WorkbookRepo装入/重放/全局维护 | 两个合法Workbook的冲突publisher声明拒绝；Work final闭包错误时全局索引先停；Remove错误保留提交归属；结构非法flows不投影original | 未变异runtime11项通过；5个精确变异捕获；没有仅凭最终SC判等价 |
| 8 | 完成请求重放与同RID内容重试 | 原始快照结构不合规不投影；合法快照效果损坏仍保留原文；清owner后完成Start可重放；新请求前旧私有原件退役 | 最终未变异runtime17项通过；11个精确变异捕获，Spec/Standards逐项接受 |
| 9 | Start归属缺失时的原文投影 | 完整Started快照仍可作为错误诊断原文 | 仅1项未变异基线通过；后续负控按用户要求暂缓，未计捕获 |

round1–8的`closure.json`、`results.json`、原始stdout、精确diff与完整`*.rs.txt`位于相应目录。共30次精确负控，29次捕获、1次未捕获；后者与round3是同一个变异，不能计成两个已关闭项。每次负控后恢复源码并逐SHA确认，编译错误不当捕获。round9只有基线，未执行负控，不计捕获。

round1两个失败draft也保留：一个误把Start重放的校验要求施加给只读状态入口；一个误要求形状一致的历史原文必须被隐藏。独立Spec reviewer依据protocol §5、repair-design §5/§13确认：本例应检查重放错误的提交归属、STORE_CORRUPT与零效果，不修改已存原文来补requires；original存在不代表语义校验通过，不能据它推进。两次draft不计PASS。

复现时从冻结候选建立独立clone，把某轮`*.rs.txt`恢复到其`closure.json.extra_tests`列出的路径，运行记录中的argv，并用记录的生产diff做负控。测试使用该副本的Cargo注入binary或直接公共runtime API；不注入未变异外部binary。round1/7/8已获独立Spec reviewer复核；round2/3/4/5/6/8已获独立Standards reviewer复核。round5父夹具曾使用普通TempDir，后续round7改为OwnedTempDir正确清理两棵只读Workbook，旧输入与输出不改写。后续新增安全复现按用户要求暂缓；本次T31完成仅按主计划明确豁免范围成立。
