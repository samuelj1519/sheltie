建议交付：F01/F02 已闭合，所采用的 macOS arm64 代码、合同及实际技术消费范围通过独立审查；用户已明确报告实际体验并接受此版客户端。未发现需进一步返工的缺陷，保留历史环境记录缺口和未验证范围。

Work `2026-10-05-001-dsh-desktop-client`，review#2.0，来自 implement#2 正常 main 边。本报告不自行推进引擎、发布源码或安装应用；交协调者按既定 deliver 路径处理。

## 身份、独立性与复用

Reviewer `/root/translation_batch_1` 未参与 Task3 准备、设计、实现或 tests/oracles 编写；review#1 中提出 F01/F02，Root 正常 back 后只修报告并补实际权限消费，产品代码与 App 无变更。旧 implement#1、review#1、停止稿、首失败与所有投入原件保持，不把旧报告追溯为合格。

源码候选 tree `ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35`，baseline commit `c3bde284ee944c506d72c046c1b47449bdeda5e9` / tree `c314533124124e14aa9d3b60f440fa51ff04c5a5`。自有 candidate `/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/candidate`，29 tracked files、10 条变更路径。default HEAD/index 仍是 baseline，当前未提交候选由实际 tree与逐文件 SHA/mode识别，不以HEAD代候选。

已读正式 review#2 brief 与四份冻结输入，并实际重核SHA：

| 输入 | SHA-256 |
| --- | --- |
| task | `106b39f203a74d4439078c6f09b092d9b302e960c0a6fe699d67312be9489fa4` |
| project | `23a46e56765fab193b82ee4cf22390a0362cb0ed14a152bdd8104d38a3ee0d6b` |
| change（implement#2） | `afdbfc73ce6df3c726acaff891370a585a69abf0953db5bb0e3a1c367890b6af` |
| checks（implement#2） | `622a09fb447e930e1c482f8bf0537febaf2d21f9bf3525ebbb60055a072041f5` |

具体正式路径在 `control/task3-code-review2-formal-input-readback.json`。旧review#1仍为SHA `321fc6edfdc487f324f75f6b6bf2d4f62670eaf70396ccc98955272e42919b8d`。没有输入漂移。

源码/content审查复用本人此前逐读 src全部六文件、contracts、全部test/actualfixture/oracle、UI、README/docs/package/lock/tsconfig的结论；本次实际逐source29 bytes/mode和完整runtime/App manifest重核同输入。复用不是省略当前身份核对，也不冒称新实现/新测试。review#2未进行UI action、模型请求、CLI、build或消费者重跑；仅写本review和自有控制记录。

## F01 关闭：报告交接自足

新的 change/checks 完整展开已冻结的补丁方法、真实argv/cwd/env、baseline、纳untracked规则、输出归属、授权独立副本和证据。报告说明这些命令是先前 executed、当前 reused；新重演必须选新路径，不能覆盖旧qualified原件。

- candidate cwd 下，隔离 `GIT_INDEX_FILE=control/report-method-qualification/source.index` 的绝对路径，依次 read-tree baseline、add --all、write-tree，再 binary/full-index diff baseline与candidate tree。`add --all` 纳入新增README/client/src/UI，遵循.gitignore排node_modules/lib/dist；保持10变更路径/all29授权源码，不靠删除文件或漏untracked压小patch。
- 独立 `git clone --shared --no-checkout` 到授权task3-delivery-check/source-copy，再read-tree/checkout-index --all/update-index --refresh/apply --index/write-tree；commands003–008记录argv/cwd/env/exit/stdout/stderr，applied tree同candidate；all29 bytes/mode核实。默认candidate index/ref不变，实际 source/upstream不试apply。
- report-method-qualification四stdout真实exit0，write-tree为ff0ba，diff逐byte等原完整patch；其后delivery command041重新生成diff亦同SHA。完整patch64598 bytes/SHA `ca23e583f1a75ea4e676ab92f7758c0ee8d60592233588e86a5b090bbc195466`，低于8MiB。

本人已核方法proof与raw，未执行Git mutation。新change已自足，F01不再开放。

## F02 关闭：真实权限请求、决定与世界状态

不以Read自动允许、preset/policy、普通ask-user或静态透传代码代实际权限问答。新增真实消费由Root在DSH Native App的现有127.0.0.1:3080/同QA session执行；本人独立从这个allowlisted QA compressed日志中，仅提取permission/sandbox/approval/tool/turn允许事件和fixture后态，不解析或输出request/header、其他session或凭据。

