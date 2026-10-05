PASS（必需技术检查范围）：同源码/App的合格检查reused，F01方法与F02真实权限问答已补；真人已实际体验并接受当前版本，新的独立审查待，历史环境记录缺口照实保留。

## 候选与检查资格

Work `2026-10-05-001-dsh-desktop-client` implement#2.0。候选tree `ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35`，baseline commit `c3bde284ee944c506d72c046c1b47449bdeda5e9` / tree `c314533124124e14aa9d3b60f440fa51ff04c5a5`。本到达未改产品源码、依赖、tests、合同或保护配置；只新增报告及公开权限消费receipt/允许fixture；report writer不写fixture、不操作模型。实际只读核source29 bytes/mode与qualified delivery proof一致、默认index SHA一致；四冻结输入及旧两报告SHA匹配brief和review，原件 `control/dsh-report-implement02-preparation-readback.json`。

执行模式与结果分开：下列原执行者executed的结果，本阶段在相同闭包资格下reused；不是本report writer重跑，也不因不重跑改成not_run。新F02由Root executed真实Native及helper，本report writer仅只读核资格；不冒称本人操作。Root独立fixtures/oracles在实现前冻结，本次没有改断言或新造fixture来冒真实模型成功；无upstream/Rust输入改变，不运行无关套件。

## 实际命令检查与原件

路径前缀：candidate=`/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/candidate`；control=`/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control`。完整PATH/LANG/LC_ALL、actualargv、cwd和exit以所列原件为准，正文不把某份pass JSON反推成未记录的shell退出码。

| 范围 | 原实际执行及环境 | 原结果／本到达模式 | 原件与限制 |
| --- | --- | --- | --- |
| strict compile | 独立checker实际Node26.5.0调用candidate的typescript/bin/tsc，`-p <candidate>/tsconfig.json --outDir <control>/task3-delivery-check/rebuilt-lib`，cwd为task3-delivery-check；NodeNext strict、固定TS7.0.2/lock | exit0；reused | delivery proof command043及raw/043.stdout/.stderr，重编译6JS与ASAR/candidate相同；不把只编译当原生或模型检查。 |
| 非零actualchild/server oracles | backend worker `node --test test/backend.test.mjs`，cwd candidate，host环境有真实本地listen与own child权限；actualPATH/LANG/LC_ALL在metadata | exit0，19pass/0fail/0skip；reused | `dsh-backend-test-005.{metadata,stdout,stderr,exit}.raw`；覆盖ready/HTTP、timeout即exit0、latecancel、强制退出、孙持pipe、404/500/hang、external不停止等。实际命令是node--test，不冒称本阶段运行了npm test。 |
| extra与dispose失败重试 | worker `node --input-type=module`，stdin为dsh-backend-extra-script.raw或dsh-backend-dispose-retry-script.raw，cwd candidate；同host环境 | extra004和dispose-retry002 exit0；reused | 各metadata/stdout/stderr/exit.raw；immediate dispose/listener重入Stop/redirect不follow/health cancel及失败后owner保留、fresh retry promise/sealed。dispose-retry001真实FAIL保留；runner只救收自己fixture的escaped孙PID。 |
| 配置与private temp | UI worker `npm run build`，及`node --input-type=module -e <完整smoke源码>`，cwd candidate；自有/private/tmp数据，受控crypto/fs边界注入，结束恢复 | build006与configsmoke003 exit0；reused | `dsh-ui-build-006.raw`、`dsh-ui-config-smoke-003.raw`、`dsh-ui-report-02.*`；unknown字段/坏JSON/explicitrepair/原备份、regular/symlink碰撞不覆写、rename cleanup、AggregateError原cause、新dir700不chmod旧dir。Rootmain700后最终全源compile由043再核。 |
| Mac package与生产内容 | Root `npm run package:mac`，candidate内strictbuild+packager20.3.0/Electron44.5.1；arm64产物随后独立checked | Root历史tool执行记录exit0；本到达reused。完整命令环境metadata未独立封存，不能全面核影响环境 | `package-mac-02.stdout/.stderr`、`app-content-build-readback.json`及Root历史结果转录`dsh-report-implement02-root-command-history.json`；exit0来自Root历史tool结果，不从stdout推算。首次01未引用regex导致shell exit2，`packaging-script-first-failure.json`保留。独立042 ASAR extract/043 rebuild/044 file/045 lipo exit0确认arm64/0.1.0和6JS2UI同源。 |
| installedruntime keyless own Home | Root实际固定Node命令`/Users/shushu/.local/share/mise/installs/node/26.5.0/bin/node /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/actual-owned-manager-check.mjs`，cwd candidate；明确CLIrc7/freshHome/cwd，spec45s/3s/7s/2s/64KiB；完整环境metadata未独立封存 | Root历史tool执行记录exit0，另receipt pass=true；reused；不从pass JSON推exit，影响环境无法全面核 | `actual-owned-manager-check.mjs/.json`、`actual-owned-manager-01.stdout/.stderr`及Root历史结果转录；52338 ready/GET200→await stop/portfree→sameHome52345重开→dispose；native/model not_run于此检查，不能代后两项。 |
| 完整binary patch／独立apply | 原4条隔离index命令、独立clone/read-tree/checkout-index/refresh/apply/write-tree，详见同阶段change全文；LANG/LC_ALL及Git env在proof | 全exit0，同tree/all29；reused | `report-method-qualification/proof.json`和1–4.stdout/.stderr；`task3-delivery-check/proof.json` commands003–010/011–039/041，具体命令、cwd、env、stdout/stderrSHA全在proof。patch64598B/SHAca23e583…；默认HEAD/index/ref不变。 |
| App归档roundtrip | 独立checker `/usr/bin/ditto -c -k --sequesterRsrc --keepParent <App> <ZIP>`及`ditto -x -k <ZIP> <artifact-roundtrip>`，cwd task3-delivery-check | commands046/047 exit0；reused | 同proof/raw046/047；582entry bytes/mode/symlink同manifest；ZIP SHA1f1d8d49…，manifest SHAbb82b974…，不是只看压缩文件hash就授安装结果。 |

