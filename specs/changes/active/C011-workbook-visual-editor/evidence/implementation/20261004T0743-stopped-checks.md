停止：完整 patch 超过 8 MiB；必需工程检查实际通过（最终 23/23），交付独立应用/浏览器/真人接受为 not_run。

候选：HEAD `5837de68c257f49c970d54254dfcc12bd7fe210d`，初始 tree `a64335ce7c5db92bbf356dc7980052e9e9439208`，当前已 stage index tree `b4ae44db1db8273f59f0b1ecf51a717913a13a0c`。
仓库 `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor`；工具 cwd `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor`。

## 当前必需检查

- 完整 `SHELTIE_EDITOR_ENGINE=/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie npm test`：`20261004T074015-28b52ba5.json`，exit 0，23 tests/23 pass/0 fail/0 skipped。真实 CLI argv/stdout/stderr/exit 在对应 stdout 的 boundary records，不把 fake-child 结果当真实引擎质量。最后输入闭包与当前工具全部字节匹配，见 `20261004T074119-a93e1aca.json`。
- 独立入口 `node scripts/smoke.mjs`：`20261004T073927-4b15964f.json`，exit 0；实际启动 `node server.mjs --sheltie <可信绝对路径> --port 4311`，HTTP check/export、完整 ZIP 同字节、PID75351 TERM正常退出，stdout/stderr完整。
- `scripts/check-docs.sh`：`20261004T074010-f0296c99.json`，exit 0，320文件。
- `scripts/check-specs.sh`：`20261004T074010-e43c72b5.json`，exit 0，11 change/1 active。
- `node --check public/app.mjs`：`20261004T074010-40c92606.json`，exit 0，只证明语法。
- `git diff --cached --check`：`20261004T074046-8da8370d.json`，exit 0，检查已 stage 的全部授权文件；原始 stdout/stderr 以 binary 原件保留尾空格，源码/JSON/Markdown不豁免。
- `git diff --check`：`20261004T074047-749a9ff3.json`，exit 0。
- 源范围/最终测试闭包/引擎 hash：`20261004T074119-a93e1aca.json`，exit 0，outside_scope_drift/final_test_input_closure_drift/outside_staged 全空；binary hash为冻结值。

最后 stage 仅追加本次实际检查原件，工具/合同/fixture字节没有改变。`git write-tree` 实际 stdout为 `b4ae44db1db8273f59f0b1ecf51a717913a13a0c\n`、exit0。final index包含所有当前授权未跟踪文件（node_modules 被工具 .gitignore 排除）。本报告在outputs路径，不进入patch，避免自引用tree。

## 覆盖与边界

文件/路径/base64测试有数量1024及多一、大小16MiB及多一、路径UTF8 4096及多一；目录先核清单后读，读后尺寸再核。真实 HTTP 恰好24MiB进入JSON判断，多一413；整组路径/重复/别名/base64/超限前检未物化。恰好16MiB/1024文件通过作者工具前检并交真实引擎拒绝缺manifest，清理根。

真实CLI装入 two-step/code-change/spec-dev，非法入口与删除后引用拒绝；合法门槛、requires、额度和回环保留。manifest/Flow/Node/instruction/input/output/Edge/manifest requires 所有未知层级改普通字段后继续真实拒绝。整数型float拒绝编辑保留原字节，仍由CLI拒绝；未知对象的float/整数/大整数保真，合法BigInt额度真实装入。

真实HTTP Host/Origin/token矩阵；缺Host由Node解析器400，其余指定不匹配403，所有拒绝都在临时物化前。固定静态与样例路由，没有任意路径路由。实际exportZIP逐文件与手写源/当前草稿比较，解压后公开add/show/verify全过；非法export只返回JSON失败，不下载ZIP。

fake边界：超时TERM→KILL、输出各流默认1MiB恰好/多一、取消、CLI失败、不完整JSON；直接child已退出而流仍开时失败保留准确root，后续观察真正close后才清理。真实HTTP断开作用于fake直接child，核PID已消失和root删除。生产期限30秒/TERM1秒按代码固定，测试fake部分使用短期限作确定的外部边界验证，不声称跑满生产30秒。

模型独立期望核未编辑Flow/二进制资源原bytes、高级非默认值、输入和边各自独立、布局不入Workbook；HTML说明保留为字面文本。实际浏览器渲染与HTML执行拒绝场景未运行，不能由模型测试代替。

## 原始失败与权限

全部失败在下表保留。npm install首次ENOTFOUND、HTTP首次listen EPERM属于沙箱环境；窄权限重试自动审核允许，未改宿主或全局配置。其余检查失败按真实缺陷/测试请求形状修复。初次EACCES自有根残留清理原件 `20261004T073626-0ac223ee.json`；不能确认流关闭测试先保留根并明确失败，未静默删除。

