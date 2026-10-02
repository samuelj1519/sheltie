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
| C002-T15 | Owner Claude 的本次任务提交；基线 `4960e9f` | fmt/check/Clippy/nextest/deny exit 0；nextest 464 passed、0 skipped、1 leaky（点名到用例）；两次负载偶发的失败运行与归因见 evidence 处置记；[原始运行](evidence/t15/README.md) | C002-T15 归属 32 例（31 新增 + 1 改写）+ 复验 T20/T23 窗口用例：五个失败窗口（下载/摘要/解包/替换/rollback）各配单条件反例与旧二进制不变、tmp 清空 oracle，update/rollback 前后 `store.db` 字节不变另在两个生命周期用例核；固定 tag 四例（latest 漂移不混包、伪造资产名/版本不越界、清单与 tag 不符）；purge 等待者与 `.lock`/根身份复核三例、self/Work 并发串行；`--json` 纯协议文档、schema 2 提示、clean home two-step、指定版本与 rollback；治理夹具七例（active 目标/RC 不要求 tag、越界版本拒绝、缺 CHANGELOG 段拒绝、浅克隆报缺历史、缺 tag 报未创建）；工作流三例（announce 挂同 SHA quality，两个单条件反例证明质量失败不能发布）；MSRV 1.85 locked 实跑 exit 0，`dist plan` 真实形状存 evidence | 独立 Reviewer（未参与实施的通用 agent）首轮「需修改」（计数、dist 形状、`store.db` 断言虚报、`checked_version` 漏 NUL），二轮「需修改」仅剩证据换真（nextest 失败运行摆在 exit 0 后），三轮「通过」；逐条处置见 [evidence/t15/README.md](evidence/t15/README.md) 独立审查段 |
| C002-T18 | `69710aa` | fmt/check/Clippy/nextest exit 0；nextest 464 passed；[原始探针与门禁](evidence/t18-api-probe-2026-09-28/README.md) | 上游文件与恢复合同、rustix/SQLite bundled API在macOS arm64与Rust 1.85.0探针通过；Linux原生探针`not_run`（用户豁免） | 独立Spec与Standards PASS；只固定机制合同，不代替T19–T31产品实现 |
| C002-T19 | `ee78118`，基线`69710aa` | fmt/check/Clippy/nextest/deny/MSRV exit 0；最终全仓nextest run `00bfcb15-586c-427a-9f6b-99a904abd2f1`：475 passed、0 skipped；[原始输出](evidence/t19-managed-fs-2026-09-28/README.md) | ManagedRelPath/目录句柄和所有旧mutating helper迁移到HomeLock根身份校验；父fd路径替换、跨Home错锁、打开后硬链接chmod拒绝、两种rename链接源拒绝、`.tmp-<uuid>` oracle、结构化NotFound锁构造重试、purge保留根/.lock；T19定向12 passed；T23完整purge失败协议与T31等待者交错保留 | 独立Spec与Standards最终PASS；typos对`WRONLY`的窄词表修正见T19 evidence；不提前关闭后继R项或M1。Linux原生验证`not_run` |
| C002-T20 | 复核基线`ee78118`；最终候选为含本行和原始日志的任务暂存树 | fmt/check/Clippy/nextest/deny/MSRV全仓门禁exit 0；最终nextest run `39ad8a9c-a292-4a00-9f68-cc9d1535c6e1`：492 passed、0 skipped、0 leaky；T20定向最终17 passed、2 leaky、475 skipped；[原始输出](evidence/t20-trusted-load-2026-09-28/README.md) | Work根、冻结manifest、历史输入/输出、audit/response/command及Raw effects闭包精确核验；Workbook行与manifest id/version/digest/time/dir绑定；完整效果形状及顺序检查，serde owner文件和旧schema2 audit原字节恢复；缺audit、重复audit、错digest、缺必需字段、路径/快照单字段篡改均在首个效果IO前拒绝 | 独立Spec与Standards最终PASS；T21枚举/摘要caller、T22封存、T23 self/purge、T24锁前初始化、T25–T28恢复/删除证明和T31窗口仍未关闭。Linux原生验证`not_run` |
| C002-T21 | 基线`22942ee`；最终任务提交包含本行和[原始证据](evidence/repairs/t21/README.md) | fmt/check/Clippy/MSRV/deny/typos/docs/specs与治理脚本exit 0；全仓Nextest run `8cc95446-f6f1-4bfd-91be-8e8653c31956`：502 passed、0 skipped、1 leaky；T21定向run `d7f8c42d-a6ea-4c97-a86f-9987d2af3a65`：10 passed、492 skipped；影响面runtime 80 passed、CLI 18 passed | 六类目录caller接入受限树；managed树从Home句柄逐段no-follow；digest/ResourceIndex/资源SHA同次读取，manifest/Flow/说明书捕获字节复用到Work begin；摘要独立向量未变；根/父软链、非UTF8、超限先拒、同inode同长度正文变化、UTF8分块与`add .`路径回归通过 | Spec `/root/spec_review` PASS、Standards `/root/standards_review` PASS。只关闭T21自身；T22–T31与M1未关闭。Linux `not_run`，无跨平台PASS |
| C002-T22 | 基线`543f9d2`；T22任务提交附本行及[evidence/repairs/t22](evidence/repairs/t22/README.md) | fmt/check/Clippy/MSRV/deny/docs/specs与治理脚本exit 0；全仓Nextest run `025d54a2-a67d-417e-aa7a-7b6002be84ee`：516 passed、0 skipped、3 leaky；T22定向run `f3c1a700-8279-466a-b79f-6ab10a560d91`：14 passed、502 skipped；影响面runtime 68 passed、CLI 5 passed | 真实submit在COMMIT后/Seal前同步换路径、改同inode bytes、增加硬链；三例均检查原ArtifactRef/effects ref、WorkId、revision+1、published0、提交错误；原/外部权限与哨兵逐项核验。V08恢复三种单条件反例；完成重放不重封；正常CLI提交置只读。Linux `not_run` | Spec `/root/spec_review` PASS；Standards `/root/standards_review` PASS。只关闭T22自身；T23–T31与M1未关闭 |
| C002-T23 | 基线`fd21a63`；待提交任务树与原始输出见[evidence/repairs/t23](evidence/repairs/t23/README.md) | fmt/check/Clippy/MSRV1.85/deny/docs/specs/test-owner脚本exit 0；全仓Nextest run `29a636e5-cd43-4218-9063-df2603c9ac68`：528 passed、0 skipped；T23定向run `5206d173-c4da-457c-8a2a-1cabecfd07bc`：12 passed、516 task-filtered；影响面runtime 61 passed、2 leaky、CLI 8 passed | 已核字节贯穿release下载、解包、SafeFile chmod、rename/exchange与落位后复核；候选同inode改字节单条件反例在移动旧版本前拒绝。RecoveryRequired显式保留不确定现场；tar `--`、两流限额、bounded子进程回收。purge有冻结树、late `store.db-shm`、unlink原锁后的最终复核反例；相对release base真实CLI测试 | Spec `/root/spec_review` PASS；Standards `/root/standards_review` PASS。Linux `not_run`；T31等待者交错与C002-M1仍未关闭 |
| C002-T24 | 基线`7e9178e`；任务输出见[evidence/repairs/t24](evidence/repairs/t24/README.md) | fmt/check/Clippy/MSRV1.85/deny/docs/specs/test-owner脚本exit 0；全仓Nextest run `4bc43a1b-65eb-4de4-b3da-bba47518259b`：552 passed、0 skipped、0 leaky；T24定向run `dd5cf04c-ebe1-4e45-b233-2c8b28fcd6fd`：25 passed、527 task-filtered | V04源文件内容变更/删除后的start重放，V12模糊前缀下历史full WorkId与wrong prefix，V13 32MiB/多一字节，V14读错不增request/sequence/work；submit/fail deleted @file重放，名称Intent状态冲突。真实Schema1+WAL通过Service/Repo/install拒绝，main/WAL字节不变且无.lock；Session锁竞争后Store删除不重建；孤儿sidecar不新建；合法SQLite旧库替换和根/锁替换均保留原对象 | Spec `/root/spec_review` PASS；Standards `/root/standards_review` PASS。Linux `not_run`；T25恢复、T31最终全窗口与C002-M1未提前关闭 |
| C002-T25 | `1d92bcd`；[任务实现与原始输出](evidence/repairs/t25/README.md) | fmt/check/Clippy/MSRV1.85/deny/docs/specs与全仓nextest exit 0；全仓run `2de4518b-59bd-45d2-87c2-79c9422f4127`：577 passed、0 skipped；T25定向run `b959050f-291f-4c4e-b1bb-b1c545fcfaba`：27 passed | Work/Workbook共用恢复编排；A阻B、snapshot投影、status card和mark顺序、损坏持久值与错误类别均有真实caller oracle；pending清理诊断明确交T28/V25 | Spec与Standards独立PASS。仅关闭T25；T26–T31/M1及Linux运行不提前关闭 |
| C002-T26 | 提交前候选，基线`1d92bcd`；[任务实现与原始输出](evidence/repairs/t26/README.md) | fmt/check/Clippy/MSRV1.85/deny/docs/specs与全仓nextest exit 0；全仓run `871f460c-ab00-4531-87c2-179b50b98d4e`：585 passed、0 skipped、1 slow；T26定向run `8bcc2825-029a-400b-91a1-94069b101a90`：8 passed、577 task-filtered | Work起始输入精确字节/长度与owner字段单条件反例；Workbook pending/final manifest/sidecar核验；9个命名sync故障点重复注入、final-only父sync重试、同rid committed=true完整original，以及sync后目录换绑反例；真实add/start/status/show caller | Spec与Standards独立PASS；仅关闭T26。Linux `not_run`，T27–T31/M1边界保留 |
| C002-T27 | `3dd224d`，基线`5a9d430`；[任务实现与原始输出](evidence/repairs/t27/README.md) | fmt/check/Clippy/MSRV1.85/deny/docs/specs与全仓nextest exit 0；全仓run `7e61974e-fbfb-486b-82ad-0bc5ca7e37cb`：593 passed、0 skipped、1 slow、1 leaky；隔离run `0c5a32ca-e8df-4f68-ad19-cefcbba611f2`：相关CLI测试1 passed、无leaky；T27定向run `e17ccf93-e371-4e9b-87b4-d4e29358cfc6`：8 passed、585 task-filtered | V21真实delete→marker crash、partial-delete crash后摘要停止、marker cross-id/bad-format与校验后替换；V22不同摘要final/payload保留；root unlink前空目录换绑保留；V23新生命周期bytes/row/reply原样；缺目录/空摘要COMMIT前拒绝 | Spec与Standards独立PASS；leaky原因未确认。保留HomeLock协作边界。Linux `not_run`；T28–T31/M1未提前关闭 |
| C002-T28 | `4526b7e`，基线`3dd224d`；最终任务提交附本行及[原始证据](evidence/repairs/t28/README.md)，源码输入sha见candidate-input.txt | fmt/check/Clippy/MSRV1.85/deny/docs/specs/core-vocab/tests/skill exit0；全仓run `6e159189-3ad3-42c1-a4c8-e41a0b85304f`：615 passed、0 skipped、1 slow；T28 run `348c5ee9-71fb-473e-920a-b5a53ec80d7e`：22 passed、593 task-filtered | V24–V27覆盖全部effects引用、readonly孤儿、异常/非空树保留、同句柄empty清理、真实rename与mark/cleanup交错、当前publisher全闭包、same-second lifecycle反例、只读主库字节/.lock与完整stderr维护载荷；审查期间四项回归失败原文保留 | Spec `/root/spec_review` PASS；Standards `/root/standards_review` task-local PASS；仅关闭T28。Linux `not_run`，早期Nextest LEAK原因未确认交T31；T29–T31/M1边界保留 |
| C002-T29 | `a4f1968`，基线`4526b7e`；最终任务提交附本行及[原始证据](evidence/repairs/t29/README.md) | fmt/check/Clippy/MSRV1.85/deny/docs/specs/core-vocab/tests/skill exit0；全仓run `85201df7-8fc1-456c-9620-a9b345dc14e3`：616 passed、0 skipped、1 slow、1 leaky；T29 run `ce4bf2ae-5c76-4f19-ab7e-3627391a13aa`：1 passed、615 task-filtered | V28真实CLI装入后writer begin交错；手写旧stats/next和新查询oracle；原代码红run `62afea1c-9e02-47ba-896c-bad43181426e`；reader读锁反实现被writer5秒timeout捕获并回收，源码逐字节恢复 | Spec与Standards独立PASS；只关闭R15/T29。静态文档测试LEAK原因未确认交T31。Linux `not_run`；T30/T31/M1边界保留 |
| C002-T30 | 基线`a4f1968`；[当前输入闭包及原始证据](evidence/repairs/t30/README.md) | fmt/check/Clippy/MSRV1.85/deny/docs/specs/core-vocab/tests/skill exit0；全仓run `2aa8b4a7-65f2-4953-8222-4894e81ff4b2`：625 passed、0 skipped、1 slow、无LEAK；Task run `94c227dc-78a8-4b5b-867c-7b4fbbf247ab`：9 passed、616 task-filtered | V29/V30 required原字节镜像、递归冷读前缀/Git/门禁、失败缺字段及人工更正、真实升级继续/整体旧fix及深历史拒绝；验收中nested Cargo构建竞态原文保留并完整迁移所有真实CLI测试 | 独立Spec/Standards含迁移增量PASS；只关闭T30/R16。Linux `not_run`；模拟worker不代表T16；T31窗口/突变与M1仍待完成 |
| C002-T31 | 基线`b926789`；最终源码输入`49d3a191`，产品提交包含本行及[最终证据](evidence/repairs/t31/README.md) | 默认675/675、零skip；fmt/check/Clippy/MSRV1.85/deny/docs/specs/core-vocab/tests/skill/dist exit0；最终task/staged记录见evidence | 正常确定性交错、真实exit70/SIGKILL窗口与恢复、R20计数界、七项实现修复及真实caller回归；完整2499项第一阶段，第二阶段245项终态；[逐ID分类](evidence/repairs/t31/mutants/final-dispositions.json)含269用户暂缓 | Spec与Standards认可2026-10-01安全验证豁免范围内T31提交；完整安全变异不PASS。Linux/M1/T16/T17保持not_run |
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

