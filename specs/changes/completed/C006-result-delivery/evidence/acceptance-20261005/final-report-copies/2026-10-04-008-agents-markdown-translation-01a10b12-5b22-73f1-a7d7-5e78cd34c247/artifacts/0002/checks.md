修后必需检查PASS；全量语义72篇资格按精确SHA复用，F002关闭仍待独审。

新tree `e25913bc8b53cdc7d749f823dcf80dd4147faa19`，相同baseline。attempt-02/input-closure.json核唯一delta及全部218路径hash/mode，无source漂移。73结构/链接、scope及实际Ruby YAML.frontmatter解析各exit0；新增parsed机器值检查针对description可译但YAML仍须可解析，73target合法且原machine values不变，无新增依赖或安装。

freeze_candidate隔离index，完整patch 270746bytes/SHAff415138db9f938235174e5aa648a6a24d01f0bebfb86ee663c7ce5314ee7a1a，71white changed（68new/3已有修）保持。apply_check.py新freshbaseline refreshstat后实际apply --index得到新tree，218file bytes/mode同；immutable archive内proof和attempt-02实际argv/cwd/stdout/stderr/exit。旧2b31/cache-stat失败/所有各batch首失败保持。

72个已全文语义PASStarget除F002外sha均相同；formal review#1正常需修改已封存SHA dcea40b05750fddf63843afb74aaea45005e1cbee695afdfce69a25880ac17b3。不将机器PASS覆盖旧语义FAIL。只运行受此次候选/新frontmatter关注直接相关检查，无无关Rust/网页重跑。

原实现checks只读保留如下，不把其旧candidate当本次新candidate。

必需机械检查通过；语义独审尚未完成，不授真人接受/发布通过。

固定候选 `2b31c7ad8b9955e04672b4d8010c2220210c9d48`；默认Git baseline/ref/index不变。

- 全73结构/字段/fences/inline/URL/href/HTML/anchor检查：python3 control/verify_translation.py，exit0，targets_checked73/errors[]；actual argv/cwd/stdout/stderr在full-verify_translation.json/.stdout/.stderr。
- 范围/源bytes/mode：python3 control/verify_scope.py，exit0，73targets/218actual文件，145不变源/资源，5原target的backup先冻；full-verify_scope原件和inventory。
- 完整patch：freeze_candidate.py 使用隔离candidate.index，baseline read-tree，仅add73target，diff--binary--full-index含新增与已有修复，71变更paths，无遗漏，无越界；270746bytes/SHA495996781e8fb2abf473f2c6c52d51000fea0fc4220af6fc9b67331b6b6cdd07，candidate-proof.json。
- 独立apply：apply_check.py自有clone baseline，仅apply--index完整patch，tree精确同候选，218bytes/mode全部一致；immutable archive/independent-apply-proof.json，full-apply-check-02原件。首次apply失败保留full-apply_check及independent-apply-9oqunc4u/4.stderr：checkout-index未refresh缓存stat；三旧target实际bytes等index blob，refresh exit0证明确属检查setup缺陷；只补refresh后新自有副本检查通过，不覆盖失败。
- 全输入资格：full-input-closure.json 保存218路径hash/mode，源码baseline与全部73target当次manifest前后无漂移。all3 writers ended。无Rust/网页输入，无关测试未重跑。
- batch1初次inline order失败、batch2两次inline order失败、batch3首失败，原stdout/stderr/exit留在各batch检查目录；修复无删源/断言/原件。Root语义修复仅一个字，受影响29机械闭包再PASS。
- /root/translation_semantic_review已独立核第一批29篇PARTIAL_PASS，首finding禁止范围已修并核。其余44需逐篇完成且所有target SHA同最终tree后才授Review PASS。

个人分钟/费用unknown；真人阅读/外部写入/dsh=not_run；不运行原配对ROI或把本轮替代旧C007义务。