## F01实际命令元数据展开

以下均为已执行原件的复述，本report writer未再执行。生成四条命令的cwd是candidate完整路径 `/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/candidate`，环境为显式隔离GIT_INDEX_FILE，各exit0、stdout/stderr在report-method-qualification的1–4原件；第三条stdout为同候选tree，第四条stdout为冻结patch：

```bash
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git read-tree c3bde284ee944c506d72c046c1b47449bdeda5e9
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git add --all
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git write-tree
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git diff --binary --full-index c3bde284ee944c506d72c046c1b47449bdeda5e9 ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35
```

add --all纳入untracked，遵循.gitignore排node_modules/lib/dist；changed paths10/allsource29，不放宽白名单。实际独立clone的cwd是 `/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/task3-delivery-check`，Git index/dir/worktree环境未设置：

```bash
git clone --shared --no-checkout /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/candidate /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/task3-delivery-check/source-copy
```

其后cwd均为 `/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/task3-delivery-check/source-copy`，用这个新副本自己的index：

```bash
git read-tree c3bde284ee944c506d72c046c1b47449bdeda5e9
git checkout-index --all
git update-index --refresh
git apply --index /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/candidates/ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35/change.patch
git write-tree
```

独立记录commands003–008均exit0；008输出同tree，009/010和011–039核29文件bytes/mode，041在source-copy重生成相同binary diff。完整patch SHA `ca23e583f1a75ea4e676ab92f7758c0ee8d60592233588e86a5b090bbc195466`，64598B；ZIP SHA `1f1d8d49fc9e25c4df64b3b68a560aa548b0f6c86383ead9e58ca6deba4a454c`；manifest SHA `bb82b974c139256cb6572701bedd6a574b2c70c4f000de8a8069e6ae5c3e4f3b`。新的实际重演必须选新的授权source-copy/index/raw目录，不覆盖上述qualified原件，不在最终source/upstream试apply。此为已有资格reused，不要求为报告返工重新应用或打包。

## 真实Native与模型技术消费

以下操作实际执行者是Root，使用真正的DSH.app，不是Chrome替代；本report writer仅复用公开receipt，没有本阶段发请求或操作菜单。没有把工具Cua的正常返回记成shell exit0。各观察对应独立时间点，后续session新事件不使旧观察时点hash变成错误。

