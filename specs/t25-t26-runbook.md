# T25、T26 执行手册

状态：`archived`。T25、T26 已完成；本文只保留 v0.1.0 的发布与真实宿主执行方法，不作为当前步骤。发布闭包见 [v0.1.0 release record](releases/v0.1.0.md)。

给执行 [plan.md](plan.md) 最后两个任务的人。两个任务都是人工任务：T25 把仓库变成可安装、可升级的 v0.1.0 发布；T26 在真实 Claude Code 里用 skill 走完一次 Work 并留记录。本手册自包含，按顺序做即可；每步写了预期结果，与预期不符就停下来对照「风险与对策」。

写作时核实过的事实基线（执行前请重跑 §1 确认）：

- 代码全绿：315 条测试、`cargo deny`、四个检查脚本全部通过。零 `todo!()`、零 `#[ignore`。
- `main` 落后工作分支 5 个提交；仓库没有任何 git remote。
- 本机 `gh` 已装但未登录；`git-cliff` 已装；`cargo-dist` 未装。
- 两个测试把版本号写死成 `0.1.0`（见 §3.1），不修就发不出预发布。
- `.github/workflows/build.yml` 的 `release` job 与 cargo-dist 的 `release.yml` 都会建 GitHub Release，同 tag 下必然打架（见 §3.2）。
- `dist-workspace.toml` 没写 `install-path`，cargo-dist 默认装进 `~/.cargo/bin`，与 README、`self update` 假设的 `~/.sheltie/bin` 不符（见 §3.3）。
- `release.yml` 是 T20 手写的，从没真正跑过；rc 发版就是它的试跑。
- 执行中补充的基线（§2、§3 实测发现）：仓库实际建在 `samuelj1519/sheltie`，而 `Cargo.toml`、`cliff.toml`、`selfmgmt.rs` 写死的是 `Samuel-J/sheltie`（见 §3.0）；cargo-dist 0.32.0 的二进制叫 `dist`，手写 release.yml 里的 `cargo dist …` 调用在 CI 会报「no such command」；T20 手写的 `dist-workspace.toml` 是 0.32.0 不认的格式（已由 §3.3 挪进 `Cargo.toml` 的 `[workspace.metadata.dist]`）；cargo-dist 以 package 名 `sheltie-cli` 命名产物与安装器（不是 `sheltie`）。

## 1. 开工前确认

```bash
git status                        # 干净
git log --oneline -3              # 知道自己在哪个提交上
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass    # 315 passed
cargo deny check
scripts/check-docs.sh && scripts/check-core-vocab.sh && scripts/check-skill.sh && scripts/check-tests.sh
```

任何一条不绿，先解决再走后面；不要在带病的基础上发版。

## 2. 准备（一次性）

1. **登录 GitHub**：`gh auth login`（交互式，在 Claude Code 会话里可以用 `! gh auth login` 让输出直接回来）。
2. **装 cargo-dist 0.32.0**（与 `dist-workspace.toml` 的 `cargo-dist-version` 一致；已按 crates.io 核实 0.32.0 就是最新发布版）：

   ```bash
   cargo install cargo-dist --version 0.32.0 --locked
   ```

3. **把 main 快进到工作分支**，之后所有提交与 tag 都在 `main` 上做：

   ```bash
   git checkout main && git merge --ff-only kimi
   ```

   注意：cargo-dist 0.32.0 的二进制叫 `dist`（不是 `cargo-dist`），命令写作 `dist plan`、`dist generate`、`dist build`；`cargo dist …` 报「no such command」是正常的。

4. **建远程仓库并推送**（已执行）。实际仓库是 `samuelj1519/sheltie`（`gh` 登录账号），与 `Cargo.toml`、`cliff.toml`、`selfmgmt.rs` 里写死的 `Samuel-J/sheltie` 不一致——这三处由 C0 修正。install.sh 要用无认证的 `curl` 下载，仓库必须公开：

   ```bash
   gh repo create samuelj1519/sheltie --public --source . --remote origin
   git push -u origin main
   ```

   本地那一串 `tNN-*` tag 是任务工具的内部基准，不必推。推送时只推 main 与后面创建的 `v*` tag。

