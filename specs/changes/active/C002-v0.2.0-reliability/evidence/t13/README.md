# C002-T13 证据

- Owner：Claude（mimo 会话）。基准 `c1a530e2a7acf89cff77c9c5c62bb48c42f64f67`（T12 提交），待提交树见提交说明。工作区起点 `git status --short` 干净。
- 输入闭包：`Cargo.lock` 未变（不跑 deny）；特性全开（`--all-features`）；平台 macOS aarch64、`rustc 1.98.1`。全部安装与校验发生在临时目录（自建源码树替身、模拟宿主 skill 目录、临时发布资产），不写真实 `~/.claude`、`~/.sheltie`。
- 改动：`scripts/skill-delivery.sh`（新，交付生成与校验）；`scripts/check-skill.sh`（补链接解析与交付自包含两条规则，加 `--delivery`）；`README.md` 装 skill 段改发布资产安装；`.github/workflows/release.yml` announce 补打包一步；`crates/sheltie-cli/tests/skill_delivery.rs`（新，八例隔离安装 fixture）；tasks.toml 白名单无需改（skills/scripts/crates/.github/README.md 都已列入）。

## 卡片第 1、2 条落在哪

| 约定 | 落点 | 由谁证明 |
| --- | --- | --- |
| 由仓库单一权威合同生成发布 references；不手工维护两份 | `scripts/skill-delivery.sh pack` 阶段 1：SKILL.md 指向 skill 目录之外的本地链接，从 `specs/contracts/` 收进交付 `references/<文件名>` 并改写链接。仓库里 `skills/sheltie/SKILL.md` 原样保留 `../../specs/contracts/` 链接——合同本身是唯一权威，没有第二份入库副本 | 测试 2 手写成员表核生成物；测试 3/4 的 `verify` 拿「重新生成一份」逐文件比对，缺文件与字节不同都失败；测试 8 用独立的 `strip_links` 归一后逐字比对权威与生成物，去链接不得吞正文 |
| 打包/安装后 references 是交付目录内可读取的文件；复制到任何目录仍可解析 | `pack` 阶段 2：交付内解析不到的本地链接去掉链接、只留文字；解析得到的（`references/protocol.md` ↔ `references/workbook.md`）保留。交付目录自包含，不指向仓库路径 | 测试 1 手写链接清单四条，逐条在交付内解析；安装副本里不含 `..` 或 `specs/` 目标 |
| README 给出与实际发布步骤一致的安装方式，不依赖用户保留源码路径 | release.yml 的 announce 在挂发布资产前跑 `scripts/skill-delivery.sh tar artifacts/sheltie-skill.tar.gz`；README 的安装命令从发布页下载同名资产解压进 `~/.claude/skills/` | 测试 2 逐字核对 README 下载地址、工作流资产名、打包入口三处一致；解包结果即安装副本 |
| 隔离临时目录安装，逐个解析本地链接、相对路径与引用命令 | 临时宿主 skills 目录里安装副本逐项解析：链接闭合、相对路径、`sheltie <group> <verb>` 命令对在交付协议 §2，再用真实 `sheltie` 二进制 `--help` 过一遍 | 测试 1 |
| 移动或移除源码树后重复检查 | 测试 1 先删掉源码树替身、再把安装目录整体改名移动，同一套解析检查重跑通过 | 测试 1 |
| 故意漏同步一份 reference 时 check-skill 必须失败 | `check-skill.sh --delivery <dir>` 走 `skill-delivery.sh verify`：链接闭合 → 命令白名单 → 与重新生成的一份逐文件比对 | 测试 3：只删 `references/workbook.md`，非零并点名该文件；相邻反例测试 4（内容过期）、5（链接逃逸）、6（生成不出）各改一个条件 |
| 不得触碰真实宿主配置 | 安装与校验全部在 `tempfile::tempdir()` 下；测试源里没有 `~/.claude`、`~/.sheltie`、`HOME` 写入 | 八例共同边界 |

## 交付形态与生成规则

发布资产 `sheltie-skill.tar.gz` 的根目录是 `sheltie/`，解进 `~/.claude/skills/` 得到 `~/.claude/skills/sheltie/`。交付文件恰三个：`SKILL.md`（两条链接改写成 `references/…`）与 `references/protocol.md`、`references/workbook.md`。

reference 不是字节副本，是确定性生成物：合同里指向交付外的本地链接（`../roadmap.md`、`../releases/v0.1.0/decisions.md`、`storage.md` 等）去掉链接、只留文字，交付内互相引用保留。动手前算过闭包——从两份合同递归收本地链接会拖进 150 余个文件（整棵 `specs/` 连 evidence 里的 .txt.gz 都被链接进来），交付不可能这么大；GF-18 又要求「复制到任何目录后全部引用仍可解析」，死链不合格。去链接是不引入死链的最小变换，链接文字原样保留。所以 `storage.md`、roadmap、decisions 这些被合同提到的文件不随包发布，交付里它们只是文字提法；要看内容回仓库 `specs/`。校验因此不是 `cmp` 权威原文，而是重新生成再逐文件比对：内容漂移、文件缺失、多出文件、链接没改写，都在这一步现形。正文保真另有一道独立核对（测试 8）：`strip_links` 归一后再逐字比，去链接变换吞掉正文也失败。

