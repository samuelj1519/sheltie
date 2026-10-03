# C005-T05 独立事前审阅

结论：准备 `PASS`。只覆盖 C005-T05 的恢复范围、前提盘点与 MSRV 工具准备。基线 `98620834109bcfae18461b742d8a621692a8a8d0`。Reviewer `/root/c005_preflight_review` 未参与准备、oracle、代码或工具编写；没有运行 MSRV、引擎写命令或改仓库。

PF01 已修：超时原版只在组长未退出时 KILL，可能遗漏同组子进程；清理还能超出 720 秒，且字符串遮掉实际退出码。修订版为每一步预留 8 秒，剩余不足时不启动；向自有新 session 进程组 TERM、宽限 3 秒后 KILL，无论组长是否退出；保留实际 returncode 和 timeout 标志，最终总时长达上限不授 completed。静态核与 freeze SHA 一致，没有执行故障演示。没有未解决必修项。

| 工程 §5 项 | 准备审结果与边界 |
| --- | --- |
| 依据 | T05–T06/M3、当前 adoption 和入口一致；只恢复 macOS aarch64 |
| 不变式 | 无产品源码改动；未扩展身份、隔离、安全扫描或发布 |
| 正反例 | 无新增产品规则；37 个 C005 注册用例均在原 951 的 PASS 原行内 |
| 真实链 | 工具执行实际 `cargo +1.85.0`；两个产物取自 Cargo JSON。export 测试显式绑定它们，普通 CLI 使用该 nextest/Cargo 的本次产物。原 1.98 产物只作资格引用 |
| 崩溃 | 无新增引擎写路径；已有 crash oracle 保持。runner timeout 仅核处置准备，实际未执行 |
| 边界 | 134 输入与 C002 t58 精确相等；192 完整输入和类型无漂移；七个真实前提均 null |
| 文档 | docs 242 / specs 8、1 active / skill / tests 951、241 cards 均 exit 0 |
| 提交 | T05 先提交准备，再执行 T06；本报告不代替 Root 的 staged scope 与提交读回 |
| 突变 | 本次没有修改生产代码或新增规则，不扩展历史突变义务 |
| 证据 | 原 62 paths 与基线 SHA 均吻合；56 原件当前同值，6 文档明确追加恢复记录；275 C004 原件无漂移。完整核对值见同名 JSON |

原 C002 `424c53bc-4a8a-4bf1-8c26-f4b600335a83` 为 951/951、0 skipped，另 5 doctest；37 项 C005 已全覆盖。fmt/check/clippy 与 freshdeny `ef6173cbc5c50ec8166f9a5b28f07834144373ee` 按精确同源输入引用，保留原命令、环境、输出 SHA 和取数时点；不称重新运行旧 C005 门禁或本刻刷新数据库。

四条历史 LEAK 的原 run/case 均读到实际原件：runtime result、future-red post_commit_card、T01 install_modify_path、T03 generated_reference。原因仍 unknown；后续无信号不能注销旧信号。两次 raw whitespace 例外、其他平台 excluded_by_user 及旧 1.85 仅编译事实保持。

C004 两个真实消费者均判断只需普通接续。没有真实 running 撤销目标、事件、旧执行者停止/隔离、接手者、原目标接受和质量成本原件；七字段保持 null。负前检有效，真实撤销试用保持 not_run，不用夹具或 sameAttempt 冷接续充数。

新 MSRV build/nextest/doctest 尚未执行。准备版 runner SHA `6d71d61c572ba1b931deecbed1f3c0287186899bfb364cb3381417e685bcdfa4`；先完成 T05 scope/单任务提交与读回，再按冻结工具和连续 12 分钟预算执行 T06。任何源码漂移、失败或超时均停止并保留原输出，不重跑求绿；实际版本、host、Cargo JSON、两个 1.85 executable SHA、counts/skip/LEAK 和每步 exit 必须在 T06 原件中核实。T06 与 M3 仍 not_run，本准备 PASS 不授工程实际测试、真实价值、人类接受、宿主隔离或发布结论。
