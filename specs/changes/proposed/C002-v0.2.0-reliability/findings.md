# MVP 统一问题清单

候选：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。结论：**需修改**。以下均为 OPEN，产品修复 `not_run`。P1 表示会破坏数据/身份/执行判断或交付链，下一版发布前必须关闭；P2 表示合同、可用性或验证缺口，同样纳入本次修复。优先级不是攻击严重度评级。

保留原 O01–O13 编号；N01 起为本次补全或从旧叙述提升为独立可追踪的问题。代码行号均对应上述候选。审查过程、产品分析和旧方案裁决见 [review.md](review.md)，全部证据位置见 [validation.md](validation.md)。

## 1. 原有问题核对

| ID | 级别 | 确认问题、依据与可观察后果 | 修复任务 |
| --- | --- | --- | --- |
| O01 | P1 | 管理根隔离未贯通。`runtime/service.rs:114,591–597`、`observe.rs:15–40`：只检查叶文件，未统一验证祖先；固定 `tmp-pending` 可跟随软链。真实 CLI 将 `works` 链到根外后 start 在根外写整棵；状态卡临时路径软链使 cancel 覆盖外部文件。违反 INV-3、storage §6。范围扩大到读、写、chmod、rename、删除与恢复，不仅输出观察 | T03、T04 |
| O02 | P1 | 请求指纹未含目标 Work。`service.rs:366–373,606–613`：cancel A 后相同 request-id cancel B 返回 `ok=true/replayed=true/revision=2/next=[]`，B 实际仍 active。违反 GF-15 和 INV-7 | T07 |
| O03 | P1 | 已装 Workbook load 忽略登记摘要。`workbook_repo.rs:178–196`：verify 已 tampered，start 仍以新摘要接受同 id/version。source 校验与复制之间也没有对最终副本重新建立可信闭包。违反 GF-05、GF-17、storage §5.1 | T05、T09 |
| O04 | P1 | 重放依赖当前观察，CLI 又读取当前状态拼历史响应。`service.rs:82–95,366–375`、`commands/attempt.rs:120–131`：已提交输出删除后同请求变 REQUEST_CONFLICT；cancel 后重放旧 submit 混入 cancelled 与旧 next；删除仓库版本后 start 重放 NOT_FOUND。违反 GF-15 | T07 |
| O05 | P1 | 历史 engine.stats 用最新 WorkState 重建。`service.rs:534–552`：begin 后 submit，删除旧 stats 再重放 begin，恢复内容从 active 变 succeeded，与原 ArtifactRef 不符。违反 GF-07、GF-29 | T07 |
| O06 | P2 | 新管理根不能直接 self install/add。CLI/`selfmgmt.rs:97` 先 open DB，父目录不存在时 STORE_CORRUPT。真实 CLI 已复现；初始化顺序和错误分类错误 | T05、T15 |
| O07 | P1 | 目录摘要实现为双 SHA256；更严重的是合同 `path\0content` 拼接有歧义。`workbook_repo.rs:199–210`：相同公共文件加 `za="zb\0X"`，与 `za="", zb="X"` 得到相同 digest。这是编码碰撞，不需要破解 SHA256。旧“保留两阶段算法兼容”的方案无效 | T09 |
| O08 | P1 | Workbook/self 写操作仅回显 request-id，未提供全局声明的重放语义。add 同 id 再执行是 WORKBOOK_EXISTS；remove/update/rollback 的重复动作也不能统一当成安全重试。协议 §1/GF-15 范围需要明确 | T08、T15 |
| O09 | P1 | spec-dev 用骨架至 HEAD 的累计 diff 验当前任务，且禁止白名单文件内全部 todo。`instructions/verify.md:17–18`：任务 2 误判任务 1 的文件越界；共享文件中未来任务占位也被误拒。临时 Git 独立 oracle 已复现累计范围错误 | T12 |
| O10 | P1 | 人工意见未完整进入后续输入。`plan-review.md:19` 与 `flows/default.toml:66–73`：scaffold 缺 decision；escalate→scaffold 缺 escalation。verify 可以升级给人却没有返回 verify 的边。仅加边仍需改 verify 的输入选择。违反 Workbook 自身的批准/继续约定 | T12 |
| O11 | P2 | skill 单目录按 README 复制后，`../../specs/contracts/` 两条链接离开交付目录。安装产物不自包含；源码内链接检查无法证明安装后可用 | T13 |
| O12 | P1 | 引擎输入与 worker 输出命名空间碰撞，路径祖先冲突未拒。`flow/parse.rs:276` 仅拒 exact brief.md，`decide.rs:637` 固定 stats.json。合法节点输出 stats.json 时无需 worker 写入也能 submit succeeded；brief.md/out 或 out 与 out/sub 安装成功但合同不可满足。须同时验证目标平台的文件名别名 | T03、T14 |
| O13 | P2 | JSON 状态卡缺失败原因、输出摘要与大小。`render.rs:314–345`：fail reason 有明确文本，但 JSON summary=null 且无 reason，outputs 只有路径。与 protocol §6“同样字段”不符。data.next 与顶层 next 也使用不同形状 | T06 |