## 6. M1 预审与追加修复材料（2026-09-28）

固定候选e1a8126的 [独立审查](review-m1-2026-09-28.md) 为“需修改”；已运行门禁和19项发现/27项矩阵见 [M1 evidence](evidence/m1-2026-09-28/README.md)，保留失败与未完成边界。该轮未完成完整kill/突变门槛，不构成M1关闭。

用户已授权按 [repair-design](repair-design.md)、[repair-plan](repair-plan.md)、[repair-validation](repair-validation.md) 实施修复，并明确豁免Linux运行。C002-T18于2026-09-28完成：合同调整已按授权写入上游；macOS arm64/Rust 1.85.0/bundled SQLite 3.50.2探针通过；`git diff --check`、docs/specs/test映射、fmt/check/Clippy/nextest通过（464 passed）；独立Spec与Standards Reviewer通过。完整命令/原始输出见 [T18探针与门禁](evidence/t18-api-probe-2026-09-28/README.md)。Linux原生探针保留`not_run`，所有后续验证和最终结论限定为macOS；不把豁免写成Linux或跨平台PASS。方案材料独立审查与静态检查见 [repair-planning evidence](evidence/repair-planning-2026-09-28/README.md)。旧done或历史测试绿不转成新问题关闭。

## 2026-09-30 实现审查七项修复与T31接续

