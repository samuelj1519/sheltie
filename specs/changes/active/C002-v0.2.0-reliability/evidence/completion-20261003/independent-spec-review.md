独立 Spec 补审结论：**需修改**。审查源码候选为 `9ee0f0114a8e407257864175eb354037e38a8451`；审查者 `/root/completion_spec_review` 未参与被审设计、实现或测试。本报告只审当前 C002 核心合同及 C004–C006 对它的后续修改，不批准完整 C001–C008、全部变异、平台或用户价值。

**F-SPEC-01：写重放的 Command/data 严格解码晚于冻结业务读。必须修复。**

依据为 `specs/engineering.md:42` 的完整载荷解码顺序、`specs/contracts/storage.md:167` 的快照资格和 `specs/contracts/protocol.md:284` 的原响应资格。`crates/sheltie-runtime/src/service.rs:1206` 在 `load_checked_request` 中先调用 `self.load(&work)`；该调用通过 `load_row_at` 编译、读取冻结 Workbook。Command 在 `service.rs:1213` 才完整解码，data 在 `service.rs:1512` 的 `validate_command_owner` 经 `snapshot::check_data` 才按完整 DTO 解码。因此非法 Command/data 也先触碰业务对象，冻结副本错误会掩盖应先被拒绝的非法载荷。

真实 CLI 反例使用主 agent 锁定的 `/private/tmp/sheltie-completion-20261003/sheltie-current`，SHA256 `9f7133e66b45e8d667724aef34513dd36db8cd89a0eb2d282ca70d7c58a43c7f`。在独立临时 Home 中 add→start→begin；对已提交 begin 的 Command 或 data 分别仅加入 `unexpected_contract_field`。随后将冻结目录移至同 Work 的保留目录作为读顺序控制。只读 `work status` 先拒绝未知字段；同 rid 的 `attempt begin` 却先返回「已发布冻结副本缺失」。两个分支的 works/requests/audit/work_sequence 完整 SQL 行均未改变，原件保留，也未伪造释放 original。这是确定的调用顺序缺陷，不主张已发生外部写入。

修复入口：在任何冻结业务读取前完整解码 Command 与快照 data，复用已检查 DTO 给业务绑定验证；坏 Command/data 继续没有 original。合法快照的效果失败仍应保留 original。回归应同时覆盖同 rid 重放与旧未发布 A 阻断新 B，保留只读顺序对照、SQL 行/原件和准确提交身份 oracle。

反例脚本：`spec-order-probe-locked.py`；原 argv/exit/stdout/stderr、binary SHA 与结果：`spec-order-probe-locked/results.json`，均相对本报告所在目录。初次独立构建副本的相同反例另存 `spec-order-probe/results.json`；正式结论以主 agent 锁定 binary 为准。

**完整效果解码的描述边界需要勘误。**

同一脚本还保存 effects 未知字段的对照，但该对照不作为上面的 Command/data 缺陷。`engineering.md:42` 的「任何业务 I/O 前」若扩展到所有 effects 错误，与 `protocol.md:284`「已核合法快照遇到效果载荷……错误时，仍提供原响应」及 storage §3.2 的分层校验冲突：Work 快照资格需要冻结图。效果错误不得因为提早返回而丢掉合法 original。

建议上游明确：完整快照/Command 先纯严格解码；效果结构、路径与整组业务闭包在任何效果动作前严格校验；允许为快照资格读取冻结定义；效果解码结果与快照资格独立，坏效果不撤销已核 original。保持既有真实正反例，不增加兼容格式、默认值或第二事实来源。

**当前 R22–R24 与后续状态合同的限定核查通过。**

