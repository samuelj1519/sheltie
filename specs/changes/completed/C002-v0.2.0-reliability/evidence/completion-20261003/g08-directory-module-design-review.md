# G08 目录身份纯 module 设计复核

**方向比例适当，可作为当前候选合同的验证路径；不是已实现PASS，也不直接批准三旧ID、T55或M2。** Reviewer只读，未改tracked。按codebase-design的in-process/internal seam原则，小module不需adapter、trait、公开类型或新状态。三个真实caller共享同一身份规则有locality收益；不把数行predicate夸称大架构deep module。

当前三处判定均为kind非Directory或dev/ino与held不同。提议pure Interface接kind与actual/expected两个身份tuple，caller拒绝!helper。在相同captured输入及保等cast下，De Morgan证明与原拒绝式点值相同：只有Directory且dev/ino均相同接受。actual必须来自该named-entry stat，expected来自该held-object fstat，不能误传root_ident；检查target平台转换无narrowing/转置。原锁、statat→fstat、error类别/路径/包装、rename/chmod/unlink/fsync、前后复核次序全部保留，不顺手增加同root设备政策或假定submount不可达。

独立pure测试必须手写语义结果，不调用helper/生产表达式计算expected。覆盖Directory/nonDirectory × dev相等/不同 × inode相等/不同8关系，明确**Directory、同数字inode、不同dev=false**；非Directory同身份false，同身份Directorytrue。零/nondefault值按原合同，不发明保留值规则。tuple是pure Interface真实输入，不声称OS返回该stat，因此不是fake fstat/假ManagedTree。这样可以验证共享判定合同；另须逐caller接线/同式证明与真实I/O回归，pure表本身不证明读取窗口、锁、error包装、原件/副作用。旧真实I/O tests不是其严格子集，必须保留，不机械套「replace don't layer」删除不同义务。

删除旧表达式不能把旧首connector mutant变成Caught/等价。它的错误式(A&&B)||C与原式不同，native不同dev同ino窗口仍未建立。每旧ID应保存source/diff SHA、MissedMutant原标签与native not_run，登记**结构替代并由共享新合同验证**：明确旧表达式→caller→helper SHA、三处同式证明、独立Interface oracle/真实非零基线/new current机制辨别力run及冻结闭包。同一次shared新机制执行可被三caller复用，但不能计作三次历史native执行，也不能给old dynamic caught=true。

**不能仅登记三首connector。** 整个A||B||C迁移还删除每处第二个dev||ino及可能比较变体/映射上下文。扫描全部受影响旧inventory，逐项区分准确current可运行target与结构替代；不能给消失的次connector套旧行号。全caught数字不代替同ino/differentdev目标关系与参数来源证据，newhelper较宽变体失败也不能反写old3 Caught。

如果里程碑明定每旧ID原native扰动实际执行，pure seam不满足，缺口保持open；若采用结构机制验证路径，必须在active package明确更新验收并独立审查，不能静默改义务。若验收目标是当前候选履行原目录联合身份合同，同式迁移+完整pure合同+真实I/O/gates闭包可以足够，不需虚构native反例或引入平台/通用observer。现阶段仅设计与代数评估，代码/oracle执行/最终门禁未做，所有native缺项如实保留。
