停止：完整 patch 为 22661296 字节，超过冻结的 8388608 字节上限；实现和必需工程检查已完成，尚不可按现有封装交付。

# 候选与依据

本次为真实 Workbook 网页作者工具任务。已完整读取 begin brief、绑定 task/project、clone 的 AGENTS/CONTEXT/specs 入口/engineering、active C011 README/plan/design、GF-34、Workbook 与 CLI 合同。没有改冻结输入、Store、根工作区或 Rust。

独立仓库：`/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor`。
初始 HEAD：`5837de68c257f49c970d54254dfcc12bd7fe210d`，初始 tree：`a64335ce7c5db92bbf356dc7980052e9e9439208`，开始时工作区干净。
当前 HEAD 保持基线，允许范围已 stage；当前 index tree：`b4ae44db1db8273f59f0b1ecf51a717913a13a0c`。
候选 tree 为真实未提交变更，不代表 review、真人接受或 C011 完成。源文件与最后一次完整 npm test 的 SHA 闭包相同，见 `20261004T074119-a93e1aca.json`。

可信引擎：`/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie`，SHA256 `04dba0274004bd46b35e68a2e5fc8f26387cf009b07c33f2e38e3caf75fb45d8`。Node `v26.5.0`、npm `12.0.1`；项目未冻结 npm 版本。依赖 smol-toml `1.7.1`、fflate `0.8.2`，package-lock 完整保留。

# 实际变更

此前没有作者工具目录。本次新增完整本地服务与可编辑界面：

- `server.mjs`：固定静态路由、固定 code-change/spec-dev 样例、会话 token、check/export；只监听 127.0.0.1，严格 Host/POST Origin/token，无任意宿主路径、shell 或客户端二进制入口。
- `public/files.mjs`、`lib/files.mjs`：目录 File 清单在读取前核限额，逐项数组在解码前核数量/大小/严格 base64 和整组路径；拒绝逃逸、重复、ASCII/NFC/目录别名和文件前缀；物化后实际集合与原字节核对。
- `lib/engine.mjs`：0700 新自有根、直接 argv/新 Home、30 秒/各流 1 MiB；TERM 1 秒后 KILL，等待直接 child 退出/流关闭；无法确认时失败并保留 residualRoot。引擎只读目录只在自有根、close 之后恢复权限清理。
- `public/model.mjs`、`toml.mjs`：Map 保留原文件，完整 TOML 对象编辑，未编辑文件不序列化。`integersAsBigInt:true` + `numbersAsFloat:true` 保留整数/float/大整数区别；不能表示的已知字段拒绝编辑而保留原字节；未知字段所有层级仍保留并继续被 CLI 拒绝。
- `public/index.html`、`app.mjs`、`style.css`：中文深灰蓝三栏；方法与多 Flow、节点增加/删除/拖动、平移/缩放、入口选择、显式边和独立 input 编辑；说明 file/text、输出/结果与高级声明、检查详情和完整新 ZIP 下载。标题/说明/路径/错误用 textContent；布局只存 localStorage。保存期间锁住编辑入口，避免检查候选漂移。
- `test/`：23 个真实运行案例；手写字节/合同期望、真实 CLI、真实 HTTP 和 ZIP 解压后的公开命令，fake 只用于 child 生命周期边界。
- `README.md`、`scripts/record.py`、`scripts/smoke.mjs`、包与 lock：可重跑安装/启动/检查入口。工具自己的 .gitignore 仅排除 node_modules。
- 本 package `evidence/implementation/`：所有失败/重试和实际检查原件；`.gitattributes` 仅把原始 stdout/stderr 作为 binary 保留，避免修改真实 Node reporter 尾空格。源码、JSON 和 Markdown 的 diff 检查保持。

权限范围外字节漂移为零，Rust/方法/fixture/构建配置保持基线。未自动 commit/push/merge/发布。

# 检查与真实缺陷修复

最终 `npm test` 为 23/23，exit 0，原件 `20261004T074015-28b52ba5.json`；入口 smoke、docs/specs、app 语法和 staged/unstaged diff 检查 exit 0。完整索引见 checks.md。