## 3. 发布前修正（提交 C0、C1、C2、C3）

四个修正各自一个提交，trailer 都写 `Task: T25` 与 `Agent: <你的名字>`。修完一个跑一遍 §1 的门禁再提下一个。

### 3.0 C0：仓库地址统一为 samuelj1519/sheltie

**为什么。** §2 建仓库时实际账号是 `samuelj1519`，而 `Cargo.toml` 的 `repository`、`cliff.toml` 的 `$REPO` 后处理器、`selfmgmt.rs` 的默认发布源三处写死的是 `Samuel-J/sheltie`。不改的话，`self update` 默认去一个不存在的仓库读清单，CHANGELOG 与 crate 元数据的链接也全错。

**改法。** 三处 `github.com/Samuel-J/sheltie` 全部换成 `github.com/samuelj1519/sheltie`。没有任何测试断言这个地址；改完跑 §1 门禁。

提交：`fix(release): 发布源与仓库地址统一为 samuelj1519/sheltie`。

### 3.1 C1：两条测试不再写死版本号

**为什么。** cargo-dist 要求 git tag 的版本与 `Cargo.toml` 完全一致（预发布后缀也算）：tag `v0.1.0-rc` 只认 `version = "0.1.0-rc"`。而下面两条测试断言字面的 `0.1.0`，版本一变成 `0.1.0-rc` 就红，rc 发不出来。断言的本意是「输出的版本等于 crate 版本」，写死字面量是骨架的疏忽：

- `crates/sheltie-cli/tests/version.rs:14`：
  `.stdout("sheltie 0.1.0\n")` → `.stdout(format!("sheltie {}\n", env!("CARGO_PKG_VERSION")))`
- `crates/sheltie-cli/tests/self_cmd.rs:14`：
  `assert_eq!(v["data"]["version"], "0.1.0")` → `assert_eq!(v["data"]["version"], env!("CARGO_PKG_VERSION"))`

**顺手登记白名单。** `scripts/check-task.sh` 要求 `tasks.toml` 里有任务条目，T25、T26 还没有。在 `tasks.toml` 末尾追加（数组必须一行写完，脚本是按行解析的）：

```toml
[T25]
files = ["README.md", "CHANGELOG.md", "cliff.toml", "Cargo.toml", "Cargo.lock", "dist-workspace.toml", ".github/workflows", "crates/sheltie-cli/tests/version.rs", "crates/sheltie-cli/tests/self_cmd.rs", "crates/sheltie-runtime/src/selfmgmt.rs", "crates/sheltie-runtime/tests/selfmgmt.rs", "specs"]
test_files = []
allow_test_changes = false

[T26]
files = ["specs"]
test_files = []
allow_test_changes = false
```

并在 plan.md 的 T25、T26 两张任务卡各补一行「文件。」，内容与上面一致（plan.md 与 tasks.toml 同源）。`selfmgmt.rs` 两处只在 §4 发现真实清单不适配时才会用到，先登记着。

提交：`fix(cli): 版本断言跟随 CARGO_PKG_VERSION，不再写死 0.1.0`，正文写清「不改它，任何预发布版本号都会让这两条测试红」。

### 3.2 C2：删掉 build.yml 的 release job

**为什么。** `build.yml` 的 `release` job（git-cliff 拼说明 + `softprops/action-gh-release` 建 Release）是 cargo-dist 之前（D-22）留下的。T20 加了 `release.yml` 之后，同一个 tag 推送会有两个工作流抢着 `gh release create`：build.yml 的链路短，几乎必然先建成一个**没有任何产物**的 Release；cargo-dist 的 `announce` 步骤后跑，`gh release create` 撞已存在的同名 Release 直接失败，四平台包传不上去。发布链以存储合同 §9 的 cargo-dist 为准。

**改法。** 删 `.github/workflows/build.yml` 末尾整个 `release:` job（从 `  release:` 到文件尾）。其余不动。

提交：`fix(ci): 删去 build.yml 的 release job，发布只走 cargo-dist`。

