# C002 实施计划

状态：`proposed`，产品修复 `not_run`
基准：`31d7ddee18921b4066c7433a752c2000e5110869`
来源：[T26 同步提交与 Work 目录复审](findings.md)

产品 delta 见 [spec.md](spec.md)，候选设计见 [design.md](design.md)。

本文是修复提案，不是当前进度权威。采用方案时，先执行 C002-T01，把上游合同与本 package 的 `tasks.toml` 同步后再改代码。

## 任务依赖

```text
C002-T01 上游合同与 repair task 治理
 ├─ C002-T02 StartRequirements 与零副作用 preflight
 ├─ C002-T03 WorkLayout readable_v2
 │    └─ C002-T04 受限输出观察与封存
 ├─ C002-T05 Workbook identity、只读根、新管理根与版本路径
 ├─ C002-T06 单一状态投影
 │    └─ C002-T07 Work 请求日志、start staging 与效果恢复
 │         └─ C002-T08 Workbook 写操作请求日志
 ├─ C002-T09 workbook-digest/v1
 ├─ C002-T10 输入发现与协调者 skill 行为
 ├─ C002-T11 article-review 打回意见绑定
 ├─ C002-T12 spec-dev 任务闭包
 ├─ C002-T13 skill 可分发布局
 └─ C002-T14 core 已校验定义收口

C002-T04 + C002-T05 + C002-T08 + C002-T10..C002-T14 ── C002-T15 CI、MSRV 与发布门禁
C002-T02..C002-T15 ── C002-M1 独立全链审查 ── C002-T16 宿主回归 ── C002-T17 发布 v0.2.0
```

## 2. 通用执行规则

C002-T01 要先修改根权威文档与本 package 的 `tasks.toml`，明确 C002-T02 起是 post-MVP repair task。它们允许同一 Owner 修改测试、类型、实现和真实 caller，`allow_test_changes = true`；仍然一个任务一个提交、白名单闭合、每个提交可编译。MVP legacy `specs/plan.md` 与根 `tasks.toml` 不再追加。

每个实现任务必须：

1. 先增加一个能复现原问题、只改变一个条件的失败测试。
2. 正例和拒绝例都通过公开 CLI 或 module interface。
3. expected 不调用生产 helper。摘要、恢复字节和目录树使用手写或独立 oracle。
4. 完成局部测试后运行受影响调用链与全量门禁。
5. 提交正文写兼容范围、验证和尚未覆盖的风险；末尾写 `Task:`、`Agent:`。

最低门禁：

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
cargo deny check
scripts/check-docs.sh
scripts/check-core-vocab.sh
scripts/check-tests.sh
scripts/check-task.sh C002-Tnn
```

改 skill 另跑 `scripts/check-skill.sh`；改发布链另跑 `cargo dist plan`；C002-M1 统一跑 mutants。

## 1. 任务卡

### C002-T01 上游合同、T26 边界与任务治理

**Owner。** 强模型。
**依赖。** 无。
**文件。** `specs/{spec,architecture,engineering}.md`、三份 contracts、新 ADR、本 package 的 `README.md`、`plan.md`、`progress.md`、`validation.md` 与 `tasks.toml`。

1. 固定 StartRequirements、WorkLayout、请求生命周期、Workbook identity、摘要兼容与 v0.2.0 宿主回归决定。
2. 协议写清：全部确定性 start 校验在序号前；新目录布局；`workbook show` 的 `start_inputs`；`INPUT_MISSING` 无任何变化。
3. 存储合同写清 staging、COMMIT/rename 崩溃恢复和 legacy layout。
4. 在本 package 的 plan/tasks 中登记 C002-T02–C002-T17、C002-M1；T26 历史保持 `done`，C002-T16 依赖 C002-M1 与 rc。
5. 修正首次真实运行中的事实冲突、机制术语与证据来源，不删除原始失败事实。

**停止条件。** 不接受独立宿主回归、legacy layout 或摘要 v1 兼容时先由人裁决；不得由实现者猜。
**提交。** `docs(specs): 固定 v0.2.0 修复合同与宿主回归边界`

### C002-T02 StartRequirements 与零副作用 preflight

**Owner。** 强模型。
**依赖。** C002-T01。
**文件。** core `flow/graph.rs`、`work/decide.rs`，runtime `service.rs`，CLI workbook show，相关 tests。

1. 实现共享的 `start_requirements` 与 `validate_start_inputs`。
2. runtime 在 `allocate_seq` 前完成 Workbook/Flow、WorkName、缺/多输入和输入 key 路径校验。
3. `decide_start` 复用同一 validator，作为最终不变式检查。
4. `workbook show` 文本与 JSON 增加每张 Flow 的有序 `start_inputs`。
5. skill 修复在 C002-T10 完成；本任务只提供可靠 interface。

**测试。** 建议新增：

- `missing_start_input_does_not_allocate_sequence_or_create_work_dir`
- `extra_start_input_has_no_side_effect`
- `invalid_work_name_has_no_side_effect`
- `missing_workbook_cli_argument_never_enters_runtime`
- `workbook_show_lists_required_start_inputs`

**正例。** `show two-step` 返回 `start_inputs: ["topic"]`，完整输入的 start 成功。
**反例。** 缺 topic 后 `works/`、`work_sequence`、`works`、`requests` 与调用前逐项相同。
**提交。** `fix(runtime): start 校验先于序号与目录副作用`

### C002-T03 WorkLayout readable_v2

**Owner。** 强模型。
**依赖。** C002-T01。
**文件。** 新增 core `work/layout.rs`；修改 WorkState、decide/render、runtime caller、协议快照、测试与 `spec-dev` 路径说明。

1. 实现 `WorkLayoutVersion::{LegacyV1, ReadableV2}` 与 `WorkLayout`。
2. 旧 state 缺字段时默认 LegacyV1；新 start 使用 ReadableV2。
3. 使用 `start-inputs`、`occurrence-NNN/attempt-NNN`、`engine/`、`outputs/`。
4. begin 成功返回前保证 output_dir 存在；brief 位于 Attempt 根。
5. 拒绝输出之间相同、大小写折叠相同和祖先冲突。
6. 删除所有 caller 中的手工路径拼接。

**测试。** 建议新增：

- `new_work_uses_readable_v2_tree`
- `legacy_state_keeps_legacy_paths`
- `attempt_id_maps_to_labeled_directories`
- `engine_stats_and_worker_stats_json_are_distinct`
- `rejects_output_path_ancestor_conflict`

**正例。** `draft#2.1` 稳定映射到 `draft/occurrence-002/attempt-001`。
**停止条件。** 旧 Work 需要批量 rename 或升数据库 schema 时停止；兼容设计不成立。
**提交。** `refactor(core): Work 目录改为可解释的版本化布局`

