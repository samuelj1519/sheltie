建议交付：独立静态审阅未发现冻结验收内需修改的缺陷；候选、授权范围与两项原始机械检查一致。完整 patch 应用、最终盲审和用户接受仍须在既定后续步骤完成。

# 身份、输入与候选

实际审查者 `/root/c007_study_coordinator/run04_review`，未参与 partial 或被审候选实现；本次是日常独立 review，不是流程外盲审或用户接受。模型按绑定继承 `gpt-6.1-sol/high`，无 override、无助手；实际 provider 身份未由本 worker 独立取得，由 Root 另核，usage/fees 均 unknown。

已完整读取本 run binding、task、project、方法目录 README/Workbook/Flow/三个阶段说明（含共有策略）、shared-method、implement-1 的 change/checks。审查读取 assigned repo 的 CONTEXT、specs README/engineering、实际 110 行指南、protocol 必要正文、storage §§2/3/5.4/5.5/6、C004 README、C005 spec/design、既有 sheltie skill 和下列实际 caller。使用同一中文文档 skill 及受控中文/API 文案参考，未新增任务标准。

Assigned repo：`/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/continuity/native`。初始 HEAD `351feb7ac22c21317a686693b732d5ae0c4b4bcc`，tree `80ea3046b1b3575f07e68313444c051ee7c5b7db`；被审 HEAD `c9823575920795db14dfd378d77c8b311ce71d87`，tree `404c3004895a9451d7cfc88b0c5a04400a4a0463`。独立查询显示 worktree 干净，全基线 diff 只有新增 `specs/guides/continuity-choices.md`；指南 13959 字节、SHA-256 `d7f53c99fae26656972b941b604cb2547fe5461790a973a7cb3dffed8cbc91aa`，与报告及检查树一致。原件 `raw/review-1/identity.json`、`.stdout`、`.stderr`，相对路径均从本 run 根解析。

副本 specs README 的历史「Active change：无」没有被当作自行采用 proposed 的授权；本次只按 project 明确的外部 C007 任务卡审查唯一授权指南。

# 冻结验收逐项核对

未发现须报告的缺陷；以下结论是实际文档、合同和调用链的静态复核，不是 CLI 演练结果。

| 冻结要求 | 实际位置与核对结果 | 标准及实现依据 |
| --- | --- | --- |
| 四种选择 | 指南 3–52 行区分重开后同 running Attempt、真实执行 fail、审查完成后 submit/back、确需行政撤销 replace。每项给触发、前提、实际命令/next 和停止。内容返工不记执行失败；number 不计 failed；回边受 max_visits。 | C005 spec §§1/3；protocol §3 begin/submit/fail；core `next.rs:75–159`、`decide.rs:799–842`。 |
| 一次替换、冻结继承、统计与拒绝 | 指南 54–75 行固定同 Occurrence 一次、当前最新 running/active、原子 superseded+running、顺序号加一；保留 entered_from、非 stats 引用和 optional null，旧草稿不转正式输出；stats 来自含新 Attempt 的状态。旧新 submit/fail 非终态 ATTEMPT_NOT_RUNNING、终态 WORK_TERMINAL，成功重放在新请求资格前；历史 next 必须刷新。未声称停进程、认证或隔离宿主。 | C005 EX-01–EX-08/design §§2–4；protocol §3 replace；storage §§5.5/6；core `decide.rs:38–83,303–518,610–661,799–842`；runtime `service.rs:444–495` 同句柄观察旧输入；CLI `attempt.rs:130–165` 实际入口。 |
| 结构化 pending/unknown | 指南 77–89 行明确 committed=true 的本请求恢复与 committed=false 的 B/A 区分，保留原 UUID/原意图，不把 pending_original 当 B 成功；original/revision 可省略，不从自然语言推业务提交状态。缺记录、身份或完整性未解决时停，不改 Store/pending、不造原件；查询仅报告待恢复事实。 | protocol §5；storage §§2.1/3.1–3.3/6，历史精确 bytes/快照核实与效果资格分离。 |
| human、门槛、终态、查询边界 | 指南 91–110 行查询后结束；resume 非空不等于 running；human 交人、缺 requires 停；gate 需明确批准、by 只是 OS 账户；其他 blocked 仅 cancel、终态 next 空。final=false 为空、合法无声明 final=true 也可为空；流程成功不代替质量/接受/发布。 | protocol §§1/3 gate、result/5/6；C004 README 成功判据；core `next.rs:148–159`；CLI `commands/mod.rs:50–80`、`work.rs:121–166`。 |
| 真实命令、链接、显式 Home/JSON、写 ID | 全部示例有 --json 和显式 --home；已有 Work 使用原管理根，新试用独立根；request-id 只用于写命令，参数/字段存在实际 CLI 与公开合同。6 个相对链接全部指向真实文件，两个 fragment 对应 protocol「3. 各操作细则」及 storage「3. 崩溃语义与恢复」。术语与机器符号保持准确。 | protocol §§1/2/3/5/6；CLI `cli.rs:7–18,89–169`、`commands/mod.rs:50–80`、`attempt.rs:14–166`；assigned CONTEXT、工程规范及共同中文 skill。 |

