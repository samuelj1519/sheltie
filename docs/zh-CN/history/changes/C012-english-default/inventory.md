# 语言清单

[English](../../../../en/history/changes/C012-english-default/inventory.md) | 简体中文

本清单记录 `eec267b7b54b1e18e75578f7b5e269e7ddf73df8` 快照中的迁移结果；行号对应该快照。当前源代码字面量声明以 [language-exceptions.json](../../../../../scripts/language-exceptions.json) 为准，本历史清单不随源码行号变化。

基线为 206a944491acfa9fa4843ced06103427e13054aa。计数指已跟踪的基线文件，不包括二进制内容。原始默认分支历史有 230 个提交；所有本地引用（包括归档和 stash）达到 260 个提交，其中 256 个带有中文提交信息。没有可达的提交被签名。现有的发布和审查标签是轻量级提交引用。

| 内容 | 包含汉字文本的基线文件 | 决策 |
| --- | --- | --- |
| 根 README、词汇表、代理指令、变更日志 | 5 | 保留显式中文副本并翻译默认内容 |
| docs | 68 | 保留 .zh-CN.md 副本并翻译默认内容；保留原始历史证据 |
| specs | 18 | 保留 .zh-CN.md 副本并翻译默认内容；迁移治理解析器 |
| crates | 135 | 翻译注释和人类可读消息；保留有意义的 Unicode 数据 |
| scripts | 30 | 翻译注释/诊断信息并更新对语言敏感的消费者 |
| tools | 23 | 翻译编辑器 UI、注释和无害的测试数据 |
| examples | 19 | 保留中文方法并翻译默认内容，同时提供显式语言选择 |
| workbooks | 27 | 保留中文规范开发内容，并以新版本翻译默认方法 |
| skills | 1 | 保留中文指令并默认使用英文 |

## 数据与证据决策

- 保留汉字范围/名称规范化测试夹具：它们测试的是所采用的工作名称契约，而非英文散文。
- 当中文路径、键和 shell 元字符组合独立测试 Unicode、字面 argv、隔离或完整性行为时，保留它们。
- 保留字节限制测试夹具，其多字节大小是独立判定依据；翻译它们会改变测试条件。
- 翻译普通演示主题、摘要、无害内容夹具、断言和调试诊断信息，这些内容的语言没有行为目的。
- 保留原始历史日志、冻结快照文件树、哈希、平台排除项、豁免、失败和 not_run 结果。翻译解释性散文，但不重新分类结果。
- 在 ~/.sheltie 下没有授权的用户数据迁移。现有的 Work/Workbook 冻结副本和工件保持不变。
- 不会仅仅因为包含中文就删除任何具有能力承载的测试或原始证据。

最终的路径级残留清单将记录迁移后实际保留的汉字文本。双语翻译是维护内容，不是被禁止的残留。

## 保留的 Rust 输入和持久化文本