用户授权直接修复全部七项并完成T31。逐条答复在 [review-response-implementation-2026-09-30.md](review-response-implementation-2026-09-30.md)，原始红/绿与门禁在 [implementation-review-fixes](evidence/repairs/t31/implementation-review-fixes/README.md)，独立 [Standards](evidence/repairs/t31/implementation-review-fixes/standards-final-review.md) 与 [Spec](evidence/repairs/t31/implementation-review-fixes/spec-final-review.md) 均关闭七项且未参与实施。最终默认全仓Nextest654/654、0 skipped，Rust四门禁、MSRV1.85 locked、取得既有缓存锁后的离线deny、规范脚本、dist plan均exit0；原T24合同矛盾FAIL与此前published补缺未闭合的653项运行原样归档，不覆盖。

新普通clone临时输入3a9f689固定170个源码/fixture/配置/脚本，另由Git提交固定治理树，baseline在opt1且debug assertions/overflow checks开启时654/654；[完整变异入口](evidence/repairs/t31/mutants/closure.md)清单2501（core724/runtime1777）。旧da2bd结果标superseded，当前完整片、第二阶段与逐存活处置执行中，不能从基线或首片通过推断T31/M1完成。Linux按用户授权not_run，Host/usage与发布继续交T16/T17。

## 2026-10-01 M1 独立全链审查与精简

