# C002 实施计划

状态：`active`。基准：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。2026-09-27 由用户采用，本计划是当前实施进度的唯一权威。

先读 [spec.md](spec.md) 与 [design.md](design.md)，按 [findings.md](findings.md) 逐项关闭。保留旧 C002 的任务 ID 便于引用，但下列范围、依赖和兼容策略取代旧卡片。任务白名单见 [tasks.toml](tasks.toml)，跨会话交接见 [progress.md](progress.md)，执行证据记入 [validation.md](validation.md)。

## 1. 采用与执行规则

采用者指定每个任务的具体 Owner 和独立 Reviewer；下表的角色是职责，不预先绑定模型。T01 建立 tasks.toml 文件白名单、允许测试修改的规则、progress 和验证记录。一个任务一个提交，提交前按工程规范运行 fmt、check、Clippy、nextest 和任务附加 gate，使用 `Change:`、`Task:`、`Agent:` trailer。未通过局部验证不得标 done。

每个行为修复都需要合法例、只改变一个条件的反例、真实 caller 与失败停止路径。摘要、字节、Git 范围和历史响应使用独立 oracle；测试 helper 不得计算自己的期望。开发使用每任务独立临时管理根；不能对真实用户 home 做格式实验。

**格式切换只有一次。** T03 的 WorkLayout、T06 的新事实视图/累计字段、T09 的新摘要、T07 的请求快照/schema 2 属于同一次持久格式切换。T03/T06/T09 先提供纯实现与独立测试，产品调用方在 T07 一次切换，不先把新语义写入 schema 1。T03/T06/T09 完成只表示准备单元完成，O07/O12/O13/N07 的产品闭环仍待 T07/M1。T07 必须同步全部 caller、fixtures、格式标识、旧库拒绝和文档，不能留下混用分支。准备单元只服务这个已采用修复，不作为未来扩展接口。

## 2. 任务与依赖

实施状态只在下表维护：`not_run / doing / done / blocked`。T16 与 T17 含只有用户能完成的动作，保持 `not_run` 不是异常。

| ID | 状态 | 任务 | 依赖 |
| --- | --- | --- | --- |
| C002-T01 | done | 固定上游合同、兼容性与任务治理 | 人采用 |
| C002-T14 | not_run | 收紧已校验定义、持久状态验证接口 | T01 |
| C002-T02 | not_run | StartRequirements 与无副作用 preflight | T14 |
| C002-T03 | not_run | 单一 WorkLayout 与输出冲突规则 | T14 |
| C002-T04 | not_run | 管理路径、文件句柄、限额和安全原子写 | T03 |
| C002-T09 | not_run | 无歧义目录摘要与独立向量 | T01 |
| C002-T05 | not_run | Workbook 身份、复制核验、只读根与初始化 | T04、T09 |
| C002-T06 | not_run | 一致事实视图与统计 | T14 |
| C002-T07 | not_run | schema 2、意图、快照、Work 发布与恢复 | T02–T06、T09 |
| C002-T08 | not_run | Workbook 事务、幂等与发布生命周期 | T05、T07 |
| C002-T10 | not_run | 输入发现、授权边界与恢复用法 | T02、T07 |
| C002-T11 | not_run | article-review 打回意见绑定 | T01 |
| C002-T12 | not_run | spec-dev 单任务与整体交付闭环 | T01 |
| C002-T13 | not_run | skill 安装产物自包含 | T10 |
| C002-T15 | not_run | self 生命周期、CI、MSRV 与发布门禁 | T08、T10–T14 |
| C002-M1 | not_run | 固定候选全链审查 | T02–T15 |
| C002-T16 | not_run | rc 真实宿主回归 | M1 |
| C002-T17 | not_run | 发布 v0.2.0 | M1、T16 |

## 3. 任务卡

### C002-T01 固定合同与采用边界

文件：宪章、spec、architecture、三份 contracts、engineering、必要 ADR、本 package 与任务白名单。

固定新 Store/schema、目录、摘要、响应、错误闭集、request-id 支持范围、pending 所有权、写锁与恢复窗口。统一 GF-29；明确 OS 主体不等于独立真人认证；删除写 shell rc 的承诺。确认新管理根与旧数据保留策略，不以“minor”掩盖破坏性变更。历史 T26 只追加有来源的证据更正，不改完成状态，不猜缺失 transcript。

验收：每个 O/N finding 映射到任务与验收；每个持久字段、响应和失败结果有唯一合同。负例是旧 Store 被自动迁移/清空、同字段存在未标记双解释、pending 仍可按年龄删除。任一出现，停止后续实现。跑 docs/specs 与人工接口闭环 review。

### C002-T14 定义与状态的可信构造面

文件：core flow/workbook/state/path/ids、runtime decode caller、API tests。

