# T26 同步提交与 Work 目录复审

审查日期：2026-09-27
固定点：`4177b57738ef5f56a6699bbb5310342d32dadd68`
候选：`31d7ddee18921b4066c7433a752c2000e5110869`
差异：`git diff 4177b57...31d7dde`
结论：**MVP 与 T26 已按项目决定完成；同步提交仍有事实归因和证据边界问题，新发现应进入后续版本提案，不能写成已经修复。**

本审查只新增 review 与修复方案文档，不修改产品代码、既有 T26 记录或任务状态。对应方案见 [C002 README](README.md) 与 [实施计划](plan.md)。

## 审查范围与证据

同步提交只有一项：

```text
31d7dde docs(specs): 记录首次真实运行
```

变更文件只有 `specs/releases/v0.1.0/decisions.md` 与 `specs/releases/v0.1.0/plan.md`。产品代码与上一轮全面审查时相同，因此上一轮确认的 13 项代码、Workbook 与交付问题没有因本次同步关闭。

本轮重新执行：

| 检查 | 结果 |
| --- | --- |
| `cargo nextest run --all-features --no-tests=pass` | PASS：315 passed，0 skipped，11.806 秒，不含编译 |
| `cargo fmt --all -- --check` | PASS |
| `scripts/check-docs.sh` | PASS：49 个文件（写入本审查文档之前） |
| `scripts/check-task.sh T26` | PASS；只证明白名单、状态和提交格式，不证明 T26 语义完成 |
| 当前 HEAD 构建的真实 CLI 探针 | 缺 `--workbook`：退出 2，无 Work 副作用；缺 `topic`：`INPUT_MISSING`，但分配序号并留下孤儿目录 |
| 当前真实 `~/.sheltie` 只读检查 | `002-default` 目录存在但数据库无 Work 行；`article-review@1.0.0` 为 `tampered`；`003-local-engine` 的 status 为 `STORE_CORRUPT` |

临时探针使用独立的 `/tmp/sheltie-post-t26-review/` 和临时管理根。对真实 `~/.sheltie` 只执行 `find`、只读 SQLite 查询、`workbook verify` 与 `work status`，没有修改用户数据。

## Standards

这一轴只评价同步提交是否符合仓库的事实、文档和审查标准。结论：5 项 finding，最高 P1。

### S1 · P1 · 把违反合同的副作用写成已接受代价

`specs/releases/v0.1.0/decisions.md:680` 把 `002` 描述为“空目录”，并用存储合同 §7.1 的“序号不回收”解释。实际目录含冻结 Workbook 和 `inputs/`，只是数据库没有 `works` 行。

协议 `work start` 明确要求先核对起始输入，再分配序号并建目录；错误码表又规定 `INPUT_MISSING` 的结果是“无变化”。§7.1 只允许**已经进入序号分配之后**的失败留下空号，不能覆盖本应在分配前完成的输入校验。

这项记录应改为 runtime 实现缺陷，并路由到代码、合同回归测试和崩溃/清理验证。

### S2 · P2 · T26 完成记录没有保存两项原始观测

`specs/releases/v0.1.0/plan.md:446` 要求记录宿主 token 观测值，`specs/releases/v0.1.0/decisions.md:691` 明确写三轮均未取得。执行手册还要求 human 节点由人写 `final.md` 并自己 submit；记录显示含打回的 Run 3 仍由协调者代执行。

真实运行、skill 发现、article-review、back 边和新会话均有叙述证据。项目已经接受 T26 与 MVP 完成；token 未采集、human 节点由协调者代执行仍应作为验收限制保留，并在后续版本的宿主回归中补测，不能改写成已经取得的证据。

### S3 · P2 · 把 Workbook 副本完整性称为输入冻结

`specs/releases/v0.1.0/decisions.md:693` 把 `.DS_Store` 触发的 Workbook 整目录摘要不符称为“输入按字节冻结”。项目词汇中，输入冻结是 `ArtifactRef.sha256` 对起始输入、上游产物和绑定输入的核对；本次验证的是存储合同 §5.1 的 Workbook 冻结副本完整性。两者都重要，但不是同一机制。

### S4 · P2 · 证据边界超过引擎能证明的范围

`specs/releases/v0.1.0/decisions.md:678` 写“全部结论都在本机用引擎复核过”。引擎可以证明 Store 中的状态、已接受命令、文件摘要和 stats，不能证明宿主会话中没有被拒命令、没有直接文件操作，也不能证明 prompt 原文。仓库没有保存原始会话记录的位置。

记录应逐项标明来源：引擎、SQLite、文件摘要、宿主 transcript 或用户回忆。缺原始证据的结论写“未验证”或收窄范围。