### 3.3 C3：install-path 定为 ~/.sheltie/bin，dist 配置挪进 Cargo.toml 并再生 release.yml

**为什么。** 执行时发现 T20 手写的 `dist-workspace.toml` 真机不可用：0.32.0 要求 workspace 清单格式（`[workspace]` 加 `cargo:` 前缀的 members），且 `github-release = true`、`create-release = true` 两个键不符合 schema（`github-release` 要字符串）。手写 release.yml 里的 `cargo dist …` 调用同样不可用——0.32.0 的二进制只叫 `dist`。另外 cargo-dist 的 shell 安装器默认装到 `~/.cargo/bin`；README 快速开始与 `self update`/`self rollback`（存储合同 §9）管理的是 `~/.sheltie/bin/sheltie`，不加 `install-path` 的话 install.sh 装一份、`self update` 更新另一份，用户 PATH 里的永远是旧版。

**改法（已按实际执行修正）。**

1. 删掉 `dist-workspace.toml`，配置挪进根 `Cargo.toml` 的 `[workspace.metadata.dist]`（engineering.md §2.1 同步改）：`installers = ["shell"]`、`install-path = "~/.sheltie/bin"`、四个 target、`ci = ["github"]`、`pr-run-mode = "plan"`、`github-release = "announce"`、`cargo-dist-version = "0.32.0"`。`rust-toolchain-version` 是废弃键（0.32.0 提示用 rust-toolchain.toml），不写。另加 `[profile.dist] inherits = "release", lto = "thin"`——`dist build` 固定用 `--profile dist` 构建，T20 的手写配置没有这个 profile，CI 构建会同样失败。
2. 顺手让 git-cliff 永久忽略预发布 tag（否则将来重新生成 CHANGELOG 时 rc 会单独成节）。`cliff.toml` 里 `ignore_tags = ""` 改为：

   ```toml
   ignore_tags = "v[0-9]+\\.[0-9]+\\.[0-9]+-"
   ```

3. 再生发布工作流并审 diff（T20 任务卡本来就安排「T25 用 `dist generate` 校对再生」）：

   ```bash
   dist generate --mode=ci
   git diff .github/workflows/release.yml   # 逐行看
   ```

   生成版与手写版差异很大是正常的：调用全部从 `cargo dist` 改为 `dist`、补了 rustup 安装与 dist 缓存。以生成版为准整体接受。之后 `dist plan` 必须能跑通并列出四个 target。
4. **注意应用名。** cargo-dist 以 package 名 `sheltie-cli` 为应用名：产物叫 `sheltie-cli-<target>.tar.xz`、安装器叫 `sheltie-cli-installer.sh`。README 的下载 URL 与 §4 的解包核对都按这个名字来；crate 不改名。

提交：`chore(release): dist 配置挪进 Cargo.toml、定 install-path 并再生 release.yml`。

## 4. 核对 `self update` 的清单适配（D-30 遗留，发 rc 之前做完）

`selfmgmt.rs` 的 `adapt_cargo_dist_manifest` 是按**设想**的 cargo-dist 完整清单格式写的，自动化测试用的夹具也是设想格式，真实字段从没验证过。适配层错了的症状是 `self update` 报 `UPDATE_UNAVAILABLE`。注意因果方向：**执行升级的是旧二进制**，v0.1.0 里修适配层救不了已经装出去的 rc——所以这一步放在 rc 之前，用一个 tag 都不打的方式验证。

**执行记录（已发生）。** 三类不符全部命中：`artifacts` 是按产物名索引的对象（适配器按数组读）；真哈希在 `checksums.sha256`，`checksum` 字段是同名的校验文件名（适配器会把文件名当哈希，必然摘要不符）；包内布局是 `<产物名去掉扩展>/sheltie`（适配器找 `sheltie/bin/sheltie`）。已修适配层与两条测试的夹具（`fix(runtime)` 提交），存储合同 §9 第 2 步同步改写，并用 `dist build` 的真实产物本地走通 install → update（0.0.0 → 0.1.0）。另外两处 T20 手写配置的硬伤也由 §3.3 修掉：`dist build` 需要的 `[profile.dist]` 缺失、cargo-dist 二进制叫 `dist` 而不是 `cargo dist`。

