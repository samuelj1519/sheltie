# Sheltie

本地运行的工作流引擎，给协调者 agent 用。人把做事方法写成 Workbook（TOML 图加自然语言说明），协调者 agent 按图派活，引擎记状态、发任务书、限定合法下一步、守门槛。引擎不判断内容好坏。

词汇见 [CONTEXT.md](CONTEXT.md)；规格见 [specs/README.md](specs/README.md)；agent 入口见 [AGENTS.md](AGENTS.md)。

## 快速开始

装引擎（macOS 与 Linux，x86_64 与 aarch64）：

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/latest/download/sheltie-cli-installer.sh | sh
export PATH="$HOME/.sheltie/bin:$PATH"    # 安装器也会提示 source ~/.sheltie/bin/env，效果相同
sheltie self version
```

拿一份样例 Workbook 并装进来（`workbook add` 的相对路径在克隆的父目录里执行）：

```bash
git clone --depth 1 https://github.com/samuelj1519/sheltie.git
sheltie workbook add sheltie/examples/two-step
```

开一个 Work 并走完它。`work start` 的响应给出 `work_id`，之后用它的唯一前缀即可（下面写作 `<work>`）：

```bash
sheltie work start --workbook two-step --flow default --input topic="给新人介绍 Sheltie"
sheltie attempt begin <work> --node outline
```

`attempt begin` 的响应给出 `brief_path`（任务书）与输出目录。读任务书，按它把提纲写到输出目录里的 `outline.md`，然后提交：

```bash
sheltie attempt submit <work> --attempt outline#1.0 --summary "三段提纲：是什么、怎么用、边界"
sheltie attempt begin <work> --node summary
```

同样读任务书、写 `summary.md`、提交：

```bash
sheltie attempt submit <work> --attempt summary#1.0 --summary "按提纲写完摘要"
sheltie work status <work>          # status: succeeded
```

不知道下一步做什么，就看每次响应里的下一步列表（文本模式下是「下一步：」，JSON 模式下是 `next` 数组，每项都是可直接执行的命令），或读 `sheltie work status <work>` 的状态卡。升级用 `sheltie self update`，出问题 `sheltie self rollback`。

在 Claude Code 里可以输入 `/sheltie` 让协调者代劳。装 skill 取发布资产 `sheltie-skill.tar.gz`（发布流程随每版挂出的自包含交付，解压即用，不依赖保留源码目录）：

```bash
mkdir -p ~/.claude/skills
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/latest/download/sheltie-skill.tar.gz | tar xz -C ~/.claude/skills/
```

## 开发

```bash
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
cargo deny check
scripts/check-docs.sh
```

工具链由 `rust-toolchain.toml` 固定为 stable（edition 2024，MSRV 1.85）。

## 许可

Sheltie 以 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双许可发布，使用者可任选其一。仓库中单独标注许可的第三方文件仍遵循各自的声明。