Raw DTO 私有，已校验定义只读；去掉可绕过 parse 的无用途反序列化与测试专用公开入口。对 Store 装入建立身份/revision/关键组合校验接口，错误是可定位的 STORE_CORRUPT；不把所有 String 普遍包装，不引入 trait/typestate 框架。

正例：当前样例和 public caller 仍工作。反例：外部不能把已校验 definition 改成非法图；持久 row 与 state identity 不符明确拒绝；损坏引用行不能在 remove 中被跳过。编译和受影响测试通过后提交。若需在 compile 复制整份 parse 校验才能保住原 public 字段，先修接口方案。

### C002-T02 StartRequirements 与 preflight

文件：core Graph/decide、runtime service、CLI workbook show、相关 tests。

共享收集/校验 start keys，show 文本与 JSON 暴露有序 start_inputs。把 Workbook/Flow、WorkName、缺/多 key、@file 错误放在分配序号和物化前；新 home 的失败 start 不建库。

正例：two-step show 给 topic，完整输入 start 成功。反例：缺 topic、多 key、非法名字、缺 workbook/flow，逐项比较 home、sequence、works、requests 与调用前相同；缺 CLI 必填参数 exit 2。拒绝后只补缺条件再成功。与 T07 的 staging 改动衔接，不能将空号解释为 preflight 可写的理由。

### C002-T03 WorkLayout 与输出合同

文件：core work/layout、Flow 编译规则、路径/API tests。实际持久 caller 切换归 T07。

实现单一目录函数，保留 Occurrence/retry 两维，engine 与 outputs 分离。拒绝重复/祖先/支持平台文件别名；明确输出路径可移植限制。将引擎 stats 与 worker stats 作为不同路径验证。

正例：嵌套合法输出、draft#2.1 映射稳定；新 outputs/brief.md/out 不再与 Attempt 根的 brief.md 冲突，必须能正常写入提交。反例：out 与 out/sub、大小写/Unicode 文件别名。纯路径测试不足以关闭 O12；T07 后必须真实 CLI begin→写输出→submit，且空执行不得把 engine.stats 当产出。不要引入 LegacyV1 与默认布局推断。

### C002-T04 根内文件操作与限额

文件：runtime home/observe/service/workbook_repo/selfmgmt 的文件 helper、相关 tests。

规范管理根，所有调用者使用受限文件操作；检查父目录、叶文件、普通文件、硬链策略和归属。安全句柄上限额读取/摘要/封存；原子写独占唯一临时名。清理不跟随软链，不按不可信 state 中裸路径直接写。删旧重复 helper 和固定 tmp-pending 路径。

正例：正常嵌套输出与显式 @file 读可用。反例：works/bin/pending 父软链、叶软链、状态卡临时软链、观察后替换、超限文件；外部哨兵字节和权限完全不变。COMMIT 前的观察拒绝保持 Store 不变；COMMIT 后的封存/投影失败保留已提交状态，并按 T07 返回 committed=true 与恢复信息。大小测试恰好上限/多一字节。目标平台各验证文件 API；不能仅再次 canonicalize 后重新开路径来宣称无竞争。

### C002-T09 目录摘要 v2

文件：runtime Workbook 摘要单元、core 摘要构造接口、独立 tests；T07 才切换持久 caller。

严格实现 design §4 的 domain prefix、BE64 数量/长度、路径排序与单 SHA256。流式读取，准确计数。保留旧两目录碰撞作为负例，不以生产 digest helper 生成 expected。

正例：手工字节流的已知向量。反例：za/zb 两种边界结果不同；顺序重排摘要不变；路径或正文改一字节摘要变化；非法文件和超限拒绝。若实现仍是 Sha256Hex::of_bytes(finalize())，任务失败。旧算法不保留为生产 fallback。

### C002-T05 Workbook 身份与本机边界

文件：WorkbookRepo、Home、Store 初始化入口、OS principal、CLI 与集成测试。

load 核登记 digest/id/version；复制后重新 parse/compile 并核最终副本。接入时遵守 T07 格式切换规则：T07 前旧生产入口仍按当前算法核对，不能写新摘要到旧库；T07 切到 v2 时删旧路径。version 拒绝点段/内部保留名。只读根最后设权限；清理前核归属。合法写操作可创建新 home，只读不创建。主体取真实 OS 身份，忽略可伪造 USER。

正例：合法安装、冻结、终态查询、删除后 Work 可读。反例：verify tampered 后 start 不得接受；复制间源变化、manifest 身份不符、version 点段准确拒绝。假 USER 不改变 audit 主体；只读不存在 home 无写。Finder 与源元数据规则分别验证。不能以同用户 chmod 可改来宣称权限无用，也不能宣称只读位不可绕过。

### C002-T06 状态与统计视图

文件：core render/state、CLI work/attempt/gate、快照与 tests。