| 合同 | 现行落点与判断 |
| --- | --- |
| original/pending_original | `workbook_repo.rs:324` 完整 metadata/audit/DTO 与 add target 投影先核业务身份，`effects.rs:452` 绑定 final/owner/digest；`service.rs:1217` 在 original 生成前核 Work/Attempt/冻结节点。`recovery.rs:264` 将本人和旧 A 的响应分层。损坏业务身份不释放成功快照；效果错误保留已核响应。 |
| 历史失败状态 | `core/work/decide.rs:755` 用截至原 Attempt 的 failed 前缀，不把创建 number、superseded 或后续失败数当历史失败数。可在后来 cancel/replace 后重放。 |
| requires | `service.rs:1586`/替换分支核冻结节点 `node_requires`；Reply 与 data 一致还不足以伪造节点声明。 |
| 资格与撤销 | `core/work/state.rs:321` 核连续 number、单次 superseded、有接替和唯一当前 running；`load.rs` 核输入、来源及路径；replace 的旧/new 相邻、时刻/理由/输入/来源在历史绑定中核对。撤销资格不声称停止进程。 |
| 可信只读装入 | `store/read.rs:106` 在同一 SQLite 读事务取得 state、requests、audit，核唯一闭包与 revision；`service.rs:703` 先完整解码全部请求再读冻结图。C004/C005 只读未知字段顺序测试确实覆盖此链。 |
| 副作用与恢复 | `service.rs:1060` 锁内恢复先于新业务；`recovery.rs:226` 整组 checked effects 执行后刷新最新卡并 mark；`effects.rs` 在动作前核形状、归属、原字节/摘要；恢复使用提交历史字节。 |
| C006 raw | `service.rs:570` 一次可信 result 读取核 revision/final/key，`result.rs:6` 仅接受已核 Ref，同 FD 验证读；raw 不维护 pending/tmp。此项为当前调用链静态核查，不扩大为全部跨设备/平台实测。 |

独立实际测试使用 nextest `0.9.145`、原默认 profile、`--locked --offline --all-features`、空 RUSTC_WRAPPER 与独立 target。CLI T34 五项 **5/5 PASS**，run `d74b900d-e244-492a-969c-a11dc26effee`；原文 `spec-t34-tests.log`，19 项 filter skipped，不是全仓结果。runtime 历史失败/替换、严格只读、效果错误八项 **8/8 PASS**，run `a5566808-b276-4ca7-a52e-b0555f5c0e2a`；原文 `spec-runtime-tests.log`，329 项 filter skipped。二者均不能抹去 F-SPEC-01。

**SK01 的精确历史边界。**

通过只读 `git show e54dcd41d8f1e186007b62b47583063cb19a4b66:.../safety-skips.json` 核得：SK01 是 `C002-T34` 的最终增量 Spec 续审，原请求范围是快照业务绑定、checked original/pending_original、Add 发布目标、历史状态及节点 requires；原候选 `e3eea899877165f8573befee3774555598ec92bd`，缺最终批准。旧候选源码也已只读核验。旧 R22–R24 的纯规则与身份分层方向成立；现行 C005 把历史失败数改为 failed 前缀，C004 加读事务/结果资格，C006 加原字节读取，不能把本报告当前候选/新测试写成旧 e3eea89 的最终整体批准。原 SK01 暂停和 SK02 215 项缺失原文保持历史事实。本轮已完成新的当前候选独立审查，结论仍为需修改；修复后需独立增量复核。

**T41 独立治理审查：限定 PASS，可按完成门禁提交。**

当前恢复入口与用户「逐一完成所有跳过步骤」一致；唯一 active 为 C002，原 macOS aarch64 v0.2.0 release/tag/资产和历史限制保持。README/根索引正确导向 active，后续 C004–C008 仍留 completed 的限定记录。T41 只授权目录迁移、链接、欠项计划；T42–T45/M2 无源码或测试修改权限。若修 F-SPEC-01，必须先设准确修复任务/范围，不能塞进 T41。

独立逐字节比较 HEAD completed 下全部 **17 份 evidence，4,383,945 字节**与 active 对应文件：0 缺失、0 字节变化，清单/各 SHA 为 `independent-t41-integrity.json`。外部根/ADR/release/legacy 变化只修链接，不改历史测试或发布事实；TOML 解析通过，所有当前 tracked 变更均在 T41 白名单。独立 docs/specs/tests/diff 检查全部 exit 0，原 argv/stdout/stderr 为 `t41-independent-governance.json`。尚未执行最终 staged check-task 或提交 trailer 复核，须按 plan 将 T41 状态记 done 后执行；本 PASS 仅批准恢复记录及保真迁移，不批准产品完成或后续源码修复。