| 文件 | 当前行号 | 用途 |
| --- | --- | --- |
| `crates/sheltie-cli/tests/result_artifact.rs` | 31, 120 | Unicode 和 shell 敏感的 artifact 键通过 argv 和结果检索按字面传递。 |
| `crates/sheltie-cli/tests/language_default.rs` | 84, 95 | 独立的端到端证明：英文简报/状态保留原始中文用户输入字节和摘要，包括 emoji。 |
| `crates/sheltie-cli/tests/work.rs` | 158, 163 | 汉字 Work 名称空白规范化以及真实 CLI WorkId 往返。 |
| `crates/sheltie-core/src/ids.rs` | 406, 407, 408, 414, 421, 554, 563, 567, 568 | 汉字 WorkName 规范化、规范反序列化、WorkId 往返，以及 51 字节 UTF-8 拒绝边界。 |
| `crates/sheltie-core/src/text.rs` | 73, 74 | 两个汉字恰好是六个 UTF-8 字节：拒绝 limit5，接受 limit6。 |
| `crates/sheltie-core/src/flow/parse.rs` | 625 | 非 ASCII 输出路径拒绝，与 ASCII/全角/分隔符拒绝控制配对。 |
| `crates/sheltie-export/tests/export.rs` | 36 | 与原始 Unicode 用户内容进行字节精确的复制/来源比较，由已完成的 fixture 共享。 |
| `crates/sheltie-export/tests/crash.rs` | 125, 204 | 与原始 Unicode 用户内容进行字节精确的崩溃/导出比较，由已完成的 fixture 共享。 |
| `crates/sheltie-export/tests/source.rs` | 134, 156, 329 | Unicode 源 artifact 叶子接受、非 ASCII Workbook 版本拒绝，以及字面 shell 敏感的 Unicode 键。 |
| `crates/sheltie-export/tests/target.rs` | 110, 147 | Unicode 结果键和重复 artifact 文件名在导出的 manifest 映射中保持可区分。 |
| `crates/sheltie-export/src/output.rs` | 223, 234, 239 | Unicode 用户目标路径在结构化和可读的英文导出响应中保留。 |
| `crates/sheltie-export/tests/common/mod.rs` | 125 | 原始 Unicode 用户任务成为选定的冻结结果输入，并且必须在不翻译的情况下导出。 |
| `crates/sheltie-runtime/tests/crash.rs` | 18, 19, 43 | Unicode Work 名称和原始 23 字节主题出现在独立手写的完整状态卡字节 oracle 中。 |
| `crates/sheltie-runtime/tests/service.rs` | 30, 463, 1321, 1967, 2070, 2072, 2151, 2173, 2319, 2358, 2404, 2412, 2477 | 原始 23 字节 Unicode 主题和单字符同长度变异保留独立手写的摘要/大小、完整性/恢复和精确字节 oracle；精确的长度单位字面量断言持久化的审计标记。 |
| `crates/sheltie-runtime/tests/start_preflight.rs` | 94, 98 | 预检失败后的汉字 Work 名称规范化保留每日序列和精确规范化的 WorkId 后缀。 |
| `crates/sheltie-runtime/tests/result.rs` | 49 | 原始 Unicode 启动输入字节大小与结果元数据引用匹配。 |
| `crates/sheltie-runtime/src/service.rs` | 2119 | 精确的 <n 字节> 持久化 audit.command_json 标记；更改它会在没有持久化格式迁移的情况下破坏历史 Command/审计相等性。 |
| `crates/sheltie-runtime/src/selfmgmt.rs` | 964 | 有效 Unicode 资产文件名接受；仅拒绝不安全的路径段。 |
| `crates/sheltie-runtime/tests/common/mod.rs` | 183 | 共享的原始 23 字节 Unicode 主题支持独立手写的摘要/大小和同长度完整性变异 oracle。 |

精确的字面量/操作位于 [retained-rust-text.json](../../../../en/history/changes/C012-english-default/retained-rust-text.json)：19 个文件中共 52 行保留源代码。这些是 Unicode/用户数据/持久化 oracle，而非未翻译的注释。没有删除任何测试或原始证据。

## 保留的编辑器输入

`tools/workbook-editor/test/clear-graph.test.mjs` 保留输入名称 `中文_方案`、`计划_模板`、`计划_模板-2` 和 `中文新名称`，以测试节点本地 Unicode/下划线名称、别名冲突、重命名和未更改的引用字节。其他编辑器 fixture 文本/标签为英文。精确位置位于 [retained-editor-text.json](../../../../en/history/changes/C012-english-default/retained-editor-text.json)。

## 其他承载语言的字面量

- `.zh-CN.md` 文件、显式本地化的 `*-zh-CN` 方法树以及 `简体中文` 选择器是维护的翻译/导航。
- `specs/contracts/storage.md` 保留 `<n 字节>`，因为它是审计机器格式标记。
- `scripts/check-specs.sh` 保留三个中文字段模式，以验证未更改的原始完成快照。
- `scripts/check-docs.sh` 保留中文禁用词模式以及英文模式，以管理两种文档语言。
- 原始发布的提交字段、上游 GitHub URL、TSV 原始 ID 列以及历史源代码树是保留的证据/身份标识。
- 管理根目录下的现有用户数据未被改动；历史重放响应和产物保留其原始语言/字节。