## 八例隔离安装 fixture

测试在 `crates/sheltie-cli/tests/skill_delivery.rs`，全部 `// Task: C002-T13`。链接解析、交付文件集合、发布资产成员表、协议 §2 命令白名单、链接归一都是本文件手写 oracle，不调用被测脚本的解析逻辑；脚本只作为被测对象运行。源码树替身按仓库同构复制 `scripts/`、`skills/`、`specs/contracts/`。

| 测试 | 唯一改变的条件 | 独立 oracle | 结果 |
| --- | --- | --- | --- |
| `installed_delivery_stays_self_contained_after_source_tree_is_removed` | 无（正例） | 交付文件集合恰为手写三条；链接清单恰为手写四条（SKILL.md→两份 references；references 互指），逐条在交付内解析、不含 `..` 或 `specs/`；命令逐条对上交付协议 §2 并经真实二进制 `--help`；随后删源码树、移动安装目录，重跑同套检查 | PASS |
| `generated_reference_keeps_authority_prose_verbatim` | 无（正例） | 手写 `strip_links` 把本地链接收成它的链接文字（圆括号目标整个丢掉），归一后权威合同与生成 reference 逐字相同——去链接变换不得吞正文（byte 比对太紧会把合法的链接改写也算差异，归一后比的是正文） | PASS |
| `release_tarball_matches_readme_install_shape` | 无（正例） | `tar tzf` 成员（去目录项）恰为手写三条；解包进模拟 skills 目录后同上解析；README 下载地址、release.yml 资产名与打包入口逐字一致 | PASS |
| `check_skill_fails_when_delivery_misses_one_reference` | 只删交付里的 `references/workbook.md` | `check-skill.sh --delivery` 非零，输出点名 `references/workbook.md` | PASS |
| `check_skill_fails_when_delivery_reference_is_stale` | 只给 `references/protocol.md` 追加一行 | 同上非零，点名 `references/protocol.md` | PASS |
| `check_skill_fails_when_delivery_link_escapes` | 只把交付 SKILL.md 的一条链接退回 `../../specs/contracts/protocol.md`（O11 的缺陷形态）；交付放在树内 `out/sheltie`，该路径在树内解析得到，失败点只剩越界 | 同上非零，报「越出交付目录」 | PASS |
| `check_skill_fails_when_reference_source_is_gone` | 只删源树替身里的 `specs/contracts/workbook.md`（生成不出那份 reference） | 默认 `check-skill.sh` 非零，点名 `workbook.md` | PASS |
| `check_skill_passes_on_repository_skill` | 无（正例） | 输出含 `check-skill: OK` | PASS |

测试 5 的条件选择说明：交付放在树外时，退回的仓库相对路径在临时目录里根本解析不到，会先报「解析不到」而不是「越出交付目录」；放进树内 `out/sheltie` 后该路径真实存在，失败原因才唯一是越界。

## 处置记