### S5 · P2 · 发现只写“随下个版本”，没有完成路由

`specs/releases/v0.1.0/decisions.md:695` 只给两条 skill 文案建议，没有对应 Task 或上游合同修订；还遗漏了 `INPUT_MISSING` 孤儿目录和 `.DS_Store` 使 Work 不可读的问题。按 `engineering.md` §7，产品、机制、顺序和实现问题必须进入各自权威文件，不能只留一句未来处理。

本提交只有文档变化，Fowler code smell baseline 不适用。

## Spec

这一轴检查同步提交是否满足 T26、协议与存储合同。结论：4 项 finding，最高 P1。

### P1 · P1 · `work start` 的实现顺序违反协议

真实 CLI 结果分成两个条件：

| 条件 | 结果 |
| --- | --- |
| CLI 缺 `--workbook` | Clap 退出 2；runtime 未执行；没有新增 Work 目录或序号 |
| 已选 `two-step`，缺 `--input topic=...` | 返回 `INPUT_MISSING`；分配 `001`；创建 `works/2026-09-26-001-default/{workbook,inputs}`；数据库没有对应 Work 行 |

根因在 `crates/sheltie-runtime/src/service.rs:106-147`：先 `allocate_seq`、创建最终 Work 目录、复制 Workbook、创建 inputs，最后才调用 core `decide_start` 校验输入键。`specs/contracts/protocol.md:77-82` 定义的顺序相反。

当前真实管理根也有相同证据：`2026-09-26-002-default/` 存在，`works` 表没有 `002`，`work_sequence.last = 5`。

### P2 · P2 · 后续版本仍需补测人审行为和 token 观测

含打回的 Run 3 证明了 back、第二次 Occurrence、独立 review agent 与 main 推进；它没有证明 human executor 的规定行为。token 项只有“未采集成功”，没有 `/cost` 前后值或等效宿主读数。T26 已完成，这两项作为 `v0.2.0` 宿主回归的新增验收条件，不回滚 MVP 状态。

### P3 · P2 · T26 事实记录与用户后续澄清冲突

用户后续明确说明开场没有提供 topic 和 Workbook；`specs/releases/v0.1.0/decisions.md:680` 却写 prompt 指定 article-review 并给了样例路径。仓库没有保存原始 prompt/transcript，无法由代码判断哪一项正确。

记录应按原始宿主证据重写；若无法取得，明确写“用户后续澄清”与“原始 transcript 未保存”，不能保留两个互斥事实。同时，“全部写操作都在 next”应收窄为“成功 start 之后的 Work 推进写操作”，因为 `workbook add` 与首次 `work start` 本来不来自某个 Work 的 next。

### P4 · P2 · 记录给出的 `workbook show` 补救路径不可执行

T26 记录建议用 `workbook show` 查询起始输入键。但当前文本和 JSON 只输出 Flow 的 id、entry、节点 id/title/executor/gate 与边，不输出 `start.<key>`。真实 `workbook show two-step --json` 没有 `topic`。

这不是只改 skill 就能解决的问题。协议和 CLI 必须先公开每张 Flow 的 `start_inputs`，skill 才能在调用 start 前检查并向用户补问。

## 目录与文件组织审查

### `1/0/` 是否必要

两个维度都必要，纯数字命名不必要。

- 第一层 `1` 是 Occurrence：同一 Node 经 back 或 re_review 再次到达时变成 `2`。它保证旧产物不被覆盖。
- 第二层 `0` 是该 Occurrence 内 Attempt 的 retry 段：首次执行为 `0`，失败重试为 `1`。它保留每次失败与重试的事实。

删掉任一层都会混淆“节点再次到达”和“同一次到达内重试”。适合修改的是目录名，而不是状态模型。

建议映射：

```text
AttemptId draft#2.1
→ attempts/draft/occurrence-002/attempt-001/
```

`occurrence-002` 和 `attempt-001` 能直接对应领域词，同时保持按文件名字典序浏览。外部 AttemptId 继续使用 `draft#2.1`，避免改 CLI 协议。

### 建议的新 Work 目录

```text
works/<work-id>/
  status-card.md
  workbook/                         # 冻结 Workbook；根和子目录均 0555，文件 0444
  start-inputs/
    topic
  attempts/
    draft/
      occurrence-001/
        attempt-000/
          brief.md
          engine/
            stats.json              # 仅声明 engine.stats 时存在
          outputs/
            article.md
      occurrence-002/
        attempt-000/
          brief.md
          outputs/
            article.md
```