1. **拿真实格式的清单**（两个来源，都做更好）：
   - cargo-dist 给自己的发布就挂着完整清单：`curl -fsSL https://github.com/axodotdev/cargo-dist/releases/latest/download/dist-manifest.json -o /tmp/real-dist-manifest.json`。
   - 本地真构建：仓库根跑 `dist build`，产物与 `dist-manifest.json` 落在 `target/distrib/`（本机只能构建当前平台的 target，够用来验证格式）。
2. **逐字段对照适配器假设**（`selfmgmt.rs` 的 `adapt_cargo_dist_manifest`）：`announcement_tag` 是 `"v…"` 形；`artifacts`（或 `assets`）是**数组**；其中 `kind == "executable-zip"` 的项有 `name`、`target_triples`（取第一个）、以及 checksum——三种写法之一：对象 `checksum.sha256`、裸串 `"checksum": "sha256:…"`、字段 `checksum_sha256`。用 `jq` 逐条核：

   ```bash
   jq '.announcement_tag' /tmp/real-dist-manifest.json
   jq '.artifacts | type' /tmp/real-dist-manifest.json          # 适配器要求 "array"
   jq '[.artifacts[] | select(.kind=="executable-zip") | {name, target_triples, checksum, checksum_sha256}] | .[0:2]' /tmp/real-dist-manifest.json
   ```

   已知的高危出入：`artifacts` 其实是按 id 索引的对象而不是数组；checksum 给的是**校验和文件名**（`….tar.gz.sha256`）而不是哈希值本身。任一命中，适配层都得改（见第 4 条）。
3. **本地端到端预演。** 不联网、不打 tag，用真实清单与真实包把「读清单 → 选资产 → 核摘要 → 解包 → 替换」完整走一遍：

   ```bash
   # 前提：第 1 条已在本地 dist build 出 target/distrib/
   # 把 workspace 版本临时改成 0.0.0 构建一个「更旧」的二进制（不改过不了 up_to_date 短路）
   #   改 Cargo.toml version = "0.0.0" → cargo build -p sheltie-cli → 用构建出的 sheltie
   SHELTIE_HOME=$(mktemp -d) <构建出的sheltie> self install
   SHELTIE_HOME=<同一个临时目录> SHELTIE_RELEASE_BASE=target/distrib <构建出的sheltie> self update
   # 预期：从 0.0.0 升到 dist 清单里的版本；self version 确认
   ```

   预演用的版本改动**不要提交**，用完 `git checkout -- Cargo.toml Cargo.lock`。（`dist build` 的清单版本取自 Cargo.toml 当前的 `0.1.0`，与「更旧」的 `0.0.0` 二进制正好构成升级。）
4. **对不上就修，在 rc 之前修。** 改 `crates/sheltie-runtime/src/selfmgmt.rs` 的适配层（白名单已含），保持瘦格式与既有夹具格式继续可用（存量测试必须全绿）；把拿到的真实清单（可裁掉无关平台的资产项）作为新夹具补进 `crates/sheltie-runtime/tests/selfmgmt.rs`。提交 `fix(runtime): 适配 cargo-dist 真实清单的 <具体字段>`，`Task: T25`。
5. **rc 发布后只做一次确认性抽查**：下载 rc 的 `dist-manifest.json`，重复第 2 条的 jq。CI 构建的清单与本地同 schema，差异应只有版本号与摘要值；有出入就修适配层、发下一个 rc（§5）再核。

## 5. C4：发 rc，试跑发布链

前提：§4 的适配核对已通过（或修复后通过）。

```bash
# Cargo.toml 的 [workspace.package] version 改为 "0.1.0-rc"，然后让 lock 跟上
cargo check -p sheltie-cli    # 会更新 Cargo.lock
# §1 门禁全绿后：
git add Cargo.toml Cargo.lock
git commit -m "chore(version): 0.1.0-rc

Task: T25
Agent: <名字>"
git tag v0.1.0-rc
git push origin main v0.1.0-rc
```