原实现候选`ca6d92f`；工作区起始干净。三位未参与实施的Reviewer分别核源码规格、Rust工程与原始证据，报告见[本轮审查](review-m1-2026-10-01.md)，O/N/R逐行矩阵与输入闭包见[证据](evidence/m1-2026-10-01/README.md)。原源码未发现新的高可信产品缺陷；本轮删除未使用参数、重复错误转发、未消费反序列化能力及持根锁后的自动CAS重算，保留单事务CAS；同步统一Store合同内部冲突并修正start意图名字描述、删除过时T01状态句。源码和合同增量独立审查通过。

精简后的675/675测试、fmt/check/Clippy、MSRV1.85 locked、离线deny、docs/specs/core-vocab/tests/skill与dist plan通过，原文保存于本轮证据。最终治理文件调整后的复验独立保存，不套用旧T31变异输入。首次deny只读缓存锁失败原文保留，权限批准后离线运行exit0；不宣称公告数据实时更新。

当前M1结论需修改：R17/N13的完整变异义务未关闭，269项仍deferred_by_user；现有授权仅明确T31在豁免范围收尾，不能自行扩大为M1通过。已请求用户明确本次M1是否恢复这些验证。Linux豁免仍not_run，T16/T17仍not_run。任务门禁要求M1 done；在范围澄清与完整门槛关闭前不伪造done来通过该门禁，也不宣称M1提交完成。

