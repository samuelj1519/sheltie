# C002-T48 独立审查

结论：**需修改**。合同期望、真实 CLI 对照、窄 helper 的目标辨别力通过；新 CLI 用例尚缺当前 plan 明定的业务文件字节不变 oracle。17 项新增突变实际 caught 原文有效，输入闭包限制须明确，未核完的 load 两项不预判。

Reviewer：`Codex /root/oracle_review`，未参与 T48 设计、测试或生产实现。只读仓库与原输出，不编写 oracle，不构建共享 target；仅写本报告。审查时 HEAD `d27816be3e919b8183abe04b7f7ca4f457046231`。

## 必须修改

F-T48-01：`crates/sheltie-cli/tests/snapshot_qualification.rs` 两条真实 CLI 用例只比较五张 Store 业务表完整行；第一条没有业务树快照，第二条仅核冻结路径 absent、保留目录 is_dir。它们不能证明错误重放未改 instruction、artifact、状态卡、历史 brief/stats 或被保留的冻结副本原字节。依据 active plan T48 的“完整Store/业务文件不变”“保留拒绝前后行/字节”和 storage §3.2、protocol §5 的坏历史不修写要求。需在本新增测试文件内对完整业务文件集合及原字节做拒绝前后独立快照；SQLite 主库/WAL/SHM 与引擎锁控制文件按既有例外单列，不把它们与业务字节混成物理无写声明。

## 范围与静态结论

三个 runtime 文件在 HEAD 的整段原源码后追加 cfg(test) 模块；原生产字节全部是当前文件前缀，比 AST 相同更强。新增 CLI 文件仅是 integration test。没有生产语义扩展或第二事实来源，INV-1–INV-7 不受此次追加影响。

审查字节 SHA256：store/read `9e1c319a9b8bb67511fe7abe52ca4bd316f3459f80e0c9ec5855613f3bdcd3fc`；effects `7c8fad349738bdf64e883162ace75e753cc4489f6c01f16647920402a768ab9f`；load `513907d5809a9ce7dd416cfffcf79797b95d1df0f6ae3443224833c0c72be8b8`；snapshot_qualification `33806b14e4183210e91af28ec55ca7f130972ba372471d20a5a0a2d17720a6be`。

- CLI 业务绑定：真实 Workbook add、start、begin、submit/fail；合法历史请求重放先通过。start 同时改 Reply/data requires 为有效但未声明的宿主条目；submit/fail 使用另一条真实 succeeded/failed Attempt，避免单纯不存在身份早退。submit 输出 digest 同步改 Reply/data 保持合法 64 hex。拒绝 expectation 来自 protocol §5、storage §3.2 的 audit/冻结事实绑定，而不是被测函数计算。准确检查 EFFECT_PENDING/STORE_CORRUPT、committed、request_id、无可信 original/revision、全表原行。
- CLI 快照解码顺序：data work_id/work_dir 单字段与 Reply 矛盾，合法控制先重放；移走 frozen 后要求 data/Reply 矛盾先于冻结读取报错。消息 `data与Reply` 区分准确优先序，不能只靠 STORE_CORRUPT 把“冻结缺失”当成功检测。第一例的业务事实与冻结定义保留，第二例主动移走副本是受控条件，缺完整字节快照见 F-T48-01。
- metadata 纯合同：四项 immutable 字段单改拒绝，published false/true 均接受。实际 callers 是 WorkService 请求窄资格→完整效果行、WorkbookRepo 同链中的 check_row；手写 strings/metadata 是字段一致性单元输入，不代表完整 ResponseSnapshot 已合格，不模拟真实并发写者。它与合同一致性约束直接对应，不因简单字段比较就认作镜像测试；不声称覆盖完整 caller 并发事件或任意外部 SQLite 写者。
- Publication 纯检查：合法 state/Command/ops 来自真实 Repo.add+Work.start 的已提交记录；合法检查先通过。每项只改一个 pending component 或 Command/final/owner/digest 事实，手写坏值不生成 expected。validate_effect_shape 的种类/数量先决条件保留；三段 pending 的前缀/叶/UUID 条件及 Start 身份检查由真实目标突变捕获证明触达。仅纯 check_work_effects 证据，未执行坏批次恢复 I/O，也不替代原响应资格/恢复整链。
- load 纯检查：从真实 two-step 成功 outline submit、running summary 取得合法状态并首先接受。namespace 同时改 Work 起始引用和 outline 冻结输入，保持二者一致，专门检起始命名空间；required 场景同时去掉上游输出和下游已绑引用，保持 None/None 形状，专门检默认 required 约束。这是同一绑定事实的联动改动，不能记为两条独立单字段 CLI 反例。validate_work_paths 不先检查 succeeded 输出完整性，因此该 None/None 能到 required guard；更上游完整 Store/state 资格可能先拒绝，故其捕获只归此私有原语，不声称真实 CLI 可以在该非法历史上走到该 guard。after 仅证明原 state_json 未被纯调用写回。

## 已读取动态原文

前序 G01 普通94消费者、19 Missed 保留历史。新增核心合同 baseline `b092b8b7-1751-4e99-bc7d-f16dac359f7c` 为3/3，559因过滤未选；Publication 有效 baseline `1424da77-ab6d-4c08-91d7-ff79e7d72110` 1/1；load corrected baseline `120ecd61-6ab3-4c18-9103-94fff491a6bd` 1/1。这5个函数不等于5个全链场景，具体循环条件按上节说明。

Publication 初次 InputValue/StartArgs 编译失败不是红；load 初次把 upstream 设 optional、downstream required 触发 FlowInvalid，不是产品 red。原文件保留，corrected baseline 属合法 fixture 修正。

`mutation-g01-oracles/result.json`：exit0、240.299s、baseline Success1、CaughtMutant17。逐一读取原 log：Publication7项在 pending/Command/final/owner/digest 断言失败；metadata4项在 immutable 字段断言失败；services4项经真实 CLI 捕获；snapshot2项准确以“改报冻结缺失而未报 data/Reply”捕获。services Submit OR 两项虽然仍在效果阶段报 STORE_CORRUPT，却返回伪造 original/revision，新增无 original 的断言准确拒绝，未把最终错误码相同当等价。

该运行记录 `source_unchanged=false`。独立比较 selection 中所有源码哈希，**唯一漂移为 load.rs 新测试模块**；本轮17的选择不包含 load 新测试，目标生产与选中四条 oracle 未变。17项证明属于已执行 scratch 的冻结闭包，不能直接登记为当前整个 T48 稳定候选通过。须保留准确旧/新输入对应及 load 两项的独立增量完成原文；生产/选中 oracle 漂移须停止重跑，不能把 hash false 静默改 true。

准确 argv/过滤与 source SHA 在 `mutation-g01-oracles/{result,selection}.json`，完整 log 在 `mutants.out/log/`。本 Reviewer 审计原文，没有另行构建或重跑。

审查不批准旧 e3eea89、不关闭全部215、T43/M2或未结束完整工程门禁。用户明确范围为全自动、macOS aarch64；未恢复其他平台，无人 trial 不得改记真实价值 PASS。
