# C006 产品方案

状态：`active`。结果资格与选择只在 [C004](../../completed/C004-verifiable-delegation/spec.md)定义。本文件定义可编辑副本，不定义第二套结果。

## 1. 用户路径与命令

1. 读取 `sheltie work result <work>`，确认需要的最终成果。
2. 需要副本时，显式给出既存目标父目录。
3. 工具接收并核验全体 Artifact，在父目录下发布唯一新目录，输出实际位置。
4. 失败时保留现场并说明哪些对象可确认。再次运行创建另一份新副本，不恢复或覆盖旧目录。

```text
sheltie-export --sheltie <absolute-binary> --home <absolute-management-root> \
  --work <full-work-id> --to <existing-absolute-parent>
sheltie-export --version
```

`--sheltie` 固定可信的引擎绝对路径，不依赖 PATH，不经 shell。`--work` 必须完整；父目录必须存在。`--json` 使用导出器自己的有版本响应，stdout 一行；人读模式给完成位置和失败处置。输入不含 request-id，也不含任意源文件路径。

父目录逐段安全打开，拒绝软链；系统别名可由用户显式提供真实路径。目标父目录与管理根重叠时拒绝，避免导出器把副本写回管理根。授权绑定本次打开的真实父目录对象，不扩展到宿主配置或其他位置。

## 2. 结果与源读取

导出器严格解码 C004 的 `work-result/v1`，要求 `status.kind=succeeded`、`final=true`、`effects_pending=false`、非空 `artifacts`。保留 `work_id`、`revision`、`workbook`、`flow`、每项来源和期望摘要/大小。字段、key 顺序与唯一性沿 C004；未知格式或非法载荷在创建暂存目录前拒绝。

C006 同时提供引擎只读入口：

```text
sheltie --home <management-root> work result <full-work-id> \
  --artifact <key> --revision <事先读取的整数>
```

该模式拒绝 `--json`、`--request-id`；stdout 只输出原文件字节，诊断写 stderr。`--artifact`、`--revision` 必须同时出现。引擎从同一次已校验读取状态中核指定 revision、最终资格和 key，再从受限普通文件句柄读并核摘要/大小。读毕且全部核验成功才以 0 退出；中途错误可能已经输出部分字节，消费者必须先暂存、检查最终退出码再发表。

支持性由 `work-result/v1` 严格解码和实际源读取命令判断，不要求引擎与导出器 release 版本相同，不增加公共能力注册表。引擎不支持读取参数时准确拒绝；导出器可以留下私有暂存，但不能发表完成目录。

## 3. 行为规则

| ID | 规则 |
| --- | --- |
| DL-01 | 只导出 C004 选定的全体最终 Artifact。不扫描历史、不猜文件名、不复制 Workbook 或 Git 仓库；没有最终选择时拒绝。 |
| DL-02 | 每次执行在授权父目录下独占创建一个随机唯一的私有暂存目录，并选择一个唯一新最终名。暂存与最终目录在同一父目录、同一文件系统。 |
| DL-03 | 接收每项源字节时固定 Work、revision、key。引擎核实际源字节；导出器独立核 size、sha256 和子进程退出码。任何不符均不发布最终目录。 |
| DL-04 | 全部成果和来源清单在私有目录内完成，文件和目录同步后，使用原子 NOREPLACE 整目录移动。已有同名目标即停止；不能检查后普通 rename、覆盖或合并。 |
| DL-05 | 所有目标路径只从已校验的排序索引和源文件叶名构造。相对目录句柄操作不跟软链，独占创建普通单链接文件；不使用 link、reflink 或源 rename。 |
| DL-06 | 完成目录中的来源清单记录结果与逐项相对路径、摘要和大小。它证明发布时核验事实；随后用户编辑副本不触发引擎状态变化或自动修复。 |
| DL-07 | 失败不发布部分目录，不接管旧暂存或不明目录，不自动删除竞争对象。重跑创建新副本。进程仍持有句柄且能证明归属时可报告自己的残留路径；不能据同名路径宣称归属。 |
| DL-08 | 原子移动成功但父目录同步或最终身份核对失败时，报告「可能已发布，需核查」，不称完成、不回滚删除。进程被杀导致没有响应时，用户核查目录；重跑仍只创建新副本。 |
| DL-09 | 导出不写 Store、不增 request/audit、不提交回执、不改变终态。只读 SQLite 控制文件维护按只读查询合同单独识别，不冒充零物理写入。 |
| DL-10 | 实测平台无法兑现受限打开、独占创建、目录同步与 NOREPLACE 时阻断支持，不降级覆盖、不启用 unsafe。 |

复制是读取与写入字节，不需要源和目标在同一设备。发布只移动目标父目录内的私有目录，因此不承诺跨设备 rename。

## 4. 输出与限制

导出器的 `work-export/v1` 响应包含 `status`、`work_id`、`revision`、`target_path`、`staging_path` 和稳定错误信息；路径不存在或不能确认时为 null。成功状态为 `complete`；失败区分 `rejected`、`failed_before_publish` 和 `publication_unconfirmed`。只有 complete 使用退出码 0。

参数或确定性资格拒绝退出 2；源、目标、完整性或 I/O 失败退出 1；移动后未确认退出 3。不解析 stderr 自然语言来分类引擎错误；源非零由源读取失败表达，保留其退出状态供诊断。

同一 OS 账户的外部进程可修改或移动副本，本工具不提供安全隔离。句柄操作防止软链或路径替换把操作重定向到另一对象；检测到父目录或暂存身份变化时停止。授权目录对象被同权限程序移动，不等于获得新路径的额外写权限；不宣称路径名是永久边界。

## 5. 采用后同步

更新 `specs/architecture.md` 的外围二进制职责、`specs/contracts/protocol.md` 的只读字节模式及错误、导出指南和工程 workspace 清单。C004 的结果选择合同不因复制改变；Store schema、Work 状态、宪章和引擎 self 组不增加例外。工具不修改 shell、agent 或宿主安装配置。