Root原件：`permission-native-qualification.json`、`permission-qualification-02-command.json` 与 `.stdout/.stderr`；独立核对原件：`task3-code-review2-permission-readback.json`。当前QA compressed74951 bytes/SHA `a8f452c35426a790bc3f0b09b0cc261b07239accf1fcaf65ca83f942fd24d04c` 与receipt一致；session为 `session-ebd6e76a-56df-477d-b170-1f2381c050cf`。

| 分支 | 独立核得的完整链 | 后态 |
| --- | --- | --- |
| turn7 允许一次 | 首write seq2941 / `call_00_jGulf6nWOauo9mQIIvvq4073` → seq2942真实FS_SANDBOX_DENIED。相同file_path/content单次retry seq3095 / `call_00_ELfPCpQLeN3RAgW9NdM32435`，只扩大该call到workspace-write并给justification；asked3096同callId，审批ID `d299b6c6-253e-400a-a0d2-6ae8e2181848`；decided3097同ID、allowed-once；tool/result3098同callId structured isError=false，turn completed。Root实际Native点击允许一次，与持久事实吻合。 | `approval-allow.txt` 精确 `APPROVAL_ALLOW_FIXTURE` 加一个换行，23bytes/SHA `0a456518647c01b65fba0a8da782b569eb559fea88360dd9220fd05829819df6`。 |
| turn8 拒绝 | 首write3554 / `call_00_xM9QUMOIPu4EzAhSJxU89711` → 3555真实FS_SANDBOX_DENIED。相同payload单次retry3694 / `call_00_L4VISkO0A48tnLbFZ0mP3659`；asked3695同callId，审批ID `98fe8b00-44dc-4910-88af-ca5fdf16d0e9`；decided3696同ID、rejected；tool/result3697同callId structured isError=true，turn completed。Root实际Native点击拒绝，与持久事实吻合。 | `approval-deny.txt` 不存在，亦无symlink；未mutation、未绕过。 |

本人核每轮只有上述两次write，没有其他tool调用；seq顺序为retry→asked→decided→result；两次retry目标/content与真实先前denial相同。standing最后仍read-only/ask，无新的永久权限扩大，旧README/hello bytes/mode均相同，工作区只有这两个旧文件和允许fixture。没有放宽Electron/browser/fileSystem保护或加入Node桥。

v02 helper exit0、script SHA `d9d81183b2194acb46d13f420d1c20e4893de8dc82cc23fcb9ef36d3d020948a` 与实际文件一致，stdout/receipt及当前QA hash一致，stderr为空。首helper错误要求meta.diffs必须truthy，实际空值；旧失败receipt/script保留，修正为同callId/isError与文件bytes。它是检查器假设修正，不是产品修复，也没有删除真实sandbox denial或用户拒绝。F02不再开放。

Native点击由Root executed；本人metadata/世界状态只读核验 executed，未冒本人亲自审批或发送请求。

## 全链技术资格与当前闭包

此前独立代码审查及实际消费者结论仍适用：唯一BackendManager/contracts权威；显式own Node/CLI/Home/cwd、port0/no-open；完整loopback ready+HTTP、非2xx/redirect/timeout拒绝；timeout即exit0/forced事实区分；双stream排空、有界诊断；cancel阻迟到connected；owned清理await childclose/known groupGone、失败保owner/retry且sealed；external不signal、不按端口猜own、不删除Home/session。remote无preload/Node、sandbox/contextIsolation；settings独立file窗口窄IPC核sender/mainframe/fileURL/payload，导航/newwindow/权限隔离；损坏配置显式repair备份、exclusive randomtemp0600/newdir0700、不chmod旧目录。未发现这些已核代码路径中的确定性合同/回归/安全缺陷。