## 2026-10-01 T32 M1审查反馈修复

用户授权恢复剩余M1验证，实际平台安全拦截才逐项记录跳过。追加T32承接审查反馈：4项runtime源码精简、Store CAS合同统一及3项真实caller回归。Spec/Standards独立最终增量review通过；678/678零skip、fmt/check/Clippy、MSRV1.85 locked、离线deny、docs/specs/core-vocab/tests/skill/dist门禁通过，原文见[T32证据](evidence/m1-2026-10-01/t32/README.md)。旧instruction正例违反64KiB合同的设计错误与编译错误均保留，不当产品失败。最终变异由M1重新冻结输入执行，未据本任务门禁判caught或M1通过。Linux/T16/T17边界不变。

## 2026-10-01 T33 根与路径解析、M1漏检回归

T33修复真实R21：Home::resolve/confine吞I/O或保留原路径，lexical_abs猜测cwd/用lossy字符串解释路径。统一失败传播、严格UTF8和唯一解析caller；删除只测试使用的Home::at并迁完caller。新增15项真实caller回归及既有Row校验补revision0；693/693零skip，Rust四门禁/MSRV/离线deny/规范/skill/dist通过，独立Spec/Standards最终增量review通过，见[T33证据](evidence/m1-2026-10-01/t33/README.md)。实际非UTF8实体路径fixture在当前APFS创建EPERM原文保留，不当安全拦截或实际验证PASS。

