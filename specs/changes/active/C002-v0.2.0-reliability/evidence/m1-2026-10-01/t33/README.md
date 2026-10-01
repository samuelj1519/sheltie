# T33 根解析修复与M1真实caller回归

实施Owner：Codex。基准T32 `326cbbb`；独立Spec/Standards Reviewer未修改本任务文件。当前完成状态只看[plan](../../../plan.md)。

## R21 根解析错误与替代路径

未变异的[权限反例](home-permission-red.txt)失败：Home::resolve吞PermissionDenied并返回原路径。独立审查确认这违反C002 design对缺祖先/权限的区分。修复canonicalize_deepest和confine：只有NotFound可按各自规则退让，其他I/O保留路径/原因；canonical/cwd/显式环境值不得用lossy字符串改选目标，非UTF8的SHELTIE_HOME不能被当缺失改选HOME。删除仅测试调用的Home::at，全caller统一真实resolve。输入文件真实父目录及current_exe也严格转换，拒绝替代路径；lexical_abs统一CLI输入/source和runtime路径词法展开，当前目录不可获取或非UTF8时失败，不猜测请求指纹。架构/storage同步明确规则，不新增兼容层、格式或业务状态。

[Home绿色原文](home-green.txt)覆盖权限、环境变量非UTF8及显式参数优先序；[confine](confine-green.txt)覆盖真实存在但祖先不可访问的base及合法对照。实体非UTF8目录fixture在本机APFS创建失败EPERM，[原文](fixture-non-utf8-path-unavailable.txt)保留为环境限制，不当平台安全拦截，不宣称该物理路径情形实跑通过。严格转换的代码/API义务由独立review核验；Linux仍not_run。

## Caller回归与同步点

普通调用链覆盖revision=0只读/写拒绝、无关Workbook删除不影响邻居、缺manifest准确Tampered、历史snapshot/audit单字段漂移、remote asset恰32MiB/+1与非法版本在传输前拒绝。fake curl检查自身FD不继承父引擎锁；只替换传输，不联网。

新增同步点只通过既有failpoint feature生效，位于regular/directory stat与open之后、source snapshot stat前后、SQLite打开之前。真实子进程验证同卷inode替换、链接、FIFO、Root/Store合法对象替换；双阶段API验证检测到替换后即使原件恢复也必须拒绝、同inode临时别名不能被跟随。独立oracle来自原bytes、SQLite行数、权限、明确错误码及完整合法对照；不用私有checker生成expected。

线程RendezvousWorker拥有两release和JoinHandle，正常及panic均先释放并join再disarm；同步点不以sleep猜窗口，子进程由既有Process守卫回收。没有引入生产worker/thread框架。

入口原文：[普通caller](caller-drift-green.txt)、[remote](remote-green.txt)、[本机边界](native-boundaries-green.txt)、[Home/观察](home-green.txt)。fixture签名/只读连接/非UTF8创建错误保留，不当产品失败；唯一真实Root失败为R21。

## 变异输入边界

先前5261e0d/2495 inventory的基线和core4/首runtime片原文完整保留；根源码检查在后续片启动前检测T33变化并停止。旧结果不能转成新输入PASS；这不是安全跳过。深度逐ID静态审查只区分严格等价/当前producer与待验证，不当caught。新候选需工程门禁、独立review和任务提交后重新冻结；最终完整变异由M1执行。[相对输入cwd回归](lexical-cwd-green.txt)通过真实CLI删除仅供本子进程使用的cwd，准确保留current_dir错误且无Work/序号/新请求；同组重放历史回归通过。当前未观察到安全平台拦截，实际跳过记录为空。