raw命令流保持原路径与原字节，`.gitattributes` 的binary表示仅服务原件传输，不删、改旧输出。早期 cached diff 因reporter尾空格失败的原件 `20261004T073703-1e1cf5d9.json` 仍保留。

Node v26.5.0，npm12.0.1，依赖精确版本/lock，cache `/private/tmp/sheltie-c011-npm-cache`，安装 `--ignore-scripts --no-audit --no-fund`。每个JSON都有argv/cwd/start/end/env/exit/HEAD与当次完整输入SHA闭包；同stem `.stdout`/`.stderr` 是完整实际流。候选列只核工具全量字节是否与最终闭包相同，不把历史绿色复用到变更后的工具。

原件根：`/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/specs/changes/active/C011-workbook-visual-editor/evidence/implementation`。

| 原件 stem（对应 .json/.stdout/.stderr） | cwd | 实际 argv | exit | 候选关系 |
| --- | --- | --- | --- | --- |
| 20261004T072119-f4cc856e | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm install --ignore-scripts --no-audit --no-fund` | 1 | 历史工具闭包 |
| 20261004T072351-3e76b792 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm install --ignore-scripts --no-audit --no-fund` | 0 | 历史工具闭包 |
| 20261004T072443-1ac05fb2 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node --test test/files.test.mjs test/engine.test.mjs` | 1 | 历史工具闭包 |
| 20261004T072502-185e14f2 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node --test test/files.test.mjs test/engine.test.mjs` | 1 | 历史工具闭包 |
| 20261004T072531-b282d1c9 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node --test test/files.test.mjs test/engine.test.mjs` | 0 | 历史工具闭包 |
| 20261004T073019-d54855f4 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm test` | 1 | 历史工具闭包 |
| 20261004T073056-e69b13c8 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm test` | 1 | 历史工具闭包 |
| 20261004T073257-4fe84e95 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm test` | 1 | 历史工具闭包 |
| 20261004T073359-be00382a | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm test` | 0 | 历史工具闭包 |
| 20261004T073418-28430f52 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `git diff --check` | 0 | 历史工具闭包 |
| 20261004T073418-aa1be5b8 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `scripts/check-specs.sh` | 0 | 历史工具闭包 |
| 20261004T073418-ef1d7265 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `scripts/check-docs.sh` | 0 | 历史工具闭包 |
| 20261004T073443-810cb36d | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node scripts/smoke.mjs` | 0 | 历史工具闭包 |
| 20261004T073530-fcee6bfa | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm test` | 0 | 历史工具闭包 |
| 20261004T073607-0e11605a | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `scripts/check-specs.sh` | 0 | 历史工具闭包 |
| 20261004T073607-7386de3b | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node --check public/app.mjs` | 0 | 历史工具闭包 |
| 20261004T073607-95d1066c | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `scripts/check-docs.sh` | 0 | 历史工具闭包 |
| 20261004T073626-0ac223ee | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node --input-type=module -e 'import {removeOwnedRoot} from "./lib/engine.mjs"; import {access} from "node:fs/promises"; for (const root of ["/var/folders/41/jg31h24j12nbqh334m6yzkh40000gn/T/sheltie-editor-E9PNwT", "/var/folders/41/jg31h24j12nbqh334m6yzkh40000gn/T/sheltie-editor-nzlwIC"]) { await access(root); await removeOwnedRoot(root); console.log(JSON.stringify({action:"cleanup-first-test-owned-residual",root,closedChildFromOriginalTest:true})); }'` | 0 | 历史工具闭包 |
| 20261004T073626-afbe7bf3 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `python3 -c 'import hashlib,json,pathlib,subprocess; root=pathlib.Path.cwd(); files=subprocess.check_output(["git","ls-files","-co","--exclude-standard"],text=True).splitlines(); paths=[p for p in files if p.startswith("tools/workbook-editor/") and (root/p).is_file()]; engine=pathlib.Path("/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie"); print(json.dumps({"node":subprocess.check_output(["node","--version"],text=True).strip(),"npm":subprocess.check_output(["npm","--version"],text=True).strip(),"engine_sha256":hashlib.sha256(engine.read_bytes()).hexdigest(),"tool_files_sha256":{p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(paths)},"head":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"baseline_tree":subprocess.check_output(["git","rev-parse","HEAD^{tree}"],text=True).strip()},ensure_ascii=False,indent=2))'` | 0 | 历史工具闭包 |
| 20261004T073702-221b8873 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `git add -A -- tools/workbook-editor specs/changes/active/C011-workbook-visual-editor/evidence/implementation` | 0 | 历史工具闭包 |
| 20261004T073703-1e1cf5d9 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `git diff --cached --check` | 2 | 历史工具闭包 |
| 20261004T073703-b0f8bdf6 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `python3 -c 'import hashlib,json,pathlib,subprocess; root=pathlib.Path.cwd(); baseline=json.loads((root/"specs/changes/active/C011-workbook-visual-editor/evidence/implementation/20261004T072119-f4cc856e.json").read_text()); allow=("tools/workbook-editor/","specs/changes/active/C011-workbook-visual-editor/evidence/implementation/"); drift=[p for p,h in baseline["candidate_files_sha256"].items() if not p.startswith(allow) and (not (root/p).is_file() or hashlib.sha256((root/p).read_bytes()).hexdigest()!=h)]; changed=subprocess.check_output(["git","diff","--cached","--name-only","HEAD"],text=True).splitlines(); outside=[p for p in changed if not p.startswith(allow)]; assert not drift and not outside, (drift,outside); print(json.dumps({"outside_scope_byte_drift":drift,"outside_scope_staged":outside,"changed_files":changed,"task_sha256":hashlib.sha256(pathlib.Path("/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/home/works/2026-10-04-001-workbook-visual-editor/start-inputs/task").read_bytes()).hexdigest(),"project_sha256":hashlib.sha256(pathlib.Path("/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/home/works/2026-10-04-001-workbook-visual-editor/start-inputs/project").read_bytes()).hexdigest()},ensure_ascii=False,indent=2))'` | 0 | 历史工具闭包 |
| 20261004T073824-0b4b141e | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node --test test/model.test.mjs` | 0 | 历史工具闭包 |
| 20261004T073848-646e5543 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm test` | 0 | 历史工具闭包 |
| 20261004T073927-4b15964f | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node scripts/smoke.mjs` | 0 | 当前工具闭包 |
| 20261004T074010-40c92606 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `node --check public/app.mjs` | 0 | 当前工具闭包 |
| 20261004T074010-e43c72b5 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `scripts/check-specs.sh` | 0 | 当前工具闭包 |
| 20261004T074010-f0296c99 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `scripts/check-docs.sh` | 0 | 当前工具闭包 |
| 20261004T074015-28b52ba5 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor/tools/workbook-editor` | `npm test` | 0 | 当前工具闭包 |
| 20261004T074046-5c47ba07 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `git add -A -- tools/workbook-editor specs/changes/active/C011-workbook-visual-editor/evidence/implementation` | 0 | 当前工具闭包 |
| 20261004T074046-8da8370d | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `git diff --cached --check` | 0 | 当前工具闭包 |
| 20261004T074047-749a9ff3 | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `git diff --check` | 0 | 当前工具闭包 |
| 20261004T074119-a93e1aca | `/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor` | `python3 -c 'import hashlib,json,pathlib,subprocess; root=pathlib.Path.cwd(); e=root/"specs/changes/active/C011-workbook-visual-editor/evidence/implementation"; initial=json.loads((e/"20261004T072119-f4cc856e.json").read_text()); final=json.loads((e/"20261004T074015-28b52ba5.json").read_text()); allow=("tools/workbook-editor/","specs/changes/active/C011-workbook-visual-editor/evidence/implementation/"); outside=[p for p,h in initial["candidate_files_sha256"].items() if not p.startswith(allow) and (not (root/p).is_file() or hashlib.sha256((root/p).read_bytes()).hexdigest()!=h)]; closure=[p for p,h in final["candidate_files_sha256"].items() if not (root/p).is_file() or hashlib.sha256((root/p).read_bytes()).hexdigest()!=h]; changed=subprocess.check_output(["git","diff","--cached","--name-only","HEAD"],text=True).splitlines(); bad=[p for p in changed if not p.startswith(allow)]; assert not outside and not closure and not bad,(outside,closure,bad); print(json.dumps({"outside_scope_drift":outside,"final_test_input_closure_drift":closure,"outside_staged":bad,"node":subprocess.check_output(["node","--version"],text=True).strip(),"npm":subprocess.check_output(["npm","--version"],text=True).strip(),"engine_sha256":hashlib.sha256(pathlib.Path("/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie").read_bytes()).hexdigest(),"tool_files_sha256":{p:h for p,h in final["candidate_files_sha256"].items() if p.startswith("tools/workbook-editor/")}},ensure_ascii=False,indent=2))'` | 0 | 当前工具闭包 |

## 限额与未执行

完整冻结argv：`git diff --cached --binary 5837de68c257f49c970d54254dfcc12bd7fe210d -- tools/workbook-editor specs/changes/active/C011-workbook-visual-editor/evidence/implementation`。实际exit `0`，stdout `22661296`字节，sha256 `198462438e69647956777a225b996056240e3dbdfc5a5c8fb9e21a921ab51f4f`，stderr ``；没有写出超限patch文件。当前tree可重生成同一字节；授权独立检查副本尚未操作。

独立apply/check/index tree比较、浏览器、真人接受、独立review均 `not_run`。Rust输入保持，不重跑无关948测试；不把未运行称PASS。费用/usage/真实用户时间未知。两份报告均未超262144字节，当前阶段结论为停止、待协调者处理无损封装选择。
