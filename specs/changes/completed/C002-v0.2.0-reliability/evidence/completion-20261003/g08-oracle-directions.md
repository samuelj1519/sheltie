# G08 独立期望准备（33 个精确对应 ID）

范围：只读当前调用链、历史对应和合同；未编写或执行 oracle，未改源码/测试/plan，不给动态 PASS 或完整等价处分。仅 macOS aarch64。准确旧/当前 ID、原 diff SHA、分组、来源 SHA 见同名 JSON；库存中的行号是归档身份，不是插入 observer 后的工作树行号。

## 共享能力与独立判据

| 组 | 数量 | 真实入口 | 主要独立事实 |
|---|---:|---|---|
| V1 发布闭包 | 6 | recovery→effects；全局 PendingReferenceIndex→Workbook list/维护 | 唯一原件、摘要/引用、四格终态、冲突的准确 cause |
| V2 目录句柄与名称绑定 | 9 | ManagedFs 目录发布/删除、效果 wrapper、tmp 维护 | 持有原树与当前名字的 dev/inode、根 epoch、竞争者原件 |
| V3 删除 marker | 5 | Workbook remove 已提交恢复 | marker 与本请求绑定、两端点均无、拒绝前后的原件 |
| V4 递归同步、purge、mode | 8 | sync/remove/purge/set_tree_readonly | 全根预检前零修改；名字替换后的竞争者字节与权限 |
| V5 只读定位 | 5 | read_publish_dir→Workbook/Work 装入 | 同轮前后存在性、唯一原件、暂时 IO 与确定 NotFound |

### V1：复用真实提交和实际冲突，不伪造 CheckedEffects

先由实际 add/start/remove 构造请求，保留原 requests/audit/业务行及原件清单；以独立读取的端点、dev/inode、字节摘要判定。正常发布只有 final，正常删除只有本请求合法 marker、两个目录端点均无。效果失败须保持 published=0，返回合法 original；旧请求 A 阻断新 B 时核 B 身份和 pending_original。不要只断言 is_err。

`verify_publish_final_state` 两项：终态 final 仍是已核同一目录而额外 pending 出现，原实现必须拒绝；AND 可漏这个单独事实，删除整个函数也可漏最终绑定。`publish_after_tree_sync` 在 pending 分支的重新绑定、rename 之前，太早，不能被当成完成终态窗口。建议仅在最终原件验证完成、终态检查之前设置有界 rendezvous；另核观察点不会绕过前层。

`verify_owned_digest→Ok` 是真实外部读取，不是纯重复。它也在 Workbook 删除路径中直接使用：构造已提交未删除的 remove，改变其 final/pending 登记原件内容，合法 marker 尚未建立；原实现须在 rename/删除前拒绝并保留被改现场。删树后的其他检查或 Work 状态卡重新装入可能让 mutant 最终仍报错，因此判据必须包括第一次移动/删除的停止点。Work 的 frozen Workbook 和起始输入可用相同完整原件关系补正例，不用逐字段镜像 guard。

`classify_committed_path_error` AlreadyExists：真实 begin 历史文件缺失修补，复用 `atomic_write_after_file_sync`，在 NOREPLACE rename 前建立竞争目标。原实现的顶层 EFFECT_PENDING cause 应为 STORE_CORRUPT（出现已有对象），mutant 保留 IO；竞争目标字节/身份不得改变。该点在 effects 的缺失检查之后且原子临时文件已经同步，不猜随机临时名。也核合法已有、匹配的历史文件原字节不变。

`validate_publish_reference` Work 分支：保留合法 work owner、digest/root、request.work_id，只将 final 改到另一个合法 Work 路径。全局 `PendingReferenceIndex::from_rows` 必须先拒绝所有损坏目录登记；`WorkbookRepo::list` 先载全局索引，因此可用真实 Workbook list 暴露被错误忽略的 Work 登记，不让后层 Work-specific 校验遮住。owner 经 WorkId parse 后的比较恒 false，不能据此宣称整串 OR→AND 等价：它恰好会吞掉独立 final mismatch。当前 Work bundle 查询只核关联 Work 的边界继续保留。

