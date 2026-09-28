# Spec 轴结论：需修改

固定候选 `e1a8126a432981df40023628dd06feec7884d458`，基线 `a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。只读审查；未参与实施；未改仓库。所有探针仅操作 `/private/tmp` 内新建夹具。未另起 cargo，使用主 Reviewer 本候选生成的 binary。

C002 的价值方向成立：请求绑定真实目标、完整提交快照、显式破坏性格式切换、冻结定义和准确恢复，都是协调者每次重复承担的机械责任；保持三 crate、纯 core、业务判断留给 Workbook 的结构正确。未发现这一范围的业务判定侵入或不必要框架。不过当前实现不能关闭以下五项，`done` 和绿门禁不能替代反例。

1. **P1：损坏 state 的路径可令合法 cancel 写出管理根。** `Store::decode_row` 只做纯状态关系校验，没有核 `work_dir` 必须等于本 Home 的 `works/<work_id>`；`service.rs:456` 从其冻结目录读取，`service.rs:648` 用其状态卡路径写投影。`fsx.rs:163,176–179` 以词法 `starts_with` 放行 `home/../outside`，并跳过 ParentDir，实际写操作仍使用原路径。仅改合法 state 的 `work_dir` 为 `home/../outside`，外部放置匹配的冻结副本和哨兵，`work status` 与 `work cancel` 都 rc 0，哨兵被状态卡覆盖。违反 INV-3、GF-32、architecture §5 和 C002 design §7 的持久路径归属要求。修复要由 runtime 从 Home/WorkId 核身份；文件边界拒绝 dot segments；不是在 core 引入 I/O。

2. **P1：Workbook 未完成发布的显式重放跳过恢复，误报成功。** `workbook_repo.rs:130–137` 和 `351–358` 命中 request 后直接 decode snapshot，查询不带 `published`，根本到不了锁内恢复。已提交 add、payload 完整而 final 未发布时，同 r-add 返回 rc 0、ok true、replayed true，但 published 仍 0、final 仍缺。违反 GF-15、architecture §4“意图相同先完成未发布效果，再返回原响应”、storage §3.2。remove 是同一错误分支。新请求读取源前查重正确，但命中必须按持久效果完成或返回已提交错误。

3. **P1：待发布 Workbook 读视图和 start preflight 没实现。** `workbook_repo.rs:314,481–487` 仅访问 final；CLI workbook list/show/verify 都没有 pending_publish。相同合法 pending 夹具中 list 成功不标待发布、show/start 返回 IO，verify 错报 WORKBOOK_TAMPERED/missing。用户不能用一次有效 start 恢复先前 add。违反 storage.md:168 和 protocol §3 workbook add 的明确 pending 读取/标记合同。应从本版本 Store effect 与合法侧车发现同一 pending 原件，有限重读 final/pending；不得靠任意源目录 fallback。

4. **P1：目录摘要仍绕过受限文件入口。** `workbook_digest.rs:65` root 直接 read_dir，`84` 取 metadata 后 `108` File::open 重开，没有 SafeFile 对象核验。只把合法 installed root 目录移到 Home 外并将原位置换成软链，workbook verify 和 show 均 rc 0；摘要相同就被视为可信。违反 T09 §1“复用 T04 的受限目录枚举与文件句柄”、GF-32、design §4。后者 stat/open 竞争还有源码层面的缺口，但动态证据仅声称 root 软链反例。

5. **P2：spec-dev 重规划未把旧计划/任务交给新 worker。** `flows/default.toml:32–40` 没有 previous plan/tasks 输入；`instructions/plan.md:10,17` 却要求原样抄上一版原始基线、保留已完成任务。真实 CLI plan-review→plan#2 的 brief 只给 spec/project/模板/意见，没有上一版任何路径。独立 fresh worker 无法仅凭任务书履行该要求；多任务后重规划可能缩小最终审查范围。违反 C002 plan.md:156–158 和 spec delta §7。现有 scenario_spec_dev.rs:1416–1418 从测试进程缓存的 baseline0 手写新版计划，没证实输入交接。应给 plan.plan / plan.tasks optional 绑定，首次尚无，重规划依冻结旧输入；补从第二次 brief 取得旧基线的真实 caller。

# 独立证据

- `python3 /private/tmp/sheltie-m1-review-e1a8126/spec/state_path_probe.py`，原始输出 `state-path-results.jsonl`：F1。每次创建独立目录，只改 state 一个字段。
- `python3 /private/tmp/sheltie-m1-review-e1a8126/spec/probe.py`，原始输出 `results.jsonl`：F2/F3/F4。pending 场景复用仓内 `workbook_txn.rs::add_publish_window_recovered_by_next_write` 的精确窗口造法（final 撤回登记 payload；published 0），并保留合法 owner。合法 add/verify/show 为对照；软链场景只替换 installed root。
- `python3 /private/tmp/sheltie-m1-review-e1a8126/spec/replan_probe.py`，原始输出 `replan-results.jsonl`：F5。真实 CLI 创建 spec-dev，spec→plan→plan-review→back plan，最终原始输出包含完整第二次 brief 与第一次 plan 路径，二者可直接比较。

T11 代码及现有真实 CLI 测试体现正确 optional review 路径绑定；T10 输入发现与历史 next 指引、T13 references 生成设计合理。它们不替代 T16 的真实 agent 质量验收；本轴未执行 Host/发布，也未声称其通过。全门禁、突变及 service 主链的其他反例由主 Reviewer 统一记录。

# O/N 归属和修复提交映射

以下是被审实现的提交，不表示反例已关闭。最终归属合并由主 Reviewer 裁定。

| 本轴项 | C002 tracking | 被审修复 commit | 需要负责修复的 Owner | 正例 / 单条件反例与真实入口 |
| --- | --- | --- | --- | --- |
| F1 根外状态卡写 | O01；T14 持久路径验证 | `63d4d48` T14、`5196cb1` T04、`bcfa0e6` T07 | runtime Store/load 与统一 fsx 边界 Owner | state_path_probe：原始合法 add/start，随后仅 work_dir 改一个字段；真实 status/cancel；预期 STORE_CORRUPT 且哨兵不变，实际 rc0 覆盖 |
| F2 Workbook replay | O04/O08/N02，GF-15/GF-31 | `766dbe2` T08、`bcfa0e6` T07 | WorkbookRepo request/recovery Owner | probe：正常完成 add 原 snapshot；撤回登记 payload 并 published0 模拟同一提交窗口，真实同 request add replay；预期发布完成或已提交错误，实际假成功 |
| F3 Workbook pending 读取 | N02/N13，GF-31，storage §3.3 | `766dbe2` T08、`bcfa0e6` T07 | WorkbookRepo load/verify、CLI pending 标记、start preflight Owner | probe：已发布时 show/verify/start 可用；同一合法待发布原件时 list/show/verify/start；预期 pending 读取/准确标记，不得 missing |
| F4 digest unsafe root | O01/O03，GF-32 | `429cb1d` T09、`5196cb1` T04、`d38b6e1` T05 | Workbook 摘要与 fsx 受限枚举 Owner | probe：正常 add/show/verify；只把 installed root 换成指向相同字节根外目录的 symlink；预期拒绝，实际 show/verify rc0 |
| F5 replan 旧输入缺失 | N08/N13，T12 | `c1a530e` T12 | spec-dev Flow/instructions/真实 caller tests Owner | replan_probe：第一次 plan 输入首次无旧产物合法；成功 plan→plan-review→back 第二次 plan；预期 brief 有冻结旧 plan/tasks 路径，实际缺失。需补 task1 已提交再 replan 的从 brief 获取 baseline oracle；该更完整独立 Git 场景本次未执行，现有测试不满足此 oracle |
