# 独立实施项目

仓库绝对路径：/private/tmp/sheltie-usability-20261004.qvx4gv7v/workbook-editor
初始HEAD：5837de68c257f49c970d54254dfcc12bd7fe210d
初始tree：a64335ce7c5db92bbf356dc7980052e9e9439208
基线性质：本地临时快照，含用户已授权当前C011规格与先前验收记录；不是已发布产品commit。工作区初始干净。
允许写：tools/workbook-editor/；specs/changes/active/C011-workbook-visual-editor/evidence/implementation/。其他源码/测试/规格/既有证据保持；你不是唯一agent，不覆盖或撤销任何其他人的工作。输出报告只写本次begin给的outputs位置。
可信engine绝对路径：/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie
可信engine SHA256：04dba0274004bd46b35e68a2e5fc8f26387cf009b07c33f2e38e3caf75fb45d8
运行语义0.3.0-rc.1/schema4；原RootHome与当前Work管理根都不用于作者工具的草稿校验。
工具目录：tools/workbook-editor/。包为private，type=module，Node>=22。依赖先用确切smol-toml1.7.1和fflate0.8.2，产生package-lock；TOML/ZIP用公开库，不自写parser。不要引入web框架/运行平台或改Cargo/dist。安装使用npm_config_cache=/private/tmp/sheltie-c011-npm-cache及--ignore-scripts --no-audit --no-fund；只装项目依赖，不装全局/宿主资源。网络/权限拒绝按真实证据申请窄sandbox权限，不绕过。
必需检查：
- 工具目录，SHELTIE_EDITOR_ENGINE=/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie npm test；每项真实argv/stdout/stderr/退出/非零count，区分fake边界与真实CLI。
- 启动node入口/根命令README可执行，使用--sheltie可信路径和明确端口，真实HTTP check/export读取完整ZIP；进程归属清楚。
- 仓库根 scripts/check-docs.sh、scripts/check-specs.sh、git diff --check。
- 源码Rust/方法/fixture/构建配置未变，勿重跑无关948 Rust；相关真实CLI从本次可信产物调用。
原件：允许evidence/implementation目录，每命令单独JSON加stdout/stderr（可gzip），原始失败/重试留新文件，不覆盖。实际运行环境/锁/依赖/SHA和输入匹配，编译过/0退出不是内容质量。

完整patch方法（deliver执行）：
1. 初始检查副本 /private/tmp/sheltie-usability-20261004.qvx4gv7v/patch-check 必须仍在 5837de68c257f49c970d54254dfcc12bd7fe210d且干净，不能使用原Root或任何另一任务副本。
2. 在工作副本只stage允许路径：git add -A -- tools/workbook-editor specs/changes/active/C011-workbook-visual-editor/evidence/implementation。工具自己的.gitignore排除node_modules及临时运行产物；package-lock与必要源码全部入patch。
3. 用git diff --cached --binary 5837de68c257f49c970d54254dfcc12bd7fe210d -- tools/workbook-editor specs/changes/active/C011-workbook-visual-editor/evidence/implementation生成完整change.patch，包含授权未跟踪文件，最大8388608bytes。
4. 在明确独立检查副本git apply --check后git apply --index完整patch；工作index和检查副本index的git write-tree必须相同。保存实际argv/结果/文件集，不改原Root。
5. delivery报告绑定实际工作index tree、初始HEAD/完整patch/应用结果/README运行步骤及未执行项。没有独立应用/超限/授权缺失就停止。

不得自动push/merge/发布、安装应用、改宿主配置或用户正式~/.sheltie。Root后续处理受审patch整合和真人操作，工作者不能自报真人接受。