### V2：同 FD 不支配外部名称；先说明 precedence

`ManagedTree` 实际由 `open_tree_locked` 创建；生产 caller 是 effects 的 publish/delete 和过期 tmp 维护。wrapper 每次 `ManagedFs::open_existing(home)`，不能把上次根与名字的核验当成本次不可变事实。`verify_managed_tree_at→Ok` 还会跳过重新打开根/父目录/名称的全部检查。`check_tree_root` 同时约束 root 字符串和 root inode epoch；caller 闭包未证明 epoch 恒定，不能只用相同 home 字符串处分。

复用 `publish_after_tree_sync` 做同设备目录 B 替换：原树 A 另存，B 位于原路径。原实现必须在移动前拒绝，A/B 完整 inventory 保留。对 rename 的 source guard，前面的 verify 可能先拒绝，因此若要判别这个独立阶段，窗口应在上一层 verify 完成后、source 新 stat/rename 之间。若 mutant 已错误 rename，随后 verify 变 RecoveryRequired，最终 is_err 并不证明正确；必须核未移动竞争者的停止义务。删除用 `delete_before_root_unlink`：已空的持有 A 移走、当前名字改 B，原实现须保留 B 并报 RecoveryRequired；不要要求此前已经合法删掉的 A 内容凭空恢复。

设 A=kind非目录，B=dev不同，C=inode不同。首连接符突变为 `(A&&B)||C`，后连接符为 `A||(B&&C)`。同设备不同 inode 的目录仅能证明后连接符缺陷。当前三个首连接符（verify_tree_at :658、rename_tree_new :699、verify_tree_entry_at :1691）不能用同设备替换运行没捕获就记等价；稳定同 dev/inode 的同一对象类型确实不可改变，但还需完整证明所有当前 producer/consumer 都禁止 dev 不同而数字 inode 相同的事实。没有这个证明则保留 open；不引入挂载、其他平台或假 stat 制造结果。

root epoch 反例方向不需要 inode 复用：把原管理根 A 移到保存路径，在同名位置创建 B，并将持有的原 tree 对象移入 B 的对应路径，叶对象 dev/inode 仍相同、根 epoch 已不同。核实际无锁 verify wrapper 是否仍可抵达及准确早期诊断；其他持锁方法可能先被 check_lock 拒绝，必须按真实 caller 分别记录。不能把“稍后另一个 guard 会拒绝”推广为全部 root gate 的等价，也不声称这个 fixture 会让整个引擎成功。

### V3：marker 四格检查必须按阶段核

四个 delete_dir OR→AND 分别在既有 marker 的首次/同步后，以及删树后/新 marker 同步后。共享正例是本请求 canonical `.deleted` 且 pending/final 都无；负例是仅一个端点出现，竞争者原件不能被删除，不能猜测生成完成标记。

`delete_marker_after_validation_before_sync` 可控制既有 marker 校验后、同步及第二四格检查前的变化。`delete_after_tree_removed_before_marker`、`delete_marker_synced_before_mark` 是 exit 点，适合构造真实中断现场，但重启会重新进入更早检查，不能以重启时被前层拒绝证明后层 caught。需要区分四个检查时，只加同次 invocation 的准确有界 rendezvous。无标记、坏标记、仅 pending、仅 final、正常完成的组合用一份独立端点/marker 判据覆盖。

### V4：全根预检不能被最终报错替代

`purge_contents` 先预检全部 root，再 fchmod 管理根、调整全部目录权限、逐根删除。构造前面合法只读目录、后面优先级较晚目录中的真实硬链接，保存整个根 inventory、权限、Store/.lock inode。原实现须在任何 chmod/unlink 前拒绝。`preflight_remove_at→Ok`、`nlink==1→true` 允许后来进入改权限或删除阶段；后层 remove_at 即使拒绝硬链接，也不能恢复预检的零修改义务。另核 final-control rescan caller 的单链接普通 SQLite 控制文件前置范围，不与首轮全根义务混写。