| 范围 | 实际证据／结果 | 本到达模式与限制 |
| --- | --- | --- |
| 打开／外部连接／目录选择 | `native-launch-observation.json`、`native-cold-reopen.json`、`picker-observation-correction.json`：打开arm64 App，连接外部55344/3080；用户实际看到系统目录选择器并选公开QA，Native AX确认model-qa-workspace | reused；最初只看主窗口AX误判picker失败已withdraw，不是产品bug，不改permission/bridge。用户选folder不等于真人整体接受。 |
| 新会话／真实模型／工具Read／流式与继续 | `model-native-observations-01.json`、`public-model-selection-qualification.json`、`qa-session-persistence-before-app-reopen.json`：同session六轮，实际Read hello.txt与预期fixture回复；Deepseek-official/deepseek-flash/high的公开request config | reused范围通过；Read Only自动允许只证明Read消费，不能证明F02实际权限请求。未复制私密raw/headers/凭据；QA源文件保持。 |
| 停止当前请求后继续 | 第一1000请求stop前已completed，不授取消PASS；第二10000请求观察running并点Stop；`qa-after-cold-continuation.json`记录typed turn/end aborted/user，后续「继续成功」 | reused第二个真实取消及继续；reason原件由字符串误判修为判别对象，有旧parser错误receipt保留；不把首轮过快completed冒取消。 |
| 冷退出与已有会话重开 | `qa-session-after-native-quit.json`：旧App67596 Native菜单退出、external保留、原压缩日志SHA6c2abef6…同prequit；随后新App重开恢复历史，`qa-after-cold-continuation.json`为同session cold request与hello.txt回复，旧时点46466B/SHA204c51ac… | reused；具体会话id/公开durable统计在receipt，不复制私密log。该日志hash是原receipt时点，不擅改成F02后的当前hash。 |
| 自有启动／退出／重开／切外部 | `native-owned-mode-and-restoration.json`、`native-model-final-summary.json`：wrongNode ENOENT且external/旧参数保留；own59705/PID96299实际启动，quit后ownPID/port/App gone且55344 kept；coldown60340同Home、无key复制；switch external前ownport释放、QA会话恢复 | reused；own Home仍存在，无凭据拷贝。技术操作不冒称reviewer亲自执行或人整体接受。 |
| 完整runtime与源码闭包消费后不变 | `runtime-after-native-model-readback.json`：10700文件/87164490bytes无漂移、source29不变；对应原Node/7pkg冻结manifest | reused；不以CLI版本或单launcher SHA代完整闭包。本到达source29另只读executed同身份。 |
| F02真实权限问答 | `permission-native-qualification.json`：turn7/8普通write真实FS_SANDBOX_DENIED→相同payload workspace-write重试→Native允许一次/拒绝；两组asked/decided ID与callId同，allow精确23B，deny不存在且structured isError=true，standing read-only/ask不变，原README/hello bytes/mode不变，无其他tool绕路 | Root executed真实Native后新增证据；本report writer只读核。不是Read自动允许、preset/policy或静态源码推断；用户运行审批不代整体接受。 |

## 新增F02实际消费、命令与修复原件

用户明确说「DSH app 审批通过」，授权继续运行当前本地构建App及公开QA权限验收。`control/native-reaccess-authorized.json`记录07:20:54Z的新`cua.getApp`已成功。它本身是运行与QA授权，不是整体产品可用性接受。随后用户另对明确的整体体验问题回答「已实际体验，接受这版客户端」，07:26:01Z的 `control/human-app-acceptance.json`记录当前候选macOS App的actual/PASS接受；此前not_run/not_reported原件不删改。前一次auto_review拒绝及当时pending停止稿保持在 `native-reaccess-approval-pending.json` 和 `dsh-report-implement02-before-permission-receipt/`，不追溯删改。

Root随后在真正DSH.app的现有127.0.0.1:3080 Web UI、同QA session `session-ebd6e76a-56df-477d-b170-1f2381c050cf` 执行两次工具权限消费。standing仍read-only/ask，README/hello旧字节与mode未变，无其他工具绕路或权限preset扩大。原件 `control/permission-native-qualification.json` 给出实际callId、seq、approval ID、结果和文件后态；不是静态推断，也不是普通ask-user。