## 2. 补全的问题

| ID | 级别 | 问题、证据与边界 | 修复任务 |
| --- | --- | --- | --- |
| N01 | P1 | start preflight 在副作用之后。`service.rs:101–147` 缺 topic 先分配序号并建冻结副本/inputs，最后 INPUT_MISSING；库无 Work 行但最终目录存在。真实 CLI 复现。storage 允许后续失败空号，不能豁免确定性输入检查 | T02、T07 |
| N02 | P1 | Workbook 发布缺恢复归属。`workbook_repo.rs:98,134–143,220–228`：add 先插行后 rename；目录冲突后 row 留存，重试 WORKBOOK_EXISTS。每次 add 清空整棵 staging 会影响其他操作；remove 的引用检查与删行分属连接。add rename 失败后的残留行与重试失败已复现；并发清理、引用检查竞争和 kill 窗口本次为静态确认 | T05、T08 |
| N03 | P2 | Store 建库 DDL/user_version 未在一个显式事务中，首次并发建库亦有 exists/open 竞争。`store/mod.rs:46–61`。decode_state 仅 serde，不校验行 work_id/revision 与 JSON 身份、状态关键组合；`works_referencing` 解码失败直接 continue。GF-16 要求准确拒绝，不能跳过损坏记录批准删除。静态确认，未做随机 kill 压测 | T07、T14 |
| N04 | P2 | 身份取 USER/USERNAME 环境变量，不能等同“当前 OS 用户”；gate 无独立人工认证。`observe.rs:101–105`、protocol §1、宪章 §4 的承诺不一致。修真实主体来源并收窄单用户信任声明，不把身份修复误写成真人认证 | T01、T05、T10 |
| N05 | P2 | `self install --modify-path` 写宿主 rc，与根规则只写管理根冲突；拼接未 shell quote；--json 时输出额外提示破坏单 JSON。`selfmgmt.rs:478–501`。隔离假 HOME 已复现 JSON 污染；删除该写宿主模式优于维护 shell 分支 | T01、T15 |
| N06 | P2 | `self update --version` 只读取 latest 并比较字符串，不能实际选择历史版本；清单与资产两次 latest 解析会漂移。`selfmgmt.rs:130–143`。静态确认，未联网升级；应先解析固定 release/tag，再以同一版本定位资产 | T15 |
| N07 | P2 | stats 事实口径不闭合。`render.rs:451,493–514` 丢弃 EdgeKind；NoLegalEdge 后 cancel 使 blocked_count 从 1 降到 0。真实 CLI 已复现。GF-29“哪种边/累计受阻”与 protocol 的当前状态推导公式冲突，采用时先统一上游 | T01、T06 |
| N08 | P1 | spec-dev 改方案时重设基线，最终 review 只看新基线差异。`instructions/plan.md:10`、`review.md:10`。临时 Git oracle 证明任务 1 的变更从整体 review 范围消失；须保留 Work 原始基线与独立的每任务基线 | T12 |
| N09 | P2 | article-review draft 未声明 optional review.verdict，back 后意见只靠聊天补充。Flow 与说明静态确认。绑定上一轮文档后验证第一次尚无、第二次正确来源 | T11 |
| N10 | P2 | set_tree_readonly 不设 Workbook 根只读。`workbook_repo.rs:418–441`。自动生成文件会改变冻结定义摘要，导致整个 Work 不可读。权限缺口已静态确认；此前真实用户目录的 .DS_Store 事件本次未重验。只读位只能减少误写，不能证明同用户防篡改 | T05 |
| N11 | P1 | check-specs 要求历史 tag/commit，而 build docs job 使用默认浅 checkout。隔离 `git clone --depth 1 --no-local file://…` 后检查明确失败。检查器末段还要求 Cargo 当前版本已有 release record/tag，阻断 active/RC 开发。C003 本地 PASS 未覆盖该入口 | T15 |
| N12 | P2 | release 工作流独立触发，没有依赖相同候选的质量结果；stable 通过不证明 MSRV 1.85。`build.yml`/`release.yml` 静态确认。未断言目前 lock 必定不兼容 1.85；本机没有该工具链，本项是缺验证门禁 | T15 |
| N13 | P2 | 测试有自证与假并发。runtime tests/service.rs:205–212 的 lazy spawn→join 链逐个执行；workbook_repo 测试用同 digest helper 生成 expected。崩溃测试只覆盖少数点，spec-dev 只有图结构测试。315 PASS 未覆盖本表真实反例 | 各修复任务、M1 |
| N14 | P2 | 文件观察/资源索引先全文读取再验证大小，digest 收集整树内容；run_command 重复 load/build，stats 再读一次 status。`observe.rs:40,86`、`workbook_repo.rs:200–208`、`service.rs:365–400`。大于合同上限的输入仍可先占用大内存。修有限读取与重复扫描；本次未做 OOM/性能基准，不声称速度回退比例 | T04、T05、T07 |

