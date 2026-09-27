# C002 审查证据与修复验证状态

审查候选：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。开始时工作区干净。日期：2026-09-27。§1–§3 是审查与方案整合的证据（已完成）；§4 起是实施验证记录，未执行的任务保持 `not_run`，不填 PASS。

## 1. 本次代码审查验证

平台：macOS aarch64；`rustc 1.98.1`、`cargo 1.98.1`。Cargo 构建使用 `RUSTC_WRAPPER=`、独立 `CARGO_TARGET_DIR=/tmp/sheltie-review-a664e75/target`，避免写用户全局 target。默认构建曾被 sccache 沙箱权限拒绝，关闭 wrapper 后成功；该环境问题不计入产品 finding。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 当前候选构建 | executed | HEAD + Cargo.lock + all features | `cargo build --all-features --message-format=json` | PASS | Cargo 报告的 executable 用于下列 probes |
| 格式 | executed | 同候选 | `cargo fmt --all -- --check` | PASS | [fmt.txt](evidence/2026-09-27/fmt.txt) |
| 编译 | executed | 全 targets/features | `cargo check --all-targets --all-features` | PASS | [check.txt](evidence/2026-09-27/check.txt) |
| Clippy | executed | 全 targets/features | `cargo clippy --all-targets --all-features -- -D warnings` | PASS | [clippy.txt](evidence/2026-09-27/clippy.txt) |
| 既有测试 | executed | 22 binaries、315 tests | nextest `aac13440-9f58-48d0-a91d-926dbc02c559` | PASS：315 passed，0 skipped | [nextest.txt](evidence/2026-09-27/nextest.txt)；测试执行 11.849 秒，不含编译 |
| 依赖检查 | executed | 原 deny 规则，仅 advisory 缓存路径改到临时副本 | `cargo deny --offline --locked --config /tmp/sheltie-review-a664e75/deny.toml check` | PASS，含告警 | [deny-isolated.txt](evidence/2026-09-27/deny-isolated.txt) |
| 当前文档与任务归属 | executed | 改文档前 HEAD | docs/specs/tests/core-vocab/skill scripts | PASS | [证据目录](evidence/2026-09-27/README.md) |
| 跨目标与重放 | executed | 真实 CLI、独立临时 home | request_probe | 缺陷确认：O02/O04/O05/N01 | [raw JSON](evidence/2026-09-27/request-probes.json) |
| 文件、Workbook、self | executed | 真实 CLI、假 HOME、临时哨兵 | boundary_probe | 缺陷确认：O01/O03/O06/O07/O08/N02/N05 | [raw JSON](evidence/2026-09-27/standards-probes.json) |
| 摘要边界碰撞 | executed | 两棵合法 Workbook | hash_probe | 缺陷确认：O07 | [raw JSON](evidence/2026-09-27/hash-framing.json) |
| 输出与视图 | executed | 真实 CLI、自造合法 Flow | output/status/stats probes | 缺陷确认：O12/O13/N07 | [输出](evidence/2026-09-27/output-probes.json)、[状态](evidence/2026-09-27/status-probes.json)、[统计](evidence/2026-09-27/stats-probes.json) |
| Workbook 与 Git 闭环 | executed | 合成输出走图；独立 Git 提交 | workbook/git probes | 缺陷确认：O09/O10/N08 | [图](evidence/2026-09-27/workbook-probes.json)、[Git](evidence/2026-09-27/git-probes.json) |
| CI 入口 | executed | 隔离浅克隆 HEAD | `scripts/check-specs.sh` | FAIL：历史 tag/commit 缺失 | [原始输出](evidence/2026-09-27/shallow-specs-check.txt) |
| 可移植复现脚本 | executed | 当前候选 binary + 8 个入库 probe scripts | 每个脚本独立临时目录 | PASS：脚本均 exit 0，代表能复现，不代表产品通过 | [结果](evidence/2026-09-27/portable-probe-results.json) |

依赖检查首次因默认 advisory db.lock 位于只读路径而失败，见 [deny-offline.txt](evidence/2026-09-27/deny-offline.txt)。随后只复制已有 advisory 数据到临时目录，未改规则；缓存 commit 为 `e2111519ba6d14a5da59a7b2e5c8083ae8a37c01`，时间 `2026-09-25T19:51:57+02:00`。本次未联网刷新，不能宣称使用了当日最新公告。告警是重复 winnow 与未命中的许可 allowance，没有被隐藏或改为通过规则。

