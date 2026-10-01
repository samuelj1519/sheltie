# M1 全链审查证据

基准：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`；原实现候选：`ca6d92fa83494feeb9ea55840bfd4bca5f14629c`。工作区起始干净，本轮实施仅限M1白名单。

独立审查发现的文档矛盾和接口冗余见[审查记录](../../review-m1-2026-10-01.md)。最终逐行证据见[矩阵](coverage-matrix.md)，输入闭包见[candidate-input.json](candidate-input.json)。原实现候选与本轮精简候选分开绑定，不复用原变异输入。

[gates](gates/gate-results.json)保存首次候选正常门禁；运行期间本轮实施开始，因此后续输出只作阶段证据，不作为最终闭包PASS。[final-gates](final-gates/gate-results.json)是源码精简后正常门禁，治理文件最终变更另行复验。全仓Nextest覆盖原有真实CLI/runtime、确定性交错与exit70/SIGKILL窗口；不新增此前暂缓的安全插桩或变异复验。

离线deny第一次因公告缓存路径只读而失败，原文[保留](deny.stdout.txt)；取得缓存锁权限后的[运行](deny-approved.stdout.txt)exit 0，仅证明本地已有公告数据，不声称公告库实时更新。

Linux `not_run`；269项`deferred_by_user`；T16/T17 `not_run`。M1当前结论需结合完整变异门槛与本次范围澄清，不把T31 done或源码审查通过转为完整M1通过。

最终治理闭包复验：[原文](final-governance-nextest.stdout.txt)675/675、零skip、exit0；[元数据](final-governance-nextest.metadata.json)绑定candidate-input SHA。[源码复核](source-closure-recheck.json)169项源码/配置/fixture未再变化。最初未添加T31已备nextest工具路径的调用因系统0.9.140不满足仓库0.9.145要求退出92，原文`old-tool-version.*`保留；正确PATH前缀指向已存在工具，不安装或更新宿主。