tag 推送同时触发 `build.yml`（docs + 两平台门禁）与 `release.yml`（plan → 四平台构建 → 全局产物 → host → announce）。预期：

- `gh run list --branch v0.1.0-rc` 两个工作流都绿。四平台构建大约 15–40 分钟。
- `gh release view v0.1.0-rc`：标了 **Pre-release**，资产里有四个 `sheltie-cli-<target>.tar.xz`、`sheltie-cli-installer.sh`、`dist-manifest.json` 及配套 `.sha256`（cargo-dist 以 package 名 `sheltie-cli` 命名产物）。
- 下载安装器核对安装路径：`curl -fsSL https://github.com/samuelj1519/sheltie/releases/download/v0.1.0-rc/sheltie-cli-installer.sh | grep -m2 '.sheltie/bin'` 能命中。
- 按 §4 第 5 条抽查真实 `dist-manifest.json`。

release.yml 是首跑，挂了就看日志修（白名单含 `.github/workflows`），修复提交写 `Task: T25`，重发新 rc（版本 `0.1.0-rc.2`，tag `v0.1.0-rc.2`，以此类推；`release.yml` 的 tag 模式与 cargo-dist 都接受带后缀的预发布）。

## 6. 快速开始实测（「新人上手」场景）

这是规格 §7 最后一行的验收。实测者是从未接触过项目的人，或一个新开的 agent 会话；只能按 README 的文字操作。先改 README，再实测；卡住就改文档再测，直到一次走通。

1. **环境。** 干净的 `~/.sheltie`：`[ -e ~/.sheltie ] && mv ~/.sheltie ~/.sheltie.bak`。实测期间不要设 `SHELTIE_HOME`——install.sh 不认识它，它把二进制装进字面意义的 `~/.sheltie/bin`，升级测试也要对这份安装做。
2. **先修 README 的已知硬伤**（一个 `docs(readme):` 提交，`Task: T25`）：
   - `<owner>` 换成 `samuelj1519`。
   - 删掉「仓库当前是脚手架，尚未实现规格」与「目标形态，随 MVP 落地生效」两处措辞——发布后它们不再成立。
   - 补取 Workbook 的一步：install.sh 装的是引擎，不带样例。快速开始需要 `git clone --depth 1 https://github.com/samuelj1519/sheltie.git`，然后 `sheltie workbook add sheltie/examples/two-step`。
   - 把流程补完整：现状停在 `work status`，任务卡要求**走通** two-step（到 `succeeded`）。补 `attempt begin` →（读者自己当工作 agent 写 `outline.md`）→ `attempt submit` → `attempt begin summary` → 写 `summary.md` → `submit` → 状态变 `succeeded` 的完整命令序列。注意 attempt id 形如 `outline#1.0`；`work status` 的例子别写死日期前缀，用 `<work_id 前缀>` 一类占位。
   - 改完跑 `scripts/check-docs.sh`（断链与禁用词会拦）。
3. **实测。** 用 rc 的安装器走完整快速开始。只有 rc 存在时 `releases/latest/download/` 可能指不到它，rc 阶段用钉死版本的 URL：

   ```bash
   curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/download/v0.1.0-rc/sheltie-cli-installer.sh | sh
   export PATH="$HOME/.sheltie/bin:$PATH"
   # 之后逐字按改好的 README 走，直到 work status 显示 succeeded
   ```

4. **记录。** 每一步记下：实测者、日期、卡在哪里、改了什么。卡点修复是额外的 `docs(readme):` 提交（`Task: T25`），修完从头再测，直到有一次全程无卡。

## 7. C5：正式发布 v0.1.0

1. 生成 CHANGELOG 并升版本（同一个提交，tag 会指在它上面，CHANGELOG 因此进发布件）：

   ```bash
   # Cargo.toml version 改为 "0.1.0"，cargo check -p sheltie-cli 让 lock 跟上
   git cliff --tag v0.1.0 --output CHANGELOG.md
   ```

   §3.3 已在 cliff.toml 里设了 `ignore_tags`，生成结果应该只有 `## [0.1.0]` 一个版本节、不带 rc 节；不对就检查那行配置。生成与打 tag 同一天做，日期即发布日期。