## 3. 证据重点与最小修复

### 请求身份、响应与文件恢复

[request-probes.json](evidence/2026-09-27/request-probes.json) 保存每次真实 argv、exit、stdout/stderr。O02 的响应显示目标 B 仍 active 却成功“重放取消”；O04 的旧 submit 响应保留 revision 3 和可 begin summary 的 next，却把 work_status 改成 cancelled。请求应绑定解析后的目标与用户参数；响应字段从提交时 snapshot 返回。

历史响应可以保留历史 next，这是重放定义，但调用者要以新的 status 查询继续操作。当前 status-card 必须反映最新 Store；不能为修 O05 把所有文件都按历史字节覆盖。

### 目录摘要

[hash-framing.json](evidence/2026-09-27/hash-framing.json) 的两个合法源目录摘要相同。因为 `za\0zb\0X` 可以解释成一个文件的内容，也可以解释成两个文件。必须把路径与内容长度编码进摘要流，并对该流做一次 SHA256；独立测试手工组装字节。

这是格式修复。用新 Store/schema 明确拒旧，比让同一字段同时代表错误旧摘要和新摘要更清楚。旧用户数据必须原样保留，不能“为了测试通过”清空或改写。

### 文件和发布生命周期

[standards-probes.json](evidence/2026-09-27/standards-probes.json) 记录根外写入、固定临时路径覆盖、add 失败残留行、clean home 与 JSON 反例。修复必须贯穿 Service、WorkbookRepo、self 与恢复；局部增加 observe_confined_file 仍会遗漏 start、原子写和删除。

已提交的未发布目录含唯一的输入/Workbook 字节，属于持久事实的待发布部分。它不能放在“24 小时可删”的临时区；只能按 Store 归属确认后清理。过期、时间戳或目录名都不构成单独删除依据。

### Workbook 的业务闭环

[git-probes.json](evidence/2026-09-27/git-probes.json)、[workbook-probes.json](evidence/2026-09-27/workbook-probes.json) 分别证明 Git 范围与机械绑定缺口。后者使用合成输出走图，只证明图/输入/next，不证明模型完成真实开发或真人批准。

## 4. 不重复计数的历史限制与设计建议

旧 S1/P1 已归 N01，P4 已归 T02 的输入发现；S2/P2 是历史 human/token 证据限制，保留为 T16 回归要求；S3/S4/P3 是历史来源与措辞问题。没有原始 transcript 时，只标记“历史叙述/用户后续澄清/未验证”，不臆造 prompt。S5 的路由已由本 package 承担。

`Manifest/FlowDef/NodeDef` 的公开可变字段、测试 helper 暴露、过期注释属于 T14 的接口与维护性修剪。没有实际调用链证据的泛化 smell 不升级为 P1，不要求全仓重写、普遍 newtype 或新的 trait 框架。