先完成一个纯事实视图及状态转换实现和独立测试；实际持久字段、CLI 响应/caller 在 T07 统一接入。以事实视图生成文本和 JSON，包含 fail_reason、完整 ArtifactRef 与统一 next。stats/next 用同一次加载，来源保留 node+edge。累计受阻的必要事实由 core 状态转换记录；新字段接入与 T07 schema 切换一起完成，不在旧 state 上默认猜值。

正例：非零时间/次数/失败原因两格式一致。反例：NoLegalEdge→cancel 计数不减少；相同来源不同边可区分；删除输出 digest/bytes 或 reason 时真实 JSON 测试失败。本任务用独立手写状态验证；T07 切换后再跑终态、失败和 NoLegalEdge→cancel 的真实 CLI，完成前不关闭 O13/N07。避免用两个 render 函数互相当 oracle。

### C002-T07 请求、schema 2 与 Work 恢复

文件：runtime request/service/store、core 新持久字段与 layout caller、CLI 成功/错误响应、全部受影响 fixtures/replay/crash tests。

一次接入 T03/T06/T09 的新格式，创建完整 schema 2；所有 caller 同步，只接受新格式。Store 建库 DDL 与 user_version 原子，首次并发建库串行；旧库拒绝前无写。按 design 实现管理根写锁、RequestIntent、完整 ResponseSnapshot、受 Store 保护 pending、效果归属与完成标记。命中先于 load/observe，事务内再查重。CLI 不再读取当前 Store 拼历史 reply。

正例：正常 start/begin/submit/gate/fail/cancel；与旧快照逐字段比较重放。反例：跨 Work request-id、文件变化后 submit 重放、Workbook 删除后 start 重放、cancel 后旧 submit、后续状态下 stats 缺失恢复、已存在历史文件摘要不符、旧请求不能回退 status-card、两个写者竞争。每种意图覆盖 COMMIT 前/后、发布前/后窗口，真实子进程退出并以同请求恢复。

验证 schema 1 只读拒绝且旧文件字节不变；schema 2 只包含一种布局/摘要/outcome。不得保留旧 hash_command、legacy request 猜测或隐式默认字段。效果错误必须能证明已提交/未提交边界。任一历史字节被重算、pending 被 tmp 清理或新目录先于提交暴露，停止任务。

### C002-T08 Workbook 事务与生命周期

文件：WorkbookRepo、Store request/commit/read、CLI workbook、failpoints/tests。

add/remove 的状态行、审计、请求与效果同事务；remove 引用检查也在事务内。每次 add 只操作自己的 pending；写锁覆盖发布，恢复未完成效果后才执行后续写。完成效果不会因旧 request-id 重放重新执行。

正例：add/remove 重放返回原 snapshot。反例：并行 add 不删对方 staging；提交后 rename 失败可恢复；先 remove 再新请求 add 同版本，重放旧 remove 不得删除新对象；重放旧 add 不得覆盖新生命周期。损坏引用行必须停止。请求空载荷/重复/冲突与 Work 共用全局去重。给出状态、数据库、原件字节和目录归属四类 oracle。

### C002-T10 输入发现与协调者行为

文件：skill、CLI show、协议使用说明、宿主验收案例。

展示 start_inputs；用户已给的信息直接使用，未指定/缺输入才询问，不用失败 start 探测。只做 next 限定于 Work 推进；发现命令、add/start 有独立入口。已指定但未装 Workbook 不静默替换。说明 request-id 应预先保存、支持范围、重放后查当前 status；human/gate 遵守实际授权并如实记录代执行。

正例：完整意图自动开工、重放后从当前 next 继续。反例：缺输入、指定未安装、用户已授权仍重复问、把历史 next 当当前状态、把 OS 身份当真人证明。离线文案/CLI 验证和 T16 宿主行为分开记录。

### C002-T11 article-review 回环

文件：样例 Flow、draft 说明与场景测试。

draft 增加 optional review.verdict；首次尚无，back 后绑定最近成功 review 的产物并按意见修改。只传路径，不内联全部历史。真实 begin 返回和 brief 均验证输入来源；不能手工追加聊天当作测试通过。

### C002-T12 spec-dev 交付闭环

文件：spec-dev Flow、说明、模板、checklists，以及独立临时 Git/CLI 回归。

固定原始整体基线、每任务基线/候选两种职责；改方案不能缩小最终全链 review。verify 仅核当前任务范围与本任务占位，允许合法的未来占位。plan-review 条件要对应获批 plan/spec 版本并交给需要它的节点；修订方案后不得沿用旧批准。escalate→scaffold 与 verify 返回的输入/边/说明成套修复。

