# C002 候选设计

状态：`active`。机制细节以架构与三份合同为准。本文件保留设计意图与取舍，不代替当前合同。此设计替换旧版 C002 的隐式双格式兼容、错误摘要固化和历史状态卡重放方案。

## 1. 保留的结构与版本边界

继续使用 `cli → runtime → core`、同步调用、一个 SQLite Store。core 负责合法转换与纯路径/视图；runtime 负责实际文件、主体、事务与效果；CLI 只解析和渲染。新增模块只按这些真实职责拆分，不引入通用框架。

Store schema 从 1 升到 2，原子创建表和 user_version。旧 schema 先只读识别并拒绝，在拒绝前不改 journal mode、内容或文件。新版本不写旧格式，不用 `serde(default)` 猜布局，也不在 reply_json 中容纳无标记的新旧两种解释。新响应若改变既有字段结构，采用 `cli-result/v2`；T01 固定完整字段与错误闭集。

默认用新管理根开始 v0.2.0，保留旧管理根和旧二进制。仅切换二进制的 rollback 不等于 Store downgrade；旧二进制也必须拒绝 schema 2。安装/更新说明必须包含这一点。迁移只有出现真实续接需求才另行设计，不能用自动清库解决。

## 2. 用户意图与提交快照

runtime 提供内部 `RequestIntent`：操作种类、完整目标身份、用户参数。新 Work 请求的前缀只读解析为完整 WorkId；历史 request-id 从 `requests.work_id` 取原目标并核原始前缀，不因后来出现同前缀 Work 而改绑或拒绝。已有 Work 行即使终态也能解析，不能靠读取 Workbook 才解析。request-id 是有界、不作路径的 opaque key；内部 staging ID 由 runtime 随机生成。

- Work intent 含完整 WorkId、node/attempt 与 summary/reason 等用户参数，不含重新计算的观察结果、时钟和模型自报事实。`--summary @file` 按文件路径记录意图，首次执行才读取内容；重放不要求源文件仍存在。
- start intent 含用户给的 Workbook selector、Flow、名字参数原值及省略状态与起始输入参数。字面值按值记录；`@file` 按规范化绝对路径记录，文件内容是首次执行时的观察结果。没有显式版本时第一次解析的实际版本进入提交响应；重放先查记录，不读当前 `@file` 或 Workbook，也不重新解释“最新版本”。
- add intent 含不访问文件系统即可词法规范化的源目录绝对路径和明确操作；相同请求成功后源目录变化或消失也不重新安装，返回原结果。新 request-id 才表示新的安装意图；同 id/version 的另一安装遵守冲突规则。
- remove intent 含完整 id/version。self 收到 request-id 直接参数错误，查询也不虚构 request-id。

第一次提交保存完整 `ResponseSnapshot`（包括 Work 身份、status、next、批准记录、输出引用与 revision），CLI 不再提交后重新读 Store 拼数据。重放只附 `replayed=true`，其他业务字段不变。历史 next 是历史响应的一部分；skill 在恢复后通过 status 获取当前 next。

事务内仍复查 request-id 与 revision，解决预检后的竞争。失败且尚未提交的请求不保存成功记录；同 request-id 可在故障排除后重试。

## 3. preflight 与路径

core 共享两个纯函数：`start_requirements(graph)` 和 `validate_start_inputs(graph, provided_keys)`。decide、runtime preflight 与 workbook show 使用同一事实来源。runtime 在任何序号分配/目录物化前核对 Workbook/Flow、名字、输入键；CLI 的 @file 读取失败同样在此前结束。

一个 `WorkLayout` 集中生成路径，新 Store 只有一种布局：

```text
works/<work-id>/
  status-card.md
  workbook/
  start-inputs/<key>
  attempts/<node>/occurrence-001/attempt-000/
    brief.md
    engine/stats.json
    outputs/<declared-path>
```

Occurrence 与 retry 是两个真实维度，均保留。`AttemptId=node#n.retry` 保持原含义；目录标签仅改善浏览。begin 返回前创建输出目录。T01 已将输出路径限定为可移植 ASCII：编译拒绝非 ASCII、重复、祖先冲突与 ASCII 大小写折叠后的别名。测试覆盖合法嵌套路径、大小写别名和 Unicode 路径准确拒绝，不做 Unicode 归一化猜测。

Home 在入口确定规范根，所有 managed 路径（Store、workbooks、works、pending、tmp、bin 及恢复/删除目标）都从它派生。检查到叶与最近存在祖先；不允许不可信父软链把根重新定义成根外位置。缺祖先与权限错误要区分，不能把任意 canonicalize 失败视为安全。