原失败全部保留：首次 npm ENOTFOUND；沙箱 HTTP listen EPERM；真实引擎只读副本导致早期 cleanup EACCES；响应错误地假设 schema 字段；fake 启动期限过短；HTTP 测试 GET 多余正文/缺 Host 的 Node 解析边界；16 MiB base64 大正则栈限制；raw reporter 尾空格的 cached diff 检查。已按真实原因修复实现或具体测试输入，不删除失败。初次 EACCES 的两个自有残留已在观察原直接 child 结束后准确清理，见 `20261004T073626-0ac223ee.json`。

协调者在准备时指出整数型 float 不能被标题修改洗成整数，本次采用库的类型保真组合并加了真实 CLI 拒绝与合法 BigInt 回归；没有修改任务标准。

# 运行

在仓库根：

```bash
npm_config_cache=/private/tmp/sheltie-c011-npm-cache npm --prefix tools/workbook-editor ci --ignore-scripts --no-audit --no-fund
node tools/workbook-editor/server.mjs --sheltie /private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie --port 4311
```

打开 `http://127.0.0.1:4311`。新建方法或打开目录/固定样例，选择节点填写属性，增加显式边并独立配置 input，检查后保存 ZIP；解压后重新打开目录。详情见工具 README。

接口：GET `/api/session`、`/api/sample/code-change`、`/api/sample/spec-dev`；POST `/api/check` 和 `/api/export`，`Content-Type: application/json`、精确 Origin 和 X-Editor-Token，正文 `[[path,base64],...]`。

本次 smoke 启动的 PID 75351 已正常退出，exit 0，端口释放。当前没有由本工作 agent 保留的服务进程。

# 冻结 patch 方法与阻断

适用基线 `5837de68c257f49c970d54254dfcc12bd7fe210d`；授权检查副本 `/private/tmp/sheltie-usability-20261004.qvx4gv7v/patch-check` 必须仍在基线且干净，不能使用 Root 或其他任务副本。

project 的完整生成/应用方法保持：

```bash
git add -A -- tools/workbook-editor specs/changes/active/C011-workbook-visual-editor/evidence/implementation
git diff --cached --binary 5837de68c257f49c970d54254dfcc12bd7fe210d -- tools/workbook-editor specs/changes/active/C011-workbook-visual-editor/evidence/implementation > change.patch
git -C /private/tmp/sheltie-usability-20261004.qvx4gv7v/patch-check apply --check /absolute/path/change.patch
git -C /private/tmp/sheltie-usability-20261004.qvx4gv7v/patch-check apply --index /absolute/path/change.patch
git write-tree
git -C /private/tmp/sheltie-usability-20261004.qvx4gv7v/patch-check write-tree
```

必须包含授权未跟踪文件、lock 与必要源码；两个 index tree 必须相同，patch 至多 8388608 字节。本工作者只 stage 自有范围，未操作检查副本。

当前用同一真实 git argv 在内存量测完整输出：exit `0`，stdout `22661296` 字节，sha256 `198462438e69647956777a225b996056240e3dbdfc5a5c8fb9e21a921ab51f4f`，stderr ``。超过上限，未写出超限 change.patch，独立 apply 为 `not_run`。主要体积是每条命令完整全仓 SHA 闭包的重复 JSON。

按 brief 达限停止，没有截掉事实、删掉失败或扩大上限。已把无损压缩/原件读取容器选择交协调者；当前未改变 JSON 原件或封装规则来绕过停止。

# 限制与交接

浏览器真实交互、DOM 的 HTML 拒绝实测、窄屏/焦点/布局截图：`not_run`，由协调者和独立 reviewer 核验。真人新建/编辑/关闭重开及接受：`not_run`。日常独立 review：`not_run`。完整 patch 文件与独立应用：因超限 `not_run`。不把工程通过写成这些接受结果。

草稿不跨页面持久化，关闭/刷新前需下载 ZIP；没有 ZIP 导入。高级合法声明保真，不能表示的非法声明需先在外部修正后重开。引擎错误的字段路径原文保留，底部提供方法和各 Flow 原文入口，不猜错误来自哪个 Flow。资源文件保留，不增加资源管理系统。只承诺直接 child 的生命周期，不声称整棵进程树或同一 OS 账户下的人类认证。

使用量、费用、真实用户个人活动时间均未知。本次为真实新功能实施和可用性任务，不属于 fixture 或公平六 run 对照。后续协调者先处理封装限额，再安排 review、真实浏览器和真人接受。
