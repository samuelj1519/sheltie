# C002 实施计划

状态：`active`。基准：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。2026-09-27 用户采用。本文件是当前实施进度的唯一权威；产品语义见根规格、架构与合同。

阅读 [CONTEXT.md](../../../../CONTEXT.md)、[文档地图](../../../README.md) 与 [工程规范](../../../engineering.md)，再按本计划定位任务。问题索引见 [findings.md](findings.md)，已采用机制见 [design.md](design.md)，候选与运行见 [validation.md](validation.md)，最终审查见 [review.md](review.md)，文件范围见 [tasks.toml](tasks.toml)。

## 授权与验收范围

- 2026-09-28 的 M1 预审发现 R01–R19，追加 T18–T31；后续 R20–R24 分别纳入 T31/T33/T34。具体历史任务卡与失败原文从 [固定快照](validation.md#历史证据恢复)恢复。
- 用户豁免 T18–T31 与 M1 的 Linux 原生运行，保留 `not_run`，不记跨平台 PASS。
- T31 于 2026-10-01 按当时 269 项暂缓范围完成。M1 随后获准恢复暂缓验证，只跳过实际触发平台提示而暂停的任务；原 T31 豁免不改写为完整验证通过。
- 2026-10-02 用户采用能力分组流程：复用同闭包完整运行和精确静态证明，用真实 consumer 选择 CLI 组或共享 oracle，影响不明才跑全 workspace。中断、未运行与受控观察副本分开计，不改写正式 mutation 标签。
- M1 按实际 SK01/SK02 例外完成：缺最终 Spec 批准及 215 项额外执行。2514 项记录无漏无重，完整变异与安全验证仍未通过。T16/T17 保持各自门槛；此次文档收敛不改变这些状态。

## 1. 采用与执行规则

package Owner 负责指派每个任务的实施者与未参与该任务修改的 Reviewer；实际姓名或 agent 身份、审查范围和结论写入 validation。Reviewer 不以测试通过代替语义审查。通常一个任务一个提交，提交前按工程规范运行 fmt、check、Clippy、nextest 和任务附加 gate，使用 `Change:`、`Task:`、`Agent:` trailer。T01 已有失败候选 `ceadc465`，本次勘误作为明确记录的纠正提交；不得把首次提交的旧运行冒充勘误验证。未通过局部验证和独立 review 不得标 `done`。

每个行为修复都需要合法例、只改变一个条件的反例、真实 caller 与失败停止路径。摘要、字节、Git 范围和历史响应使用独立 oracle；测试 helper 不得计算自己的期望。开发使用每任务独立临时管理根；不能对真实用户 home 做格式实验。

**格式切换只有一次。** T03 的 WorkLayout、T06 的新事实视图/累计字段、T09 的新摘要、T07 的请求快照/schema 2 属于同一次持久格式切换。T03/T06/T09 先提供纯实现与独立测试，产品调用方在 T07 一次切换，不先把新语义写入 schema 1。T03/T06/T09 当时仅为准备单元；产品接入与验收由后续 T07/M1 负责，实际结果见 validation.md。T07 必须同步全部 caller、fixtures、格式标识、旧库拒绝和文档，不能留下混用分支。准备单元只服务这个已采用修复，不作为未来扩展接口。

### 每个任务的执行循环

1. **定位。** 在表中确认依赖已 `done`，从任务卡列出的目录找真实入口与调用者；用 `git status --short` 记录工作区原状。遇到无关改动不覆盖、不清理。对照 [findings.md](findings.md) 中相应 O/N 项与上游合同，写出本任务的可观察结果。
2. **划边界。** 只改 [tasks.toml](tasks.toml) 白名单内的文件；若上游合同勘误确需新文件，先在本任务中更新白名单、写明原因，再改文件，不能靠跳过 `check-task` 放行。实现前列出合法例、只改变一个条件的反例、实际 CLI/Store/Workbook caller、失败时停止和恢复路径；摘要、字节、Git 范围、历史响应的期望值用独立 oracle。每个测试标 `// Task: C002-Tnn`。所有文件与进程使用本任务独立的临时管理根；不把真实 `~/.sheltie` 当测试夹具。
3. **实现与局部验证。** 先跑受影响测试以确认反例，再修生产调用链。T03/T06/T09 只交纯单元和独立测试，不能把新持久语义接进 schema 1。运行任务卡要求的正反例、真实入口、故障窗口；任何未覆盖的条件记 `not_run`，不能从纯单元推断 CLI 闭环。
4. **冻结待审候选。** 记录 `HEAD`、`Cargo.lock`、特性、平台、测试目录与 `git diff --check`。按任务卡运行附加 gate；改文档运行 `scripts/check-docs.sh`、`scripts/check-specs.sh`；改依赖运行 `cargo deny check`。再运行工程规范 §2.3 的四条全仓门禁。将命令、原始输出路径、退出码、输入闭包与候选文件清单写入 [validation.md](validation.md)，不能只写 PASS。
5. **独立审查。** Reviewer 对照任务卡、合同、正反例和真实 caller 检查待提交 diff；发现缺口回到步骤 2，保留 FAIL 记录。通过后把该任务状态改为 `done`，运行 `scripts/check-task.sh C002-Tnn --staged`，把状态、验证和审查记录一起提交。提交后核对 `git show --stat`、trailer、提交 hash 与工作区；没有得到成功退出码不声称已提交。M1、T16、T17 用各自的完成门槛。

如果上游文档与实现要求冲突，先在拥有该事实的规格、架构或合同中修正，再继续代码；不得添加第二套持久格式、隐式默认字段或临时兼容层。若新证据改变任务依赖，先更新本表并记录原因。任何必需的反例失败、历史字节被改写、旧库被写、根外哨兵被改或未提交最终目录可见时停止当前任务，不标 `done`。

## 2. 任务与依赖

实施状态只在下表维护：`not_run / doing / done / blocked`。T16 与 T17 含只有用户能完成的动作，保持 `not_run` 不是异常。

| ID | 状态 | 任务 | 依赖 |
| --- | --- | --- | --- |
| C002-T01 | done | 固定上游合同、兼容性与任务治理；勘误首次提交 | 人采用 |
| C002-T14 | done | 收紧已校验定义、持久状态验证接口 | T01 |
| C002-T02 | done | StartRequirements 与无副作用 preflight | T14 |
| C002-T03 | done | 单一 WorkLayout 与输出冲突规则 | T14 |
| C002-T04 | done | 管理路径、文件句柄、限额和安全原子写 | T03 |
| C002-T09 | done | 无歧义目录摘要与独立向量 | T04 |
| C002-T05 | done | Workbook 身份、复制核验、只读根与初始化 | T04、T09 |
| C002-T06 | done | 一致事实视图与统计 | T14 |
| C002-T07 | done | schema 2、意图、快照、Work 发布与恢复 | T02–T06、T09 |
| C002-T08 | done | Workbook 事务、幂等与发布生命周期 | T05、T07 |
| C002-T10 | done | 输入发现、授权边界与恢复用法 | T02、T07 |
| C002-T11 | done | article-review 打回意见绑定 | T01 |
| C002-T12 | done | spec-dev 单任务与整体交付闭环 | T01 |
| C002-T13 | done | skill 安装产物自包含 | T10 |
| C002-T15 | done | self 生命周期、CI、MSRV 与发布门禁 | T08、T10–T14 |
| C002-T18 | done | 固定修复合同、接口与平台API门槛 | 用户豁免Linux并保留not_run；macOS门禁与双轴独立review通过 |
| C002-T19 | done | 受管路径、目录句柄与安全文件原语 | T18 done；Linux运行豁免且not_run；[macOS门禁与双轴独立review](validation.md#历史证据恢复) |
| C002-T20 | done | 持久路径与效果的可信装入 | T19 done；[macOS门禁与双轴独立review](validation.md#历史证据恢复) |
| C002-T21 | done | 受限目录枚举、摘要和复制 | T19；[macOS全树观察与复制](validation.md#历史证据恢复)，Linux `not_run` |
| C002-T22 | done | 同句柄输出观察与封存 | T19、T20、T21；[macOS真实COMMIT至封存窗口](validation.md#历史证据恢复)，Linux `not_run` |
| C002-T23 | done | self文件链与purge锁生命周期 | T19、T21、T18合同采用；[macOS门禁与双轴Review](validation.md#历史证据恢复)；Linux `not_run` |
| C002-T24 | done | 请求解析、历史目标与锁内建库 | T20、T21；[macOS门禁与双轴Review](validation.md#历史证据恢复)；并发初始化 follow-up `edb86f1` 及[真实add/install复验](validation.md#历史证据恢复)；Linux `not_run` |
| C002-T25 | done | 统一恢复与准确提交错误 | T20、T21、T22、T24；[macOS全链门禁与双轴独立Review](validation.md#历史证据恢复)；Linux `not_run`；当时交T28/V25的清理与告警已纳入后续T28/T31验收 |
| C002-T26 | done | 发布完整归属与必要sync | T19–T21、T25；[macOS门禁与双轴独立Review](validation.md#历史证据恢复)，Linux `not_run` |
| C002-T27 | done | 同对象删除与完成证明 | T25、T26；[macOS门禁与双轴独立Review](validation.md#历史证据恢复)，Linux `not_run` |
| C002-T28 | done | pending安全清理与只读发现 | T25–T27；[macOS全仓门禁与双轴独立Review](validation.md#历史证据恢复)，Linux `not_run` |
| C002-T29 | done | stats与next同次装入 | T25、T28；[macOS真实CLI交错与双轴Review](validation.md#历史证据恢复)，Linux `not_run` |
| C002-T30 | done | spec-dev重规划旧输入交接 | T18；[真实CLI/Git冷读、完整binary迁移与双轴Review](validation.md#历史证据恢复)；Linux `not_run` |
| C002-T31 | done | 确定性交错、正常窗口与变异登记（含2026-10-01用户安全验证豁免） | T19–T30 |
| C002-T32 | done | M1审查反馈精简与真实caller回归 | T31 |
| C002-T33 | done | 根解析修复与M1存活体的真实caller回归 | T32 |
| C002-T34 | done | 快照业务绑定修复与M1存活项回归 | T33；699项与工程门禁、Standards通过；Spec后续实际平台暂停按SK01授权跳过 |
| C002-M1 | done | 修复后固定候选全链审查（含实际SK01/SK02授权跳过） | T01–T15、T18–T34；[最终验收](review.md)，215项额外执行缺失及最终Spec批准缺失明确保留 |
| C002-T16 | not_run | rc 真实宿主回归 | M1 |
| C002-T17 | not_run | 发布 v0.2.0 | M1、T16 |
| C002-T35 | done | 项目许可统一为MIT | 用户2026-10-02明确授权；元数据/包文件/文档及提交门禁 |
| C002-T36 | done | C002文档及历史证据收敛 | 用户2026-10-02明确授权；固定快照、9常规文件、引用与状态复核 |
| C002-T37 | done | 执行耗时复盘与通用验证预算方案 | 用户要求分析并提交；只记录事实与建议，不采用新门禁 |
| C002-T38 | done | 产品与全代码分析及保持行为的精简 | 用户2026-10-02明确授权；M1完成范围与例外保持原记录 |

## 3. 任务卡

### C002-T01 固定合同与采用边界

Owner：文档与合同；Reviewer：未参与勘误的实现/规格审查者。文件：`CONTEXT.md`、宪章、根 spec、architecture、三份 contracts、engineering、D-033–D-036、本 package、change 索引与任务白名单。

1. 对照 [findings.md](findings.md) 的 O01–O13/N01–N14，逐项确认任务 Owner、正反例与最终验证入口。保留 schema 1 旧数据和旧二进制；新 schema 2、摘要 v2、目录布局和 `cli-result/v2` 只在 T07 一次接入。历史 T26 只修有来源的事实错误，不改完成状态或补造 transcript。
2. 逐字段核对 `RequestIntent`、`ResponseSnapshot`、`effects_json`、pending 归属、状态卡与错误闭集。重放查询必须先于读取当前 Workbook、`@file` 或源目录；冻结副本只在 COMMIT 后从 `pending/.../payload/` 发布；Attempt 目录由 `prepare_attempt` 恢复；remove 从已校验的 `state_json` 判断引用。`EFFECT_PENDING` 要区分当前请求已提交与旧请求阻断新请求，不只记日志。
3. 统一 GF-29/GF-31、新管理根与旧数据保留、OS 账户记账不等于独立真人认证、`self` 不写 shell rc。D-036 使用符合全仓 `unsafe_code = "forbid"` 的安全 API。修正空 proposed 目录与检查器的关系，以及 archive、progress、索引中的采用/任务状态。
4. 运行 docs/specs、四条 Rust 门禁与 `scripts/check-task.sh C002-T01 --staged`，把原始输出与候选闭包交独立 Reviewer。反例：旧 Store 被迁移或清空、同字段双解释、pending 按年龄删除、已提交请求因当前文件缺失不能重放、提交前最终目录可见。任一出现，T01 留 `doing`，后续任务停止。

### C002-T14 定义与状态的可信构造面

Owner：`sheltie-core` 已校验定义和 `runtime/store/read.rs`、`workbook_repo.rs` 的装入 caller。文件：core flow/workbook/state/path/ids、runtime decode caller、API tests。

1. 找出 `Manifest/FlowDef/NodeDef` 的 public 可变字段、反序列化入口和测试专用公开构造。把 Raw DTO 留在解析边界，已校验定义只读；现有 public caller 通过合法构造/访问方法迁移，不复制整份 parse 校验。
2. 建立一个持久 Work 行验证入口：检查行 `work_id`、revision、冗余 status 与 `state_json` 的身份和关键状态组合；损坏时返回带行定位的 `STORE_CORRUPT`。`Store::read` 和 Workbook remove 的引用扫描复用它，不跳过任何损坏行。
3. 用当前样例和 public API 作合法例；反例只改 definition 非法字段、row 身份、revision 或 status 中的一个条件。外部不能构造非法已校验图；损坏行不能令 remove 放行。编译和受影响测试过后审查。不普遍包装 String，不引入 trait/typestate 框架。

### C002-T02 StartRequirements 与 preflight

Owner：core Graph/decide、runtime service 与 CLI workbook show。文件：对应源文件和 API/CLI tests。

1. 在 core 实现 `start_requirements` 与 `validate_start_inputs`；decide、runtime preflight、show 共享同一有序键集合。show 文本与 JSON `flows[].start_inputs` 都暴露该列表。
2. 新请求的 Workbook/Flow、WorkName、缺/多 key 和 `@file` 读取错误都在序号分配、目录物化、建新 home 之前拒绝。为 T07 保留“先用未读文件的用户参数查重放，再做新请求 preflight”的顺序；T02 不自行切换持久请求格式。
3. 真实 CLI 正例：two-step show 返回 `topic`，完整输入 start 成功。逐个反例只改缺 topic、多 key、非法名字、缺 Workbook、缺 Flow 或不可读 `@file`；比较 home、sequence、works、requests 与调用前相同。缺 CLI 必填参数退出码 2；补齐唯一缺条件后同请求可成功。任何确定性拒绝烧号即失败。

### C002-T03 WorkLayout 与输出合同

Owner：core `work/layout` 与 Flow 编译规则。实际持久 caller 切换归 T07。

1. 用一个 WorkLayout 函数生成 Occurrence/retry 两维路径，把引擎文件放 `engine/`、worker 文件放 `outputs/`。按 Workbook 合同 §3.2 校验输出相对路径：可移植 ASCII、无重复/祖先关系、ASCII 大小写折叠后无别名。
2. 纯测试核对嵌套合法输出、`draft#2.1`、`outputs/brief.md/out`，以及 `out`/`out/sub`、`OUT.md`/`out.md`、`Out`/`out/sub`、非 ASCII 的单条件拒绝。`engine/stats.json` 与 `outputs/stats.json` 必须分别可表示。
3. T03 只交路径单元与独立 oracle，不切 schema 1 的实际目录。T07 接线后再用真实 CLI begin→写输出→submit 验证 O12；空执行不得把 engine.stats 当 worker 产物。不要加入 LegacyV1 或默认布局推断。

### C002-T04 根内文件操作与限额

Owner：runtime 的 `home`、`observe` 与服务层文件边界；T15 接 self caller。文件：runtime home/observe/service/workbook_repo/selfmgmt 的文件 helper 与 tests。

1. 从 Home 派生并限制所有 managed 路径；查最近存在祖先、父目录、叶文件、链接数与对象归属。读取、摘要、封存使用同一打开的句柄和实际读到的字节数；原子写使用独占唯一临时名。删重复 helper 和固定 tmp-pending 路径。清理不跟随软链，不直接信任 state 中的裸路径。
2. 正例覆盖嵌套 worker 输出与显式 `@file`。单条件反例覆盖 `works/bin/pending` 父软链、叶软链、状态卡临时软链、观察后替换、超限文件；核对根外哨兵字节与权限不变。每个大小上限测恰好上限接受、多一字节拒绝。
3. COMMIT 前观察拒绝必须保持 Store 不变；COMMIT 后封存或投影失败保留已提交状态，T07 用 `EFFECT_PENDING` 与原响应接通。分别在 macOS/Linux 验证目标文件 API；不能把“再次 canonicalize 后重新打开路径”当作无竞争证明。

### C002-T09 目录摘要 v2

Owner：runtime Workbook 摘要单元与 core 摘要类型。T07 才切换持久 caller。

1. 复用 T04 的受限目录枚举与文件句柄，按存储合同 §5.1 的精确字节流实现 domain prefix、BE64 文件数/路径长度/内容长度、路径字节排序与一次 SHA256；流式读取并按实际字节计数，不整树缓存。
2. 独立手工字节向量给出期望摘要。反例保留旧 `za`/`zb` 边界碰撞：v2 必须不同；目录枚举顺序变化摘要不变，路径/正文一字节变化摘要变化，链接与超限文件拒绝。
3. 若仍以 `Sha256Hex::of_bytes(finalize())` 二次摘要、用生产 helper 生成 expected，或把 v2 写入 schema 1，本任务失败。旧算法不保留为 T07 后生产 fallback。

### C002-T05 Workbook 身份与本机边界

Owner：WorkbookRepo、Home、Store 初始化入口与 OS principal。文件：对应 runtime/core/CLI 与集成测试。

1. `load` 核登记 id/version/digest；add 和 start 在复制后对最终副本重新 parse/compile/摘要核验，登记最终副本身份。version 拒绝 `.`、`..` 与内部保留名；Finder 元数据准确报文件名。只读权限最后设置到根目录和文件；删除前先核归属。
2. 用 D-036 选定的 `users` 安全 API 取 effective uid/账户名；先核目标平台、MSRV 与许可，再加依赖。设置假 `USER` 不得改变 audit 或 gate 主体；无 UTF-8 账户名时按合同退到 `uid:<数值>`。
3. T07 前生产入口继续按 schema 1 的旧摘要核验，不向旧库写 v2；T07 切换时删旧生产路径。真实 CLI 正例：新 home 合法安装、冻结、终态查询和删除已装 Workbook 后 Work 可读。反例：verify tampered 后 start 拒绝、复制间源变化形成不合规副本时拒绝、manifest 身份不符、version 点段、只读不存在 home 无写；完整合法副本登记实际最终字节。不能把只读权限说成同用户不可绕过的认证。

### C002-T06 状态与统计视图

Owner：core state/render 与 CLI 事实视图；T07 接持久字段与实际 CLI caller。

1. 用一份纯事实视图驱动文本/JSON，包含 fail_reason、完整 `ArtifactRef`、同形 `next` 与来源 node+edge。stats/next 用同一次已校验状态加载。累计受阻在 core 状态转换发生时记录，不从当前状态倒推。
2. 用独立手写状态作 oracle：非零时间、次数、失败原因在两种格式一致；NoLegalEdge→cancel 不减少 blocked；同来源不同边能区分；去掉 reason/digest/bytes 时断言失败。不要让两个 render 函数互相生成期望。
3. 新持久字段及 CLI 接线留给 T07，不在旧 state 中猜默认值。T07 后再跑终态、失败与 NoLegalEdge→cancel 的真实 CLI；之前不关闭 O13/N07。

### C002-T07 请求、schema 2 与 Work 恢复

Owner：runtime request/service/store、core 新持久字段/layout caller 与 CLI 响应。文件：这些入口、所有受影响 fixtures、replay/crash tests。

1. 一次建立完整 schema 2：DDL 与 `user_version` 同事务；旧库在任何写 PRAGMA 前只读拒绝并比较旧文件字节；首次并发建库由管理根写锁串行。T03/T06/T09 的布局、状态事实、摘要在同一提交切到所有产品 caller；同步 fixtures、响应 `cli-result/v2` 与文档，删旧生产分支。
2. 按存储合同 §2.1 构造带完整目标的 `RequestIntent`：先解析原始用户参数，命中 `request_id` 后不读当前 Workbook、`@file`、源目录或输出；锁内复核和事务内再次去重。完整 `ResponseSnapshot` 与意图、状态、审计、效果同事务保存；CLI 只渲染快照，不回读当前 Store 拼历史 reply。固定 canonical JSON 独立字节/摘要向量。
3. 先独占创建 `pending/<id>.owner` 侧车，再创建 `pending/<id>/payload/`；预提交阶段只写私有 payload，COMMIT 后按 `effects_json` 发布到最终目录。`attempt begin` 的 `prepare_attempt` 效果先建目录再写 brief/stats；状态卡只从最新状态刷新。效果全部完成后标 `published = 1`；已完成 begin 的显式重放仍可按登记字节补缺失的历史文件，不重做已完成 submit 的封存。恢复核归属、Store 引用、摘要和历史字节；无 Store 引用的 pending 只在核合法侧车后清理，不能按年龄删。
4. 真实 CLI 正例走 start/begin/submit/gate/fail/cancel，并逐字段比对首次响应与重放（除 `replayed`）。单条件反例至少覆盖：跨 Work request-id、只读命令带 request-id 退出码 2、历史 Work 前缀后来变成歧义仍重放原目标、`@file` 内容变化或消失后的 start 重放、Workbook 删除后的 start 重放、`--summary @file` 源文件变化或消失后的 submit 重放、输出变化后的 submit 重放、cancel 后旧 submit、后续状态下旧 stats 缺失按原字节补齐、历史文件已有不同字节时不覆盖、旧请求不得回退卡、两个写者竞争。恢复旧效果阻断新请求时，核对新请求未提交的响应形状。
5. 本任务的每类 Work 写意图用真实子进程退出覆盖 COMMIT 前、COMMIT 后、发布前/后；对照 Store 行、最终目录、pending 归属、原件字节和同请求恢复结果。Workbook 意图的相同窗口由 T08 补齐。旧库字节变化、历史字节被重算、pending 被 tmp 清理、未提交最终目录可见或 schema 2 混用旧布局/摘要/outcome，任一出现即停止，不标 `done`。

### C002-T08 Workbook 事务与生命周期

Owner：WorkbookRepo、Store request/commit/read 与 CLI workbook。文件：对应源文件、failpoints/tests。

1. 把 add/remove 的 Workbook 行、审计、请求快照和效果登记放在一个事务。add 在本操作的 pending payload 复制后重新校验身份、Flow 与摘要；旧 add 的未完成效果在同一写锁下先恢复。remove 在事务内遍历并验证全部 Work 行，再从 `state_json` 判断非终态引用，不用冗余 `status` 列预筛。
2. `delete_dir` 记录旧生命周期的准确路径、归属与删除前 digest；提交后只移走/删除该对象，完成删除后写 `.deleted` 持久标记。两处原件都缺而无合法标记时结果不明，返回 `EFFECT_PENDING` 并停止，不猜成功。完成效果不因旧 request-id 重放再次执行；同 id/version 的新 add 是新生命周期，旧 remove/add 重放均不能碰它。
3. 真实 CLI 正例：add/remove 原响应逐字段重放；源目录变化或消失后的旧 add 仍返回原快照。反例：并行 add 不删对方 pending、提交后 rename 失败时 list/show/verify 从受保护 pending 准确读取且 start 无锁预检不误报缺失、remove→新请求 add 同版本→重放旧请求、损坏引用行停止、空载荷/重复/跨操作冲突按全局 request-id 去重。add/remove 各用子进程退出覆盖 COMMIT 前后与发布前后；尤其验证删完 payload、写 `.deleted` 前被杀时按“结果不明”停止，不把外部缺失判为成功。每个例子分别核状态、数据库、原件字节和目录归属。用同步点制造真实并发，不串行 spawn→join。

### C002-T10 输入发现与协调者行为

Owner：`skills/sheltie/SKILL.md`、CLI show 的对外用法与协议使用说明。T02 提供 show 字段，T10 完成协调者行为。

1. 在 skill 中写清发现路径：用户未指定 Workbook 时列出可选项；已指定但未安装时准确报告，不静默替换；show 读取 `start_inputs`，已有输入直接使用，只对缺项询问。Work 推进只从当前 `next` 选择；发现、add 与 start 有独立入口。
2. 写清 `request_id` 应在写调用前保存；Work/Workbook 写操作支持，self 不支持。重放的历史 `next` 不代表当前状态，续接先 `work status`。human/gate 按实际授权执行，代执行记录事实，OS 身份不等于真人证明。
3. 离线验收：完整意图自动开工并能从当前 next 续接。反例：缺输入、指定未安装、已授权仍重复问、使用旧 next、把 OS 账户当真人认证。CLI 行为与文案分别核对；真实宿主交互留给 T16，不能以离线脚本替代。

### C002-T11 article-review 回环

Owner：`examples/article-review/flows/default.toml`、`instructions/draft.md` 与场景测试。

1. 给 draft 增加 `required = false` 的 `review.verdict` 输入；说明首次没有意见，沿 back 返回时读取最近成功 review 的产物并据此修改。只传文件路径，不内联历史正文。
2. 真实 CLI 跑首次 draft→review→back→第二次 draft；第一次 begin/brief 标“尚无”，第二次都指向正确的 review 产物及来源 Occurrence。若只能靠聊天补传意见，T11 不通过。

### C002-T12 spec-dev 交付闭环

Owner：`workbooks/spec-dev/flows/default.toml`、instructions、templates、checklists 与独立临时 Git/CLI 回归。

1. 固定 Work 开始时的整体原始基线，另为每个任务记录本任务基线/候选；任务 verify 只核当前任务改动及其占位，允许合法未来任务占位。改 plan 后，最终 review 仍对原始整体基线检查全部交付。
2. 把 plan-review 条件绑定到获批的 plan/spec 版本并沿 Flow 输入交给 scaffold/implement/verify；方案修订使旧批准失效。修复 escalate→scaffold 和 verify 升级返回的边、输入、说明书三件套。不要把 Spec/Plan/Git 状态塞进引擎。
3. 用独立临时 Git 仓库和真实 CLI 证明五组闭环：两个任务改不同文件；两任务共用文件且留未来占位；附条件批准进入 scaffold/implement/verify；verify/scaffold 升级后按人意见继续；任务 1 完成后改方案，最终 review 仍包含任务 1。每组固定 Git 提交/文件集合的独立 oracle，并核 brief 输入绑定。真实 agent 交付质量留给 T16。

### C002-T13 自包含 skill 交付

Owner：`skills/sheltie`、交付生成脚本、根 README、`scripts/check-skill.sh` 与隔离安装 fixture。

1. 由仓库单一权威合同生成发布 references；仓库内可保留链接，但打包/安装后 references 必须是交付目录内可读取的文件。README 给出与实际发布步骤一致的安装方式，不依赖用户保留源码路径。
2. 在隔离临时目录安装，逐个解析本地链接、相对路径与引用命令；移动或移除源码树后重复检查。故意漏同步一份 reference 时 check-skill 必须失败。不得触碰真实宿主配置。

### C002-T15 self、CI 与发布质量

Owner：runtime `selfmgmt`、CLI self、Cargo/toolchain/deny、`.github/workflows`、`scripts/check-specs.sh` 与根 README。

1. 删除 `--modify-path` 写 shell rc 的入口，`--json` stdout 只输出协议 JSON，self 带 request-id 退出码 2。按固定 tag 解析 `--version`、manifest 与资产；下载、摘要、解包、替换和 rollback 分别核失败窗口。所有 managed self 路径使用 T04 的根内文件边界与同一写锁。
2. purge在同一锁内删除用户数据但保留空管理根与原`.lock`；等待者沿同一锁继续，合法install/add可初始化，旧Work命令返回NOT_FOUND且不重建。另覆盖锁外意外替换根/锁的身份复核与整体重试。加入self/Work并发和purge等待者测试。安装/更新提示明确Store schema 2与旧数据保留，不能误导用户只换二进制即可降级。
3. 让治理 job 获取需要的历史 tags/commits；check-specs 分开验证已发布 release 与 active target/RC，不要求开发中的版本已有 tag。release 依赖同一 SHA 的质量 job。实际运行 MSRV 1.85 locked gate；失败时用证据决定修依赖或提高声明，stable 通过不能代替。
4. 用本地 release fixture 测指定版本、latest、rollback、clean home two-step 与 `cargo dist plan` 的真实发布形状。反例测校验失败旧二进制不变、latest 漂移不混包、伪造路径不越界、质量失败不能发布、缺历史给准确诊断。四平台已发布资产取证留给 T17。

## 4. C002-M1 固定候选全链 review

Owner：未参与 T02–T15、T18–T31 被审代码实施的独立 Reviewer；实施者只提供候选、证据与逐条答复。2026-09-28的审查结论为“需修改”，不构成M1关闭；最终候选与范围见 review.md，历史预审不构成当前批准。

1. 冻结候选 commit 与输入闭包。对 O01–O13/N01–N14 建矩阵：每行写 finding、修复 commit、正例、单条件反例、真实入口、原始 run、结果、Owner 与剩余风险。没有独立 oracle、真实 caller 或失败停止路径的项保持未关闭，不由 task `done` 或测试数量推成 PASS。
2. 在同一候选运行 fmt/check/Clippy/nextest/deny、docs/specs/core-vocab/tests/skill、MSRV 与 `cargo dist plan`。并发测试先同时启动并用同步点制造交错，再 join；摘要、Git 范围、状态卡字节用独立 oracle。补 Work/Workbook 的 COMMIT 前后、发布前后故障窗口；删除后但完成标记前被杀的“结果不明”必须准确停止并返回已提交错误，不能记自动恢复 PASS。core/runtime 突变按能力处置每个存活体，不机械要求每个都加测试。
3. Reviewer 逐项检查不变式、三 crate 边界、合同与实际 caller、证据原文和实现者答复。审查后若修改影响候选输入闭包，按影响链重跑并记录新 hash。review.md 结论只用“通过 / 需修改 / 阻断”，M1 通过只证明源码与离线闭环，不能代表 Host 或发布。
4. 同时关闭R01–R24；R21是T33确认的根解析错误，R22–R24是T34确认的原响应、状态及节点资源绑定缺口，详见[审查记录](review.md)。按 [验收矩阵](validation.md#验收矩阵) 记录每行修复commit、合法例/单条件反例/真实入口/独立oracle/原始run/Reviewer/结果。完整窗口、突变存活体处置或必需平台证据缺失时保持未通过；用户明确授权的实际平台暂停及Linux例外单独列明。不能以局部task done或历史464全绿替代。

## 5. C002-T16 rc 真实宿主回归

Owner：用户/真实宿主操作者；实施者准备绑定 M1 候选的 rc 二进制、独立管理根、自包含 skill、逐命令记录模板与清理说明。只有用户/Host Owner 能完成的动作保持 `not_run`，不能用合成图替代。

- 未指定/未安装 Workbook、缺输入时正确交互；已给信息不重复问。
- two-step、article-review back、gated-release；人工节点由人执行并记录，gate 展示产物后取得实际批准。
- 关闭并重开会话，仅通过状态查询继续；错误请求后正确恢复，无第二套进度。
- spec-dev 完成至少两个任务、一次人工条件与一次改方案，最终覆盖整个原始需求。
- 实测 Finder 与完整性；记录逐命令响应、人工操作、产物摘要、宿主 usage、耗时及质量结果。usage 不可得就记录缺失，不能记成 0。

执行顺序：先记录 rc SHA、Host 版本、skill 交付 hash 与临时 home；再按上面五类场景逐步执行，保存请求/响应、人工授权与产物摘要；最后重开会话从状态继续，并核对 usage、耗时、质量与缺失项。若要声称节省成本，另做同任务直接 agent/skill/Sheltie 对照；没有对照只报告绝对观测，不阻塞可靠性修复的客观结果。宿主失败回到对应 Owner 修复，不用一句“模型没遵守”关闭问题。

## 6. C002-T17 发布

Owner：发布操作者；发布动作需要用户授权，且 M1 与 T16 全部必需项通过。授权前先准备可复核的发布候选、CHANGELOG、release record 草稿与四平台资产/manifest/checksum/quality 的同 SHA 对照。新管理根安装、指定版本更新和 rollback 用独立 home 实测；schema 1 数据保持原样，旧 binary 只配旧 home 查旧记录，新 home 不被旧 binary 误写。

取得发布授权后才执行对外发布与 package 生命周期变更。release record 写候选、门禁、Host 证据、兼容限制与已知问题；发布失败不标 `completed`。T16 未运行或不通过时 T17 保持 `not_run`，不能把本计划或离线审查写成产品发布成功。

## 已完成修复任务的维护入口

T18–T31 的原逐步实施手册与 V01–V33 样例已归档。本表保留维护入口和关键义务；任务状态以第 2 节为准，详细正反例/原运行见 [validation.md](validation.md)。发生新行为修复时仍须按第 1 节补真实 caller 和独立 oracle，不套用旧 PASS。

| 任务 | 入口与责任 | 关键义务 |
| --- | --- | --- |
| T18 | 上游合同、架构、D-035/D-038/D-039、文件/SQLite API | 先采用 purge/只读控制/参数错误合同，核 macOS 与 MSRV 可行性；Linux not_run |
| T19 | runtime fsx/Home | 受管路径、目录/普通文件句柄、原子写、sync、类型/链接拒绝 |
| T20 | Store/load/effects | 持久身份、路径与整组效果可信装入，错误时不改坏事实 |
| T21 | Workbook/digest/observe | 受限树、framing 摘要、独立向量、exact/+1 和最终副本身份 |
| T22 | submit/observe/seal | 观察句柄跨 COMMIT；再次核原字节/nlink，恢复只认登记引用 |
| T23 | selfmgmt/CLI self | install/update/rollback 文件链与 purge 原锁生命周期 |
| T24 | 请求解析/WriteSession/Store/CLI | 历史目标与 @file 查重优先，锁内按操作限定初始化 |
| T25 | recovery/load/协议错误 | 两服务统一效果；自己提交与旧 A 阻新 B 的完整响应归属 |
| T26 | publish/owner/同步 | pending/final 完整闭包、必要 sync 失败不 mark |
| T27 | delete/完成标记 | 同对象移入与删除；无合法完成证明时停止，新生命周期不动 |
| T28 | pending/定位与清理 | 全 Store 引用、合法孤儿、只读有界重读、独立 cleanup 告警 |
| T29 | stats/next caller | 同一装入快照与真实 writer 交错 |
| T30 | spec-dev Flow/说明/模板 | 被审镜像、原始整体基线、累计 verify 与 fresh worker/Git 正反例 |
| T31 | 全链测试/并发/变异与 R20 | 真实同步、exit70/SIGKILL、独立 oracle、精确逐 ID 核算与明示缺失 |

## M1追加修复 C002-T32

Owner：Codex；Reviewer：未参与修改的Spec/Standards Reviewer。承接本轮M1中已确认的接口冗余、Store合同冲突、测试辨别力缺口。范围包括runtime源码、对应真实caller测试及本package；上游Store合同只勘误CAS停止行为。补测采用原合同独立值，不为每个存活体镜像实现写测试。退役接口须核全仓consumer，保留历史变异ID与删除理由。按Rust四门禁、deny/MSRV/文档/skill/dist门禁及独立review完成任务提交，再固定M1变异输入。M1仍负责全部当前已完成任务的最终闭环，安全跳过仅适用于实际平台拦截。

## M1追加验证 C002-T33

Owner：Codex；独立Reviewer：Spec/Standards Reviewer。本任务集中补M1逐ID审查确认的真实caller测试缺口，保留独立oracle与合法对照；必要观察点只在既有failpoint feature下生效，不引入新的业务状态或生产协议。新候选改变测试输入后，旧部分变异原文保留，不套用新输入PASS。完成工程门禁、独立审查与任务提交后，重新冻结M1完整验证输入。

## M1追加修复 C002-T34

Owner：Codex；独立Reviewer：Spec/Standards Reviewer。R22由未变异T33候选的真实CLI复现：历史remove快照的合法id/version改为与audit不符后，业务正确拒绝，但错误仍附错误的成功`original`。修复runtime原响应投影的请求、审计与业务绑定；同一义务覆盖`pending_original`。R23/R24的真实CLI又确认Fail重放接受不可能状态、Begin重放接受冻结节点未声明的资源；core提供现有状态/资源规则的同一纯函数，runtime只核历史记录与这些事实，不重建当前状态或复制业务决策。core源码改变后不再复用旧720项core变异，重新执行core/runtime完整inventory。保留已提交身份、冲突优先序及合法效果失败的原响应，不修写坏Store、不让CLI重建业务规则。复用并分离现有可信装入规则，避免第二套校验或状态；补真实CLI单字段反例、合法I/O失败对照及存活项必要回归。工程门禁与独立review通过后提交；旧冻结变异原文保留为历史，新输入按影响链重跑，不复用改变依赖闭包的PASS。实际平台安全跳过与Linux/T16/T17边界不变。


## C002-T35 MIT许可

仅修改项目自身许可声明及Apache许可文件；三个crate继续继承workspace许可并携带MIT正文。第三方许可与依赖允许清单保留各自声明。Task文件范围以tasks.toml为准，验证三个Cargo包的metadata和package --list、文档治理及工程规范提交门禁。该元数据变更不改写历史M1输入或验证结果。

## C002-T36 文档收敛

只合并本package已采用设计、完成任务入口与最终验收摘要，并修正D-039及MVP计划的历史引用；原文固定在validation.md所列归档，不改源码、测试、任务验收范围或T16/T17状态。按工程规范提交门禁与文档治理验证，MIT另属T35。


## C002-T37 执行效率复盘

将归档计时、根因与通用方案提交为文档。区分日历、运行区间及并行命令耗时；保留真实修复与验证价值、SK01/SK02及未执行边界。只修改本package记录和通用指南，不修改源码、测试、钩子或现行门禁；验证文档治理、任务范围和工程规范提交门禁。

## C002-T38 产品与代码分析及精简

Owner：code-simplifier负责生产Rust源码，Codex负责规格分析、逐处diff复核与验证。基准为`4b86279`，初始工作区干净。阅读当前规格、架构、合同、C002设计/验收和三个crate全部生产源码，沿consumer读取相关测试并核全仓用例索引，分别评价产品方向、C002实现覆盖、Rust工程实践与复杂度。分析和结果只追加本package现有文件；架构选型、状态校验表述与协议响应字段若偏离现有C002快照和真实caller，作事实勘误，不改公开行为。

只合并有相同合同的重复组装/校验、移除已证明不可达的内部兜底与无消费者实现。保持公开协议、错误字段与优先序、持久格式、next顺序和文件I/O顺序；不删除根内句柄、整组严格解码、锁、sync或恢复归属校验，不实施proposed方案。功能缺陷单独记录，不混入保持行为的精简。测试和快照保持原字节。

先复核diff与消费者，再运行fmt/check/Clippy/nextest四门禁、deny、docs/specs/core-vocab/tests/skill、MSRV 1.85 locked及dist plan；保存本候选输入和完整输出。使用显式基准运行`check-task.sh C002-T38 4b86279 --staged`，另核内联测试与外部测试/快照零改动。既有M1与变异结果仍绑定历史候选，不给新源码复用PASS，不重启历史变异流水线。任务完成以本次分析、精简、复核与验证为准，不新增M1/Host/发布通过声明；本次不自动提交或发布。


### C002-T38 提交授权与证据保存

本任务的上述“未提交/不自动提交”是此前工作区交接事实。用户随后明确授权提交；本提交保留该任务实际源字节和对应原运行，原文已按README索引压缩归档，不以当前最终运行替代早期候选证据。