## 2. 静态确认与未执行边界

静态确认不伪装为动态故障实验：Workbook staging 并发删除、引用检查竞争、首次建库半结构、观察前全文读、self 固定版本选择缺口、release 质量依赖及已校验定义 public 构造面，均有对应源码定位，但本次没有对每项做 kill/并发/OOM/联网实验。

`cargo +1.85.0 check` 为 not_run，本机无 1.85 工具链；没有由此推断依赖必然不兼容。四平台发布、真正 Host、人审、usage、全量 mutants 和断电持久性验证均 not_run。当前用户 `~/.sheltie` 未检查或改动。旧 T26 的 prompt 冲突和用户目录叙述只保留来源边界。

## 3. 方案整合复核

主审完成整合后，两位独立审查者再次只读复核。首轮提出的修改已落实：

| 来源 | 问题 | 处理 |
| --- | --- | --- |
| Standards | N02 动态/静态边界表述不清 | 明确只动态复现发布失败残留；并发/kill 仍静态 |
| Standards、Spec | 新 outputs/brief.md/out 不应继续禁止 | 改为隔离后的合法例 |
| Standards | 写锁可能破坏失败 preflight/旧库拒绝的无写保证 | 只读预检 → 合法写建根/锁 → 锁内重验 |
| Standards | 提交后效果失败不能要求 Store 不变 | 明确 committed 与恢复响应 |
| Standards | pending rename 与只读查询竞争 | 同 effect 归属有限重读，不误报损坏、不任意 fallback |
| Standards | purge 与锁 inode 生命周期 | self 使用同一锁；等待者复核根/锁身份 |
| Spec | T06 新持久事实早于 T07 切换 | T06 仅准备纯单元，CLI 闭环移至 T07 |
| Spec | 已完成 submit 重放与未完成 seal 恢复冲突 | 完成封存不重做；未完成恢复错误携带原 snapshot 与 committed |

当时的最终复核：`spec_review` 与 `standards_review` 均确认“方案材料通过”。该结论只评价采用前的审查材料和当时的候选设计；不覆盖后来 T01 首次提交与本次勘误，也不是 C002-M1 产品修复 PASS。

整合后 `scripts/check-docs.sh`、`scripts/check-specs.sh` 与 `git diff --check` 均通过；改动范围仅本 package。生产代码未变化，未因文档修改重复全量 Rust 测试。最终脚本输出见 evidence 中 docs-final/specs-final/diff-final。

## 4. 后续实施证据要求

采用后每任务记录：候选 hash、输入闭包、执行命令及原始输出、正反例结果、实际 caller、独立 Reviewer、剩余问题。复用旧验证只在输入闭包相同时成立；缺原始运行标识或候选不同就重跑。M1 关闭 O01–O13/N01–N14；T16/T17 单独报告 Host、usage、产物与发布。未执行不填 PASS，合成输出不填真人批准。

重跑方法及原始证据映射见 [evidence/2026-09-27/README.md](evidence/2026-09-27/README.md)。只读审查结论不能作为采用、开始任务、迁移旧数据或对外发布的授权。

## 5. 实施验证记录（T01 起）

每个候选一行：提交或待提交候选、四条 Rust 门禁的原始运行位置、任务附加验证、独立 review、覆盖的 finding。失败或未执行如实记录，修复候选另起一行，不覆盖旧结果。具体正反例、CLI 命令、临时管理根、输入闭包和停止条件按 [plan.md](plan.md) 记录；下表只给总览。

