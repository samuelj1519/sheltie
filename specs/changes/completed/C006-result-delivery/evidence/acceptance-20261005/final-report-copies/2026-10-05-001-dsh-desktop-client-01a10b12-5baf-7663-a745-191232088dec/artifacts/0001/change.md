可审查：F01完整补丁交接方法与F02真实Native权限问答已补齐，源码及App不变；真人已实际体验并接受当前macOS版本，报告修复待新的独立审查。

## 目标、当前范围与身份

Work `2026-10-05-001-dsh-desktop-client`，implement#2.0，来自 review#1 的正常 back。目标是可打开的 macOS arm64 dsh 薄客户端：连接已存在的本机服务，或用明确 Node/CLI/Home/cwd 启动自己的后端；原 dsh Web UI 负责工作区、会话、模型、流式回复、工具及权限询问。当前到达只修报告及补权限消费证据，不修改产品代码，也不重跑同输入的合格检查。

初始 baseline commit `c3bde284ee944c506d72c046c1b47449bdeda5e9`，baseline tree `c314533124124e14aa9d3b60f440fa51ff04c5a5`。当前源码 tree 仍为 `ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35`，自有候选 `/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/candidate`。29 个 tracked 文件、10 条变更路径：`README.md`、`docs/client.md`、`package.json`、`src/backend.ts`、`src/config.ts`、`src/main.ts`、`src/preload.cts`、`src/renderer.ts`、`ui/index.html`、`ui/style.css`。本报告 writer 只读逐文件核得源码 29 bytes/mode 与已合格 proof 一致，默认 index SHA 亦一致，原件为 `control/dsh-report-implement02-preparation-readback.json`，F02后新增只读复核为 `control/dsh-report-implement02-final-evidence-readback.json`。没有把仍指向 baseline 的 HEAD 当作当前未提交候选。

冻结 task/project/previous-review SHA 分别为 `106b39f203a74d4439078c6f09b092d9b302e960c0a6fe699d67312be9489fa4`、`23a46e56765fab193b82ee4cf22390a0362cb0ed14a152bdd8104d38a3ee0d6b`、`321fc6edfdc487f324f75f6b6bf2d4f62670eaf70396ccc98955272e42919b8d`。旧 implement#1 两报告及全部原件只读保留；本次新增这两份 implement#2 outputs 与独立 control 草稿/身份记录，不使旧报告追溯合格。

## 实际行为与产品边界

相对初始基线，完整实现包括中文原生设置页、可取消启动/错误/重试、Node/CLI/Home/cwd 明确选择、成功参数持久化与冷重开恢复、原生编辑/缩放菜单，以及所连服务原 Web UI。remote window 无 preload/Node 桥，启用 sandbox/contextIsolation；独立 local window 的窄 IPC 核 sender、主 frame、精确 file URL 和 payload 字段。导航/新窗口受限，不复制 upstream rc.5 RPC。

BackendManager 是唯一连接状态与资源归属权威。只认 own child 的 loopback ready 与 HTTP 就绪；排空双 stream、有界诊断；取消后阻止迟到 ready/health；owned 成功停止需 child close 与 known group gone，失败保留 owner/error，可重试 dispose 且不解除 sealed。external 只断开，不按端口猜归属、不停止 Node55344、不删除会话。设置损坏明确报错，显式修复备份；随机 sibling 临时文件 exclusive wx/0600，新建目录0700，清理只针对本次成功创建的文件，不 chmod 既有目录。

实际 runtime 是已装 CLI rc.7、解析的 Web 等依赖 rc.8，原 `/Users/shushu/tools/dsh/deepseek-harness` rc.5 源保持只读。输入闭包是实际 Node 与 10700 runtime 文件、87164490 bytes、7 个解析包和69个前端输入；`control/runtime-after-native-model-readback.json` 记录消费后无漂移。HTTP就绪不证明模型成功；技术消费也不替代真人整体接受。

## F01：完整补丁生成与独立应用方法

F01 是报告契约缺口，不是产品代码缺陷。以下完整复述 project 已冻结方法、实际 argv/cwd/env 和原件，纳入未跟踪新增文件。方法已有真实资格记录，本次复用，未再次执行 Git 操作。若交付时需要重演，必须使用新的授权独立检查目录/index/原件路径；不得覆盖下列旧资格原件，亦不得在实际最终 source 或 upstream 试 apply。

生成工作目录：`/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/candidate`。四条命令都使用隔离 `GIT_INDEX_FILE`，不修改候选默认 index/ref：

