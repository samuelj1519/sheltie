# C004 设计

状态：`proposed`。新增路径与命令均为拟实现内容。根合同同步由 T03 完成，之后初级实现者按阶段骨架执行。

## 1. 架构与真实入口

```text
协调者 → sheltie-cli → sheltie-runtime → sheltie-core
                  └→ sheltie-verify（私有 library，无独立二进制）
```

core 仍是纯状态、定义和引用规则。runtime 提供短事务、受管文件句柄、观察记录和效果恢复。`sheltie-verify` 只处理 Git 快照、冻结检查配置、子进程与工具报告；不直接写 Store，不选下一步，不依赖 runtime。CLI 编排两条调用链，不直接读写文件。一个具体工具模块即够用，不新增 runner trait、环境平台或宿主插件接口。

现有入口：`core/flow/{def,parse,compile}.rs` 定义与编译节点；`core/work/{command,decide,state,next,layout,render}.rs` 管运行事实；runtime `WorkService::run_command/commit/replay`、`Store::commit`、`RequestIntent`、`PersistedResponse` 管短写与快照；`fsx::SafeFile`、`observe_loaded_input` 管同文件观察。当前没有工具执行入口，不能把下述职责误写成已经可复用的现成接口。

## 2. 冻结检查配置

配置为普通 `start.validation` 文件，格式 `local-check/v1`，由工具 crate 严格解析；core/runtime 只冻结文件引用，不理解 Git。最小字段：

| 字段 | 内容与约束 |
| --- | --- |
| `repository` | 已授权本地仓库的绝对路径；仅作读取源和定位提示 |
| `baseline` | 完整基线 commit，Git 解析后的完整标识必须一致 |
| `protected_paths` | 基线中必须保持字节的显式相对文件路径，不能从自然语言推导；不支持 glob |
| `checks` | 1–16 个唯一 id 的检查；每项是绝对 `program`、参数数组、可选相对 `script`、显式环境变量和超时 |
| `timeout_seconds` | 每项 1–600 秒；整批最大 1800 秒，按实际运行时间截止 |
| `max_log_bytes` | 每项 stdout/stderr 各 1–16 MiB；达到上限后停止并记未完整验证，不截断成成功 |
| `unverified_conditions` | 有界文字清单，说明机器命令覆盖不到的标准，仅给协调者与用户读 |

每条命令的 cwd 固定为本次 scratch 的 commit 检出根。需要项目脚本时通过 `script` 相对路径声明，必须是该 commit 内的普通受保护文件，作为解释器的首个文件参数；其余 args 是冻结字面参数，不允许另指原仓库、外部或未跟踪脚本，不使用 `-c` 等内联脚本入口。程序自身加载的工具链与依赖不被冒充冻结脚本。无法机械确认的隐式配置依赖列为未冻结前提，首版不能宣传完整标准闭包。实际运行前后在同一快照上核 script 与 protected 文件字节，不能只核源仓库。

环境从最小空环境加配置的显式键构造；引擎内部控制变量不注入候选进程。外部工具链、缓存、服务、凭据和未冻结依赖在报告中列为环境前提，不宣称它们随 Work 冻结。报告默认只显示显式环境键名，值全部隐藏；原始命令日志可能包含项目工具输出，不对其作完整自动脱敏承诺，实验与分发证据由 Owner 检查后保存。原始冻结配置仅保存在用户授权的本地范围。

首版没有额外文件覆盖和保留样本。需要未跟踪资源或外置内容的任务不在该快照合同内，准确报告缺前提。候选相对 baseline 改动任何 protected 文件（含删除、重命名和类型变化），不执行命令，报告标准发生变化；用户重新确认配置后以新 Work 开始。未知字段、重复 id、越界路径、无界超时和无可运行检查均拒绝。

## 3. 快照和命令

`sheltie verify run <work> --attempt <id> --candidate <完整commit> --profile-input validation --output check --json` 是唯一公开原生检查入口；`validation` 必须是当前 Attempt 已冻结输入，`check` 必须是该节点声明的工具输出逻辑名。不接受 report 文件、来源标签、退出码或外部回填参数。

工具先在只读源仓库固定完整候选身份，再在引擎提供的自有 scratch 中建立不共享源对象的完整本地 clone，检出并核完整 commit。源 HEAD、分支随后变化不改变本次对象；复制期间缺对象或快照无法闭合时失败，不退回新 HEAD。拒绝 submodule、LFS 外置内容和不能按普通文件/目录封闭的树；文件内容摘要、执行权限和符号链接拒绝策略由 T03 固定。源 `.git` 不新增 worktree 或引擎提交。

