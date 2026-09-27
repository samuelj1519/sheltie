# 2026-09-27 审查原始证据

代码候选：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。这些文件记录缺陷复现与既有门禁，不是修复通过证据。JSON 中的临时绝对路径、日期和 request-id 是原始运行值，不要求重跑完全相同。

## 重跑

在待审候选的仓库根，用独立 target 构建，然后从 Cargo JSON 中取得 executable。不要猜用户全局 target 位置。

```bash
review_dir="$(mktemp -d /tmp/sheltie-review.XXXXXX)"
env RUSTC_WRAPPER= CARGO_TARGET_DIR="$review_dir/target" \
  cargo build --locked --all-features --message-format=json > "$review_dir/build.jsonl"
export SHELTIE_REVIEW_BIN="$(python3 -c 'import json,sys; print(next(x["executable"] for x in map(json.loads,open(sys.argv[1])) if x.get("executable") and x.get("target",{}).get("name")=="sheltie"))' "$review_dir/build.jsonl")"
export SHELTIE_REVIEW_REPO="$PWD"
python3 specs/changes/proposed/C002-v0.2.0-reliability/evidence/2026-09-27/request_probe.py
```

其他 `*_probe.py` 以相同方式逐个运行。每个脚本在自己的临时目录中使用假数据；`boundary_probe.py` 的 self install 使用假 HOME，只修改该临时树。脚本用于观察当前候选，不把复现 bug 的 exit 0 当产品 PASS；修复者应把相应预期转成正式回归断言。`workbook_probe.py` 使用合成业务文档，不构成人工审批或真实 agent 交付证据。

## 文件映射

| 文件 | 内容 |
| --- | --- |
| request_probe.py / request-probes.json / probes.txt | 跨 Work 请求、输出变化重放、历史响应、删除 Workbook 后 start、缺输入目录、历史 stats |
| boundary_probe.py / standards-probes.json | 新 home、双 hash、tampered start、根外软链写、固定临时文件覆盖、add 发布失败、add request 重放、JSON stdout |
| hash_probe.py / hash-framing.json | 两棵不同目录具有同一个旧编码摘要 |
| output_probe.py / output-probes.json | stats 输入/输出碰撞、brief 祖先、输出祖先冲突 |
| status_probe.py / status-probes.json | JSON 失败原因与文本差异 |
| stats_probe.py / stats-probes.json | NoLegalEdge 取消后计数减少、边类型丢失 |
| workbook_probe.py / workbook-probes.json | 人意见绑定与 escalate next 缺口 |
| git_probe.py / git-probes.json | 当前任务 diff 和最终整体 diff 的独立 Git oracle |
| shallow-specs-check.txt / rc-specs-check.txt | 浅克隆历史缺失，以及仅改 active 版本时的检查器假设 |
| nextest.txt / fmt.txt / check.txt / clippy.txt | 原始 Cargo 门禁输出 |
| docs-before.txt / specs-before.txt / test-owners.txt / vocab.txt / skill.txt | 审查前原始脚本输出 |
| deny-offline.txt | 默认 advisory 缓存锁被沙箱拒绝；后续隔离缓存验证见 validation |
| independent-before-c002.md | 主审在读取旧 C002 前记录的独立发现 |

浅克隆反例：在新临时路径运行 `git clone --depth 1 --no-local file://<候选仓库绝对路径> <临时路径>/repo`，进入 clone 跑 `scripts/check-specs.sh`。RC 反例在这个隔离 clone 中改 Cargo 版本为 `0.2.0-rc.1` 后重跑；单改版本会同时触发缺 changelog/record，检查器要求 tag 的静态依据见脚本末段。正式修复测试应另建完整 active/RC fixture，不能仅补 changelog 就声称已覆盖生命周期。