| 分支 | 真实调用、审批与结果 | 文件后态 |
| --- | --- | --- |
| 允许一次，turn7 | 普通write call `call_00_jGulf6nWOauo9mQIIvvq4073` / seq2941，结果seq2942为FS_SANDBOX_DENIED。相同file_path/content重试 `call_00_ELfPCpQLeN3RAgW9NdM32435` / seq3095，仅增workspace-write与justification；approval/asked seq3096的ID `d299b6c6-253e-400a-a0d2-6ae8e2181848` 对应这个callId。Root真实Native点「允许一次」；decided seq3097同ID/outcome=allowed-once，tool/result seq3098的structured isError=false，turn7 completed。 | `control/model-qa-workspace/approval-allow.txt` 精确23bytes，为 `APPROVAL_ALLOW_FIXTURE` 加单个换行；SHA `0a456518647c01b65fba0a8da782b569eb559fea88360dd9220fd05829819df6`。 |
| 拒绝，turn8 | 普通write `call_00_xM9QUMOIPu4EzAhSJxU89711` / seq3554，seq3555真实FS_SANDBOX_DENIED。相同file_path/content重试 `call_00_L4VISkO0A48tnLbFZ0mP3659` / seq3694，仅增workspace-write与justification；asked seq3695 ID `98fe8b00-44dc-4910-88af-ca5fdf16d0e9` 同callId。Root真实Native点「拒绝」；decided seq3696同ID/outcome=rejected，seq3697的structured isError=true及用户拒绝消息；turn8 completed，未换其他tool绕路。 | `approval-deny.txt` 不存在，包括不存在symlink；未发生mutation。 |

两次回答后Native仍显示Read Only，实际permission/preset最后值read-only、sandbox/mode最后值read-only，approval/policy为ask。Root第一次权限面板的实际Cua截图保留在对话工具结果中；当前公开receipt只摘本次QA许可字段和fixture，不复制私密headers/raw或其他session。07:23:35Z的压缩日志为74951bytes/SHA `a8f452c35426a790bc3f0b09b0cc261b07239accf1fcaf65ca83f942fd24d04c`、276rows；这是新观察时点，旧六轮日志hash仍保留各自时间点。

资格helper首轮真实失败是错误要求新文件结果meta.diffs必须truthy（seq3098实际为空）。`control/permission-qualification-01-failure.json`与 `qualify_permission_events.py` 保留该parser错误，不归为产品故障。v02按structured tool-result isError、对应callId和精确文件bytes核，不删除真实拒绝/失败。Root实际执行 `python3 /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/qualify_permission_events_v02.py`，cwd control；command JSON完整记录PATH/LANG/LC_ALL、script SHA `d9d81183b2194acb46d13f420d1c20e4893de8dc82cc23fcb9ef36d3d020948a` 和exit0，stdout/stderr为 `permission-qualification-02.stdout/.stderr`。

本report writer只读核source29、ASAR、README/hello bytes/mode、allow精确内容、deny不存在和v02 script身份，原件 `control/dsh-report-implement02-final-evidence-readback.json`；未再执行helper/消费者/模型/UI/CLI或读取私密session raw。静态 `permission-flow-diagnosis.md` 和 `permission-fixture-preflight.json` 仍仅是诊断/准备，不被升格为真实F02证据。F01/F02实施补项已完成，交新的独审核对，未代审查人宣最终合格。

## 本到达结论与边界

F01新报告已自足展开实际生成/独立应用方法；F02新增真实两分支消费原件和helper v02元数据已补。当前未改source/runtime/App，29源码与ASAR仍同冻结tree；既有合格消费者、Native六轮、artifact/patch资格按相同闭包reused，不重复模型或重打包覆盖。Root用户运行授权解决此前工具审批阻断，不改Electron保护，也不把原auto_review拒绝当产品缺陷。旧冻结implement#1/review/首失败/成本与本阶段停止稿完整保留。

必需技术范围具备以上PASS证据，报告自身与F01/F02修复仍待新的独立review；没有将作者自核或Root helper称为独立质量盲审。真人整体App接受已由 `control/human-app-acceptance.json`单独记录actual/PASS（当前macOS候选）；最终独立盲审、Windows/Linuxnative、商业签名/公证/自动更新/全局安装/发布仍not_run或待；fair/ROI未采用。package02/actual-owned-manager的exit0来自Root历史tool记录，完整环境metadata未封存、影响环境无法全面核，独立artifact/source证明和Native消费者范围另外列明，不隐去该局部记录缺口。fees、usage、个人活动/总等待分钟unknown，120分钟/60小时上限不重置，不从Attempt/test时长推算。