2. 提交 `chore(version): 0.1.0`（cliff 配置会把这个提交本身排除在变更日志外），含 `Cargo.toml`、`Cargo.lock`、`CHANGELOG.md`。trailer `Task: T25`、`Agent:`。

3. 打 tag 并推送：

   ```bash
   git tag v0.1.0 && git push origin main v0.1.0
   ```

4. 等 release 工作流跑完，核对 `gh release view v0.1.0`：**不是** prerelease；四个平台包、`sheltie-cli-installer.sh`、`dist-manifest.json` 都在。

## 8. 真实网络升级（v0.1.0-rc → v0.1.0）

对 §6 装好的 rc 二进制执行（它就是普通用户的升级路径）：

```bash
sheltie self version     # 预期 0.1.0-rc
sheltie self update      # 预期：已从 0.1.0-rc 升到 0.1.0
sheltie self version     # 预期 0.1.0
sheltie self rollback    # 预期：换回 0.1.0-rc
sheltie self update      # 再回到 0.1.0，留在正式版
```

逐字留存输出，摘要写进 §9 的提交信息。这一步顺带完成了 §6 欠的一刀：`releases/latest/download/` 现在指向 v0.1.0，可以按 README 的 `latest` URL 再验一次安装器可达。

**如果升级失败**：错误码是 `UPDATE_UNAVAILABLE`（清单/下载问题）还是 `UPDATE_CHECKSUM_MISMATCH`（摘要问题）决定修哪。修完后执行升级的仍然是旧适配层，只能再发一个新版本（`v0.1.0-rc.n+1` 或 `v0.1.1`）来验证；v0.1.0 若带坏适配层出厂，用 `gh release delete` 撤掉再发，不要把坏版本留在 `latest` 上。

## 9. C6：T25 收口提交

最后一步，一个提交：

- `README.md` 的最终状态（§6 的修正如果已经单独提交，这里可能没有 README 改动）。
- `specs/plan.md`：T25 状态改 `done`。
- 提交信息用任务卡指定的第一行 `docs(specs): 从 install.sh 实测快速开始并发布 v0.1.0`，正文写：
  1. 实测者、日期、卡点清单与对应的文档修复（§6 的记录）。
  2. rc 发版与 v0.1.0 发版的结果（工作流运行链接、产物清单）。
  3. §8 升级链的逐字输出摘要。
  4. **完成判据 5 要求的依赖树**：`cargo tree -p sheltie-core` 的完整输出原文贴进正文（不带 `-e normal` 也行，能看清全部传递依赖即可；预期只有 camino、serde、serde_json、sha2、thiserror、toml 及其传递依赖，无 I/O crate）。
- trailer：`Task: T25`、`Agent:`。
- 提交后：`scripts/check-task.sh T25` 必须 OK（基准是 `t20-review`，它会把 C1 到 C6 全部 `Task: T25` 提交的改动并集对照白名单）。

## 10. T26：真实宿主实测

T25 完成后做。目标：在真实 Claude Code 里，协调者只靠 skill 与 CLI，用 `article-review` 样例走完一次**含打回**的 Work。产生一次文档提交，不产生代码提交；发现的任何问题按 [engineering.md](engineering.md) §7 路由，不修在 T26 里。

1. **装 skill**（这是人的动作，不是引擎安装东西，`INV-3` 不涉）：

   ```bash
   mkdir -p ~/.claude/skills
   cp -R skills/sheltie ~/.claude/skills/
   cat ~/.claude/skills/sheltie/SKILL.md   # 确认在位
   ```