### C002-T04 受限输出观察与封存

**Owner。** 强模型。
**依赖。** C002-T03。
**文件。** runtime `home.rs`、`observe.rs`、`service.rs`，runtime/CLI artifact tests。

1. submit 使用 `observe_confined_file(output_dir, declared_rel, max_bytes)`。
2. canonicalize 完整已存在路径并核对 output_dir；缺文件返回 None，软链、目录与越界准确拒绝。
3. 先看 metadata 和大小，再有上限地读取与摘要。
4. chmod 前再次 confinement 与摘要核对，不能修改目录外文件。

**测试。** 父目录软链、叶软链、观察后替换、合法嵌套输出四组正反例。
**停止条件。** 任何路径可绕过 WorkLayout 或只能靠吞错误通过时停止。
**提交。** `fix(runtime): 输出观察和封存限制在声明目录内`

### C002-T05 Workbook identity、只读根与本机生命周期

**Owner。** 强模型。
**依赖。** C002-T01。
**文件。** core manifest/version 类型，runtime `home.rs`、`workbook_repo.rs`、store read，CLI 与 tests。

1. `WorkbookRepo::load_installed` 核对数据库摘要、manifest id/version 和 Flow；start/show 只走该 interface。
2. `set_tree_readonly` 最后把传入根设为 `0555`。安装副本和 Work 冻结副本使用同一实现。
3. add 遇到 `.DS_Store`、`Thumbs.db` 等合同列出的宿主元数据时拒绝，不复制、不登记。
4. 引入 `WorkbookVersion`，在写库前拒绝 `.`、`..` 与以点开头的版本路径。
5. ReadWrite 操作可创建不存在的 SHELTIE_HOME；ReadOnly 不创建。

**测试。** 建议新增：

- `installed_workbook_root_is_readonly`
- `readonly_root_can_be_renamed_by_writable_parent`
- `add_rejects_host_metadata_files`
- `start_rejects_registered_workbook_digest_drift`
- `dotdot_version_leaves_no_row_or_directory`
- `install_and_add_create_missing_home`

**正例。** 正常冻结副本可 status；目录根不能自动写 `.DS_Store`。
**反例。** 修改已装 Flow 后 start 返回 `WORKBOOK_TAMPERED`，不创建 Work。
**提交。** `fix(runtime): Workbook 按登记身份冻结并锁定目录根`

### C002-T06 单一状态投影

**Owner。** 强模型。
**依赖。** C002-T01。
**文件。** core render/state、CLI status、快照与测试。

文本状态卡与 JSON 状态卡先生成同一个 `StatusView`；`last_attempt` 同时表达 summary 与 fail_reason。两种输出的信息选择规则只有一份。

**测试。** fail 后文本与 JSON 的 reason 一致；删掉任一投影字段时一致性测试失败。
**提交。** `refactor(core): 状态卡文本与 JSON 共用事实投影`

