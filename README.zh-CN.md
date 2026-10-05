# Sheltie

[English](README.md) | 简体中文

**给协调者 agent 用的本地工作流引擎，支持中断后续接。**

人把做事方法写成 **Workbook**：TOML 图加自然语言说明。一次运行称为 **Work**。Sheltie 明确记录状态、任务书、输入、输出和合法下一步，协调者负责派发任务并判断成果。

[文档](docs/zh-CN/README.md) · [快速开始](#快速开始) · [参与贡献](CONTRIBUTING.md) · [发布版本](https://github.com/samuelj1519/sheltie/releases)

## Sheltie 提供什么

- **可复用的方法：** 在 Workbook 中声明步骤、执行者、输入、输出、审查回环和批准门槛。
- **具体的任务书：** 每个 Attempt 获得冻结输入的路径和声明的输出要求。
- **本地持久进度：** 查看状态卡，中断后从已记录状态续接。
- **明确的控制流程：** 按 CLI 返回的合法下一步操作，支持文本和 JSON 输出。

引擎是一个 Rust 二进制，使用本地 SQLite 保存状态。引擎不调用模型、不判断自然语言内容、不安装宿主资源，也不自动发布成果。这些职责由协调者承担。详见[支持范围与限制](docs/zh-CN/reference/limitations.md)。

## 选择版本

当前产品环境为 **Apple Silicon Mac（`aarch64`），使用 APFS 文件系统**。

| 版本类型 | 版本 | 从哪里开始 |
| --- | --- | --- |
| 最新有记录的发布版本 | **v0.2.0** | [发布记录](docs/zh-CN/reference/releases/v0.2.0/README.md)，或使用下方安装命令 |
| 当前源码候选 | **0.3.0-rc.1**，尚未发布 | [从源码构建](docs/zh-CN/how-to/build-from-source.md)，再运行[第一个 Work](docs/zh-CN/tutorials/first-work.md) |

当前源码以英文为默认语言，并提供 [code-change](docs/zh-CN/how-to/run-code-change.md)、[明确成果导出](docs/zh-CN/how-to/export-results.md)等新增能力。v0.2.0 二进制不包含这些能力。仓库文档说明当前源码；发布版本保留各自的 CLI 文案和行为。

v0.2.0 使用 Store schema 2，当前源码使用 schema 4。两个版本应使用独立管理根，Store 格式不会自动迁移。安装、升级、回滚和卸载的完整步骤见[安装管理指南](docs/zh-CN/how-to/manage-installation.md)。

## 快速开始

### 安装已发布引擎

在 Apple Silicon Mac 上执行：

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/download/v0.2.0/sheltie-cli-installer.sh | sh
export PATH="$HOME/.sheltie/bin:$PATH"
```

演示使用新建的临时管理根。后续命令在同一个 shell 中执行；`self version` 显示选定的管理根和二进制版本。

```bash
export SHELTIE_HOME="$(mktemp -d /private/tmp/sheltie-demo.XXXXXX)"
sheltie self version
git clone --depth 1 https://github.com/samuelj1519/sheltie.git
cd sheltie
sheltie workbook add examples/two-step
sheltie work start --workbook two-step --flow default --input 'topic=Introduce Sheltie to a newcomer'
```

### 运行两步方法

将 `<work>` 替换为返回的 `work_id` 或其唯一前缀，领取第一个任务：

```bash
sheltie attempt begin <work> --node outline
```

读取返回的 `brief_path`，把 `outline.md` 写到声明的输出位置，再提交并领取下一项任务：

```bash
sheltie attempt submit <work> --attempt outline#1.0 --summary 'Outline written'
sheltie attempt begin <work> --node summary
```

读取新任务书及其绑定的提纲，把 `summary.md` 写到声明的输出位置，再完成运行：

```bash
sheltie attempt submit <work> --attempt summary#1.0 --summary 'Summary follows the outline'
sheltie work status <work>
```

预期状态为 `succeeded`，合法下一步为空。提交检查文件合同，协调者仍需判断输出内容。使用当前源码时，[完整教程](docs/zh-CN/tutorials/first-work.md)提供逐步命令、输出示例和成果选择说明。

## 从 agent 使用 Sheltie

Claude Code 使用 `/sheltie` 调用协调者 skill。发布版本附带可独立使用的 skill 资产：

```bash
mkdir -p ~/.claude/skills
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/latest/download/sheltie-skill.tar.gz | tar xz -C ~/.claude/skills/
```

CLI 工作流程见当前[协调者指令](skills/sheltie/SKILL.zh-CN.md)。中文版指令和 Workbook 通过明确的语言链接与方法 ID 选择。引擎不负责安装或配置宿主 skill。

## 文档与贡献

- **学习使用：** [第一个 Work](docs/zh-CN/tutorials/first-work.md)、[门槛与成果](docs/zh-CN/tutorials/gate-and-result.md)。
- **执行实际任务：** [code-change](docs/zh-CN/how-to/run-code-change.md)、[spec-dev](docs/zh-CN/how-to/run-spec-dev.md)、[续接 Work](docs/zh-CN/how-to/resume-work.md)。
- **编写方法：** [编写 Workbook](docs/zh-CN/how-to/write-workbook.md)、[编辑 Workbook](docs/zh-CN/how-to/edit-workbook.md)。
- **查阅行为：** [CLI 参考](docs/zh-CN/reference/cli.md)、[规格](specs/README.md)、[领域词汇](CONTEXT.md)。
- **参与贡献或获取帮助：** [贡献指南](CONTRIBUTING.md)、[agent 指令](AGENTS.md)、[支持](SUPPORT.md)、[安全问题报告](SECURITY.md)、[行为准则](CODE_OF_CONDUCT.md)。

双语文档可从 [GitHub Wiki](https://github.com/samuelj1519/sheltie/wiki) 或上方仓库文档链接阅读。项目以英文为默认语言，保留中文使用文档和方法指令，并保留有实际用途的 Unicode 测试数据与历史原始证据。

## 许可

[MIT](LICENSE)。单独标注许可的第三方文件仍遵循各自的声明。