```bash
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git read-tree c3bde284ee944c506d72c046c1b47449bdeda5e9
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git add --all
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git write-tree
GIT_INDEX_FILE=/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/report-method-qualification/source.index git diff --binary --full-index c3bde284ee944c506d72c046c1b47449bdeda5e9 ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35
```

`git add --all` 在这个授权独立 candidate 内纳入 untracked 新源码，并按 `.gitignore` 排除可再生的 `node_modules/`、`lib/`、`dist/`。不得用只 diff 已 tracked 文件的方式遗漏新增源码。最终 changed paths 必须仍为上述10项，allsource必须为29文件；出现额外路径、源漂移或超限即停止，不能靠删文件使补丁通过。

`control/report-method-qualification/proof.json` 的四条命令均 exit0；`1.stdout`–`4.stdout` 与同名 `.stderr` 保留实际 stdout/stderr，cwd/env/exit在 proof中。`3.stdout` 输出当前 candidate tree；`4.stdout` 是完整 binary diff，逐 byte等于冻结 `change.patch`，64598 bytes（小于8MiB），SHA-256 `ca23e583f1a75ea4e676ab92f7758c0ee8d60592233588e86a5b090bbc195466`。冻结补丁位置：

`/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/candidates/ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35/change.patch`。

独立应用已经由未参与 Task3 准备、实现或 tests 的 `/root/c011_closeout_review` 执行。clone 的 cwd为 `/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/task3-delivery-check`；未设置 `GIT_INDEX_FILE/GIT_DIR/GIT_WORK_TREE`，仅操作新自有 clone：

```bash
git clone --shared --no-checkout /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/candidate /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/task3-delivery-check/source-copy
```

其后五条命令的 cwd均为 `/private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/task3-delivery-check/source-copy`，使用该副本自己的 index：

```bash
git read-tree c3bde284ee944c506d72c046c1b47449bdeda5e9
git checkout-index --all
git update-index --refresh
git apply --index /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/candidates/ff0ba3f4f81f9a88a6a5d7ca15d99bb70acb8c35/change.patch
git write-tree
```

实际记录是 `control/task3-delivery-check/proof.json` 的 commands003–008，stdout/stderr为 `raw/003.stdout`–`raw/008.stdout` 及同名 `.stderr`，全部exit0。008输出 applied tree等于candidate tree。009/010与011–039核all29 tracked文件bytes/mode；041在source-copy以相同baseline/tree重新`git diff --binary --full-index`，stdout SHA亦等于冻结完整patch。该proof记录defaultcandidate HEAD/index/ref前后不变；本次只读index复核也相符。

## F02：真实Native权限问答已补，待独审

用户明确说「DSH app 审批通过」，授权继续运行当前本地构建App及公开QA权限验收。`control/native-reaccess-authorized.json`记录07:20:54Z的新`cua.getApp`已成功。它本身是运行与QA授权，不是整体产品可用性接受。随后用户另对明确的整体体验问题回答「已实际体验，接受这版客户端」，07:26:01Z的 `control/human-app-acceptance.json`记录当前候选macOS App的actual/PASS接受；此前not_run/not_reported原件不删改。前一次auto_review拒绝及当时pending停止稿保持在 `native-reaccess-approval-pending.json` 和 `dsh-report-implement02-before-permission-receipt/`，不追溯删改。

Root随后在真正DSH.app的现有127.0.0.1:3080 Web UI、同QA session `session-ebd6e76a-56df-477d-b170-1f2381c050cf` 执行两次工具权限消费。standing仍read-only/ask，README/hello旧字节与mode未变，无其他工具绕路或权限preset扩大。原件 `control/permission-native-qualification.json` 给出实际callId、seq、approval ID、结果和文件后态；不是静态推断，也不是普通ask-user。