实际 source/caller 原件：`raw/review-1/guide.*`、`protocol.*`、`protocol-rest.*`、`continuity-authorities.*`、`c004.*`、`storage-targets.*`、`storage-and-stats.*`、`callers-map.*`、`callers.*`、`readonly-callers-map.*`、`readonly-and-link-titles.*`、`existing-sheltie-instructions.*`、`assigned-entry.*`。每组含 capture JSON、完整 stdout、stderr。

链接诊断 `links-and-commit.stdout` 的简单字符串 matcher 因标题含句点，给两个 fragment 的 matching_title 空数组；该 matcher 不证明断链。随后 `readonly-and-link-titles.stdout` 保存两个真实标题，人工核其 Markdown anchor 与指南 fragment 一致；没有改指南或通过重跑机械检查凑绿。两个外层 Python 调用产生 escape SyntaxWarning，命令退出 0，警告留在实际工具记录；不是候选缺陷或既定检查失败。

# 原始检查与范围

独立读取 `raw/implement-1/check-docs.json` 与 `check-staged-whitespace.json` 和全部 stdout/stderr，并实际复算原件摘要相等；argv/cwd/UTC/control_environment/exit 完整，未超时、within_deadline=true。两项实际退出 0：

- `scripts/check-docs.sh specs/guides/continuity-choices.md`，stdout `check-docs: OK (1 个文件)`，stderr 空。
- `git diff --cached --check`，stdout/stderr 空。

检查前后 staged tree 为 `404c3004895a9451d7cfc88b0c5a04400a4a0463`，同 candidate commit tree。`checked-input`、`candidate-identity` 保存 PATH/locale/Git overrides；LANG=en_US.UTF-8、LC_ALL/LC_CTYPE=C.UTF-8，Git index/dir/worktree 与 Sheltie 控制变量未设置。独立原件读取及摘要核验见 `raw/review-1/check-originals.*`。同输入闭包的机械检查引用原件，没有重复执行。它们只证明单文件链接路径/既有文案规则与 staged 空白，不代替上述内容判断、动态 CLI 或最终质量。

candidate 本地提交格式及 Change/Task/Sample/Arm/实际 Agent trailer 与冻结 project 相符；snapshot Git author 不是实际 worker 身份证据。初始目标指南确实不存在。新增文件已 stage/commit，不能用 unstaged diff 交付。完整 patch 生成命令、基线、独立检查副本、apply --check/apply --index/write-tree 和字节/路径相等要求在 change 报告中准确复述 project 原规则，未临时换标准。

# 冷接续证据与剩余边界

只核本 run 的持久事实：`stage.json` 指向 review-1、implement/review visits 1/1、retry 0/0，partial helper 的 normal final 与原时间留存；`raw/partial-implement-1/partial-state.json` 仍记录未完成草稿、原 SHA/5465 字节、当时检查 not_run；`raw/cold-implement-1/resume-activity-start.json` 记录同 implement-1、同草稿核对、旧原件保留。独立读取原件见 `raw/review-1/own-continuity.*`。未重演中断、未取上一会话正文或其他组答案；这些持久 worker 报告不单独证明真实宿主会话关闭/provider 身份或净收益。

交付前仍须按固定 deliver 在授权 Native 独立副本 `/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/native` 完成完整 patch 生成及实际应用，核基线/干净状态、完整路径集合、tree 与全部授权字节等于被审候选。review 未执行此步骤，状态 not_run，不能宣称完整 patch 已可应用。候选若漂移，本审阅和既有检查不能自动覆盖新候选。

本 worker 未执行 Sheltie、Rust/build/tests、CLI 演练、故障注入、安装、发布或外部写入；未修改候选、Root 产品文件、inputs、方法、其他 run/arm、stage/run 索引或旧报告。未参与最终独立盲审或用户接受，均 not_run。新增独立事实命令经冻结 `study/capture.py.execute`，每条 120 秒上限并受原 hard deadline 限制；所有自启动命令已 await，无后台任务、无 helper。日常审查未发现需返工缺陷，协调者据此选择既定下一阶段；本 worker 不自行推进流程。

报告完成 UTC：2026-10-03T21:00:07.998687+00:00；原 hard deadline `2026-10-03T21:08:27.513464+00:00`。