合法叶 symlink 的 purge 正例应删除链接自身，外部目标保持精确字节、dev/inode 不变；删除 Symlink arm 会把合法对象落入 wildcard 而错误拒绝。递归 sync 的 symlink 是拒绝场景，见限定候选。

`remove_at` 和 `set_dir_tree_mode` dev/ino OR→AND：复用 `remove_directory_before_open`、`tree_mode_before_open`，在已完成 stat 后换同设备 B，原 A 保存；原实现不递归清空 B、不 chmod B。after_open observer 在比较后，过晚。mode 分别做目录、普通单链接文件，避免文件 nlink/type guard 抢先掩盖；原件/竞争者分别核内容与权限，不只核返回值。

### V5：一轮读事实与有限重试

正常 pending-only、已 rename 的 final-only、published final-only 均需成功读取登记原件并携带正确 pending flag；合法 rename 在有界范围内允许读完。两者同时存在时，未完成发布必须拒绝，不能任选一棵。先用稳定 both-present 覆盖首次选择守卫，再用 pending-only→both、final-only→both 覆盖第二观察守卫；第一次正确而第二次出现竞争者，不能合并为一个恒定事实。

`pending_read_after_locate` 已晚于全部存在性检查，不能证明 :214/:221 的前后窗口。若低层 existing point 无法唯一识别这一轮特定调用，只建议 locator 的窄 observer，不用轮询猜时间或 mock 任意 bool。pending_publish=true 且两端点持续缺失，原 :227 逻辑以 IO/WouldBlock 结束，OR 会变 NotFound；上层可能映射错误，需记录真正 consumer 的准确诊断，不能假设类别等价。`read_publish_dir` 只在 locate 成功后、read 发现路径消失时重试，不会修补 locate 已返回的错误。只读副作用判据是无 HomeLock、无 Store/业务文件写入，无借用另一个原件。

## 四项限定静态候选

1. effects :1086 路径/SHA 格式 OR→AND：执行来源只有私有 CheckedEffects。Work producer `check_work_effects` 从已 `load_row_at`→`validate_work_paths` 核过的 state.inputs 导出 RefJson，final 绑定 `works/<work>`，每个输入路径绑定 `workdir/start-inputs/<key>`；Sha256Hex 类型转字串再构造合法。read consumer 的另一 check_work_effects caller不执行；Workbook producer start_inputs=None、owner 非 Work；cfg test checked_submit 只有 SealOutputs/RefreshStatusCard，没有发布。两个被改谓词在全部当前执行 producer 都 false。仅可处分该 shape guard；随后实际文件 sha/bytes 是新事实，不套证明。未来任何构造 RefJson/CheckedEffects 的新 caller 要重开。
2. effects :1305 marker format/id OR→AND：单个错误字段由同一已读 bytes 的后续 canonical bytes 比较拒绝，中间只有纯序列化，无进一步业务 I/O。拒绝集合、完成资格、停止点相同，诊断从“internal_id不一致”变“字节不是合同格式”。不得记完整响应等价。
3. fsx :1616 删除 sync Symlink arm：同一不可变 stat match 值落到 wildcard，也返回 InvalidRequest，既不打开子项也不 sync 该子项；已完成的前项同步相同。诊断从符号链接变特殊文件，仅接受/拒绝和该停止点相同，不是完整错误文本等价。
4. fsx :2618 删除 preflight RegularFile fallback arm：第一 `RegularFile if nlink==1` 保留，其余普通文件落 wildcard，同样 InvalidRequest、无 chmod/unlink；诊断由硬链接变特殊文件。仅这个 arm 删除成立，不适用于 nlink guard→true 或整个 preflight→Ok。

## 执行与停止边界

此报告的全部 33 项执行状态为 not_run_by_this_preparation。四项只列限定候选；其余 open，未认定 equivalent。作者先冻结输入闭包，给共享正反例短反馈，再选对应组，保存每项完整 raw；零测试、错误 ID、observer 未命中、上游 guard 遮蔽、源漂移、超时均停止并保留未完成。先以少量真实原件/提交/端点关系覆盖能力，只有缺失窗口才增加局部 observer，不扩通用框架、平台或发布。
