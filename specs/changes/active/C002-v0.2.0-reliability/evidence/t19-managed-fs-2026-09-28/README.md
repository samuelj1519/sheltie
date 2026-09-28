# C002-T19 受管文件原语证据

候选基线：`69710aa417b2f34967a0e23372d695cd11abed3f`（T18）。实现者：Codex。平台：macOS arm64。Linux 按用户授权豁免，状态为 `not_run`；本记录不代表跨平台通过。

## 改动与正反例

- `ManagedRelPath`、`ManagedFs` 和 `ManagedDir` 将受管路径校验、目录句柄操作与 `HomeLock` 根身份绑定；原 `fsx` 写入、同步、rename、权限和删除 helper 已改为要求 `Home` 与同根锁。
- 目录句柄路径替换后仍在旧目录对象写入；错误 Home 的锁在动作前拒绝；打开后增加根外硬链接时，chmod 在任何权限副作用前拒绝；临时文件残留检查匹配 `.tmp-<uuid>` 命名。
- 两个 `rename_new` 入口共用源类型/link-count检查。软链源拒绝后仍留在原位，目标不存在，外部哨兵字节和权限不变。
- `Home::acquire_lock` 将根内结构化 `NotFound` 和根内 `Io(NotFound)` 作为构造窗口瞬时缺失整体重试；组件式路径比较拒绝相邻根。根锁身份错误和其他错误不进入这条重试分支。
- purge 底层保留原管理根与同一个 `.lock`。本任务只保证受管原语及基本保留行为；Store 最后删除、完整预检/部分失败协议归 T23，等待者确定性交错归 T23/T31，未据此关闭 R04。

## 命令与结果

以下原始 stdout、stderr 和 exit code 分文件保存。最终全仓 Nextest run ID 为 `00bfcb15-586c-427a-9f6b-99a904abd2f1`，475 passed、0 skipped；`check_specs_full_history_resolves_release_commits` 在该候选运行中未触发泄漏标记。T19 定向任务 run ID 为 `2ecf8245-8a54-49c5-860d-a9053cc249d3`，12 passed、463 skipped。较早同候选运行 `0ce17545-17ba-4864-8206-057e6e77d99f` 曾报告该 governance test 为 leaky；最终完整重跑结果见上述日志。

| Gate | 命令 | Exit | 原始输出 |
| --- | --- | ---: | --- |
| fmt | `cargo fmt --all -- --check` | 0 | `fmt.*` |
| check | `cargo check --offline --all-targets --all-features` | 0 | `check.*` |
| Clippy | `cargo clippy --offline --all-targets --all-features -- -D warnings` | 0 | `clippy.*` |
| 全仓测试 | `cargo nextest run --offline --workspace --all-features --no-tests=pass --no-fail-fast` | 0 | `nextest.*` |
| T19 定向测试 | `scripts/task.sh C002-T19` | 0 | `task.*` |
| deny | `cargo deny --offline check` | 0 | `deny.*` |
| MSRV | `cargo +1.85.0 check --offline --workspace --all-targets --all-features --locked` | 0 | `msrv.*` |
| 文档/spec | `scripts/check-docs.sh`; `scripts/check-specs.sh` | 0 | `docs.*`, `specs.*` |
| 辅助门禁 | `scripts/check-core-vocab.sh`; `scripts/check-tests.sh`; `scripts/check-skill.sh`; `git diff --check` | 0 | 对应 `*.stdout.txt`、`*.exit` |
| 拼写门禁 | `pre-commit run typos --files crates/sheltie-runtime/src/fsx.rs _typos.toml` | 0 | `typos.*` |

`cargo deny --offline check` 使用临时 `CARGO_HOME` 下预置的本机 advisory database，避免网络访问；advisories、bans、licenses、sources 均通过。deny 现有配置的未命中许可白名单及 `winnow` 多版本只产生原有 warning，未变成失败。其 stderr 含 cargo-deny 的表格空格，原始字节以 base64 保存于 `deny.stderr.raw.b64`（用 `base64 -D < deny.stderr.raw.b64` 还原），避免改写原输出或让空格触发 `git diff --check`。

首次提交钩子的 `typos` 把有效常量 `OFlags::WRONLY` 自动改成无效拼写，后续 Cargo hook 检出编译失败，提交因此被中止。已恢复源码并只在 `_typos.toml` 加入精确词条 `WRONLY`，扩展 T19 文件白名单后重跑拼写门禁与全套门禁；没有排除源码文件。该中止尝试未创建提交。

## 独立审查与边界

- Spec reviewer 最终通过：pending owner/container 的持久顺序、无 fallback、句柄与 rename 行为、根锁重试分类均符合本任务合同。T23 purge 全链与 T31 等待者交错仍未关闭。
- Standards reviewer 最终通过：锁旁路、chmod 前 link-count、临时文件 oracle 和两个 rename 源对象条件均已修正；确认 T19 范围不替代后继任务。
- 原始 T15 证据保留。被 T18 生命周期合同取代的 `sleep(100ms)` 删根 waiter 不再作为当前行为测试；其替代职责分别保留给 T23 与 T31。

门禁原始文件名：`<gate>.stdout.txt`、`<gate>.stderr.txt`、`<gate>.exit`；T19的暂存区任务核验见 `check-task.*`。本证据限定当前 macOS 候选，不表示 M1 完成。
