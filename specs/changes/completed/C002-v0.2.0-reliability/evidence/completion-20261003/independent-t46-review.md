独立增量结论：**PASS**。F-SPEC-01 的现行修复与工程规范勘误通过，无剩余必改。本结论限定历史 Command/data 解码顺序、原响应资格及受影响消费者，不批准全部 C001–C008、全部变异、安全、跨平台或用户价值。

审查者 `/root/completion_spec_review` 未参与本修复或新增测试编写。候选为 HEAD `8167165a5a707bbb51a29fd1aa1047f387c9e9d3` 加当前 T46 工作区 diff。实际代码、测试、工程规范、Cargo.lock、默认 nextest 配置及主要独立 oracle 的 SHA256/尺寸见 `independent-t46-inputs.json`。修复前审查与真实反例保留在 `independent-spec-review.md`，不覆盖旧 FAIL。

已逐处核完整生产 diff。`service.rs` 在 `self.load` 前严格解码完整 `Command`，并经 `snapshot::check_data` 解码完整 data/核 Reply-data 身份。装入冻结图后直接复用 `validate_command_owner_data` 核业务归属；旧仅转发解码的 `validate_command_owner` 无剩余 caller 后删除。不新增载荷入口、默认值、格式、状态或装入层。原响应生成、效果读取/检查及 RequestLoadError 分层路径保持不变。

`engineering.md` 现明确 Command/成功快照先于冻结业务读；效果完整解码与闭包校验先于效果动作。只读冻结定义仍可用于原响应资格，合法效果错误保留已核 original。该边界与 protocol §5、storage §3.2 一致，没有放宽未知字段接受集合或跳过完整效果验证。

新增 `crates/sheltie-cli/tests/strict_state.rs:277` 用真实 CLI 临时 Home 建合法历史 begin，先核合法同 rid 重放与完整持久内容不变，再逐一构造以下四种反例。Command/data 每次仅加 `unexpected_contract_field`；冻结目录另行保留以识别是否提前业务读。旧 A 场景仅将该已提交请求的 published 改为 0。

| 情境 | 独立核查的结果 |
| --- | --- |
| Command 未知字段＋本人重放 | EFFECT_PENDING/STORE_CORRUPT，message准确含未知字段；committed=true/request_id=A；无original和revision。 |
| data 未知字段＋本人重放 | 同上，data纯严格解码先于冻结读。 |
| Command 未知字段＋旧A阻断新B | committed=false/request_id=B/detail.pending_request_id=A；准确STORE_CORRUPT；无original、pending_original和revision；B没有登记。 |
| data 未知字段＋旧A阻断新B | 同上，不能将坏历史快照投影为旧A的可信成功响应。 |

`persisted` 的独立 oracle 调用 common/store 的 `SELECT *`，保存 workbooks/works/work_sequence/requests/audit 五表每列原 SQLite 类型与内容；同时递归保存 works/workbooks/pending 的全部目录/文件原字节。拒绝前后完整比较，包括保留的冻结目录、brief与pending原件，而非只断言目录存在。期望来自原请求、手写合同值和坏字段，未调用被测装入/renderer计算期望。

已核作者真实 red run `e423d628-cba5-40e8-9c7e-0cbe544c66f0`，旧实现实际在未知Command前报缺冻结副本，不是编译红。作者 green 及扩展四情境 run `07b34410-ac28-4da3-a7da-229ac382cef9` 保留。两个原输出不替代本次独立执行。

本次独立使用 nextest 0.9.145、默认 profile、locked/offline/all-features、空 RUSTC_WRAPPER 与独立 `/private/tmp/sheltie-spec-review-target`：

- strict_state：5/5 PASS、0 skipped，run `62e51347-d761-4ab0-b74f-e01b8b88c25a`；包含上述4反例与既有嵌套拒绝链。原文 `independent-t46-strict-tests.log`。
- effect_contracts：4/4 PASS，run `92f34369-78cf-49cb-9449-17e7f87faf0e`；13 filter skipped。核 begin合法快照的损坏content/hash、缺效果；submit封存引用/形状；Workbook完成历史形状及身份。`assert_effect_pending` 明确要求已提交合法快照的 original 可严格解析，另一原响应字段为空；专项断言核原data/revision，并比较完整行/原bytes/permissions不变。原文 `independent-t46-effect-tests.log`。
- docs/specs/tests/diff检查 exit0；测试治理为843测试/204任务卡。未代替主agent的完整稳定候选工程门禁；提交时仍须固定最终输入和task门禁。

原审查的R22–R24五项CLI与八项runtime历史/资格消费者，候选和原run仍以 `independent-spec-review.md` 为准；本次只重跑受解码顺序直接影响的消费者，不宣称全部旧运行输入相同。

**SK01 补验范围：现行独立审查通过，旧候选历史不回写。** 本次接续先前当前候选Spec审查，修复其唯一明确生产缺陷。已核当前 original/pending_original、Add target、历史状态/失败前缀、冻结节点requires、严格装入与副作用/恢复关键链，现行这些受审合同无剩余必改。旧 `e3eea899877165f8573befee3774555598ec92bd` 的SK01原暂停、没有最终批准仍是历史事实；本次当前候选PASS不能登记成旧e3eea89的最终整体批准。SK02的215项执行、其他平台、LEAK及用户价值需要各自实际闭环，不随本审查变为PASS。