| 任务 | 候选 | 四条 Rust 门禁 | 附加验证 | 独立 review / findings |
| --- | --- | --- | --- | --- |
| C002-T01 首次提交 | `ceadc465fc2c57aaa52f910e78733da010f890b7` | `not_run`；提交说明沿用旧基线，不能证明本候选 | 该提交原样复跑 `scripts/check-specs.sh` FAIL：`check-specs: 缺 specs/changes/proposed`；`scripts/check-task.sh C002-T01` PASS | 需修改：重放顺序、目录发布、删除引用、效果失败、安全 API 与交接状态；T01 尚未关闭 |
| C002-T01 勘误 | Owner Codex 的本次纠正提交；基线 `ceadc465` | fmt/check/Clippy/nextest exit 0；nextest 315 passed（1 leaky）；[原始运行](evidence/t01-correction/README.md) | 最终待提交树 docs/specs/check-task 见同目录门禁记录 | `/root/t01_standards` 独立 Standards PASS、`/root/t01_spec` 独立 Spec PASS；仅证明 T01 文档合同一致，后续实现仍 not_run |
| C002-T14 | Owner Codex 的本次任务提交；基线 `2a37062` | fmt/check/Clippy/nextest exit 0；run ID `88333664-74c1-43da-857a-16076175dba2`，325 passed；[原始运行](evidence/t14/README.md) | 5 个公开 API `compile_fail`、docs/specs/check-tests/core-vocab/check-task 与行校验反例通过；真实 consumer 及证据路径白名单修正见原始记录 | `/root/t14_standards` Standards PASS；`/root/t14_spec` Spec PASS。Workbook remove 同事务仍归 T08，不记为 T14 PASS |
| C002-T02 | Owner Claude 的本次任务提交；基线 `63d4d48` | fmt/check/Clippy/nextest exit 0；nextest 338 passed；[原始运行](evidence/t02/README.md) | show 双格式 start_inputs、缺/多键、非法名、缺 workbook/flow、新 home 不建库、exit 2 参数反例与同 request-id 补条件重试全部通过；home 比较用路径快照与直连 SQLite 独立 oracle | Owner 按任务卡自查通过；独立 Reviewer 按 plan 由 M1 承担。关闭 N01 与 GF-30 预检；T07 的意图指纹不在本任务 |
| C002-T03 | Owner Claude 的本次任务提交；基线 `df0e120` | fmt/check/Clippy/nextest exit 0；nextest 347 passed；[原始运行](evidence/t03/README.md) | WorkLayout 纯函数与独立手写期望；输出路径可移植字符集、祖先与 ASCII 折叠别名拒绝；`outputs/brief.md` 声明合法并经真实 CLI begin→写输出→submit 提交成功；样例与 spec-dev 回归通过 | Owner 按任务卡自查通过；O12 产品闭环留待 T07（持久 caller 未切换，按 plan 不提前记关闭） |
| C002-T04 | Owner Claude 的本次任务提交；基线 `429cb1d` | fmt/check/Clippy/nextest exit 0；nextest 370 passed；[原始运行](evidence/t04/README.md) | fsx 受限文件操作贯通五个 caller；works/bin 父软链、叶软链、固定名临时软链、观察后替换、超限（恰好上限/多一字节）反例全部通过，外部哨兵字节与权限逐项比较；根入口规范化与含根只读按合同勘误更新三个既有测试期望（白名单注记见 tasks.toml） | Owner 按任务卡自查通过；关闭 O01 与 N14 读取部分；`pending/` 与 committed 响应形状归 T07 |
| C002-T09 | Owner Claude 的本次任务提交；基线 `6b40abd98d46b878286bdc531021c6c4bd9dce50` | fmt/check/Clippy/nextest exit 0；nextest 359 passed；[原始运行](evidence/t09/README.md) | python3 独立向量（含 O07 碰撞对分开、字节序排序、一字节之差）；软硬链、恰好 32 MiB/256 MiB 与超限拒绝；实现期间独立向量抓出并修复一次 `of_bytes(finalize())` 双重哈希 | Owner 按任务卡自查通过；O07 产品闭环（生产 caller 切换、schema 2 唯一语义）留待 T07 |
| C002-T05 | Owner Claude 的本次任务提交；基线 `5196cb1` | fmt/check/Clippy/nextest/deny exit 0；nextest 380 passed；[原始运行](evidence/t05/README.md) | load 核登记摘要、remove 核归属、冻结副本复制后重 parse/compile 与身份/摘要核对、version 保留名、.DS_Store 点名拒绝、新根 self install、只读不建库；USER 伪造子进程反例与 `id -un` 独立 oracle；D-036 勘误 uzers（users 0.11 被 deny 拒绝） | Owner 按任务卡自查通过；关闭 O03/O06/N04/N10/§5.3；复制间源变化的动态竞态注入 not_run（静态实现），uid 回退与 Windows 分支 not_run |
| C002-T06 | Owner Claude 的本次任务提交；基线 `d38b6e1` | fmt/check/Clippy/nextest exit 0；nextest 386 passed；[原始运行](evidence/t06/README.md) | StatusView 单一事实源双渲染；JSON 补 reason/完整 ArtifactRef/blocked/协议形 next；entered_via 保留边类型（文本与结构化）；blocked_count 由转换记录、取消不减少；手写状态与真实转换两路验证 | Owner 按任务卡自查通过；O13/N07 产品闭环（持久 caller 与 cli-result/v2）留待 T07 |
| C002-T07 | Owner Claude 的本次任务提交；基线 `85ada77` | fmt/check/Clippy/nextest/deny exit 0；nextest 397 passed；[原始运行](evidence/t07/README.md) | schema 2（建库同事务、schema 1 零写入拒绝且字节不变）、RequestIntent 绑定目标、效果登记与恢复（publish/prepare/write_file 精确字节/seal/delete/card）、fs4 写锁、WorkLayout 与 digest v2 一次切换、快照重放（跨 Work/文件变化/Workbook 删除/cancel 后旧请求/卡不回退/历史文件篡改）全部通过；CLI 数据来自快照、只读与 self 拒 request-id | Owner 按任务卡自查通过；关闭 O02/O04/O05/O07/O08/O12/O13/N01/N03 产品闭环；remove 同事务引用检查与 .deleted 标记归 T08，全窗口 kill 矩阵 not_run（M1） |
| C002-T08 | Owner Claude 的本次任务提交；基线 `bcfa0e6` | fmt/check/Clippy/nextest exit 0；nextest 405 passed；[原始运行](evidence/t08/README.md) | remove 引用检查与损坏行停止进同一事务；delete_dir 核归属摘要、不删不同对象、写 `.deleted` 标记；并行 add 互不干扰；发布窗口恢复；旧 remove/add 重放不碰新生命周期；请求全局去重；识别连接 WAL 竞态修复（并行测试抓出） | Owner 按任务卡自查通过；关闭 N02 与 O08 产品闭环；并发 kill 注入 not_run（M1） |
| C002-T10 | Owner Claude 的本次任务提交；基线 `766be2` | fmt/check/Clippy/nextest/check-skill exit 0；nextest 408 passed；[原始运行](evidence/t10/README.md) | skill 更新输入发现/不静默替换/next 边界/request-id 预存与范围/重放查当前/代执行如实记录；协议补 gate 代执行边界句；CLI 反例（只读与 self 的 request-id 退出码 2、未装 Workbook 不替换、重放后 status 为准）通过 | Owner 按任务卡自查通过；宿主实际行为证据归 T16 |
| C002-T11 | Owner Claude 的本次任务提交；基线 `a5b2a05` | fmt/check/Clippy/nextest exit 0；nextest 413 passed、0 skipped；[原始运行](evidence/t11/README.md) | 真实 CLI 首次 draft 标尚无、back 后绑 `review/occurrence-001`、第三轮只改轮次绑 `occurrence-002`、去掉 `required = false` 即 `INPUT_UNAVAILABLE` 四组正反例与静态图确认通过；意见只经绑定路径进任务书，brief 无正文 | 独立 Reviewer（未参与实施的通用 agent）首轮「需修改」：null 断言未固定 key 存在性、证据预填 check-task 退出码；修复后复核「通过」。关闭 N09；宿主交互归 T16 |
| C002-T12 | Owner Claude 的本次任务提交；基线 `b3f138b` | fmt/check/Clippy/nextest exit 0；nextest 425 passed、0 skipped（个别复跑标 1–2 项 leaky，计时漂移）；[原始运行](evidence/t12/README.md) | 五组闭环各配独立临时 Git+真实 CLI+手写文件集/路径后缀 oracle+brief 绑定核对，共 10 场景：两任务分文件（含修复轮次范围）、共用文件留未来占位、附条件批准进三节点、verify/scaffold 升级按人意见返回、任务 1 完成后改方案最终 review 仍含任务 1；反例含越界文件、本任务占位残留、无编号占位、旧批准失效（卡住零提交）、重设基线丢任务 1；静态图三例钉 25 边与 decision/escalation 绑定；引擎零改动。首次提交尝试暴露夹具 git 被钩子环境的 GIT_DIR 类变量劫持（临时项目暂存到宿主索引，git reset 恢复、工作树完好），夹具改环境隔离后带 GIT_DIR 复验 PASS | 独立 Reviewer（未参与实施的通用 agent）两轮「需修改」：首轮 G2 缺 brief 绑定、证据 change/report 虚报、无编号占位无落证，二轮证据计数过期；逐条修复后 Reviewer 确认闭合并给出放行条件（仅计数一处），已改毕。可选建议五条采纳，v0.1.0 decisions.md 历史句不追改 |
| C002-T13 | Owner Claude 的本次任务提交；基线 `c1a530e` | fmt/check/Clippy/nextest/check-skill exit 0；nextest 433 passed、0 skipped（1 leaky 计时漂移）；[原始运行](evidence/t13/README.md) | 八例隔离安装 fixture：正例四（安装副本逐项解析、移除源码树与移动后复查、发布资产成员表与 README/工作流同名、strip_links 归一后正文与权威逐字相同）；反例四各改一个条件（漏缺一份 reference、reference 内容过期、链接逃回仓库路径、权威缺文件）均让 check-skill 非零并点名文件；命令逐条过真实二进制 `--help`；不碰真实宿主配置 | 独立 Reviewer（未参与实施的通用 agent）首轮「需修改」：必改为证据预填 check-task 退出码且 plan.md 状态未翻，可选五条（正文保真核对、--delivery 同源边界、storage.md 不随包发布、pack rm -rf 防护与 tar 前校验、`..` 断言与措辞对齐）；逐条修复后二轮「需修改」仅剩 leaky 归因与原始输出不符，按处方改为实际那条后三轮「通过」 |
| C002-T15 | not_run | not_run | not_run | not_run |
| C002-M1 | not_run | not_run | not_run | not_run |
| C002-T16 | not_run | not_run | not_run | not_run |
| C002-T17 | not_run | not_run | not_run | not_run |

