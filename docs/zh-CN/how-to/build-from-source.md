# 构建当前源码

[English](../../en/how-to/build-from-source.md) | 简体中文

用于运行 0.3.0 源码基线。构建直接取得可信二进制，不安装或替换用户版本；需要 macOS aarch64／APFS 环境中的源码副本、仓库工具链、Git、Python 3 和 Bash。

以下命令在源码仓库根、同一 Bash 会话运行；任一命令失败立即停止，保留 stdout、stderr 和退出码。首次使用不指向旧 Store，也不清理旧数据。

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

解析失败就停止。核响应的 `data.version=0.3.0`、`data.schema_version=4` 和 `data.home` 为本次 `source_home`。记录 `engine_binary`、`source_home`、`session_dir` 的实际绝对路径；所有后续调用始终带同一 `--home`，重开会话恢复这些值，不能回落到 `SHELTIE_HOME` 或默认 `~/.sheltie`。新的管理根尚未创建，`self version` 不创建它。

## 使用构建结果

学习基本流程时继续[教程](../tutorials/first-work.md)；执行真实仓库任务时继续[code-change](run-code-change.md)。上述构建创建独立会话目录，`self version` 不创建业务 Home。装入方法或启动 Work 后才由相应写操作初始化管理根。构建不会替换已安装二进制或旧 Store。
