# M1 全链审查证据

基准：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`；原实现候选：`ca6d92fa83494feeb9ea55840bfd4bca5f14629c`。工作区起始干净，本轮按M1与追加任务T32–T34白名单执行。

独立审查发现的文档矛盾、接口冗余及R21根解析错误见[审查记录](../../review-m1-2026-10-01.md)。逐行证据见[矩阵](coverage-matrix.md)。[T32](t32/README.md)和[T33](t33/README.md)分别记录精简、修复及真实caller回归；T34已修复R22–R24，最终699项与工程门禁见[T34](t34/README.md)。T33旧变异输入见[归档](mutants/superseded-before-t34/source-input.json)，新的完整M1变异已冻结为`95d78e0`，输入见[当前manifest](mutants/source-input.json)，基线699/699、零skip，inventory2514（core743/runtime1771）；首轮与能力分组执行已完成，最终逐ID核算与限制见下方。各阶段输入和原文分别保留，历史结果不冒充新输入运行。

[gates](gates/gate-results.json)保存首次候选正常门禁；运行期间本轮实施开始，因此后续输出只作阶段证据。[final-gates](final-gates/gate-results.json)和`final-governance-nextest.*`是T32/T33之前的阶段证据。T33历史基线为693/693、零skip；T34门禁为699/699、零skip。完整变异验证尚有明示缺失，最终M1按实际授权例外完成。全仓Nextest覆盖真实CLI/runtime、确定性交错与exit70/SIGKILL窗口，T33追加精确观察点回归。

离线deny第一次因公告缓存路径只读而失败，原文[保留](deny.stdout.txt)；取得缓存锁权限后的[运行](deny-approved.stdout.txt)exit 0，仅证明本地已有公告数据，不声称公告库实时更新。

用户已授权恢复原269项暂缓验证，只有实际出现平台安全提示且暂停的具体项目才记录并跳过。[实际跳过清单](safety-skips.json)记录了SK01独立Spec后续复核暂停，按授权跳过、不重试、不记Spec通过。Linux仍`not_run`，T16/T17仍`not_run`。当前完成状态以plan.md和下方最终验收为准。

早期治理闭包复验：[原文](final-governance-nextest.stdout.txt)675/675、零skip、exit0；[元数据](final-governance-nextest.metadata.json)绑定当时candidate-input SHA。[源码复核](source-closure-recheck.json)对应当时169项源码/配置/fixture。最初未添加T31已备nextest工具路径的调用因系统0.9.140不满足仓库0.9.145要求退出92，原文`old-tool-version.*`保留；正确PATH前缀指向已存在工具，不安装或更新宿主。

## 2026-10-02 验证提速

按用户要求停止自动逐项全workspace队列，改为精确处分证明、共享直接oracle与能力测试组，影响不明者仍做全workspace。2514项第一阶段全部完成；core17与runtime135项workspace完整原文保留原run身份。先前进程中断的29项、此次取消队列的18项都仅作未完成WIP，未计正式通过；见[恢复原文](mutants/continuation-2026-10-02/closure.json)与[提速决策](mutants/adaptive-validation-2026-10-02/decision.json)。

已完成5个单项变异的实际CLI补充检测，见[独立审计](mutants/supplemental-mutants-independent-audit.json)；另1个末次SIZE校验由单独绑定的受控观察副本验证，见[独立审计](mutants/supplemental-late-seal-size/independent-execution-audit.json)。这些结果单记补充检测，不改写正式Missed。此段记录分组验证的早期进度；当前最终验收见下方，SK01仍不记Spec批准。


## 2026-10-02 最终验收入口

[最终审查](../../review-m1-2026-10-02.md)结论通过，限定实际授权例外内的macOS源码与离线验收。[当前门禁](current-acceptance/gates/gate-results.json)699/699、零skip；[账本](mutants/adaptive-validation-2026-10-02/final-ledger.json)2514项逐ID核算；[Reviewer持久记录](mutants/reviewer-records/relocation.json)保留原文与SHA。SK01缺最终Spec批准，SK02缺215项额外执行；[验收范围](acceptance-scope.json)保持完整验证与安全验证false。所有运行队列已结束或停止，历史WIP不计通过。M1在plan标done，Linux/T16/T17保持not_run。