旧5261e0d/2495输入原文保留为superseded：core4完整3caught1missed、runtime第0片222项167caught/13unviable/42missed均经独立证据Reviewer核实际build/test/diagnostic及逐成员SHA；源码变化检测在下片启动前停止，不套新候选。最终完整变异由M1重新冻结本任务提交后输入执行。Linux not_run，T16/T17 not_run，实际平台安全拦截记录为空。


## 2026-10-01 T34 快照业务绑定修复

实施Owner Codex，父提交T33 `06c3af3`。R22/R23/R24独立真实CLI红色证据与逐字段/全Store/业务文件oracle见[evidence/m1-2026-10-01/t34](evidence/m1-2026-10-01/t34/README.md)。原响应经元数据、audit、业务绑定校验后提供，合法后效果失败仍携原响应；core生成/验证共用状态与节点资源事实，历史结果不比较当前visits。退役raw投影、未消费查询及冗余判断，Box cause关闭Clippy大Err。

最终source输入`8657e13fb93167a486511f4d6ee542bfb5eacf4fc5e497e22abde03765cb6d8f`，全仓run `79c91669-2f11-422e-9cc8-4b91b7de5df3` 699/699、零skip、1 slow，exit0。fmt/check/Clippy、Rust1.85 locked、docs/specs/core-vocab/tests/skill/dist均exit0；offline deny exit0，仅使用记录中的本地公告缓存。原始argv/exit/SHA在T34 gates及metadata；先前fixture编译/只读连接、资格旧断言和test-owner注释失败均保留，未掩盖失败。

独立Standards当前源码通过。Spec已返回的反例/seam/方向记录保留，其后续复核实际触发平台风险提示并暂停，按用户明确授权记录`M1-safety-skip-001`、跳过、不重试；不写最终Spec通过。T34 done只适用于本次明确例外；缺失义务见[清单](evidence/m1-2026-10-01/safety-skips.json)。

旧5344候选runtime第一阶段1773完整、core旧720及core4workspace保留；源码guard在runtime workspace创建前停止exit1，非安全跳过。T34改变core/runtime，新M1不得复用任何旧运行为当前PASS，必须重新冻结并全量运行两阶段及存活处置。Linux not_run；M1 doing；T16/T17 not_run。任务/staged、提交与提交后门禁另记。


## 2026-10-02 M1 最终分组验收

产品候选e3eea899、变异冻结候选95d78e0，输入闭包与运行身份见[最终审查](review-m1-2026-10-02.md)。[当前门禁](evidence/m1-2026-10-01/current-acceptance/gates/gate-results.json)全部exit0，Nextest run `2e126f6a-1496-4b0b-8d71-583e22d4065e`为699/699、零skip；MSRV1.85 locked通过，deny仅核本地缓存公告。源码未再改动，最终文档门禁另存current-acceptance/finalization。

[独立账本核算](evidence/m1-2026-10-01/mutants/adaptive-validation-2026-10-02/independent-final-accounting.json)通过：2514=1812首轮caught+328编译unviable+74限定静态处分+33完整workspace捕获+30能力CLI捕获+22补充检测+215未完成额外执行。22项包含16直接、6受控观察，原正式Missed标签不改写；中断29项与取消18项均不计完成。

SK01最终Spec续审与SK02新共享事件oracle任务均实际暂停且未重试；[逐项缺失映射](evidence/m1-2026-10-01/mutants/adaptive-validation-2026-10-02/SK02-missing-execution-map.json)不虚构215次提示。[验收范围](evidence/m1-2026-10-01/acceptance-scope.json)区分M1授权例外内完成与full_validation_pass=false。Linux、T16/T17保持not_run；物理非法UTF8目录fixture因环境EPERM未创建。
