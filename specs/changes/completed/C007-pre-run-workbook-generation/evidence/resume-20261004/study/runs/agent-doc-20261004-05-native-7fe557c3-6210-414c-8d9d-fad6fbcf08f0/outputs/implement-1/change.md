可审查：新增冻结时点的当前验收与用户交付卡，最终两项必需检查退出 0；独立 review、deliver、patch 应用与用户接受尚未执行。

# 目标、实际变更与候选

本阶段只新增 `/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/acceptance-handoff/native/specs/guides/current-acceptance-handoff.md`。初始快照没有该 guide；现在首次接手者可分别定位当前源码/已发布线、C001–C006限定闭合范围、原生与MSRV两个951 run、旧215处置、C004真实接续/消费者、C005七null与C006编辑/跨虚拟APFS范围，再按最少入口继续。事实只来自 assigned snapshot，冻结 workspace source `00e010ba8466b6ffc78ba603f5c923174b3d12af`；C007/C008正式真实试用仍not_run，没有混入Root此后实验。

- 初始 HEAD：`351feb7ac22c21317a686693b732d5ae0c4b4bcc`；tree：`80ea3046b1b3575f07e68313444c051ee7c5b7db`；开工 clean。
- 当前本地 candidate：`567b6d3acb6bba50781927ee8cf8de9ed0397347`；tree：`819a00b26f388fd2b98492cbd912942061339c0a`；收尾 clean。
- 最终 staged tree = candidate tree：`819a00b26f388fd2b98492cbd912942061339c0a`。
- 唯一文件 SHA-256：`c73f6d7bc7e386bc2689ef1fd284caa1f36c755d74be89991890971179d6abad`；完整基线diff路径集合只有该授权文件。
- 候选绑定原件：`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-05-native-7fe557c3-6210-414c-8d9d-fad6fbcf08f0/raw/implement-1/committed-candidate-binding.json`。

# 事实依据与使用说明

读取了 task/project、完整 Native implement instruction，共享中文 skill 与受控写作参考，并在 assigned repo 读取 CONTEXT、specsREADME、engineering、C001/C003 README、C002当前最终验收段、C004/C005/C006完整validation、result-export指南。为核字段/命令与精确证据，另定向读 assigned README、Cargo版本、protocol/storage相关段及 C002最终gates/deny/dist、C005 preflight/msrv-execution原件。全部source links用guide相对仓库路径；当前标准/源码/测试未改。

卡片区分134工程、190 C002变异/源码和192后续tracked输入，不混为同域；两个951与各自5doc不相加为新run。215为185动态/21限定静态/9结构，37/148是T43时点分组，不把T58新2helper算入旧215。原stage1 Missed、旧native not_run、LEAK cause unknown和旧工具/缓存/fetch失败保留。C004原B观察偏差与新消费者补验分开，C005无真实撤销事件不造样本，C006代理资料用途与真人/物理/其他OS边界分开。读者先看guide末节，再按当前specs/README、C004、export指南进入。

# 自审修正与旧检查

未绑定previous-review，无review返工。本阶段自审发现初稿误写不存在的 `work resume` 命令；protocol只提供 `work status` 的 `resume` 字段，已更正。首次成功检查对应旧tree `f40774b46e3427f812b9b5bf6e2ffa9239a2b3d3`，原件 retained，不能证明当前candidate；最终字节重新stage与执行两项必需检查，另存final-*原件。没有真实命令失败、没有重跑凑绿或改标准。

# deliver 必须沿用的完整 patch 规则

本阶段不生成最终delivery patch、不创建或写验证副本、不代review/deliver。deliver应从本候选生成完整patch（包括新增文件，不能只取unstaged diff），记录实际stdout文件及SHA：

```bash
git -C /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/acceptance-handoff/native diff --binary --full-index 351feb7ac22c21317a686693b732d5ae0c4b4bcc HEAD -- specs/guides/current-acceptance-handoff.md > <本run声明的完整patch路径>
```

实际 argv：`["git", "diff", "--binary", "--full-index", "351feb7ac22c21317a686693b732d5ae0c4b4bcc", "HEAD", "--", "specs/guides/current-acceptance-handoff.md"]`，cwd 为 `/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/acceptance-handoff/native`，HEAD须等于 `567b6d3acb6bba50781927ee8cf8de9ed0397347`。将完整基线diff路径集合核为exact allow_files；required newfile已经stage/commit，不能把未跟踪文件漏掉。patch限额8388608 bytes，报告各限262144 bytes。

仅在项目声明的 Native 验证副本 `/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/acceptance-handoff/native` 应用。副本须从原始archive/full initial snapshot建立；本阶段没有创建，deliver必须记录真实创建来源/argv，并先核该副本initial HEAD/tree等于 `351feb7ac22c21317a686693b732d5ae0c4b4bcc` / `80ea3046b1b3575f07e68313444c051ee7c5b7db`。禁止在source或opposite arm应用，禁止扩大文件范围。应用顺序：

```bash
git -C /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/acceptance-handoff/native apply --check <本run声明的完整patch路径>
git -C /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/acceptance-handoff/native apply --index <本run声明的完整patch路径>
git -C /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/acceptance-handoff/native write-tree
git -C /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/acceptance-handoff/native rev-parse HEAD^{tree}
```

验证副本write-tree必须等于 `819a00b26f388fd2b98492cbd912942061339c0a`；再逐SHA/字节比较全部授权文件与candidate，记录原件。apply/check未执行不能先授PASS。

# 新增要求、权限和流程边界

本阶段没有新增policy或验收要求。任何新增要求、扩大权限、改变固定方法或候选无法对应必须停止并交协调者，保留原记录。只能本scope；不安装、不改宿主、不push/merge/release。固定implement→review→deliver；implement/review最多3visits、每次最多1真实失败retry，deliver最多1visit。当前implement-1实际完成一次；首次自审字节修正不是独立review或流程返工。required checks每条120s且受本run统一deadline `2026-10-03T21:27:02.004791+00:00`，失败/超时保原件停，不换标准。

# 限制与未完成

check-docs只核该1文档的机械规则和链接，cached diff检查只核最终staged空白；零退出不代内容审查。Rust/build、动态演练、引擎/Store、安装发布、独立review、deliver完整patch独立应用、最终质量及用户接受全部not_run。本卡不改变产品合同，也不将document/package completed授作用户接受。真实原生/MSRV/副本事实仅引用冻结source原运行，不声称本trial重跑。

实际helper身份 `/root/c007_study_coordinator/run05_implement`；继承声明gpt-6.1-sol/high，无override、无spawnhelpers。runtime模型精确元数据未由本worker独立读取，Root另核；usage/费用/globalhuman/modelhistory未知。完成时UTC由本阶段原件记录，不补造此前读命令起止。只读命令未通过capture时仅引用真实工具chunk/session待Root提取，不伪造分流stdout或历史开始时点。