| 已采用范围 | 原执行资格／本次方式 | 核对依据与结果 |
| --- | --- | --- |
| strictcompile、19 actualchildren/server tests | worker/checker executed；sameclosure reused | 19pass/0fail/0skip；不是mock或零测试。含timeout0、latecancel、stdout/stderr、descendant持pipe、forced、404/500/hang与external不停止；本次无源码/tests变更。 |
| extra4、dispose失败重试、config安全smoke | worker executed自核；本人独立审raw/code后sameclosure reused | 缓存rejected Promise旧FAIL保，freshretry/owner/seal通过；private/exclusive/corruptrepair/backup/cleanup/new700通过。自核身份不改为独审。 |
| installedruntime freshHome启停重开 | Root executed；源码/runtime身份与Native own消费者资格下reused | actualCLIrc7/resolvedrc8 startready/HTTP/stop/portfree/restart/Home retention/dispose；不拿该keyless结果冒模型调用。 |
| Native existing/own与真实模型 | Root executed；本人review#1只读Cua观当前真实历史、本次复用同App/runtime | 选择公开QA、新会话/真实Read/流式、第二长请求实际aborted/user、停止后继续、cold同会话hello.txt、wrongNode actionable、own59705/PID96299退出gone、coldown60340同Home、有序switch前ownport释放，external55344保持。第一1000请求completed不授cancel，AX-onlypicker负断言withdraw与binding错误如实保留。 |
| 新权限允许一次/拒绝 | Root Native executed；本人允许事件/fixture独立只读executed | 上述F02链完整，不从preset或静态支持推实际成功。 |
| 完整patch/apply/compiled/ASAR/ZIP | 独立deliverychecker executed；本人当前byte/mode/hash核验executed，消费者reused | freshbaseline apply同tree/all29，six rebuiltJS/two staticUI同candidate/ASAR；App582精确entry/bytes/mode/symlink；ZIP roundtrip匹配。 |

本次 `task3-code-review2-closure-readback.json` 实际逐哈希核29源码bytes/mode、完整10700 frozenruntime文件87164490 bytes/mode、actualNode SHA、7 resolvedpackage版本，以及App582完整entry集合/每个regular bytes/mode/symlink目标。ASAR8当前byte同，defaultindex SHA保持 `92ea5a481c5e7eb7b5194768b60d98d1e5d7540829d1789a4c69ea843e060814`。runtime manifest SHA `5842a9fe4f55bbddbfe73c2178c53439c380590c4af106f0c81378aa07092ea1`；Appmanifest SHA `bb82b974c139256cb6572701bedd6a574b2c70c4f000de8a8069e6ae5c3e4f3b`；ASAR SHA `c96b81af40f4710eeddfe74800f4ae56121613b01c7cd21392d656c395574091`。完整源码/产物/runtime闭包稳定，不以CLI单SHA代依赖。

ZIP127078664 bytes/SHA `1f1d8d49fc9e25c4df64b3b68a560aa548b0f6c86383ead9e58ca6deba4a454c`；完整patch和独立apply的49 raw哈希已由review#1本人核，当前reuse身份不变，不重复生成/覆盖它们。检查内容与原始证据的详细索引见新checks及保留的review#1，不能将reused改称本人新跑。

## 用户接受与边界

`control/human-app-acceptance.json` 对应同候选/App，问题明确询问实际整体体验，用户实际回答：「已实际体验，接受这版客户端」。此为用户reported accepted，不是Root技术QA、运行审批或reviewer替人体验。较早「DSH app 审批通过」仅授权运行当前App及QA，单独记录，未拿它冒整体接受。用户实际报告已到，因此当前不再标整体接受pending；旧not_run/not_reported是各自过去观察时点，保留不改写。

新checks明确保留package-mac02和actual-owned-manager两项历史完整环境metadata未独立封存：exit0来自Root历史tool结果转录，不能从pass JSON/stdout反推；影响环境不能全面核。本人不补造当时环境，不授这两条历史命令全面环境可复现资格。其已观察行为另由有明确输入身份的独立source/artifact编译readback、实际Native own与完整currentruntime/App闭包支持；该局部存档限制不等于产品代码缺陷，保留在交付报告。

本report自身是正式内容审查，不是最终质量盲审或公平/ROI实验。Windows/Linux native、商业签名/公证、自动更新、全局安装/发布及最终盲审仍not_run或待；fair/ROI未采用。不因工程/模型/用户接受而宣其他平台、安全全覆盖、净收益或引擎全Work自动完成。费用、usage、个人活动/等待分钟unknown，预算不返工重置。

建议协调者交付当前已审候选；最终新source目录 `/Users/shushu/tools/dsh/dsh-desktop` 与实物交接另按project核不存在/漂移/权限和完整新副本，不在upstream或实际源试apply，不装Applications。本人未发布、安装或推进CLI；交付前若源/闭包/实物变更，这份review不自动转授新候选。