| 当前名称 | 结论 | 调整理由 |
| --- | --- | --- |
| `works/<work-id>/` | 保留 | WorkId 已含日期、序号和可读名称 |
| `workbook/` | 保留，修权限 | 领域词准确；根目录必须与子目录一样只读，防 Finder 写 `.DS_Store` |
| `inputs/` | 改为 `start-inputs/` | 当前名称容易与每次 Attempt 的绑定输入混淆；实际只存 `start.<key>` |
| `attempts/` | 保留 | 与领域词 Attempt 一致 |
| `<node>/` | 保留 | 按 Node 聚合历史，便于定位 |
| `<n>/<retry>/` | 改为 `occurrence-NNN/attempt-NNN/` | 两层语义必要，纯数字不可自解释 |
| `brief.md` | 保留在 Attempt 根 | 每个 Attempt 的主要入口，名称明确 |
| `stats.json` | 移到 `engine/stats.json` | 标明由引擎生成，并与 worker 输出分开 |
| 声明输出与 `brief.md` 混放 | 移到 `outputs/` | 提高浏览性，同时消除 `brief.md`、`stats.json` 与声明输出碰撞 |
| `status-card.md` | 保留 | 它是 Work 的入口投影，名称与词汇表一致 |

目录布局必须由一个 `WorkLayout` module 统一产生。core、runtime、渲染和测试不得继续分别拼字符串。

### 冻结 Workbook 根目录不是只读

`set_tree_readonly` 只把子目录改成 `0555`、文件改成 `0444`，传入的 `workbook/` 根保持 `0755`。注释称 macOS 移动或删除目录需要根目录可写；实测把根改为 `0555` 后，只要父目录可写，rename 仍成功。删除前已有 `make_tree_writable` 可显式恢复权限。

当前真实 `article-review@1.0.0` 已因 `.DS_Store` 变为 `tampered`，`003-local-engine` 的 status 也因此不可读。这不是完整性机制“按设计工作”的全部结论，而是**自动生成的宿主元数据可以破坏一个完成 Work 的可读性**。应修根目录权限，并保留摘要拒绝作为第二道防线；不应简单把 `.DS_Store` 从摘要中忽略。

## 上一轮 finding 的当前状态

同步提交未改产品代码，以下 finding 全部仍为 OPEN：

| 编号 | 问题 |
| --- | --- |
| O01 | 输出父目录符号链接可越出 Attempt 根并 chmod 外部文件 |
| O02 | request-id 没绑定目标 Work，可跨 Work 虚假重放 |
| O03 | 已装 Workbook load 忽略登记摘要，同版本内容可漂移 |
| O04 | 重放依赖当前文件和当前状态，不能稳定返回原响应 |
| O05 | 历史 `stats.json` 用最新状态重建，字节会变化 |
| O06 | 不存在的管理根无法直接 install/add |
| O07 | Workbook 目录摘要实现与文档公式不一致 |
| O08 | Workbook 写操作回显 request-id 但不支持重放 |
| O09 | `spec-dev` 单任务 verify 使用累计 diff，第二个任务会误报越界 |
| O10 | `spec-dev` 人工批准条件没有进入后续任务书 |
| O11 | skill 按 README 安装后，两条合同链接断裂 |
| O12 | 工作输出可与 `brief.md`、`stats.json` 等引擎文件碰撞 |
| O13 | JSON 状态卡遗漏 Attempt 失败原因 |

T26 又提供了两个新的 Workbook/skill 证据：

1. `article-review` 的 draft 没有把 `review.verdict` 声明为 optional input；打回意见依靠协调者临时追加，而不是由 Workbook 机械绑定。
2. 指定 Workbook 未安装时不得静默换用其他 Workbook；用户未指定 Workbook 时必须先问。当前 skill 已写后一条，真实协调者没有遵守，说明需要加入明确反例和宿主回归，而不是只追加一句近义文案。

## 调整后的总评

产品定位与三 crate 架构仍然成立；315 个测试也继续证明既有状态机主路径。准确边界是：

- `v0.1.0` 已发布。
- T26 真实宿主执行、skill 调用、back 回环和产物链已经发生。
- MVP 与 T26 已由项目接受为完成；人审动作与 token 观测是记录中的明确限制。
- T26 暴露的 start 副作用、Workbook 冻结目录、输入发现和目录可读性问题尚未修复。
- 上一轮 13 项 finding 仍然 OPEN。

因此本次 review 对候选 `31d7dde` 的结论是 **MVP 完成，后续版本需修改**。修复方案当前仍是 proposed/`not_run`；采用时先改上游合同和任务链，再实施代码，不应只修 skill 文案或清理现有孤儿目录。
