# C004-T09 事前独立窄审

**通过，仅事前准备。** 审阅者 `/root/c004_execution_review` 未修改 observer/规程/消息/freeze，未派消费者、未执行包装 CLI；只写本报告与同名 JSON。实际 T09 消费及 M3 尚未通过。

84 个冻结文件逐 SHA 无漂移，包括原 Work 的 task/project、binary、三成果、完整 C005 原件与现行合同/方法。consumer-freeze SHA 为 `a9a9e6875feadb74b10c5a9890db1c03271dbb324b94690b3d4924e61d20862e`；observer SHA 为 `0fc971e0421eddbf8e448fbf5f467206eb0607038d9c82ce508fbb8adf7dfdc9`。只用 compile(string) 核语法，没有执行 observer 或写 pycache。

observer 只接受 status/result。每次以 SQLite mode=ro/BEGIN 读取五表全部行，比较完整业务对象路径集合、dev/ino/mode/nlink/普通文件 SHA；SQLite 控制载体例外不混入业务原件。原 CLI argv、stdout/stderr、实际退出码与包装观察退出码分别记录；正常路径原 bytes 透传，normal/timeout 的原 bytes 均另存 base64。观察失败或逾期时包装器非零，不把原 CLI 的成功当观察成功。

证据目录在 Home 外；原 observation 存在即拒绝，新写使用 exclusive x，不覆盖旧记录。本轮尚无两个 observation 或 consumer-report。截止沿原18:41:37.324097Z，CLI前重核剩余时间、timeout=min(30,remaining)，查询后快照结束时过期不判成功。超时保留 partial bytes 与前后快照，原CLI exit未知为null；参数/漂移/期限的前置拒绝由实际工具stderr/exit留证，不造已执行快照。规程禁止失败重跑、绕过observer、后台进程或修改Home/产物/C005。

初版 deadline 缺口、task/project漏冻结、退出码混淆与partial raw保留问题已在派发前修正，初版finding另存。新消费者须实际完成两次包装查询、按三refs读与核成果、执行交付首个只读动作，并写自己的C005准备判断。既有running窗口引用原件；此补验不重做文档生产，不制造撤销事件、不增加引擎节点或状态。旧B两条历史查询快照缺失永久保留；事前通过不能称新消费者已完成或F02已关闭。

T08提交 `d8e03491b7bcb60a768a17e6351d607f9c1ffee7` 已独立逐blob读回：21份旧raw的 ccf7 blob、T08提交内容与当前物理字节完全一致。F-C004-M3-01关闭。F02仍待新消费者实际原件与独立增量核；原真人/净收益/LEAK/未知成本留原。T09准备提交与实际治理由Root按任务卡执行。

审阅时点：`2026-10-03T18:13:00.693554+00:00`。