### C002-T07 Work 请求日志、start staging 与效果恢复

**Owner。** 强模型。
**依赖。** C002-T02、C002-T03、C002-T06。
**文件。** 新增 runtime `request.rs`；修改 service、store commit/read、CLI 写响应、replay/crash tests。

1. 实现 RequestIntent、ResponseSnapshot、DurableEffect、StoredOutcome。
2. CLI 写响应不再提交后读 Store；snapshot 包含提交时 status、next、批准记录与身份字段。
3. 请求命中先于 load/observe；intent 含目标 Work，不含当前文件观察。
4. brief、engine/stats、status-card 保存 exact bytes；重放只补做保存效果。
5. start 在 `tmp/starts/<request-id>` 物化，状态提交后 rename 到最终 Work 目录；效果可重放。
6. legacy 请求不能精确恢复时明确失败，不用当前状态猜。

**测试。** 覆盖跨 Work request-id、文件变化后 submit 重放、Workbook 删除后 start 重放、cancel 后旧响应稳定、后续 fail 后 stats 原字节恢复、start COMMIT/rename 崩溃窗口。

**停止条件。** 新请求仍使用 `hash_command` 或历史效果由当前 WorkState 重算时任务未完成。
**提交。** `fix(runtime): 请求按用户意图提交并恢复原始效果`

### C002-T08 Workbook 写操作接入请求日志

**Owner。** 强模型。
**依赖。** C002-T05、C002-T07。
**文件。** workbook_repo、request、store commit/read、CLI workbook/self、failpoint 与 tests。

add/remove 的数据库变更、audit 与 StoredOutcome 在同一事务。staging 按请求独占，rename/delete 是 durable effect；并行 add 不能清掉对方 staging。Workbook 写操作自动生成并返回 request-id；self 命令显式拒绝 request-id。

**测试。** add/remove 同 id 重放、不同 intent 冲突、COMMIT 后崩溃恢复、并行 staging、self 参数拒绝。
**提交。** `fix(runtime): Workbook 写操作接入事务化请求日志`

### C002-T09 固定 workbook-digest/v1

**Owner。** 标准模型。
**依赖。** C002-T01。

把当前算法命名为 `digest_dir_v1`，合同写出真实两阶段 hash，使用独立手写向量和 v0.1.0 fixture。结果字节必须保持不变。若决定改成单次 hash，本任务停止并另写完整数据迁移方案。

**提交。** `fix(runtime): 固定 Workbook 摘要 v1 并加入独立向量`

### C002-T10 输入发现与协调者行为

**Owner。** 强模型。
**依赖。** C002-T02。
**文件。** `skills/sheltie/SKILL.md`、协议、CLI workbook show tests、本 package 的宿主回归说明与 validation。

1. skill 在 start 前读取 `workbook show --json` 的 `start_inputs`，缺值就问用户。
2. 用户未指定 Workbook/Flow 时必须问；指定但未安装时报告缺失并请求来源，不得替换成其他 Workbook。
3. “只做 next”收窄为成功 start 后的推进写操作；只读发现命令与 add/start 的入口流程单独说明。
4. human executor 默认把任务书交给人。只有用户明确授权代执行时，协调者才可代写/submit，并在记录中标注。
5. 增加宿主回归 prompt：未指定、指定未安装、缺 topic、human 节点四种反例。

**提交。** `fix(skill): start 前发现输入并禁止静默替换 Workbook`

### C002-T11 article-review 机械绑定打回意见

**Owner。** 标准模型。
**依赖。** C002-T01。
**文件。** `examples/article-review/`、core example tests、CLI scenario test。

draft 增加 optional `review.verdict` 输入；第一次到达显示尚无，back 后 draft#2 的 brief 必须绑定 review#1 输出。draft 说明书要求有 feedback 时逐条处理。测试不得由协调者手工追加意见来伪造绑定。

**提交。** `fix(example): article-review 打回时绑定上一轮审查意见`

### C002-T12 修复 spec-dev 任务闭包

**Owner。** 强模型。
**依赖。** C002-T01。

1. plan-review decision 绑定到 scaffold、implement、verify、review、deliver。
2. change/report 携带当前任务基线与候选提交；verify 比较该基线到候选，不比较骨架到全部 HEAD。
3. 增加 `escalate -> verify` 合法边并更新“继续”映射。
4. 用两任务临时 Git 场景和人工条件进入后续 brief 的场景验证。

**提交。** `fix(workbook): 修复 spec-dev 验证范围与人工条件交接`

### C002-T13 让 skill 安装产物自包含

**Owner。** 标准模型。
**依赖。** C002-T01。

在 `skills/sheltie/references/` 使用指向权威合同的仓库内符号链接；README 使用 `cp -RL` 安装，使目标目录得到普通文件。检查脚本在临时宿主目录验证全部链接、命令和引用均脱离源码树可用。

