# C002-T55 限定静态与结构计数复核

**4项限定静态证明通过；9项结构迁移的源码/关系/接线前提通过，最终执行闭包尚待，不由本报告直接关闭。** Reviewer只读未写oracle或生产代码。精确旧/current ID、diff/sourceSHA与每项范围见JSON。

1. verify_start_input_references路径/SHA shape OR：CheckedEffects字段private，全部执行factory仅Work/Workbook两处。WorkService执行装入先load_row_at/validate_work_paths，把每state.inputs.path绑定本work/start-inputs/key；RefJson相对path由相同home.to_rel，sha由有效typedSha.as_str；Publish final同式works/work。Workbook startInputsNone/owner非Work早return；read-side check不执行；test checked_submit仅Seal/Refresh。因此该两个predicate在完整当前执行producer都false，守卫不改变普通成功/拒绝/I/O。原OR会额外clone/parse有效SHA，AND短路省略，所以不是资源/CPU/OOM等价。实际bytes/sha后续新事实绝不能套此证明。

2. deleted-marker format/id OR：strictDTO来自同一次4096以内bytes读取，单字段错variant后续canonical bytes必拒（字节相等会蕴含两字段正确，矛盾），中间只有pure序列化；两字段错都早拒，正确但非canonical也同拒。普通拒绝集合/业务I/O停止相同，但detail由internal_id不一致变不是合同格式，额外pure分配/序列化亦不称等价。不能说完整error JSON相同。

3. sync_dir_tree删除Symlink arm：同一次immutable stat.kind的Symlink落wildcard，同为InvalidRequest，均不打开/sync该子项；之前合法项的同步相同，之后不执行。reason符号链接→特殊文件，限定停止/拒绝，不是全文错误相等。

4. preflight_remove_at删除RegularFile fallback：首个RegularFile ifnlink1保留，其余regular落wildcard，同InvalidRequest且未chmod/unlink，immutable nlink不重读。reason硬链接→特殊文件。此证明不含nlink guard变true或整个preflight变Ok。

结构计数独立核：33旧G08中**5目录connector+4locator connector=9**；不是6目录欠账。6目录源码语法位置中旧1595不在215，旧formal stage1Missed后已有focusedCLI后补Caught/no missing，另存原文SHA，不新增第34项。verify_tree_at旧568 Fn→Ok还在24direct内，须生成当前target执行。locator抽出4处candidate&&!other、role交换正确、bool值immutable；存在性读/branch/sleep与published例外/最终Io/NotFound顺序保持，不加入未采用beforeRetry保证。

9结构项最终须逐旧ID保存原stage1与native not_run，标target移除、新helper/pure关系/wiring/current共享机制run及真实caller/finalgates。原2locator此前sampleCaught仅旧候选证据，不复用为新candidate currentCaught；newShared9不是9次旧native。当前mapping里target=null的current_diff_sha256仍是替换前候选diff，终账本须清楚改名/标明previous-candidate身份，不能当现存target。

directory真实verify/rename/delete新case覆盖metadata来源/阶段；fresh registered_add通过WriteSession空tmp清理，真实begin通过PrepareAttempt目录同步也走第三helper，完整caller须在最终同源回归核。两纯module interface表不是全业务I/O oracle，kind/device/ino及sole布尔含义已审。此报告不授20direct动态或T55/M2整体PASS，亦不把diagnostic scopes、普通完成或资源限制扩大。