临时文件用同目录唯一名与独占创建，拒绝软链和意外已有项；写完按持久性合同 sync，再 rename。输出以安全打开的文件句柄检查类型、字节数和摘要，并在同一对象上封存；避免“检查路径、重开另一对象、再 chmod”的窗口。允许用户显式 @file 读管理根外文件，不允许由该例外派生根外写。

## 4. 摘要与 Workbook 身份

新目录摘要命名为 `workbook-digest/v2`，采用下列精确字节流，最后只做一次 SHA256：

```text
ASCII "sheltie-workbook-digest/v2\0"
BE64(file_count)
对规范 UTF-8 相对路径按字节序排序的每个普通文件：
  BE64(path_byte_length) || path_bytes || BE64(content_byte_length) || content_bytes
```

路径不含 `.`、`..`、NUL 或空段，不作静默 Unicode/大小写转换；拒绝软链、硬链、特殊文件和不可表示路径。空目录不参与摘要，这一点写入合同。用手工拼字节的独立 oracle 验证单次 hash、文件边界与排序。

在读取前检查单文件/目录总量，之后用有上限的流式读取和计数避免文件增长绕限。资源索引只为 instruction 检查 UTF-8；不为每个 resource 全文转字符串。一次操作复用同一加载/观察结果，避免 preflight 再完整重复扫描。

add 先复制到自己的 pending source，再对这份最终字节集合 parse/compile/digest；登记的 manifest 身份来自该副本。start 先核对 installed 的登记摘要与 id/version，复制后再次核对冻结副本，拒绝校验与复制间的变化。version 必须能作为单个安全目录段，至少拒绝 `.`、`..` 和内部保留名。

Workbook 与冻结副本的文件设 0444、目录含根设 0555；合法移除时只对已核归属树放开权限。权限只是减少误写，摘要仍用于检查。源目录有明确列举的宿主元数据时准确拒绝，不静默忽略字节；Finder 场景必须实测，不能仅以 chmod 成功证明。

## 5. 一个 Store 中的请求与文件发布

请求表保存 intent_hash、响应快照和恢复所需效果；具体 SQL 在 T01 固定，所有字段、状态与索引随 schema 2 一次定义。效果记录是 I/O 完成情况，不参与业务选边，不构成第二套 Work 状态。

先以只读方式识别已有 schema、构造不依赖当前文件内容的意图并查询可重放请求；未命中才读当前文件并完成确定性 preflight。旧库拒绝、新 home 的失败 start 都发生在创建目录/锁文件或写 PRAGMA 之前。合法写操作才创建管理根并取得锁；锁内重新核对 schema、request-id 与受并发影响的前置事实，再恢复/准备/提交。

为避免本地多个写进程交错发布/删除，runtime 使用一个管理根级写锁，覆盖锁内重放复查、恢复、准备、事务与效果发布；进程退出由 OS 释放。只读操作不获取写锁，不创建锁文件。self 的写入口也遵守这把锁。确认purge删除用户数据但保留空管理根和同一个`.lock`后，排队者沿同一锁继续；合法install/add可初始化空Store，旧Work写命令对已删除Store/Work返回`NOT_FOUND`且不重建。锁外因素意外删除/替换根或锁时，仍须身份复核并整体重试，不能沿陈旧inode写入。这是文件生命周期串行化；SQLite revision/CAS保留为事务边界校验，不做自动业务重试框架。锁只针对本地协作进程，不声称约束同用户手工改文件。

start 和 add 的staging位于专用`pending/<internal-id>/payload/`；引擎先独占创建并fsync`pending/<internal-id>.owner`侧车，再创建payload，绝不把它放进可按年龄清理的tmp。remove先建owner/container，不预建payload；提交后核对final再移入payload并安全删除。先sync原件和必要目录，再在一个事务中记录Work/Workbook、audit、request snapshot与发布效果，随后rename`payload/`到最终目录、刷新当前状态卡并完成效果标记。拒绝在内存中“记住”唯一恢复信息。

