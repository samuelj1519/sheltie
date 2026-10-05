# 取得可编辑的最终成果副本

当前源码提供只读 raw 成果入口和外围 `sheltie-export` 工具。工具未发布或安装；从仓库临时构建。完整语义见[协议§8](../contracts/protocol.md#8-最终-artifact-原字节与外围导出)，设计与验证范围见[C006 摘要](../changes/completed/C006-result-delivery/README.md)。

## 构建与输入

在仓库根执行，Cargo JSON中的`executable`才是实际binary路径：

```bash
(
set -e
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-export-build \
  cargo build -p sheltie-cli -p sheltie-export --locked --message-format=json \
  > /private/tmp/sheltie-export-build.json
python3 - <<'PY'
import json, hashlib
from pathlib import Path
paths = {}
for line in Path('/private/tmp/sheltie-export-build.json').read_text().splitlines():
    item = json.loads(line)
    if item.get('reason') == 'compiler-artifact' and item.get('executable'):
        name = item['target']['name']
        if name in ('sheltie', 'sheltie-export'):
            paths[name] = Path(item['executable'])
assert set(paths) == {'sheltie', 'sheltie-export'}, paths
for name, p in paths.items():
    print(name, p, hashlib.sha256(p.read_bytes()).hexdigest())
PY
)
```

构建或解析非零立即停止；只有得到两项实际路径与SHA后才执行后续操作。

把实际路径分别赋给`engine_binary`、`export_binary`。准备`management_root`（已有真实无链接绝对管理根）、完整`work_id`和`destination_parent`（已有真实无链接绝对目标父目录）。Home与目标父目录不能相互包含。二进制必须是可信绝对路径，工具不搜索PATH。

## 查询和复制

先核结果的`final=true`、`status.kind=succeeded`、没有待完成effects且选集非空。仅需要引用时使用查询结果；需要可编辑文件时执行复制：

```bash
"$engine_binary" --home "$management_root" --json work result "$work_id"
"$export_binary" --sheltie "$engine_binary" --home "$management_root" \
  --work "$work_id" --to "$destination_parent" --json
```

JSON响应是一行`work-export/v1`。只有`complete`和退出码0表示规定的字节核验、整目录不替换发布和OS同步完成；从`target_path`取得副本。`manifest.json`保存同一次结果和实际成果映射，`artifacts/0001/...`按结果key顺序编号。索引目录避免空key、Unicode或相同文件名冲突，不能把key直接拼成目标路径。副本可以编辑；编辑后的字节不再由初始清单保证，源Work不变。

单个Artifact上限32MiB、选集合计256MiB、结果JSON上限1MiB。工具独立核全部size/sha及源退出码，全部成功才发布；不覆盖已有副本。再次执行创建另一份新副本，保留此前用户编辑。

若只需某项原字节，使用同一次结果的正revision与literal key。先用mktemp创建本次新私有暂存文件；shell的`>`会截断它，不能指向已有文件：

```bash
local_file=$(mktemp /private/tmp/sheltie-artifact.XXXXXX)
"$engine_binary" --home "$management_root" work result "$work_id" \
  "--artifact=$artifact_key" --revision "$result_revision" > "$local_file"
```

raw模式不能带`--json`或`--request-id`，stdout不补换行。必须核最终退出码；读取失败可能已输出部分字节，不把该文件当完整成果。

## 失败现场

- `rejected` / 2：参数或确定性资格不合格，核`error.code`与输入。
- `failed_before_publish` / 1：源、完整性或IO失败。`staging_path`只指能证明所属的本次私有暂存，不能当完成目录。
- `publication_unconfirmed` / 3：整目录移动已经发生，但最终身份或父同步未确认。保留现场，核实际对象、manifest和字节；不删除或回滚补偿。

进程被 kill 可能没有响应。保留 stdout、stderr、退出码、实际路径、权限与源结果身份，核真实对象、manifest 和文件 bytes／SHA。不能据目录名字认领对象、接管旧暂存或删除不明目录；无法确认发布时不将它当作完成成果。重跑始终创建新副本，不复用旧 UUID 或覆盖旧副本。该工具不保证停止整个宿主进程树、物理断电持久或用户编辑后的持续一致。

当前采用与实测范围为 macOS aarch64／APFS。跨 APFS 载体验收不覆盖外置物理设备专项认证或其他 OS。当前发布配置排除 exporter，临时构建不自动进入 release；实际发布范围见[发布记录](../releases/README.md)。技术核验、实际副本消费与人类成本结论的边界见[当前限制](../limitations.md)。