固定命令顺序执行，正常非零后可继续其余检查；未启动、超时、日志超限、信号中断或准备失败时停止整批，尚未运行的项标未运行。退出码、信号和启动失败来自实际进程；非零不自动解释为候选缺陷。快照在检查前后分别核基线规则文件和候选来源身份；工具报告区分初始源码树、运行中生成/修改的文件及未冻结环境，不能声称所有执行字节只有 commit。

超时和可处理信号尝试终止并回收所持进程组。父进程遭 SIGKILL 或无法证明后代停止时，记录未证明完成；handoff 提醒核残留，不能自动再跑。清理只处理自己已证明归属的 scratch，清理失败单列残留，不抹掉已提交结果。

## 4. 原生观察与长执行

每个 Attempt 最多一个观察批次，放入现有 WorkState 的该 Attempt 附属字段；不另建证据数据库。记录包含 `execution_id`、冻结输入引用、opaque subject 摘要、`output_name`、登记时间/OS 主体、状态、工具输出与日志引用。core 只核身份、闭集状态和归属；Git commit 和工具细节保存在工具生成的报告中，runtime 不解释它们。

状态仅为 `open / completed / interrupted`。`open` 表示本次执行已经登记而没有完整结果，不声称子进程已经启动。`completed` 表示工具已形成完整报告，报告可以包含非零或未完整验证；`interrupted` 表示登记后没有可提交完整报告。两种终结状态不可改写。

1. **登记。** 短管理根锁内查当前 Attempt、输入摘要、工具输出声明和未占用批次。生成执行身份及两个内部短操作 request-id，登记 `open`、scratch 归属和准备效果；内部开始响应按现有提交快照保存。准备效果不完整时不启动项目命令。
2. **运行。** 完成准备并释放所有 WriteSession、Store clone 与锁 Arc，才启动工具。正常只读状态查询和其他 Work 写操作不被长命令锁住。当前 Attempt 有 `open` 时 `next` 不提供 submit，只提供 fail/cancel 和查询指针；第二次原生 run 拒绝。
3. **完成。** 重新取短锁，核同一 Work、Attempt、execution_id 和仍为 open 的记录；其他 Work 导致的全局写入不使它失效。按锁内当前 revision 提交，不能要求最初 revision 原样保持。当前 Attempt 被 fail、replace 或 Work 被 cancel 后，旧完成拒绝，不能写进新 Attempt。
4. **发布。** 工具报告和日志经 runtime 同句柄核大小、摘要、归属与封存，随完成状态、审计、效果和完整响应快照同事务登记；文件效果沿 C002 管道恢复。只有发布完成才能交给下一节点。

运行期间 fail/cancel 原子把未完成记录关闭为 interrupted；实际进程停止仍需执行入口或使用者确认。失败领取新 Attempt 才能重新运行；C005 replace 也须关闭旧 open，不能复制其完成资格。

公开 `verify run` 不支持 `--request-id`：它是一次长观察的组合入口，并非当前一次事务的写命令。开始和完成各有不同的内部请求，不覆盖开始的历史快照；首次公开响应来自完成操作，附 execution_id。重复调用不重跑，也不伪装成同 id 历史重放，使用只读 status/handoff 查询。准备或运行中进程消失时不能根据日志文件推定完成。恢复器只发布已经登记的字节或效果，不重启项目命令；用户确认停止后 fail，再显式重试。T03 必须将这个例外写进 GF-15 和协议，不能沿用普通写操作的请求承诺。

## 5. 工具输出与普通提交

节点输出新增闭集 `producer = "worker" | "engine"`，默认 worker。首版每个 Node 最多声明一个 engine 输出，必须 required，专用于该批次原生检查报告；两个 engine 输出或可选 engine 输出装入即拒绝。日志是报告关联引用，不另占输出槽。engine输出同时声明 `profile_input`，必须引用本节点一个 required 输入；worker输出不得带该字段。工具入口的 --profile-input/--output 必须与该声明一致，不能自行挑另一份冻结输入。工具入口只能填 engine 输出；submit 不能把 worker 目录文件当成 engine 产物。engine 文件位于 `engine/observations/<execution_id>/`，不能与 worker 的 `outputs/` 重叠。

工具完成时，报告引用存在附属观察中，Running Attempt 的正式 `outputs` 仍为空。普通 submit 从附属完整记录取得该 engine 输出，核摘要后与 worker 输出一起封存进入正式 outputs。这样保持当前 Running 输出为空的持久校验义务。声明必需 engine 输出却没有完整原生记录时 submit 拒绝；对应 fail 是合法停止路径。只有完整报告而内部检查未完整验证时，协调者按普通失败处理，不把报告文件存在当作检查通过。

对有必需engine输出的Running Attempt，`next`按附属记录给出完整可调用路径：无记录时提供收集该输出的操作、fail/cancel，不能提供submit；open时只有fail/cancel和只读查询指针；completed且文件效果已发布时，当前可执行视图才提供submit/fail/cancel。普通worker节点保持当前next规则。core只产生通用 `CollectOutput`，CLI将其渲染为 `verify run`，不在core解释Git。