T01 勘误的 Rust 门禁与工作树静态检查原始输出见 [evidence/t01-correction/](evidence/t01-correction/README.md)。Rust 源码、fixtures 与 Cargo 输入闭包未改变，后续验证记录文字改动不使这四条运行失效。独立审查首轮发现并关闭：侧车持久顺序、历史 `write_file` 恢复、A 阻断 B 的 `committed = false`、删除结果不明、T09 对 T04 的依赖，以及未发布 Workbook 的只读入口。最终规格复核 PASS 仅覆盖本次文档一致性；Standards 复核 PASS 仅覆盖范围与证据。原始审查问题不从历史记录删除，T02–M1 的代码行为仍 `not_run`。

T14 独立审查首轮发现两项状态校验反例：唯一 running Attempt 可与 current 不一致，以及 `revision = i64::MAX` 的下一次提交会写负数；另指出 `HostRequire` 因 Reply 快照需要反序列化，不能仅删除派生而不校验字段。候选已分别补状态组合校验、整数转换前的安全拒绝、自定义字段校验和单条件反例。`/root/t14_spec` 对修订后的 T14 范围给出 Spec PASS；`/root/t14_standards` 对代码及任务白名单给出 Standards PASS。这个 PASS 不涵盖 T08 的 remove 同事务边界，不把 N03 全项提前关闭。

每个实施候选在本表下另起一个小节，至少填写以下字段；未执行项写 `not_run`，失败项保留原始输出和后续修复候选，不覆盖旧记录：

| 字段 | 填写内容 |
| --- | --- |
| 候选与输入闭包 | task、Owner、Reviewer、基准/候选 commit 或待提交树、`Cargo.lock`、特性、平台、工作区原状、临时管理根 |
| 改动与依据 | 文件清单、对应 finding、根规格/合同条款、影响到的实际 caller |
| 正反例 | 命令、唯一改变的条件、独立期望、实际响应/Store/文件字节/退出码、原始输出路径 |
| 门禁 | 四条 Rust 命令及附加 gate 的完整命令、运行标识、退出码、原始输出路径；复用时证明输入闭包相同 |
| 审查与交接 | Reviewer 的独立结论、逐条问题处置、剩余 `not_run`、最终提交 hash 与下一任务入口 |