| 分支 | 真实调用、审批与结果 | 文件后态 |
| --- | --- | --- |
| 允许一次，turn7 | 普通write call `call_00_jGulf6nWOauo9mQIIvvq4073` / seq2941，结果seq2942为FS_SANDBOX_DENIED。相同file_path/content重试 `call_00_ELfPCpQLeN3RAgW9NdM32435` / seq3095，仅增workspace-write与justification；approval/asked seq3096的ID `d299b6c6-253e-400a-a0d2-6ae8e2181848` 对应这个callId。Root真实Native点「允许一次」；decided seq3097同ID/outcome=allowed-once，tool/result seq3098的structured isError=false，turn7 completed。 | `control/model-qa-workspace/approval-allow.txt` 精确23bytes，为 `APPROVAL_ALLOW_FIXTURE` 加单个换行；SHA `0a456518647c01b65fba0a8da782b569eb559fea88360dd9220fd05829819df6`。 |
| 拒绝，turn8 | 普通write `call_00_xM9QUMOIPu4EzAhSJxU89711` / seq3554，seq3555真实FS_SANDBOX_DENIED。相同file_path/content重试 `call_00_L4VISkO0A48tnLbFZ0mP3659` / seq3694，仅增workspace-write与justification；asked seq3695 ID `98fe8b00-44dc-4910-88af-ca5fdf16d0e9` 同callId。Root真实Native点「拒绝」；decided seq3696同ID/outcome=rejected，seq3697的structured isError=true及用户拒绝消息；turn8 completed，未换其他tool绕路。 | `approval-deny.txt` 不存在，包括不存在symlink；未发生mutation。 |

两次回答后Native仍显示Read Only，实际permission/preset最后值read-only、sandbox/mode最后值read-only，approval/policy为ask。Root第一次权限面板的实际Cua截图保留在对话工具结果中；当前公开receipt只摘本次QA许可字段和fixture，不复制私密headers/raw或其他session。07:23:35Z的压缩日志为74951bytes/SHA `a8f452c35426a790bc3f0b09b0cc261b07239accf1fcaf65ca83f942fd24d04c`、276rows；这是新观察时点，旧六轮日志hash仍保留各自时间点。

资格helper首轮真实失败是错误要求新文件结果meta.diffs必须truthy（seq3098实际为空）。`control/permission-qualification-01-failure.json`与 `qualify_permission_events.py` 保留该parser错误，不归为产品故障。v02按structured tool-result isError、对应callId和精确文件bytes核，不删除真实拒绝/失败。Root实际执行 `python3 /private/tmp/sheltie-dsh-desktop-20261005-uz_h1s9b/control/qualify_permission_events_v02.py`，cwd control；command JSON完整记录PATH/LANG/LC_ALL、script SHA `d9d81183b2194acb46d13f420d1c20e4893de8dc82cc23fcb9ef36d3d020948a` 和exit0，stdout/stderr为 `permission-qualification-02.stdout/.stderr`。

本report writer只读核source29、ASAR、README/hello bytes/mode、allow精确内容、deny不存在和v02 script身份，原件 `control/dsh-report-implement02-final-evidence-readback.json`；未再执行helper/消费者/模型/UI/CLI或读取私密session raw。静态 `permission-flow-diagnosis.md` 和 `permission-fixture-preflight.json` 仍仅是诊断/准备，不被升格为真实F02证据。F01/F02实施补项已完成，交新的独审核对，未代审查人宣最终合格。

## 既有技术证据、使用与限制

实际Mac arm64 `.app` 为0.1.0，ASAR SHA `c96b81af40f4710eeddfe74800f4ae56121613b01c7cd21392d656c395574091`。独立apply的29源码与独立重编译的6 JS/2 UI、candidate及ASAR内容一致；整App582-entry manifest和ZIP roundtrip bytes/mode/symlink一致。ZIP127078664 bytes，SHA `1f1d8d49fc9e25c4df64b3b68a560aa548b0f6c86383ead9e58ca6deba4a454c`，实物/manifest在 `control/task3-delivery-check`。

打开 `candidate/dist/DSH-darwin-arm64/DSH.app` 默认连接现有3080；也可在「客户端启动」明确选Node/CLI/Home/cwd，不复制key。菜单「连接设置…」「停止客户端后端 / 断开现有服务」与退出区别见README。最终独立source计划为 `/Users/shushu/tools/dsh/dsh-desktop`，尚未发布，发布前仍须Root核新目录不存在、漂移与权限；不装到Applications。

既有检查在相同source/runtime/artifact输入闭包下reused，本报告writer只执行只读身份核对，不冒称重做19tests、Native菜单、真实API、退出重开或artifact制作；详见同阶段checks。真实权限问答技术范围已实际补齐；真人整体App接受已由 `control/human-app-acceptance.json`单独记录actual/PASS（当前macOS候选）；其他OSnative、最终质量盲审、商业签名/公证/自动更新/全局安装/发布均not_run或待；fair/ROI未采用。费用、usage、活动/等待分钟unknown，120分钟个人/60小时总等待预算不重置，也不从Attempt或测试时长推算。旧模型日志hash对应原观察时点，后续新事件应新增receipt，不覆盖旧记录。