**提交。** `fix(skill): 交付自包含的 Sheltie 协调者说明`

### C002-T14 收紧 core 已校验定义的 interface

**Owner。** 强模型。
**依赖。** C002-T03、C002-T05。

Raw TOML DTO 保持私有；Manifest、FlowDef、NodeDef 对外只读；parse 是公开构造入口；compile 不再接收可被外部改成非法状态的 public fields。移除不需要的 Deserialize，持久化输入继续拒绝未知字段。

**停止条件。** 不得在 compile 里复制 parse 的全部校验来维持公开可变字段。
**提交。** `refactor(core): 已校验 Workbook 与 Flow 定义只读化`

### C002-T15 CI、MSRV 与发布门禁

**Owner。** 强模型。
**依赖。** C002-T04、C002-T05、C002-T08、C002-T10–C002-T14。

build 与 release 复用同一质量 workflow；release plan 依赖质量 job。stable 检查之外增加 `cargo +1.85.0 check --locked --all-targets --all-features`。release candidate 测试包含缺输入无副作用、legacy/new layout、干净 home、skill 安装布局与 v0.1.0 Store fixture。

**提交。** `ci(release): 发布候选复用质量门禁并验证 MSRV`

## 2. C002-M1 独立全链审查

**Owner。** 未参与 C002-T02–C002-T15 的强模型。

1. 审查 `v0.1.0..HEAD`，逐项关闭复审中的 O01–O13 和新增 finding。
2. 重跑本次真实 CLI 探针；`INPUT_MISSING` 前后目录、序号、works、requests 必须完全相同。
3. 从 v0.1.0 fixture 升级，旧 Work 保持 legacy 路径，新 Work 使用 readable_v2。
4. 重跑 COMMIT 前、COMMIT 后/效果前、start rename 前、Workbook add rename 前故障窗口。
5. 运行全量门禁、core/runtime mutants、MSRV check 与 `cargo dist plan`。
6. 结论只用“通过 / 需修改 / 阻断”。结构 PASS 不等于真实宿主 PASS。

C002-M1 PASS 后才打 `v0.2.0-rc`。

## 1. C002-T16 v0.2.0 宿主回归

使用 release candidate，不用开发 target：

1. 全新 SHELTIE_HOME 和全新 Claude Code 会话安装 skill。
2. 不指定 Workbook，确认协调者先问；指定未安装 Workbook，确认不替换；缺 topic 时确认先问、不执行 start。
3. article-review 完成一次 back；draft#2 必须从任务书绑定 review#1，而非聊天补充。
4. publish 由人自己写 final 并执行任务书 submit。
5. 关闭会话后新开会话，仅靠 `work status --json` 继续。
6. 记录 `/cost` 前后值或等效 token 读数、逐条 CLI/response、人工介入与耗时。
7. Finder 打开 frozen workbook 后，目录摘要和 status 仍正常；显式修改文件仍报错。
8. 把回归证据写入本 change package 的 validation/review；T26 历史记录不改写。

## 2. C002-T17 发布 v0.2.0

**Owner。** 人。
**依赖。** C002-M1 与 C002-T16 PASS。

生成 CHANGELOG；从 v0.1.0 与 rc 各走 update/rollback；确认四平台产物、manifest、checksum 与 quality job 指向同一 commit；在全新管理根用 release installer 完成 two-step smoke。

## 1. 完成判据

修复完成当且仅当：

1. C002-T01–C002-T17、C002-M1 与 C002-T16 在当前 change plan 中有真实状态与证据；MVP/T26 历史状态保持完成。
2. O01–O13 和本轮 finding 各有合法例、单条件拒绝例与真实 caller 验证。
3. 缺 topic 不消耗序号、不创建最终目录；start 的其他失败只允许留下可回收 staging 和合同允许的空号。
4. 新目录可从名称区分 Occurrence 与 Attempt；旧 Work 无迁移可读。
5. Workbook 根目录不可被 Finder 自动写入；登记摘要漂移在新建 Work 前被拒绝。
6. 同请求重放返回提交时 snapshot 与 exact engine effects。
7. C002-T16 保存 human 动作、token 与逐条宿主证据。
8. 没有引入第二套 Work 状态、第二套路由器、业务判断或宿主安装逻辑。

## 2. 不阻断 v0.2.0 的后续工作

- 提供只读 `work tree`/`work inspect` 视图，避免用户必须直接浏览管理根。
- 为已有孤儿目录提供只报告、不自动删除的 doctor 命令；清理必须核对数据库无行和目录归属。
- 对 Workbook 全目录采用流式摘要，降低大资源的峰值内存。
- 用同一任务比较直接 agent、skill、skill + Sheltie 的质量、token 和人工介入；没有对照数据前不承诺节省比例。
