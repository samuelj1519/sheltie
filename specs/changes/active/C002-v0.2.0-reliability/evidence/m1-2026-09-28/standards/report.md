# Standards：需修改

候选 `e1a8126a432981df40023628dd06feec7884d458`，基线 `a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。只读审查，未修改仓库，未运行 cargo。临时 probe 使用根 Reviewer 已构建的 CLI/rlib。

## Findings

1. **P1，封存仍重新按路径 chmod。** `crates/sheltie-runtime/src/fsx.rs:153`，真实 caller `effects.rs:155–165`。`SafeFile` 已观察文件后，rename 原对象并在原路径放指向根外哨兵的软链，`set_readonly()` 返回 `Ok`，根外哨兵 `0600→0444`，原观察对象仍 `0644`。违反 storage §4.3、T04 同句柄封存和 INV-3；现有句柄替换测试只验证再次读取，不测试封存。应使用安全的句柄权限 API；不能以 macOS 注释理由降级合同。

2. **P1，self 的部分 caller 绕过受限祖先路径。** `selfmgmt.rs:128` install 的 `tmp` 与 `:250–272` rollback 的 `bin/tmp` 未做 T04 边界核验。真实 CLI：`tmp` 指向根外时 install 成功且根外出现 UUID staging；`bin` 指向根外时 rollback 成功并删除外部当前 binary、将外部 previous 重命名。违反 T15.1“所有 managed self 路径使用 T04 边界”、INV-3。Owner T15；统一真实 caller 的路径校验，并加入只变祖先链接的拒绝例。

3. **P1，持久效果路径没有可信构造面。** `effects.rs:72,115,139`、`home.rs:114`、`fsx.rs:176–178`。待恢复 request 的 `WriteFile.path='../outside/payload.md'` 通过 serde 解码、Home 裸拼路径、词法 starts_with，ParentDir 被跳过。真实新 CLI 写命令恢复时已创建管理根外文件，之后命令才报 WORKBOOK_EXISTS。违反 engineering §2.2 路径 newtype、§5 外部路径 confine、T04/存储合同 §3.3 根内路径复核。Owner T04/T07；所有效果路径须先校验相对路径和对象归属，禁止遇非法数据仍执行。

4. **P1，正常冻结 Workbook 使 purge 半途失败。** `selfmgmt.rs:288`、`fsx.rs:359–373`。真实 workbook add 成功后 `self uninstall --purge --yes` 返回 IO/EACCES：冻结目录 `0555` 未安全放开权限便递归删文件。该次失败已删掉 `store.db`、`.lock`，Workbook 留存，形成无数据库的半空根并破坏锁连续性。违反 self purge 生命周期及 T15 失败边界。Owner T15；必须覆盖真实含冻结树管理根，不以只装 binary 的 fixture 代替。

5. **P2，N13 假并发仍未闭合。** `tests/service.rs:229–238` 的 lazy `map(spawn).map(join).collect` 每个线程先 join 后才启动下一个；T15 的 purge_waiter 又以 100ms sleep 代同步点。M1 §4.2 明文要求先启动所有线程、同步制造交错、最后 join。摘要独立向量已经改善，但绿色测试不能关闭这条并发证据欠账。Owner M1/测试；强化同步点后核锁及终态。

## 工程品味判断

三个 crate 的职责、纯 core、新布局和独立摘要向量方向正确；没有依据要求改成通用 trait/typestate 框架。主要工程问题是 `fsx` 的“受限”接口并未在类型和构造面维持承诺，调用方能够持裸 AbsPath/String 绕过校验；self 又混用受限 helper 与直接 std::fs 操作。修复应收紧真实文件边界并迁移 caller，不是泛化重构。工具自动检查的格式、lint 和命名不另列 finding。

## 原始证据

- `cli_boundary_probe.py` / `.json`：真实 CLI rollback、install、purge；archive symlink 被正确拒绝，不列 finding。
- `safe_file_seal_probe.rs` / `.txt`：public SafeFile 观察后对象替换；使用现成 rlib rustc 链接。
- `effects_path_probe.py` / `.json`：临时 SQLite 设置待恢复请求并仅注入效果路径；触发真实 CLI 恢复，Python hashlib 独立 oracle。

全部位于 `/private/tmp/sheltie-m1-review-e1a8126/standards/`。probe 临时管理根位于 `/private/tmp`；无真实用户 home 修改。candidate worktree `git status --short` 为空。