| 窗口 | 行为与恢复 |
| --- | --- |
| preflight 失败 | 无序号、业务行、request 或物化目录；不存在的 home 也不为失败 start 建库 |
| 分配后、COMMIT 前 | 可留下空号和本操作未提交 pending；下次持写锁按“无 Store 引用 + 本引擎归属”清理，不影响其他请求 |
| COMMIT 后、rename 前 | Store 有提交结果；pending 是唯一原件，不得按年龄删除；下次写操作先按 Store 效果发布，再 load/observe 新命令 |
| rename 后、效果标记前 | 恢复核对最终对象的归属、摘要与目标；同对象视为已完成，不能覆盖不同对象 |
| 效果失败 | 不回滚已提交 Work；响应明确 `committed`、revision、request-id 和恢复动作，不能误报“无变化”或诱导换新请求 |
| 只读查询遇到 pending | 不执行恢复；可从 Store 指向的已提交 pending 冻结定义读取状态，并显式报告文件尚待发布。只读若与 rename 交错，只能按同一个 Store effect 的归属在 pending/final 两个位置有限重读，不能把第一次 NotFound 直接当损坏；超出重试界限报准确的暂时 I/O 错误，不写文件或 fallback 到任意安装版本 |

Workbook remove 的“查非终态引用、删行、写请求/审计/效果”在同一个事务；损坏的引用状态必须报 STORE_CORRUPT，不能跳过。提交后将准确归属的旧目录移入自己的待删除位置再删除。add/remove 在写锁下先完成前序 pending，防止删除与重加交错。

完成的发布/删除/封存效果不会因历史请求重放再次执行。正常已完成 submit 的重放直接返回原 snapshot，不重新观察或 chmod 当前输出；只有尚未完成的 seal 恢复才核对原 ArtifactRef。历史 brief/stats 是例外：显式重放已完成 begin 时，若文件缺失且父目录可信，按登记字节补齐；已有不同字节或父目录缺失则报已提交恢复错误，不覆盖。相同 id/version 的新生命周期不能被旧 add/remove 重放覆盖或删除。发布/删除后的唯一字节若被外部删除，只能明确报缺失，不得臆造恢复；测试需覆盖这一条件。

## 6. 历史文件与当前投影分别恢复

brief 与 engine.stats 的内容在提交前确定，持久保存精确字节及目标身份。显式重放已完成 begin 时：缺失且父目录可信则补齐；存在且摘要相同则不写；存在但不同或父目录缺失则报已提交恢复错误，不能掩盖修改。尚未完成的 SealOutputs 恢复针对原产物引用；不存在或改变时不重造、不越界 chmod。已提交请求的效果恢复错误携带 committed=true、原 revision/request-id 和 original_response（已保存业务 snapshot）；文件完整或成功补齐时返回原 snapshot。不得重新执行业务，也不得把恢复错误当成未提交。

status-card 是当前 Store 投影，不存历史卡字节。`refresh_status_card` 作为请求效果登记完成状态，但每次恢复都在同一写锁下从最新状态生成；失败返回 `EFFECT_PENDING`，旧请求不能把卡写回旧 revision。故障测试覆盖新状态已提交、旧请求晚到、两个写者和投影丢失。

## 7. 事实视图、身份与定义边界

core 先生成一个 StatusView，再分别渲染文本与 JSON；失败原因、完整 ArtifactRef 和 next 选择规则只有一份。stats 与 next 在同一加载快照上计算。GF-29 采用累计受阻和来源 node+edge：NoLegalEdge 的发生事实必须保留到 cancel 后，可在唯一 WorkState 中记录受阻计数/历史必要字段，由 core 在转换时更新；不依赖当前 status 猜历史，也不创建第二套推进表。

OS 主体从真实进程身份取得，不采信 USER/USERNAME。无认证时只记录“哪个 OS 用户调用了 approve”，不标记独立真人已验证。上游宪章、规格、协议与 skill 同步该边界。

Raw TOML DTO 私有；已校验定义的公开构造与修改面收口，按实际外部调用者提供只读访问。持久状态读取校验行/JSON 身份、revision、关键状态组合及 managed 路径归属。内部固定常量不重复层层校验；测试 helper 不当作安全边界。

## 8. 协调者、Workbook 与发布

skill 在 start 前通过 show 发现输入；用户已明确指定时不重复询问，未指定或缺实质信息才问；不得静默换 Workbook。human 节点与 gate 遵守已授权范围，代理代执行要如实记录。skill 安装产物的 references 随发布生成并校验，源码合同保持单一权威；不要求用户安装后仍保留仓库路径。

spec-dev 在 Workbook 文档内保存原始整体基线、每任务基线/候选和当前批准条件版本。重规划不重置整体审查范围。条件绑定到有效 plan/spec revision，避免旧批准永久附着新方案；escalate 的去向与输入、说明成套更新。引擎保持业务无关。

self 去掉 modify-path 写宿主模式；stdout 只交付协议内容。update 先固定发布 tag，再下载同 tag 的清单和资产；本地 fixture 与真实发布形状经过同一受限解析路径。release 依赖同一 SHA 的质量 gate，治理 job 显式取得所需历史，check-specs 区分已发布版本和 active 开发/RC。