2. **开一个新的 Claude Code 会话**（让 skill 被发现），确认 `which sheltie` 指向 `~/.sheltie/bin/sheltie` 且是 v0.1.0。输入 `/sheltie`，要求按 `article-review` 开 Work，起始输入 `topic` 自选。
3. **保证一次打回。** 审查一次通过也是合法结果，但本任务要观察 `back` 边。稳妥做法：作为用户在开场告诉协调者「第一稿按从严标准审」，协调者可以按协议 §4 在任务书后追加上下文——这不违反任何规则。观察协调者是否读懂 `review.md` 第一行的结论并选了 `back` 边、`draft#2` 是否出现、打回后任务书「来自」行是否正确。
4. **人审节点。** `publish` 是 `executor = "human"`：协调者应把任务书交给你，由你自己写 `final.md` 并跑任务书末尾的 `sheltie attempt submit …`。
5. **观测清单**（记录进 decisions.md）：
   - 协调者是否只用了 `next` 里的命令（每次写操作的响应都带 `next`；对照会话记录逐条核）。
   - 有没有试图绕过：直接改 `~/.sheltie` 下的文件、跳过 submit、被拒后换路径重试等。
   - 任务书是否够用：工作 agent 有没有需要任务书之外的信息才能完成步骤。
   - token 用量的宿主观测值：会话前后各记一次 `/cost`（或宿主等效读数），差值即本次 Work 的协调开销；记的是宿主观测，不是模型自报（GF-24 的口径）。
6. **落档。** 把记录写进 [decisions.md](decisions.md) 末节「首次真实运行」，替换现在的占位行。写明：日期、宿主与版本、Workbook 与 `work_id`、打回是否发生、上面四条观测、发现的问题及路由去向（没路由走的就写「无」）。`specs/plan.md` 的 T26 状态改 `done`。
7. **提交。** `docs(specs): 记录首次真实运行`，trailer `Task: T26`、`Agent:`。提交后 `scripts/check-task.sh T26` 必须 OK。
8. 本手册（`specs/t25-t26-runbook.md`）留去由你定：它是过程文档，规格库只留规范，可以在 T26 提交里一并删掉（`specs/` 在白名单里），历史里仍能查到。

## 11. 收尾：MVP 完成判据总核对

T26 提交后逐条核 [plan.md](plan.md) §3：

1. `grep -E '^\| (T|M)[0-9]+ \|' specs/plan.md | grep -v done` 应为空——T01 到 T25 与 M1 到 M3 全 `done`，每个任务有对应提交。
2. 规格 §7 十三个场景各有通过中的自动化测试（M3 报告已逐行对上），「新人上手」有 §6 的实测记录，T26 有 decisions.md 的记录。
3. `grep -rn 'todo!()' crates --include='*.rs'` 与 `grep -rn '#\[ignore' crates --include='*.rs'` 均为零命中。
4. `check-docs.sh`、`check-core-vocab.sh`、`check-skill.sh` 在 CI 的 `docs` job 里且绿——看 v0.1.0 tag 上的那次 `build.yml` 运行。
5. `cargo tree -p sheltie-core` 全文已贴进 C6 的提交信息。

全绿即 MVP 完成。

## 风险与对策速查

| 风险 | 症状 | 对策 |
| --- | --- | --- |
| 版本断言写死 | 改 `0.1.0-rc` 后两条测试红 | §3.1 已修；确认没有第三处（`grep -rn "0\.1\.0" crates --include='*.rs'`） |
| 双 Release 打架 | cargo-dist announce 报 release 已存在 | §3.2 已删 build.yml 的 release job |
| 装错目录 | install.sh 装进 `~/.cargo/bin` | §3.3 的 install-path；§5 用 grep 核对安装器 |
| 清单格式不符 | `self update` 报 `UPDATE_UNAVAILABLE` | §4 先 jq 对照再本地预演；修复赶在第一个 rc 之前 |
| release.yml 首跑失败 | rc 的工作流红 | 看日志修，重发 rc.n+1；这正是 rc 存在的意义 |
| rc 阶段 `latest` URL 不可用 | curl 404 | rc 阶段用钉死版本的下载 URL（§6）；v0.1.0 发布后 latest 才指它 |
| 适配层随 v0.1.0 出厂即坏 | 用户永远升不了级 | §4 的本地预演就是在防这个；真发生了撤 Release 重发（§8） |
