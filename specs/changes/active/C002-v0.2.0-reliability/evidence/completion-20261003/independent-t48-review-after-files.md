# C002-T48 业务字节补齐后的独立增量审查

结论：**通过**。F-T48-01 已关闭；五条新增 oracle 的合同期望、实际触达、冻结后 19 项辨别力和完整工程门禁均通过。初审 `independent-t48-review.md` 需修改原文与旧17运行 source_unchanged=false 保留，不反写为稳定候选。此次 PASS 只覆盖 T48/G01 这19个精确当前ID，不批准旧 candidate 或全部215，不关闭其他平台/真实价值/M2。

Reviewer：`Codex /root/oracle_review`。未参与设计、oracle、实现或本次修正。只读仓库和已完成原文，没有编写测试、改源码、构建或重跑；仅写独立报告及复核 JSON。

冻结基准：`d27816be3e919b8183abe04b7f7ca4f457046231`。实际候选源码 SHA256：

| 文件 | SHA256 |
| --- | --- |
| `crates/sheltie-cli/tests/snapshot_qualification.rs` | `1738f0803502b2680b6948d7cf6c841c464be82fe0389658e5a0c989a61855d3` |
| `crates/sheltie-runtime/src/store/read.rs` | `9e1c319a9b8bb67511fe7abe52ca4bd316f3459f80e0c9ec5855613f3bdcd3fc` |
| `crates/sheltie-runtime/src/effects.rs` | `7c8fad349738bdf64e883162ace75e753cc4489f6c01f16647920402a768ab9f` |
| `crates/sheltie-runtime/src/load.rs` | `513907d5809a9ce7dd416cfffcf79797b95d1df0f6ae3443224833c0c72be8b8` |

## F-T48-01 的修正

新增测试专用 `business_files` 独立遍历 works/workbooks/pending，记录完整路径集合、每个对象的 mode 和原文件 bytes。`Some(bytes)` 与 `None` 区分文件与目录，symlink_metadata 后拒绝链接及特殊对象。它不调用生产 Store/效果/摘要 helper 生成期望。

两条 CLI 用例都在调用错误重放前保存原树、拒绝后完整相等比较；移走的 `retained-workbook` 仍位于 works 树，因此冻结 manifest/Flow/instruction 的原字节和权限均被覆盖。增加、删除、内容、对象类型或权限改变都会失败。SQLite 主库/WAL/SHM、引擎锁控制对象不混入业务字节声明，Store 五表完整原行继续独立比较；tmp 不是本报告的业务成果集合。此证据不声明 inode、物理持久性或任意并发外部编辑安全。

## load 两条新增纯契约的独立性

合法输入由真实 WorkbookRepo.add、WorkService.start/begin、真实输出写入、submit、下一节点 begin 产生；有效 two-step 默认 required 定义来自已采用 Workbook §3.2/§4。合法状态先被 `validate_work_paths` 接受。

namespace 反例同时改 Work 起始输入与 outline 冻结引用为同一个根内 alias，排除仅因二者不一致早退；require_path→Ok 的实际突变在 `wrong_namespace` 断言失败。required 反例去掉原 outline 输出、把 summary 绑定置 None，保持 None/None 形状；!required guard→true 的实际突变在 `missing_required` 断言失败。它们是同一业务绑定事实的联动反例，不记成单字段端到端证明。

两次坏状态只在内存里交给私有 `validate_work_paths`，原 Store state_json 不写回；合法真实 producer 使输入来源可信，目标突变证明该 helper 内先决守卫没有遮挡。但完整 CLI 的上游资格可能更早拒绝这些 corrupt states，故这两项只归私有原语辨别力，不冒称完整历史重放链事件。Publication/metadata 的纯检查范围和真实 CLI 的 original/data 资格范围保持初审说明。

## 冻结后的完整动态证据

- 五条 oracle baseline：run `c6b00042-58ca-40be-8c63-02db9dc8ae35`，5/5，559 项因过滤未选。两条 CLI 函数有四种同步有效形状的业务绑定条件和两种身份矛盾条件；其余三个函数属于纯契约，不能按函数数宣称全链场景数。
- `mutation-g01-final/result.json`：exit0，359.282s，baseline Success1、CaughtMutant19，0 missed/timeout/unviable。outcomes 结束于 `2026-10-03T04:50:37.529226Z`。选择集合与原 G01 普通94测试后仍 Missed 的19项完全相同。
- 独立逐 log 复核19项都 Build Success、Test Failure100 且有真实 panic/断言失败；非编译失败或空过滤。分布为 Publication7、load2、service4、snapshot2、metadata4。CLI 的错误码相同但泄露伪造 original 仍被准确捕获；纯 snapshot 矛盾被移走冻结副本条件准确区分读顺序。
- 最终完整门禁：fmt/check/clippy/nextest/doctest 全 exit0。Nextest run `aec12b7c-fead-400c-a1e0-eaf665c12648`，853/853、0 skip、3 slow，test 阶段185.594s（整命令207.718s）；未见 LEAK。doc 命令为 workspace `cargo test --doc --all-features --locked`，core 的5个 compile-fail 保留，其他 crate 0 doctest 不冒称新增覆盖。

`independent-t48-final-audit.json` 保存复核细节：mutation selection 的全部源码当前哈希无差异，134份已登记工程输入当前哈希无差异，五份门禁原文 SHA256 全匹配。mutation 和 gate 记录的 source_unchanged=true 与独立核验一致。三个 runtime 文件原源码全部逐字保留为 cfg(test) 追加前缀，无生产 AST/行为变化。examples/workbooks/Cargo.toml/Cargo.lock/nextest config 当前与基准无 diff。

可重跑准确 argv、filter 和 SHA 分别在 `mutation-g01-final/{selection,result}.json`、package evidence 的 `t48-final-gates.json`；原输出在 `mutants.out/log/` 和 `t48-final-*.txt`。旧17来源漂移运行、初次 compile/fixture 错误原文继续保留，不改成产品红例或正式旧 Missed 结果。

无剩余 T48 oracle/源码审查 finding。docs/specs/测试治理、任务状态/白名单/提交格式由执行者在任务收尾核验；本报告不预判尚未执行的提交动作。用户当前采用范围为全自动、macOS aarch64；无人 trial 和平台排除不改记真实价值或跨平台资格 PASS。
