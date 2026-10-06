# 发布双语 GitHub Wiki

[English](../../en/how-to/manage-wiki.md) | 简体中文

维护者按本指南将人用文档发布到 <https://github.com/samuelj1519/sheltie/wiki>。主仓库维护文档源文件，独立的 Wiki Git checkout 维护生成的展示文件。源码、规格和文档在同一次审查中更新，无需 submodule 或第二个文档编写仓库。

## 编写与审查

- 英文页面放在 `docs/en/`，对应中文页面放在 `docs/zh-CN/`。两个目录使用相同文件名和信息结构。
- 英文是默认权威。语义变化同时更新两个版本，保留要求、限制、证据状态和版本范围。
- 提供双向语言链接。引用权威英文规格，不重复定义合同。中文 Workbook 与 skill 指令保留在明确的方法和指令位置。
- 修改源文件并审查对应源码提交。Wiki 页面由工具生成；在准备替换内容前，先处理受管 Wiki 文件中的手工修改。

提交前运行：

```bash
scripts/check-docs.sh
scripts/check-specs.sh
python3 scripts/check-language.py
python3 scripts/wiki.py check
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -p test_wiki.py
```

`check` 允许检查未提交的源码，验证双语配对、页面名唯一性、相对链接、锚点和导出转换。代码块与行内代码保持原样。生成和验证发布产物要求源码已提交且工作区干净。CI 和 pre-commit 执行结构检查；这些检查不证明翻译质量或 agent 语言效果。

## 初始化与克隆 Wiki

前提：已审查的源码提交、Git、Python 3.9+、仓库 Wiki 写权限，以及明确的发布授权。首次发布时，先按下方命令生成并验证一个新快照，再通过 GitHub Wiki 界面创建 Home 页面，原样使用生成的 `Home.md` 内容。GitHub 随后提供独立的 `.wiki.git` 仓库。参见 [GitHub 官方说明](https://docs.github.com/en/communities/documenting-your-project-with-wikis/adding-or-editing-wiki-pages)。

在被忽略的 output 或其他明确选定目录中使用独立 checkout。下例创建新 checkout；后续使用已有的干净 checkout，并运行 `git pull --ff-only`。

```bash
git clone https://github.com/samuelj1519/sheltie.wiki.git output/github-wiki
```

沿用已有 Git 认证，无需新增应用、持久凭据或修改宿主配置。

## 生成、检查与发布

从主仓库根运行，每次选一个尚不存在的快照目录：

```bash
python3 scripts/wiki.py generate --output output/wiki-snapshot
python3 scripts/wiki.py verify --output output/wiki-snapshot
python3 scripts/wiki.py stage --output output/wiki-snapshot --checkout output/github-wiki
git -C output/github-wiki diff --cached --stat
git -C output/github-wiki diff --cached -- Home.md Zh-CN-Home.md _Sidebar.md
```

若已为初始化 Home 生成快照，继续使用已验证的目录，不要在已有目录上再次生成。

导出器使用英文 Home、独立中文 Home、由语言与完整路径生成的稳定页面名，以及共享侧栏。人用文档链接转换为 Wiki 页面链接，源码链接固定到实际源码提交，历史数据链接指向复制的 Wiki 资产。manifest 记录源码和生成文件的哈希。`verify` 从同一个干净 HEAD 重建完整预期产物；源码提交变化或产物被修改时拒绝通过。

`stage` 要求 Wiki checkout 干净、独立，且 origin 指向本项目的 Wiki。它仅准备生成路径，仅删除 manifest 管理的过时文件，保留其他文件，并拒绝受管内容被修改或新页面名与不同的非受管内容冲突的情况。它不提交或推送。

审查准备好的变更，并确认发布已获授权后运行：

```bash
git -C output/github-wiki commit -m "docs(wiki): synchronize bilingual documentation"
git -C output/github-wiki push origin HEAD
```

推送采用正常快进保护。若远端已变化，保留本地工作，检查并处理远端变化后再发布；日常 Wiki 更新不使用强制推送。在实际 Wiki 中核验英文首页、中文首页、教程、语言切换，以及源码和资产链接。

## 历史与恢复

主仓库源码提交和 Wiki 提交是两个身份，发布记录同时保留二者。原始档案仍在 Git 中，生成的 manifest 不替代原始验证证据；浅克隆缺少已记录的快照时，获取完整历史。

恢复较早的 Wiki 发布时，先检查对应 Git 提交，恢复需要的受管文件，再审查并发布一个正常新提交。历史文档快照不为当前产品提供验收资格。