五组闭环：两任务分别改不同文件；两任务共用文件且留未来占位；有条件批准进入 scaffold/implement/verify；verify/scaffold 升级后按人意见继续；完成任务 1 后改方案，最终 review 仍覆盖任务 1。正反例用 Git 提交/文件集合与 brief 绑定独立验证，另在 T16 取真实 agent 执行证据。不要给引擎增加 Spec/Plan/Git 状态。

### C002-T13 自包含 skill 交付

文件：skills/sheltie、交付生成脚本、README、check-skill 与安装 fixture。

单一权威合同生成发布 references；安装结果全部是可读取文件，脱离源码树仍能导航。可以在仓库使用符号链接配构建时解引用，但发布物必须实测，不能要求用户记住 cp 的隐藏前提。覆盖合同漂移检查，避免手工维护两份规则。

正例：隔离目标目录可解析全部本地链接与命令。反例：移动/移除源码路径后不存在断链；忘记同步 references 时 gate 失败。不访问真实宿主配置。

### C002-T15 self、CI 与发布质量

文件：selfmgmt、CLI self、Cargo/toolchain/deny、build/release workflow、check-specs、README 与集成测试。

删除 modify-path 写 rc 路径，所有 --json stdout 只有协议 JSON；self 拒绝 request-id。按固定 tag 解析 --version/清单/资产，检查下载、解包与回滚窗口。所有 managed self 路径使用 T04 边界和同一管理根写锁。purge 持锁；等待者获锁后复核根与锁对象身份，根已删除/重建时退出重试，不能沿旧 inode 继续写。加入 self/Work 并发和 purge 等待者测试。更新前说明 Store 不兼容及旧数据保留方法。

治理 job 获取历史 tags/commits；check-specs 分别接受已发布 release 和 active target/RC，不能要求开发中的版本先发布。release 明确依赖同一 SHA 的质量 job。新增实际 MSRV 1.85 locked gate，若不满足则通过证据决定修依赖还是提高声明，不把 stable 当替代。

正例：本地 release fixture 的指定版本/最新/rollback；clean home two-step；浅克隆补齐所需历史后 gate 通过；active RC 在无 tag 时合法。反例：校验失败旧二进制不变，latest 变化不混包，伪造路径不能越界，质量失败不能发布，缺失历史给准确诊断。`cargo dist plan` 加真实发布形状 fixture，不能仅测自制瘦格式。已发表的四平台产物取证归 T17。

## 4. C002-M1 固定候选全链 review

未参与实施的 Reviewer 审查完整候选与 v0.1.0 的变化，对 O01–O13/N01–N14 建关闭矩阵；未关闭项不能靠 task done 或测试数量代替。逐条保留 finding、修复 commit、反例、结果及原始 run。

运行全仓 fmt/check/Clippy/nextest/deny、docs/specs/core-vocab/tests/skill、MSRV、dist plan。真正并发测试先批量启动并用同步点制造交错，再 join；摘要、Git 范围、状态卡字节使用独立 oracle。补 Work/Workbook 发布与恢复故障窗口。core/runtime 突变结果按能力处置，不把全部存活体一概当必须加测试。

修复代码候选冻结后审查；审查修改影响输入闭包时，按受影响链重验并记录新 hash。结果只有“通过 / 需修改 / 阻断”。M1 通过只证明源码与离线闭环，不证明 Host 或发布完成。

## 5. C002-T16 rc 真实宿主回归

使用绑定 M1 候选的 rc 二进制、独立管理根和自包含 skill。用户/Host Owner 执行只有他们能完成的动作；未执行写 not_run，不能用合成图替代。

- 未指定/未安装 Workbook、缺输入时正确交互；已给信息不重复问。
- two-step、article-review back、gated-release；人工节点由人执行并记录，gate 展示产物后取得实际批准。
- 关闭并重开会话，仅通过状态查询继续；错误请求后正确恢复，无第二套进度。
- spec-dev 完成至少两个任务、一次人工条件与一次改方案，最终覆盖整个原始需求。
- 实测 Finder 与完整性；记录逐命令响应、人工操作、产物摘要、宿主 usage、耗时及质量结果。usage 不可得就记录缺失，不能记成 0。

若要声称节省成本，另做同任务直接 agent/skill/Sheltie 对照；没有对照只报告绝对观测，不阻塞可靠性修复的客观结果。宿主失败回到对应 Owner 修复，不用一句“模型没遵守”关闭问题。

## 6. C002-T17 发布

发布需用户授权，依赖 M1 与 T16 的全部必需项通过。更新 CHANGELOG，确认四平台资产/manifest/checksum/quality 对应同一 commit；全新根安装、指定版本更新与回滚实测。schema 1 数据保持原样；回滚按旧 binary+旧 home 验证，新 home 不能被旧二进制误写。

release record 记录候选、门禁、Host 证据、兼容限制与已知问题，再按治理流程完成 package。发布失败不标 completed；不把这份 proposed 计划或本次 review 写成修复成功。
