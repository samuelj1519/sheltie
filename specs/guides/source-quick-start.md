# 当前源码快速开始

本文面向首次使用当前源码的协调者，覆盖 `0.3.0-rc.1`、Store schema 4 和 `cli-result/v4`。源码候选尚未发布；[README 的远端安装](../../README.md#快速开始)仍是已发布的 v0.2.0。这里直接运行构建产物，不安装、更新或发布。

使用 [code-change](../../examples/code-change/README.md) 完成已获准的本地仓库任务：implement → review → deliver，review 可返工到 implement。工作 agent 执行各节点内容，协调者核对报告、候选与检查，选择合法下一步；引擎只记状态、冻结文件并守输出合同。方法默认没有 gate，不授予合并、部署或发布权限。

## 1. 构建并固定本次路径

前提是已有源码副本、仓库指定的 Rust 工具链、Git、Python 3 和 Bash，以及明确的任务授权。以下命令在源码仓库根、同一 Bash 会话运行；任一命令失败立即停止，保留 stdout、stderr 和退出码。首次使用不指向旧 Store，也不清理旧数据。

```bash
project_repo=$(pwd -P)
session_dir=$(mktemp -d /private/tmp/sheltie-source.XXXXXX)
source_home="$session_dir/home"
RUSTC_WRAPPER= CARGO_TARGET_DIR="$session_dir/target" \
  cargo build -p sheltie-cli --bin sheltie --locked --message-format=json \
  > "$session_dir/build.json"
```

构建退出码为 0 后，从 Cargo JSON 取得实际 `executable`，不猜 `target/debug` 路径：

```bash
engine_binary=$(python3 - "$session_dir/build.json" <<'PY'
import json, os, sys
from pathlib import Path
paths = set()
for line in Path(sys.argv[1]).read_text().splitlines():
    item = json.loads(line)
    if (item.get('reason') == 'compiler-artifact'
            and item.get('target', {}).get('name') == 'sheltie'
            and item.get('executable')):
        paths.add(item['executable'])
assert len(paths) == 1, paths
binary = Path(paths.pop())
assert binary.is_absolute() and binary.is_file() and os.access(binary, os.X_OK)
print(binary)
PY
)
"$engine_binary" --home "$source_home" --json self version
```

解析失败就停止。核响应的 `data.version=0.3.0-rc.1`、`data.schema_version=4` 和 `data.home` 为本次 `source_home`。记录 `engine_binary`、`source_home`、`session_dir` 的实际绝对路径；所有后续调用始终带同一 `--home`，重开会话恢复这些值，不能回落到 `SHELTIE_HOME` 或默认 `~/.sheltie`。新的 `home` 尚未创建，`self version` 不创建它。

## 2. 准备真实 task 与 project

task 写目标、约束、验收和停止预算；project 写仓库绝对路径、初始候选、允许文件、工作区前提和必需检查。两份输入在 start 时按字节冻结；标准缺失时先停止补齐，目标改变后另开 Work。

下面是可执行的文档任务示例，仅适用于已获准修改本源码副本的 `README.md`。其他任务在 start 前替换两份材料；不改 Workbook 来迁就任务。先运行只读检查，确认工作区干净，否则记录已有改动、与授权方确认前提，不能清理他人修改：

```bash
git status --short --untracked-files=all
project_head=$(git rev-parse HEAD)
cat > "$session_dir/task.md" <<'TASK'
# 任务
在 README.md 的“开发”命令块补充 cargo fmt --all -- --check。
保留全部原命令与其他正文；只修改 README.md。
验收：新增命令准确、只出现一次，文档链接检查与 diff 空白检查成功。
检查失败、权限或标准不足时停止，保留原始输出；不得安装、合并或发布。
停止预算：遵守方法的到达和失败重试上限，不为通过而删除检查或扩大范围。
TASK
cat > "$session_dir/project.md" <<PROJECT
# 项目
仓库绝对路径：$project_repo
初始 HEAD：$project_head
工作区前提：已只读确认干净；保留他人文件，不自动清理或回退。
允许修改：README.md。任务输入位于仓库外，不作为产品文件提交。
必需检查（在仓库根执行）：
- scripts/check-docs.sh README.md
- git diff --check
报告需绑定实际候选、修改文件和完整检查命令、cwd、环境、stdout、stderr、退出码。
执行检查后候选变化则旧结果失效；未知与未执行项如实记录。
PROJECT
```

## 3. 装入方法并开 Work

```bash
"$engine_binary" --home "$source_home" --json workbook add examples/code-change
"$engine_binary" --home "$source_home" --json workbook verify code-change@1.0.0
"$engine_binary" --home "$source_home" --json workbook show code-change@1.0.0
```

逐条核成功响应。verify 的结果应为 `ok`；show 的 `default` Flow 起始键 `data.flows[].start_inputs` 应为 `task`、`project`。检查 `requires`；缺声明的宿主资源就停止，引擎不检查或安装它们。本示例没有声明宿主资源。

Work / Workbook 写操作若需安全重试，调用前生成并记录 UUID，用 `--request-id` 传入，重试保持同一 ID 和参数。只读操作及整个 `self` 组不接受它。以下为 start 示例：

```bash
start_request_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
"$engine_binary" --home "$source_home" --json --request-id "$start_request_id" \
  work start --workbook code-change@1.0.0 --flow default \
  --input "task=@$session_dir/task.md" --input "project=@$session_dir/project.md"
read -r -p '粘贴 start 成功响应的 data.work_id: ' work_id
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

保存完整 Work ID 和原响应。当前 status 的 `next` 应包含进入 implement 的操作；先核 `effects_pending=false`、`pending_publish=false`，再推进。

## 4. 领取任务书、派活与提交

每次 begin 前从当前 `next` 核对应 node；下列命令按正常首轮顺序展示，不是无条件执行的脚本：

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node implement
```

读取响应的 `data.attempt`、`data.brief_path`、`data.inputs`、`data.outputs`，把真实 `brief_path` 交给工作 agent。工作 agent 完整读任务书和冻结 task/project，完成授权修改与检查，在 `outputs.change` 指定位置写 `change.md`；报告第一行给实际结论，绑定候选、原始检查与失败、unknown、`not_run`。首次 `previous-review` 为 null；不能把未绑定材料当反馈。输入和已封存输出只读，禁止写 Store 或管理元数据。

协调者读报告并核实际候选、检查和必需文件齐全后提交；Attempt ID 必须来自实际 begin 响应，不猜后缀：

```bash
read -r -p '粘贴 implement begin 响应的 data.attempt: ' attempt_id
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$attempt_id" --summary "实现与检查已完成，实际结论见 change.md"
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

引擎提交只校验声明输出的文件属性、限额与封存，不核报告中的代码或检查结论。输出不齐或超限时 Attempt 仍 running，按错误原因处理，不把拒绝记成成功。

## 5. 独立审查、返工与交付

当前 `next` 允许 review 后执行：

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node review
```

交给未参与实现的审查 agent，读取新任务书、冻结输入和实际候选，按 task 标准独立核查，在 `outputs.review` 写 `review.md`。协调者读取报告，使用 review begin 的实际 Attempt ID 提交：

```bash
read -r -p '粘贴 review begin 响应的 data.attempt: ' attempt_id
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$attempt_id" --summary "独立审查完成，结论见 review.md"
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

审查要求内容返工时仍 submit，协调者从当前 `next` 选 `edge=back` 的 implement；begin 新任务书，读取绑定的具体 `previous-review`，修复后再次进入独立 review。审查完成且可交付时选 `edge=main` 的 deliver。报告里的“通过”不自动选边，也不等于执行 `gate approve`。

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node deliver
```

派交付 agent 读真实任务书，核 change/review 指向同一候选，按 `outputs.delivery` 写 `delivery.md`，列实际成果位置、检查原件、审查结论、使用说明和未完成义务。协调者核对后使用本次实际 ID 提交：

```bash
read -r -p '粘贴 deliver begin 响应的 data.attempt: ' attempt_id
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$attempt_id" --summary "成果已整理，未完成义务见 delivery.md"
"$engine_binary" --home "$source_home" --json work result "$work_id"
```

只有 `data.final=true` 且 `effects_pending=false` 才取最终选集。本方法选 change、review、delivery 三项；每项 `path`、`sha256`、`bytes`、`source` 指向具体 deliver Attempt 绑定的输入或封存输出。`final=false` 时选集为空；不要猜最新报告。查询列举冻结引用，不重新证明源字节、代码质量或用户接受；消费原件仍核实际字节。需要新的可编辑副本时见 [成果导出指南](result-export.md)，复制、合并与发布各按实际授权办理。

## 6. 中断、错误与停止

重开会话先恢复原二进制和显式 Home 路径，再只读发现或查询当前状态：

```bash
"$engine_binary" --home "$source_home" --json work list
"$engine_binary" --home "$source_home" --json work status "$work_id"
"$engine_binary" --home "$source_home" --json work stats "$work_id"
```

核 `data.revision`、`status`、`effects_pending`、`pending_publish`、当前 `next` 与 `resume`。`resume=null` 表示没有当前 Attempt；否则按 `resume.brief_path`、冻结 `inputs` 和 `draft_outputs` 接续。草稿路径不证明文件存在、完整或已封存。先确认旧执行者和共享工作区已妥善处置，再继续原 running Attempt；普通会话重开不调用 fail、begin 或 replace 制造新尝试。历史写请求重放的 `next` 是当时快照，不能用它推进当前 Work。

| 观察 | 处理边界 |
| --- | --- |
| 非零退出、`ok=false` 或未知是否提交 | 停止后续命令，保存命令、cwd、环境、stdout、stderr、退出码、request-id 与候选；无证据不称成功，不改标准。写请求结果未知时按已记录的同一 ID/参数核实，不能新开 Work 猜测补做。 |
| `EFFECT_PENDING` / pending 标志 | 只读查询不恢复效果。`committed=true` 时用本次同 request-id 重试；`committed=false` 时先按 `pending_request_id` 恢复旧请求，再重试本次原 ID。准确保留 `cause`；错误持续、原意图或身份不明时停止，不直接改管理文件。 |
| 工作 agent 确实崩溃、超时或无法交付 | 仅当前 next 允许时以真实原因调用 `attempt fail`；内容审查不通过走 submit/back。implement、review 各最多到达 3 次，每次允许 1 次执行失败重试；deliver 最多到达 1 次、允许 1 次失败重试。不得增次数或节点绕过预算。 |
| `blocked(retries_exhausted)` / `blocked(no_legal_edge)` | 停止派活并报告；只从当前 next 选择已授权操作。取消是写操作，不因受阻自动执行。外部时间、调用或费用预算若另有规定也必须停止；未知费用记 unknown。 |
| `status.kind=succeeded` / `cancelled` | Work 已终态，不能继续 begin/submit/fail；成功后只读取明确成果，取消不产生成功选集。终态或流程成功不代替独立质量与用户接受。 |
| `STORE_SCHEMA_MISMATCH` | schema 1/2/3 旧 Store 整体拒绝业务打开；保留旧管理根原件，用旧二进制配旧根查询。使用新的显式 Home，不迁移、清空或回滚数据库。 |
| `ARTIFACT_MODIFIED` / `STORE_CORRUPT` / 标准或权限不足 | 保留原件与准确错误，停止调查或交协调者处理，不改冻结输入、封存报告或验收去取得成功。 |

确需标真实失败时，取得当前 status 中的实际 Attempt ID，并记录具体原因后调用：

```bash
"$engine_binary" --home "$source_home" --json attempt fail "$work_id" \
  --attempt "$attempt_id" --reason "实际执行失败原因"
```

命令、状态和恢复细则见 [协议合同](../contracts/protocol.md)；协调职责见 [Sheltie skill](../../skills/sheltie/SKILL.md)。
