# 取得可编辑的最终成果副本

[English](../../en/how-to/export-results.md) | 简体中文

用于把明确最终成果交给后续消费者，或建立可以继续编辑的新副本。当前源码提供原字节入口与未发布的外围 `sheltie-export`。参数、格式、限额和退出状态见[导出参考](../reference/export.md)。

需要当前可信引擎、已有 Work 的准确管理根，以及明确授权的目标父目录。Home 与目标父目录必须是真实无符号链接的绝对目录，且不能相互包含。示例的 `management_root`、`work_id` 与 `destination_parent` 必须先设为实际值，Work ID 使用完整字符串。

## 1. 构建并记录实际二进制

在源码仓库根、同一 Bash 会话运行。构建失败立即停止，不解析或执行旧产物：

```bash
export_session=$(mktemp -d /private/tmp/sheltie-export.XXXXXX)
RUSTC_WRAPPER= CARGO_TARGET_DIR="$export_session/target" \
  cargo build -p sheltie-cli -p sheltie-export --locked --message-format=json \
  > "$export_session/build.json"
```

构建成功后，从 Cargo JSON 取得两个实际 `executable`。不猜 target 目录，不使用共享固定日志：

```bash
python3 - "$export_session/build.json" <<'PYTHON'
import hashlib, os, sys
from pathlib import Path
import json
paths = {}
for line in Path(sys.argv[1]).read_text().splitlines():
    item = json.loads(line)
    if item.get('reason') != 'compiler-artifact' or not item.get('executable'):
        continue
    name = item['target']['name']
    if name in ('sheltie', 'sheltie-export'):
        paths[name] = Path(item['executable'])
assert set(paths) == {'sheltie', 'sheltie-export'}, paths
for name, path in sorted(paths.items()):
    assert path.is_absolute() and path.is_file() and os.access(path, os.X_OK)
    print(name, path, hashlib.sha256(path.read_bytes()).hexdigest())
PYTHON
```

将实际路径分别赋给 `engine_binary` 与 `export_binary`，保存输出中的 sha256。用 `self version` 核引擎版本、Home 和 Store 格式与该 Work 相匹配。已有当前源码的可信构建可以直接使用，不必重建；已发布旧格式不能用当前 exporter 解释。

## 2. 查询同一次成果

```bash
"$engine_binary" --home "$management_root" --json work result "$work_id" \
  > "$export_session/result.json"
cat "$export_session/result.json"
```

命令成功后，核 `data.final=true`、`data.status.kind=succeeded`、`data.effects_pending=false` 和非空 `data.artifacts`。`final=false` 时先按当前状态完成合法运行或恢复；`final=true` 但集合为空表示方法没有选择成果，不从历史目录补猜一份文件。

普通查询只列引用，取得原件仍须核实际字节。只需要引用时到此结束。

## 3. 发布新副本

确认目标父目录与复制授权后：

```bash
"$export_binary" --sheltie "$engine_binary" --home "$management_root" \
  --work "$work_id" --to "$destination_parent" --json \
  > "$export_session/export.json"
cat "$export_session/export.json"
```

只有退出码 0 且 `status=complete` 才使用 `target_path` 中的副本。工具重新查询并核源结果与原字节，不使用前一步保存的 JSON 代替源读取。

`manifest.json` 保存源结果和副本文件映射，`artifacts/0001/...` 等索引目录按 key 顺序编号。使用 manifest 中的相对路径找到对应文件；key 不直接作为路径。副本可以编辑，编辑后的 bytes 不再由初始清单保证，源 Work 不变。

重跑另建新副本，保留此前编辑，不覆盖或合并已有目录。工具限额见[导出参考](../reference/export.md#文件与格式)。

## 4. 只读取单项原字节

从保存的同一次结果取得正 revision，将 `artifact_key` 设为其中一项准确 key。用 mktemp 建立本次新文件，避免 shell 重定向截断已有用户文件：

```bash
result_revision=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["revision"])' "$export_session/result.json")
local_file=$(mktemp /private/tmp/sheltie-artifact.XXXXXX)
"$engine_binary" --home "$management_root" work result "$work_id" \
  "--artifact=$artifact_key" --revision "$result_revision" > "$local_file"
```

raw 不接受 `--json` 或 `--request-id`，不补换行。先核最终退出码，再核文件的实际 bytes 与 sha256 和该项记录相同；读取失败可能留下部分 bytes，不能交付这个暂存文件。

## 失败现场

| 状态与退出码 | 处理 |
| --- | --- |
| `rejected`／2 | 核 `error.code`、路径、结果资格与参数；未成功不能使用目标 |
| `failed_before_publish`／1 | 保留源、完整性或 I/O 错误；`staging_path` 仅为本次可核所属暂存，不是完成副本 |
| `publication_unconfirmed`／3 | 移动已发生但最终身份或父同步未确认；保留现场，核实际对象、manifest 和文件 bytes，不自动删除或回滚 |

进程被终止可能没有响应。保存 stdout、stderr、退出码、实际路径、权限与源结果身份，核实际对象和清单。不能按名字认领对象、接管旧暂存或删除不明目录；无法确认时报告未确认，不把目录存在当作完成。重跑仍另建副本。

支持范围为 macOS aarch64／APFS。规定 OS 同步不等于物理断电持久，跨 APFS 载体验收不覆盖外置物理设备专项认证；工具也不保证整棵进程树停止、同权限隔离或用户编辑后的持续一致。完整边界见[限制](../reference/limitations.md)。
