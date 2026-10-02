# C005 验证

Candidate: `none`

状态：`proposed`；全部实现测试、实验和实施里程碑 review 为 `not_run`。采用后在执行前固定候选、Workbook/图摘要、配置、工具链、测试过滤器和环境；结果、模式与证据分开记录。

## 1. 机制验收矩阵

| 要求 | 合法例 | 只改一个条件的拒绝例 / 故障例 | 归属 |
| --- | --- | --- | --- |
| 原子替换 | active/latest running，同 Occurrence 创建新 Attempt | 目标不是 latest 或状态非 running，完全无变化 | T02 |
| 原生执行记录 | core 同 Decision 关闭旧 open 为 interrupted，completed 保留；runtime 同事务提交 | 旧 runner 迟到完成 CAS 拒绝；新 Attempt 不继承完成资格 | T02 / T04 |
| 输入冻结 | 新旧非 engine.stats 输入引用完全一致 | 原引用文件摘要改变，整次拒绝 | T02 / T04 |
| 顺序号与失败数独立 | 2 次替换后 number=2，failed=0；再 fail 使 failed=1 | max_retries=0 的第 1 次 fail 必须阻塞 | T02 |
| 替换有界 | 上限 1 接受第 1 次；上限 32 可表示 | 第 2 次拒绝且旧仍 running；字段 33 拒绝 | T02 |
| 旧提交资格 | 新 Attempt 正常 submit | 旧 submit / fail 都 ATTEMPT_NOT_RUNNING | T04 / T06 |
| 并发 | 单次 replace 完整提交 | submit 与 replace、两次 replace 竞争至多一个成功 | T04 |
| 相同请求 | 同 request-id、同 reason 参数返回原新 Attempt | 改目标或理由参数 REQUEST_CONFLICT | T04 |
| 历史字节 | 重放不需要原 reason 文件；brief/stats 逐字节相同 | 损坏字节不覆盖，不重新生成历史 | T04 |
| 历史资格 | 新 Attempt 后来成功或再次替换，原 replace 可重放 | 快照新旧身份/输入/requires 与原审计不符，拒绝 | T04 |
| COMMIT 前 | 普通替换成功 | 提交前终止，旧 running、无新尝试/成功请求 | T04 |
| COMMIT 后 | 恢复一个新 Attempt 与原 brief | 发布失败准确 EFFECT_PENDING，恢复不再次替换 | T04 |
| 当前状态卡 | 旧请求重放后卡保持最新 revision | 历史 next 不能写成当前 next | T04 / T06 |
| 系统身份与时间 | 固定系统测试边界提供非默认时间/主体 | USER 改动不改变审计主体；CLI 不接受时间注入 | T04 |
| 版本边界 | 新格式完整读写 | 旧 Store 拒绝且原库业务字节不变 | T04 |
| 普通恢复 | 原会话重开后原 Attempt submit | 不得强制 replace 或人工等待转 blocked | T06 |
| 候选与门槛 | 替换后同候选验证、原 gate 仍需批准 | 替换不能抹掉未完成检查或绕过门槛 | T06 |

## 2. 真实接续实验

目的不是证明跨宿主必然优于同宿主，而是确认交接视图已经承担信息恢复，replace 只解决撤销提交资格。选择确有继续工作的真实任务；不为证明功能人为耗尽账号额度或制造第二个宿主。

对照 A：原会话恢复或新会话继续原 Attempt。实验 B：同类任务中确需撤销旧资格，明确旧进程状态后显式 replace。两者使用同一 Workbook 标准、候选要求与交接材料。必要的子进程故障注入只计机制测试，不计真实价值。

| 指标 | 记录口径 |
| --- | --- |
| 重新解释 | 接手者额外要求用户提供的事实及人工分钟 |
| 约束保持 | 冻结标准、输入、候选身份、门槛逐项比对；关键缺失直接失败 |
| 重做 | 接手者重复已完成工作的步骤与分钟，注明必要重新验证 |
| 撤销收益 | 哪个迟到提交风险被正式接口拒绝；是否确实需要新 Attempt |
| 总投入 | 用户操作、两次会话和必要核对的总时间，不只比较某个命令 |
| 副作用范围 | 旧进程是否停止、共享工作区是否发生写入；不宣称引擎隔离 |

单次成功只证明该场景可用。没有真实撤销需求时实验 B 保持 not_run，结论是暂不扩大 C005，不把普通恢复成功当成替换需求成立。

## 3. 验证记录

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| core 合同与计数 | not_run | 待 T01 固定 | 未执行 | not_run | 无 |
| Store、幂等、故障恢复 | not_run | 待 T03 固定 | 未执行 | not_run | 无 |
| CLI 完整调用链 | not_run | 待 T05 固定 | 未执行 | not_run | 无 |
| 真实恢复对照 A | not_run | 待实际任务 | 未执行 | not_run | 无 |
| 真实替换实验 B | not_run | 待实际撤销需求 | 未执行 | not_run | 无 |
| M1 / M2 / M3 独立 review | not_run | 待固定各阶段候选 | 未执行 | not_run | 无 |

`executed`、`reused`、`covered` 是执行模式，不是 PASS。复用测试必须保持完整输入闭包并给原 run ID；超时、中断、不完整结果均不能写 PASS。实际实验失败回实施 Owner，不修改预定成功标准。

## 方案静态核验

文档闭包：`21e36675dd42658809bc9c793f498961665f88df5e26db1bc1b573a208eedec9`，范围及独立审查见 [review](review.md)。链接/措辞、change结构、任务表与TOML/白名单对齐已核验；测试治理检查只核现有测试声明与归属，没有执行产品测试。以上不计入产品Candidate或实验PASS。未来实现、M里程碑、用户实验和平台仍保留上表not_run。