- release.yml 是 dist 生成文件，announce 里的打包一步是自定义步骤，注释写明重新 `dist init` 会丢掉、需照此补回。不改 `Cargo.toml` 的 dist 配置：skill 是平台无关物，挂独立资产比塞进四个平台包直接，且 Cargo.toml 不在本任务白名单。
- `skills/sheltie/SKILL.md` 一字未动，仓库内链接照旧指向 `specs/contracts/`（卡片「仓库内可保留链接」）。
- bash 里 `$t` 紧跟全角括号会被当成变量名的一部分（`$t（` → unbound variable），报错消息里改 `${t}`。
- 交付文件集合与链接清单是手写冻结形状：SKILL.md 或两份合同的引用面变化时这两张清单要跟着改，属预期维护点，不是测试过拟合。
- v0.1.0 的 runbook 里 `cp -R skills/sheltie ~/.claude/skills/` 是归档文档当时的步骤，不追改（specs/ 不在本任务白名单，且归档纪律）。
- `pack` 不再 `rm -rf` 输出目录：目标只准尚不存在或已空，空字符串、`/`、仓库根、skill 源目录一概拒绝。复验五种形状全拒，非空目录里的文件原样留下。`tar_pack` 也先对临时交付跑一遍 `verify` 再打包——挂出去的资产一定是过了校验的那份。
- `verify` 第 3 步比对的是「当前这棵树」重新生成的一份，所以 `--delivery` 只适合同一源码的交付（打包即校验、T16 核 rc 装出的副本）；旧版本交付对着新树报「不一致」是预期。两条边界（同源、storage.md 类文件不随包发布）写进了两个脚本的头注释。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；433 passed、0 skipped（1 leaky 为 `scenario_article_review max_visits_exhaustion_blocks_with_no_legal_edge` 的计时漂移，与 T12 同类，不影响结果；见 nextest.txt 唯一一条 LEAK） | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh` | 0 | [docs.txt](docs.txt) |
| `scripts/check-specs.sh` | 0 | [specs.txt](specs.txt) |
| `scripts/check-tests.sh` | 0；433 个测试、任务卡 226 条 | [tests.txt](tests.txt) |
| `scripts/check-core-vocab.sh` | 0 | [core-vocab.txt](core-vocab.txt) |
| `scripts/check-skill.sh` | 0 | [skill.txt](skill.txt) |
| `git diff --check` | 0 | [diff-check.txt](diff-check.txt) |
| `scripts/check-task.sh C002-T13 --staged` | 0 | [check-task.txt](check-task.txt) |

## 独立审查

Reviewer：未参与本任务修改的通用 agent（独立会话，只审不改）。

首轮结论「需修改」：机制与实现无硬伤（引擎零改动、白名单不越界、check-skill 旧规则仍咬、发布链三处同名一致、去链接只动链接行——protocol.md 26 行、workbook.md 6 行），问题在证据纪律与两处防护缺口：

| 问题 | 处置 |
| --- | --- |
| 必改：门禁表预填 `check-task` 退出码 0、链向并不存在的 `check-task.txt`，实际命令当时退出 1（plan.md 状态未翻）；progress.md 的「已提交」与此矛盾 | plan.md 本任务状态在本提交里翻成 `done`；check-task 真跑、原始输出存 [check-task.txt](check-task.txt)，门禁表只写真实结果（T11 先例）；plan.md 与任务提交同批入库，矛盾消除 |
| 可选：正文保真缺独立核对——字节比对被合法的链接改写卡住，去链接吞掉正文无从发现 | 加手写 `strip_links` 归一后逐字比对（测试 8），权威与生成物正文必须相同 |
| 可选：`--delivery` 拿「当前树」的生成物比对，适用范围没写明 | 两个脚本头注释写明：只适合同一源码的交付（打包即校验、T16 核 rc 副本）；旧版本交付对着新树报「不一致」是预期 |
| 可选：`storage.md` 等被去链接的文件不随包发布，没写明 | 脚本头注释与「交付形态与生成规则」段写明：不随包发布，内容回仓库 `specs/` |
| 可选：`pack` 的 `rm -rf "$out"` 有误删风险；`tar` 前没校验 | `pack` 目标只准尚不存在或已空，拒绝空串、`/`、仓库根、skill 源目录，全程不再 `rm -rf`；`tar_pack` 先对临时交付跑 `verify` 再打包。五种拒绝形状手工复验，非空目录里的文件原样留下 |
| 可选：证据措辞「不含任何 `..`」与断言 `!t.starts_with("..")` 不一致 | 断言加强为 `!t.contains("..")`，措辞对齐为「不含 `..`」 |

复审二轮「需修改」：上表六项处置核实闭合五项半（check-task.txt 真跑、strip_links oracle 独立有牙齿、pack 拒绝面五种实测且正常路径无回归、两条边界已写明），仅剩门禁表把 leaky 归因第 8 例，而 [nextest.txt](nextest.txt) 唯一一条 LEAK 是 `scenario_article_review max_visits_exhaustion_blocks_with_no_legal_edge`——与首轮必改同类（证据只准写真实结果）。按处方改为实际那条并与原始输出逐字对齐。

复审三轮「通过」：leaky 归因收口无误（`grep -n LEAK` 仅第 23 行、与点名逐字一致），`check-task.txt` 仍是真实运行，全改动在暂存区无残渣，无旧归因残留。前两轮核过的闭合项未回退：任务卡两条与 GF-18 全落实、八例反例各改一条件、发布链三处同名同形、check-skill 旧规则仍咬、引擎零改动、改动全在 T13 白名单内。

## 覆盖与边界

- 关闭：卡片第 1、2 条的全部要求——生成 references、交付自包含、README 与发布步骤一致、隔离安装逐项解析、移除源码树后复查、漏同步让 check-skill 失败、不碰真实宿主配置。
- 不含：真实发布挂资产与下载实测归 T17（本地已核打包命令、资产名与工作流步骤一致，工作流本身要在 GitHub 上跑才算数）；真实宿主里 skill 的安装、触发与协调者行为归 T16。README 的下载地址在 v0.2.0 发布前指向 latest 的旧发布（v0.1.0 没有这个资产），T17 发布后即正确——发布类文档先于发布就位是本仓库的常态，这里如实记下窗口。
