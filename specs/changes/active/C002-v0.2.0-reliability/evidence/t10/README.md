# C002-T10 证据

- Owner：Claude（glm 会话）。基准 `766dbe29313be505e67634c2abc5b0e6e31ed1f4`，待提交树见提交说明。
- 输入闭包：`Cargo.lock` 未变；特性全开；平台 macOS aarch64、`rustc 1.98.1`。
- 改动：`skills/sheltie/SKILL.md`——选 Workbook（未安装即停、不静默替换）、输入从 `show` 的 `start_inputs` 发现且用户已给的信息直接用（不用失败 start 探测）、`next` 只限定 Work 推进（发现/管理命令有独立入口、空 next 不是无事可做）、request-id 预先保存与支持范围（只读/self 拒绝）与重放后查当前 status、gate/human 的代执行如实记录 OS 账户；`specs/contracts/protocol.md` gate approve 补「代执行记录的是 agent 进程账户」边界句。CLI 测试新增三例（work.rs）。

## 正反例

| 例 | 独立期望 | 结果 |
| --- | --- | --- |
| 只读与 self 给 request-id（反） | list/status/self version 均退出码 2、`INVALID_REQUEST` | PASS |
| 指定未装 Workbook（反） | `NOT_FOUND` 点名 ghost；无 Work 创建（不静默替换） | PASS |
| 重放后续接（正） | 同 id 重放 submit 得原快照（replayed=true）；`work status` 仍是 cancelled | PASS |
| show 暴露有序 start_inputs（正，T02 已测） | `flows[].start_inputs` 与文本「起始输入」 | PASS（回归） |

离线文案与 CLI 行为已验证；宿主实际行为（不重复问、代执行告知）归 T16 真实宿主回归。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；408 passed | [nextest.txt](nextest.txt) |
| `scripts/check-skill.sh` | 0 | [skill.txt](skill.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 关闭：T02 发现面 + 本任务的行为指引（skill 文本与 CLI 拒绝路径）。宿主行为证据（N04 的 gate 代执行告知、不重复问）归 T16，未执行前不声称。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
