import datetime,hashlib,json,sys
from pathlib import Path
run=Path(sys.argv[1]);raw=run/'raw/deliver-1';out=run/'outputs/deliver-1';summary=json.loads((raw/'closure-summary.json').read_text());now=datetime.datetime.now(datetime.timezone.utc).isoformat()
report=f"""交付材料已整理：完整 patch 已在预声明 Native 独立基线副本实际应用，所得 tree 与日常 review 的候选一致；最终质量盲审和用户接受尚未执行。

# 任务、身份与冻结输入

本 run `{run.name}` / continuity / native 的任务是新增操作者接续与资格撤销选择指南，只授权 `specs/guides/continuity-choices.md`。实际 deliver worker 为 `/root/c007_study_coordinator/run04_deliver`，按绑定继承 `gpt-6.1-sol/high`、无 override、无助手；实际 provider 元数据由 Root 另核，usage/fees unknown。身份、工具、相关环境和时间在 `raw/deliver-1/actor.json`。首次入口同时读过 Root 的 CONTEXT、specs README/engineering；随后全文读取 assigned repo 的同名入口，未把 Root 状态用于候选判断或修改。本阶段未读取其他 run/arm 答案，未搜索上级资料。

已完整读取本 run binding、task/project、deliver 方法全文（含共有策略）、三份输入报告，并应用共同中文 skill。三个输入报告指向同一 HEAD/tree；review 第一行明确建议交付，无阻断项。未改这三份报告，未改候选、共同方法、标准、stage/run 元数据或 Root 产品文件。

源仓库：`{summary['source_repo']}`。初始 HEAD `{summary['initial_head']}`、tree `{summary['initial_tree']}`。日常已审候选及当前 HEAD `{summary['candidate']}`、tree `{summary['candidate_tree']}`；生成前及结束时的 HEAD/tree 相同，源 worktree/index 干净。完整未过滤基线 diff 只有新增授权指南，原件 `raw/deliver-1/candidate-head.*`、`candidate-clean.*`、`candidate-full-paths.*`、`candidate-final-head.*`、`candidate-final-clean.*`。

# 输入检查与日常 review 的范围

implement 报告描述补齐四种选择、replace 一次/Occurrence 与冻结输入/stats、EFFECT_PENDING、unknown/human/gate/终态边界；这是实现者报告。日常 review 独立核对指南、合同与实际 caller，未发现冻结验收内需修改的缺陷；本 deliver 引用该静态审阅，不重复宣称执行过 CLI 场景。

必需两项检查的原 JSON、完整 stdout/stderr、实际 argv/cwd/UTC/exit 和摘要已独立核对，退出码均为 0、未超时、within_deadline=true：

- `scripts/check-docs.sh specs/guides/continuity-choices.md`，原件 `raw/implement-1/check-docs.*`，stdout 为 `check-docs: OK (1 个文件)`、stderr 空。
- `git diff --cached --check`，原件 `raw/implement-1/check-staged-whitespace.*`，stdout/stderr 空。

`raw/implement-1/staged-tree.*`、`post-check-tree.*` 均为 `{summary['candidate_tree']}`，与已提交候选相同。当前实际核对保存于 `raw/deliver-1/input-originals.json`，包括全部原件摘要；没有重跑这两项凑绿。文档机械检查和 staged 空白检查不能替代内容审阅、CLI 演练或最终质量。

# 完整 patch 与实际独立应用

按 project 的原 argv，在源仓库执行：

```bash
git diff --binary --full-index {summary['initial_head']} HEAD -- specs/guides/continuity-choices.md
```

完整 stdout 原 bytes 独占写入 `outputs/deliver-1/change.patch`，与 `raw/deliver-1/generate-patch.stdout` 逐字节一致，原 argv/cwd/env/UTC/exit 在 `generate-patch.json`、stderr 在 `generate-patch.stderr`。完整未过滤路径集合已先核，新增文件确已提交；没有使用普通 unstaged diff。patch {summary['patch']['bytes']} 字节，SHA-256 `{summary['patch']['sha256']}`，低于 8388608 字节上限。

只在预声明 Native 独立副本 `{summary['verify_repo']}` 应用。副本已经存在，因此未复制、重建、覆写或换位置。`verify-initial-head.*` 实际显示上述初始 HEAD/tree，`verify-initial-clean.*` stdout 空；初始授权指南不存在。依序实际执行，三项均退出 0、stderr 空：

```bash
git apply --check "{out/'change.patch'}"
git apply --index "{out/'change.patch'}"
git write-tree
```

三项 cwd 均为该独立副本。原件为 `raw/deliver-1/apply-check.*`、`apply-index.*`、`applied-tree.*`，每组含 JSON、完整 stdout、stderr。所得 tree `{summary['applied_tree']}` 与已审候选严格相同；独立副本 HEAD 留在初始基线，index/worktree staged 新增指南，没有为该验证另建提交。

`applied-full-paths.*` 与 `applied-status.*` 仅含新增指南；`applied-unstaged.*` stdout 空。指南在候选和应用副本均为 {summary['authorized_bytes']['bytes']} 字节、SHA-256 `{summary['authorized_bytes']['sha256']}`，非可执行文件，实际逐字节相等。`protected-initial.json` 记录原基线全部 {summary['protected_tracked_files']} 个 tracked 文件的内容/模式；应用后全部一致。`candidate-files.*`、`applied-files.*` 和 `candidate-applied-file-manifest.json` 保存两份完整 {summary['all_candidate_tracked_files']} 文件集合与磁盘内容/模式摘要，全集合和字节一致。tree 相等同时绑定所有 tracked 路径与 Git 模式，不只核一个授权文件。汇总原件 `raw/deliver-1/closure-summary.json`。

所有新增实际命令使用冻结 `study/capture.py.execute`，单项 120 秒且受原 hard deadline `2026-10-03T21:08:27.513464+00:00` 剩余时间限制。完整脚本、外层执行记录分别为 `raw/deliver-1/actual-deliver.py`、`actual-delivery-orchestration.*`；实际应用闭包结束 UTC `{summary['end_utc']}`，当时剩余 {summary['remaining_s']:.3f} 秒。Git/PATH/locale 等影响结果的环境记录在 actor；各 command JSON 保存 control_environment。无实际失败、无重试、无超时；没有截断必要 stdout/stderr 或扩大范围。

# 五份成果与使用说明

最终成果只包含下列三份冻结输入与两份新输出，路径从本 run 根解析；实际绝对路径、bytes 与 SHA 在 `raw/deliver-1/final-five-materials.json`，不自动加入历史草稿、脚本或证据目录：

| 角色 | 本 run 实际路径 | 字节 | SHA-256 |
| --- | --- | --- | --- |
| change | `outputs/implement-1/change.md` | {summary['reports']['change']['bytes']} | `{summary['reports']['change']['sha256']}` |
| checks | `outputs/implement-1/checks.md` | {summary['reports']['checks']['bytes']} | `{summary['reports']['checks']['sha256']}` |
| review | `outputs/review-1/review.md` | {summary['reports']['review']['bytes']} | `{summary['reports']['review']['sha256']}` |
| delivery | `outputs/deliver-1/delivery.md` | 见最终 manifest | 见最终 manifest |
| patch | `outputs/deliver-1/change.patch` | {summary['patch']['bytes']} | `{summary['patch']['sha256']}` |

交协调者核这五份原件和候选闭包。后续接受者应核目标基线和授权，再应用完整 patch；不要把已经应用的检查副本当作干净基线重试。操作者使用指南时从 status 查询开始，以现有 Work 的同一管理根、完整 ID 和当前 next 选择；新试用使用独立 Home，示例参数需替换。本文及指南命令没有在本 run 执行 Sheltie。

# 限制与结束事实

本阶段完成的是交付材料和实际 Native patch 可应用性验证，不等于最终盲审、代理/真人接受或产品发布。最终独立质量、用户接受、Sheltie/Store 操作、CLI 演练、Rust/build/tests、全仓门禁、安装、发布均 not_run；usage/fees、跨会话宿主身份、真人净收益与实际 provider 元数据 unknown。本 worker 不替代这些义务或自行推进后续流程。

本 worker 自启动命令全部同步 await 并已结束，无后台任务、无新增 helper；没有停止或声明收尾他人进程。没有运行失败后重做副本或重跑凑绿。报告写成 UTC：`{now}`；最终材料原件核验结束时间另在 `raw/deliver-1/final-five-materials.json`，协调者仍须在原 deadline 内完成本阶段收尾。
"""
b=report.encode();assert len(b)<=262144
with (out/'delivery.md').open('xb') as f:f.write(b)
materials={}
for role,rel in [('change','outputs/implement-1/change.md'),('checks','outputs/implement-1/checks.md'),('review','outputs/review-1/review.md'),('delivery','outputs/deliver-1/delivery.md'),('patch','outputs/deliver-1/change.patch')]:
 p=run/rel;data=p.read_bytes();assert len(data)<=(8388608 if role=='patch' else 262144);materials[role]=dict(path=str(p),bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
 foriginal=summary['reports'].get(role)
 if foriginal:assert materials[role]==foriginal
assert materials['patch']['sha256']==summary['patch']['sha256']
result=dict(materials=materials,candidate=summary['candidate'],candidate_tree=summary['candidate_tree'],applied_tree=summary['applied_tree'],raw_dir=str(raw),end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),self_started_background_tasks=False,helpers=[],usage='unknown',fees='unknown')
with (raw/'final-five-materials.json').open('x') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps(result,ensure_ascii=False,indent=2))
