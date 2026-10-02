# C006 设计：不可变请求与受限新增

状态：`proposed`；实现 `not_run`。本设计以 [C004 设计 §6 结果合同](../C004-verifiable-delegation/design.md#6-结果合同) 为唯一来源，不定义新的结果资格或选择算法。

## 1. 分层与现有入口

| 位置 | 职责 | 实施时的入口 |
| --- | --- | --- |
| C004 `work result` | 只读选择结果、核候选与证据关系、可信读冻结 Artifact 字节 | C004 的公开结果合同 |
| `crates/sheltie-export` | 严格解码结果、请求冻结、文件复制、恢复、人读和 JSON 输出 | 拟新增 `main.rs`、`source.rs`、`request.rs`、`target.rs`、`export.rs` |
| `crates/sheltie-runtime/src/fsx.rs` | 引擎管理根的句柄约束与安全发布 | `ManagedFs::open_regular`、`ManagedDir::write_new/rename_new`；仅作技术参考 |
| 发布文件 | 生成同一 release 的引擎与导出器独立归档及 sha256 | 根 `Cargo.toml`、`.github/workflows/release.yml`、既有平台指南 |

默认只新增导出器 crate，不先抽通用文件系统 crate。现有 `ManagedFs` 依赖 `HomeLock` 和管理根布局，强行让导出器依赖 runtime 会扩大权限与调用面。强模型在 T01 阅读真实 `fsx.rs`，在导出器私有 `target.rs` 中用当前锁定的安全 Rust OS API完成最小目标文件操作；若实证表明必须共用抽象，由 T01 明确两个真实消费者、具体接口、迁移测试和成本，再经 M1 审查后决定。初级实现者不得自行提取 crate。

Git 词汇仅存在于 CLI 结果投影和外围导出器，不进入 core/runtime 的业务类型与判断。

## 2. 结果与源字节

调用 `sheltie --json --home <management-root> work result <work>` 取得 `work-result/v1`；字段 `work_id/revision/workbook_digest/status/final/artifacts/candidate/check_records/unverified_conditions/snapshot_digest`、Artifact 项身份与限额都按 C004 §6。只有 `status=succeeded` 且 `final=true` 才可导出。多个所选原生报告的候选或输入不一致由引擎拒绝，导出器不另选最新报告或拼装候选，也不维护平行 schema。

复制每项 Artifact 时调用 `sheltie --home <management-root> work result <work> --artifact <key> --result-digest <snapshot_digest>`，拒绝 JSON/request-id。引擎从同一个受限打开的普通文件句柄边读取边核对大小和摘要，读毕以退出码确认完整性；导出器在受限目标临时文件中接收全部字节，检查退出状态，再独立计算大小和摘要。任何不符都不能发布目标文件。命令调用不经过 shell，二进制绝对路径、参数数组与 stderr 单独处理，stdout 只能是合同规定的内容。

源在开始前或读取中被替换、截断、增长或改写时，完整性核对拒绝。已结束成功的 Work 不因导出再修改；如果 C004 结果快照不稳定或缺少可信字节入口，C006-T01 停止，不让导出器直读猜测的管理根路径。

## 3. 导出请求

新请求目录：

```text
<authorized-parent>/<full-work-id>/<request-id>/
  .export.lock
  request.json
  artifacts/<stable-result-key>/<original-leaf>
  result.md
  manifest.json
```

Artifact 目标路径只由已校验的结果 key 与安全文件名构造；不直接拼接源绝对路径。T01 冻结唯一的文件名编码与重复名拒绝规则、大小限额、严格 JSON schema 和稳定序列化顺序，并给手写期望字节。`result.md` 与 `manifest.json` 根据冻结请求确定性生成，不含本次运行时间；同一请求重新生成的字节相同。

`request.json` 固定结果快照、目标父目录与请求目录的设备号/inode、Work ID、请求 ID、引擎/导出器版本、完整文件清单及每项期望字节。请求本身按原子不替换方式发布并同步；成果复制只在它完整发布后开始。它是外部导出恢复依据，不是 Store 请求、授权令牌或新的 Work 状态。请求身份限定为父目录设备号/inode、完整 Work ID 与 request-id；跨父目录或 Work 不存在全局去重。换位置或 Work 时指南要求新 UUID，不为检查这种流程要求新增索引。

导出器在请求目录用安全打开的 `.export.lock` 取得进程级独占锁，串行同一请求的写入。锁文件要求普通文件、单硬链接，锁与请求目录句柄身份一致；崩溃由 OS 释放锁。请求未发布时允许目录里只有核为普通单链接的锁文件与保留临时命名空间，重新获取结果并以新的随机临时名创建请求；不读取、删除或复用缺少归属证明的旧临时文件。任何业务文件却缺请求时拒绝，不接管不明目录。T01 用竞态测试证明首次并发不会产生两个不同请求。

## 4. 受限目标操作

1. 从文件系统根逐段打开目标父目录，拒绝链接，得到真实目录句柄；按路径和目录身份核对它与管理根不重叠。系统已有路径别名的解析规则沿 C002，并在 T01 定死，不能把任意 `canonicalize` 当成授权。
2. 在父句柄下创建或核对 Work 目录与请求目录，保存并持续核对目录身份。恢复时父目录或请求目录 inode 不同就停止，不能只凭同名路径接续。
3. 相对目录句柄逐段打开或创建成果父目录。所有相对路径拒绝绝对路径、空段、`.`、`..`、NUL、分隔符混淆和不支持的文件名。遇链接、特殊文件或不能唯一编码的名称就拒绝。
4. 缺失文件在最终父句柄中独占创建本请求临时名；写入和摘要核对作用于同一文件句柄。确认普通文件、单硬链接，`sync_all` 后用 `renameat_with(..., RenameFlags::NOREPLACE)` 或经实测等价的安全 API 发布，再同步目录。
5. 目标已存在时只读相对打开，不跟软链；检查类型、单硬链接和同一句柄字节。相同则跳过，不同则停止。检查后竞争创建同名文件时原子不替换发布必须失败；禁止普通 rename 或任何 truncate。
6. 所有成果发布后逐项读回核对，再生成 `result.md`，最后发布 `manifest.json`。存在同字节清单仍要核对成果；不同字节清单冲突停止。

跨文件系统通过字节复制实现；不把源文件 rename、link 或 reflink 到目标。只清理由当前进程持有文件句柄、inode 与本请求标识能证明归属的临时文件；崩溃后归属无法证明的旧临时文件保留且不复用，使用新随机临时名接续，不递归删除目标树。

该顺序没有承诺多文件原子性。部分成果可能已经发布，但最后清单未出现；恢复会逐项核对和补写。清单也不能保证随后用户没有改文件。导出结果报告保存核对事实和来源，不声称永久送达。

## 5. 故障恢复

| 观察 | 处理 |
| --- | --- |
| request 未发布 | 未开始成果复制；只允许锁与保留临时命名空间；旧临时文件不删除，新独占临时名重新建立请求 |
| 部分成果已发布，manifest 缺失 | 相同字节跳过；补缺失项；不同字节停止；最后发布清单 |
| manifest 已发布，某项缺失 | 仍逐项检查；补缺失项后核对旧清单相同 |
| manifest 已发布，某项被修改 | 冲突停止；不相信清单，不覆盖用户字节 |
| 源完整性不符 | 目标临时文件不发布；报告源损坏，交回引擎修复入口 |
| 父/请求目录被替换，软链/硬链被注入 | 拒绝；根外哨兵和其他请求目录不变 |
| 同一 request 指向另一结果或版本 | 请求冲突；使用新 request 另建副本，不改旧请求 |

## 6. 平台与分发

平台系统操作、竞态注入点和关键拒绝测试由强模型在 T01 完整实现并实测，不能只留 `todo!()` 给初级开发者。若当前 safe Rust API不能兑现本设计，停止并收窄发布支持；不得改 `unsafe_code = forbid`。

导出器版本来自 workspace release 版本；所调用引擎的 `self version` 必须完全匹配且支持 C004 结果 schema。开发探针也使用同一候选构建的两份二进制。T07 在临时目录检验 release 归档、sha256、文件权限和帮助文案；发行指南明确同一 tag 下两份归档的名称与来源。引擎 `self` 仍只更新自己，更新后不匹配的导出器拒绝并提示显式重新安装对应版本。T07 不发布真实 release。