收集操作的work/attempt/output/profile_input由状态和图固定，唯一未绑定用户参数是candidate。该next项增加 `parameters = [{name: "candidate", required: true, supplied_by: "coordinator"}]`，明示调用者需填写完整候选后才能执行；协调者读取实现产物中的候选提示，工具再实际解析与核对完整commit，不能将提示直接登记为事实。这是一个明确的参数合同，不是通用表达式或自由命令模板。历史完成响应的next按COMMIT时状态固定，可能已列submit；发布失败由EFFECT_PENDING明确标为不可续接，不改原快照。当前状态视图核未完成效果，写操作先成功恢复已登记文件效果才允许submit；恢复不执行项目命令、不重写历史next。所有写操作响应、文本、状态卡和持久快照都同步此shape；普通next项parameters为空。T03在采用协议中同步“next可拼命令”对这个待填参数的准确表述，并提供真实CLI正反例。

检查配置、报告和日志沿普通冻结输入交给后继节点。Review 仍普通 Node；发现、解决建议和独立性声明都是报告内容，不自动成为系统资格。

## 6. 结果合同

仅无出边的终点允许在 input/output 声明 `result = true`，默认 false。所选项必须 required；逻辑 key 在所选输入与输出中唯一，重复、可选或不存在来源装入即拒绝。没有结果声明的旧 Workbook 仍可运行，`work result` 显示空选择，不猜最近产物。

例如终点是 retro，则它将 delivery 和原生 check 报告作为普通必需 input，分别标 result=true；自己的 lessons 输出可按作者意愿选择。所有引用从使 Work succeeded 的具体终点 Attempt 已绑定 inputs 和封存 outputs 取得。非终态查询返回状态和 `final=false`，不开放原件导出；取消也不包装成成功结果。

拟命令：`sheltie work result <work> --json`，返回 `work-result/v1`：

| 字段 | 来源 |
| --- | --- |
| `work_id / revision / workbook_digest / status / final` | 同一次可信装入的 Store 和冻结图 |
| `artifacts` | 按 key 排序的明确选择；每项含 key、path、sha256、bytes、来源 Attempt |
| `candidate` | 从所选 Artifact 中匹配 Store 原生观察的工具报告取得；无原生报告为 null |
| `check_records` | 只列所选原生报告对应的执行事实，不把其他候选的历史结果拼入 |
| `unverified_conditions` | 冻结配置中的未验证说明，明确属于标准声明 |
| `snapshot_digest` | 规范结果字段的摘要；不含读时钟、重核观察或 digest 自身 |

工具外围严格解码原生报告才提供 `repository_hint / commit / input_digest`；普通文档无此权力。多个所选原生报告候选或输入不一致时返回 RESULT_CONFLICT；不能选最新一个掩盖冲突。状态卡、handoff 可以看完整历史，但不改变最终选择。

摘要编码为 `work-result/v1` 域前缀加字段顺序固定的规范 JSON UTF-8 字节长度及字节；字段和 map 顺序固定，字节数十进制，时间按现合同，禁止浮点。core 生成同一规范载荷，runtime 计算实际 sha256；golden 向量独立手写。T06 将完整结果 DTO、schema 及编码表并入协议，不留初级实现者选择序列化细节。

可信字节入口：`sheltie work result <work> --artifact <key> --result-digest <sha256>`。拒绝 `--json` 和 request-id；核当前 final 快照摘要、明确 key、源文件归属和同一普通文件句柄。stdout 仅原始字节，stderr 是错误；读取期间同时核限额和摘要，读毕不符返回失败。消费者必须暂存全部字节、核退出码与独立摘要后再发布，不能使用失败前已输出的前缀。该入口只读，不追加终态记录。

## 7. 交接视图

拟 `work handoff <work> --json` 与文本同源：Work/定义身份、revision、当前 Attempt/brief、所有冻结输入、已封存输出、观察记录及摘要指针、门槛和上限、最新 next。只列明确路径；未提交草稿不核为完成产物，提醒旧执行者状态与共享工作区风险。视图无需安装另一宿主、不包含凭据或全量聊天。

## 8. 实施约束

T03 固定持久格式和 CLI 版本、完整 DTO、限额、错误码、安全 Rust API 可行性与日志敏感字段规则。三个里程碑分别证明「定义和恢复材料」「实际工具到持久结果」「固定 Workbook 到最终结果」。每阶段强模型先写接口、真实 caller 与独立 oracle，简单模型填既定行为；强模型审查不能参与该阶段骨架或实现。发布平台只包含实际采用并验证的 macOS aarch64。
