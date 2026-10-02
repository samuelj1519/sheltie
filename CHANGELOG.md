# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [0.2.0]

### Breaking Changes

- Store 升为 schema 2；旧 schema 1 管理根拒绝读写并保持原样，不自动迁移或清库。
- 使用 `cli-result/v2`、`workbook-digest/v2` 和新的 Attempt 目录布局；历史 v0.1.0 记录由旧二进制配旧管理根读取。

### Fixed

- 请求身份绑定真实目标，重复请求返回提交时原响应；恢复历史产物时核字节与归属，当前状态卡不回退。
- Workbook 与 Work 的发布、封存、删除及崩溃恢复统一守文件句柄、事务与根内边界；严格拒绝重复 Flow、未知嵌套字段及不可信持久状态。
- 成功写操作安全维护过期 tmp，异常仅告警；pending 按归属与恢复事实处理。
- 协调者补齐输入发现、合法下一步与人工门槛说明；spec-dev 保留原始基线、已验证前缀、完整需求与真实重规划交接。
- spec-dev 0.2.1 修正反思报告的 Attempt 目录说明，旧冻结版本保留。

### Changed

- 许可统一为 MIT，自包含 skill 交付按同版本合同打包。

### Known Limits

- 本版本只发布 macOS aarch64/x86_64，Linux 按用户决定排除；原 Linux CI 失败记录保留。

- 历史 M1 的 SK01/SK02 例外、215 项额外执行与最终 Spec 批准缺失仍保留，不代表完整变异或安全验证通过。
- T16 是当前 Codex 的实际本机场景；usage 缺失，手动 skill 加载未验证自动发现，没有成本收益对照。
- 本版本两平台实物、同 SHA 质量和真实远端更新/rollback 的结果见对应 release record；历史本机验证不替代该发布证据。

## [0.1.0] - 2026-09-26

### Bug Fixes

- **(ci)** 删去 build.yml 的 release job，发布只走 cargo-dist - ([9d60711](https://github.com/samuelj1519/sheltie/commit/9d60711373bba2870059e15b82758385b937e0bd)) - Samuel-J
- **(cli)** gate approve 的 data 补上 at 字段 - ([dc05b1d](https://github.com/samuelj1519/sheltie/commit/dc05b1d9cd2c17e72824bd2df52bf7eadd20d8a9)) - Samuel-J
- **(cli)** 删掉 next_ops_of 的 JSON 往返与静默兜底 - ([36e8577](https://github.com/samuelj1519/sheltie/commit/36e8577856a4f7f7d6d20a96c6cb56f5bf240e61)) - Samuel-J
- **(cli)** 版本断言跟随 CARGO_PKG_VERSION；登记 T25/T26 白名单 - ([7e4243a](https://github.com/samuelj1519/sheltie/commit/7e4243adb76ea9a31ba127c98fc14077dbb00a66)) - Samuel-J
- **(core)** 数字段拒绝前导加号与前导零，Han 区段补齐，错误字段归位 - ([e55effd](https://github.com/samuelj1519/sheltie/commit/e55effddfbdf94914935b27c76ee2d9f962b0618)) - Samuel-J
- **(core)** 规则 5 拒绝把可选输出当必需输入 - ([5f4b6d6](https://github.com/samuelj1519/sheltie/commit/5f4b6d6ece4b8dfa1bb8f646187bae2d4226b006)) - Samuel-J
- **(core)** 图带全量 requires，版本列查声明 - ([51eda62](https://github.com/samuelj1519/sheltie/commit/51eda626d1dfbf22788451a4d2bad07b27973c15)) - Samuel-J
- **(core)** 未绑定输入行照合同写上游节点 - ([496a182](https://github.com/samuelj1519/sheltie/commit/496a182834bf5ac06dcd2218f8d2e220360d3de2)) - Samuel-J
- **(core)** 读回不豁免上限与规范化，Graph 不可反序列化 - ([f47af48](https://github.com/samuelj1519/sheltie/commit/f47af4834ec55bc9e8741d3ad8eefdca74030f95)) - Samuel-J
- **(core)** engine.stats 序列化失败就报错，只算一次 - ([03a9be9](https://github.com/samuelj1519/sheltie/commit/03a9be9bfe1642fc30ba11170de738ab8e36fdc7)) - Samuel-J
- **(core)** 说明列随类型写「此 <kind>」，耗时改走 unix_secs - ([bc14d36](https://github.com/samuelj1519/sheltie/commit/bc14d36c0383ddb506f30479bdaa4a395c2e5a3a)) - Samuel-J
- **(core)** engine.stats 口径含本次 Attempt - ([00eaeeb](https://github.com/samuelj1519/sheltie/commit/00eaeeb9eab574068f2e8e15f8e4c0818cbecf54)) - Samuel-J
- **(core)** 持久化结构补 deny_unknown_fields - ([c5d567c](https://github.com/samuelj1519/sheltie/commit/c5d567c51284fc08a171cdc311acf54ec2db9f8c)) - Samuel-J
- **(release)** 发布源与仓库地址统一为 samuelj1519/sheltie - ([7dae6d0](https://github.com/samuelj1519/sheltie/commit/7dae6d01364570085406d2dff698acc61a7d6a2f)) - Samuel-J
- **(runtime)** stats.json 重放重建、failpoint 归位与 M2 补测试 - ([1a5d7a7](https://github.com/samuelj1519/sheltie/commit/1a5d7a76f1a7bada8f198f3218765bc94d298a7b)) - Samuel-J
- **(runtime)** self install 先建 store.db 再做幂等短路 - ([67332b6](https://github.com/samuelj1519/sheltie/commit/67332b6dbe16efcac40cf7bb416b62c0c5a19372)) - Samuel-J
- **(runtime)** rollback 用 remove_file 删除被丢弃的旧二进制 - ([8c5c7cf](https://github.com/samuelj1519/sheltie/commit/8c5c7cfdc7871dbc3fed4bf90e146a322c7faade)) - Samuel-J
- **(runtime)** 适配 cargo-dist 真实清单与包内布局 - ([d5d2008](https://github.com/samuelj1519/sheltie/commit/d5d200827e1d86b5b82f00a3d2674522001d1643)) - Samuel-J
- **(scripts)** check-task.sh 兼容 macOS 自带的 bash 3.2 - ([8b33c48](https://github.com/samuelj1519/sheltie/commit/8b33c48f1fb5bdfee7d9eac9d6864d94139a4cdf)) - Samuel-J
- **(scripts)** check-task 误判混合文件、共享占位与检查时机 - ([deb31ed](https://github.com/samuelj1519/sheltie/commit/deb31ed8a6c6ce5126954069e3d20cd7fc7926e1)) - Samuel-J
- **(scripts)** check-task 范围取并集，修全角冒号与空数组 - ([904a54c](https://github.com/samuelj1519/sheltie/commit/904a54c69ee9eb1fd61e91cb062064fa6b8bcaec)) - Samuel-J
- **(scripts)** mutants 输出收进 target，不再弄脏工作树 - ([82d870b](https://github.com/samuelj1519/sheltie/commit/82d870ba7192ceff6e7a40fccfb79466830fb912)) - Samuel-J
- **(scripts)** 检查 2 对文件条目也只查 .rs - ([767084b](https://github.com/samuelj1519/sheltie/commit/767084b5b805677316dfe25e3cf82490123a1599)) - Samuel-J

### Documentation

- **(cli)** self_cmd 头注更正为「install 会建 store.db」 - ([dbebdcd](https://github.com/samuelj1519/sheltie/commit/dbebdcd90331a41e8fdfd0862932f64ea7ca0574)) - Samuel-J
- **(readme)** 快速开始改为可从 install.sh 走通的真实流程 - ([e502a92](https://github.com/samuelj1519/sheltie/commit/e502a92a0a25c38fa4493d0d9e4f2904f74254a1)) - Samuel-J
- **(readme)** 按实测卡点补三处说明 - ([022764c](https://github.com/samuelj1519/sheltie/commit/022764c66d100e2f235d7fa2c64a61589b12e13e)) - Samuel-J
- **(specs)** 建立 Sheltie 规格、合同、实现计划与 spec-dev Workbook - ([82099e1](https://github.com/samuelj1519/sheltie/commit/82099e1870c185e14085e9698947167d33785f6e)) - Samuel-J
- **(specs)** 明确 work stats 的 blocked 与 approvals 口径 - ([ac17547](https://github.com/samuelj1519/sheltie/commit/ac17547e1709db1b2021553ae6b8f2a543f32591)) - Samuel-J
- **(specs)** 修正 D-28 事实，补 9999 饱和，找回 D-25 事实段 - ([64f4647](https://github.com/samuelj1519/sheltie/commit/64f4647dcfcba8890d45dc818aeb4cafdfd31148)) - Samuel-J
- **(specs)** M2 里程碑审查报告、D-29 与流程教训 - ([268ec6d](https://github.com/samuelj1519/sheltie/commit/268ec6d7904a7ce9ea40bc7b641395fcad6f5ef8)) - Samuel-J
- **(specs)** M3 里程碑审查报告：端到端十二场景全数对上测试 - ([458e947](https://github.com/samuelj1519/sheltie/commit/458e947b021012b67ce8082f90b636d7361c1f58)) - Samuel-J
- **(specs)** M3 复审记录、D-31 与协议 install 口径统一 - ([a3b63f7](https://github.com/samuelj1519/sheltie/commit/a3b63f78dd49d4d141d5e6257bba8da3a659c0be)) - Samuel-J
- **(workspace)** 全仓中文去翻译腔，改成地道简体中文 - ([dc44577](https://github.com/samuelj1519/sheltie/commit/dc4457728352fdec105e0e25128717f09d4d75ac)) - Samuel-J
- **(workspace)** 清掉英文名词直译，换成地道中文 - ([18c65fa](https://github.com/samuelj1519/sheltie/commit/18c65fad1874c6403cf20e24596515b960991bea)) - Samuel-J
- **(workspace)** 定名统一为响应封装、唯一状态权威、引用检查 - ([fa4710b](https://github.com/samuelj1519/sheltie/commit/fa4710bbe92a3a194313000bba0c1bc6a88964c7)) - Samuel-J

### Features

- **(cli)** workbook add/list/show/remove/verify - ([71930b6](https://github.com/samuelj1519/sheltie/commit/71930b6b5caa88bb4808e5932270a84b54e75f60)) - Samuel-J
- **(cli)** work start/list/status/cancel - ([63dea79](https://github.com/samuelj1519/sheltie/commit/63dea790c2bc8a9a0255b29e57a6e748d3f1938b)) - Samuel-J
- **(cli)** attempt begin/submit/fail 与 gate approve - ([72794ea](https://github.com/samuelj1519/sheltie/commit/72794eaf6dbcd728d77bedbb9e142b636c40d080)) - Samuel-J
- **(cli)** self 命令组与 cargo-dist 发布链 - ([420157e](https://github.com/samuelj1519/sheltie/commit/420157e229e0ac998e66e2796a8de22f3bb53b57)) - Samuel-J
- **(core)** 强类型 ID、相对路径、有界文本与摘要 - ([ef584ab](https://github.com/samuelj1519/sheltie/commit/ef584ab0098104db2f73aefacd16b7a106693d95)) - Samuel-J
- **(core)** 解析 workbook.toml - ([b292a38](https://github.com/samuelj1519/sheltie/commit/b292a38663aaeb2c6b91f52305132366cc6d0d99)) - Samuel-J
- **(core)** 解析 flow/v1 的节点、边与输入来源 - ([7c45ed8](https://github.com/samuelj1519/sheltie/commit/7c45ed80d4158245b99192897760e9df14a07d18)) - Samuel-J
- **(core)** 把 Flow 编译成校验过的图 - ([7f23d81](https://github.com/samuelj1519/sheltie/commit/7f23d8167b4cecfb99d76b15f479dfc3812d3e8c)) - Samuel-J
- **(core)** Work 状态、Start 命令与首个合法下一步 - ([12a6ade](https://github.com/samuelj1519/sheltie/commit/12a6ade437baedd4e1c0800267f6ed16ae1bc8a7)) - Samuel-J
- **(core)** BeginAttempt 选边、计数与输入冻结 - ([9044612](https://github.com/samuelj1519/sheltie/commit/9044612f4ab329ca27367a3f1e4cda99c93fb060)) - Samuel-J
- **(core)** SubmitAttempt 与 FailAttempt 及输出合同校验 - ([4334925](https://github.com/samuelj1519/sheltie/commit/4334925b67ba61a4b340385fe410f351fed727c1)) - Samuel-J
- **(core)** 门槛批准、取消与终态守卫 - ([810730f](https://github.com/samuelj1519/sheltie/commit/810730fabf20d02b7e42649c2228752226eaebb1)) - Samuel-J
- **(core)** 渲染任务书、状态卡与 next 投影 - ([8af5f94](https://github.com/samuelj1519/sheltie/commit/8af5f9491d5fe8467b9f99df12a5bd6525cde513)) - Samuel-J
- **(retro)** 反思节点、engine.stats 输入与 work stats 事实视图 - ([6a26fc6](https://github.com/samuelj1519/sheltie/commit/6a26fc60d0e97329f74549c077b6224aca2cdfd7)) - Samuel-J
- **(runtime)** 管理根解析、路径约束与文件观察 - ([9b2d00f](https://github.com/samuelj1519/sheltie/commit/9b2d00f4a39742f7cb6adaaaef70d4c224d4346a)) - Samuel-J
- **(runtime)** SQLite 存储：结构校验、去重、revision CAS 与序号 - ([a860311](https://github.com/samuelj1519/sheltie/commit/a860311a94c042ef726bcd1146e5d9d74cab6199)) - Samuel-J
- **(runtime)** Workbook 仓库与 staging 原子入库 - ([00bd835](https://github.com/samuelj1519/sheltie/commit/00bd835f52e5147afcbfc8a88708ac6e0af1db20)) - Samuel-J
- **(runtime)** Workbook remove 引用检查与 verify - ([fc4f9f9](https://github.com/samuelj1519/sheltie/commit/fc4f9f97312affa62c4c69255a04c68abac74ce0)) - Samuel-J
- **(runtime)** Work 服务补齐 begin/submit/fail/approve 与只读视图 - ([2344894](https://github.com/samuelj1519/sheltie/commit/2344894fd4bae0aafe63f3200811b8c089282dd2)) - Samuel-J
- **(skill)** sheltie SKILL.md 与命令白名单检查 - ([86ef062](https://github.com/samuelj1519/sheltie/commit/86ef062e5bccbf1966cd63444776f8601b714a46)) - Samuel-J

### Miscellaneous Chores

- **(release)** dist 配置挪进 Cargo.toml、定 install-path 并再生 release.yml - ([5286e97](https://github.com/samuelj1519/sheltie/commit/5286e97d4392ede3a46dc2b4ca4306a58fef893d)) - Samuel-J
- **(review)** T02 复核，补三条边界测试，迭代填空流程 - ([9d73175](https://github.com/samuelj1519/sheltie/commit/9d731753721113077db040dd4da7db78247f6673)) - Samuel-J
- **(review)** 采纳实现者对 T02 复核的反驳，修骨架残留与工具盲区 - ([a9ed84c](https://github.com/samuelj1519/sheltie/commit/a9ed84c742546b8f3f92b7f06514a3d23ff88a5e)) - Samuel-J
- **(review)** 修正 T05 空白文本测试的替换串 - ([ac5eb9c](https://github.com/samuelj1519/sheltie/commit/ac5eb9cfdd19b0c8d07ab8db4bed373d20cdfff9)) - Samuel-J
- **(review)** 修正样例与夹具里不合 ID 规则的输入名 - ([00ca15a](https://github.com/samuelj1519/sheltie/commit/00ca15adc07f8aeb471f8e88b7ebf41c17a59ace)) - Samuel-J
- **(review)** 钉住 T05 规则 9 的拒绝例 - ([8aad2e9](https://github.com/samuelj1519/sheltie/commit/8aad2e95cefd11bd3f9cb28271a947643a47e50d)) - Samuel-J
- **(review)** 补齐 spec-dev 夹具文件并修正 review 回环夹具 - ([43ec6bf](https://github.com/samuelj1519/sheltie/commit/43ec6bf3bd95e6097b6bbf0e71b3c58fad9818a7)) - Samuel-J
- **(review)** 输入名只查唯一，回退 kebab 改名 - ([685d4f2](https://github.com/samuelj1519/sheltie/commit/685d4f2040b1a85d4693d96c7eb8c955bdac288d)) - Samuel-J
- **(workspace)** 搭起三 crate 骨架、全部禁用测试与任务工具 - ([461f5c7](https://github.com/samuelj1519/sheltie/commit/461f5c7a3ec4db47ee6042922da5eac2cf0bcb51)) - Samuel-J

### Refactoring

- **(core)** 去掉重复的摘要长度检查与冗余提前返回 - ([6290c61](https://github.com/samuelj1519/sheltie/commit/6290c613fc07354aab06f25a1e10dffaba60cc0f)) - Samuel-J

### Tests

- **(cli)** 审查回环场景：回边、访问上限与参考输入 - ([96c79f9](https://github.com/samuelj1519/sheltie/commit/96c79f9307e2e89925faa63c12d21c80017aafed)) - Samuel-J
- **(cli)** 门槛、产物完整性与 Workbook 生命周期场景 - ([3c62ead](https://github.com/samuelj1519/sheltie/commit/3c62eadd060fb5188f925e97bbc5975348578752)) - Samuel-J
- **(core)** M1 里程碑审查补测试、删死代码与流程教训 - ([6ace5a3](https://github.com/samuelj1519/sheltie/commit/6ace5a30dc74a8bf62bcd0ee394deb02cfc6d7cd)) - Samuel-J
- **(core)** M1 第二轮出题，requires 原样给声明，Timestamp 收紧 - ([caa9d23](https://github.com/samuelj1519/sheltie/commit/caa9d2365474f789b65065a837efe2637e48e50c)) - Samuel-J
- **(core)** M1 复审通过：补交叉声明与远年时间用例，关闭里程碑 - ([94519d7](https://github.com/samuelj1519/sheltie/commit/94519d7b59c2c6268c7581641894b90632fd48ed)) - Samuel-J
- **(core)** 样例 Workbook 按合同编译通过 - ([9ca2526](https://github.com/samuelj1519/sheltie/commit/9ca25261cf85a297723d631a5c9c9c1b1a792ef3)) - Samuel-J
- **(runtime)** 修 T15 缺目录测试的 chmod，目录保留可遍历 - ([fe7f5fe](https://github.com/samuelj1519/sheltie/commit/fe7f5feb056fdf77cc81d8a10c06cf9d80edbb2c)) - Samuel-J
- **(runtime)** 崩溃窗口、请求重放与升级恢复 - ([64c34ad](https://github.com/samuelj1519/sheltie/commit/64c34ad2cc0acbf90ef0b7944b9861a7b4697793)) - Samuel-J
- **(runtime)** M3 补测发布链与 install 语义 - ([fe91704](https://github.com/samuelj1519/sheltie/commit/fe91704601948d8a9c302501b875212fad3e60d7)) - Samuel-J
- **(runtime)** 后缀判定的 .tgz 区段补一条解包测试 - ([7008803](https://github.com/samuelj1519/sheltie/commit/700880308c03deea8eaa4611f657e39bf46c304d)) - Samuel-J
- 测试名改写行为，任务归属改用 // Task 注释 - ([f1a8715](https://github.com/samuelj1519/sheltie/commit/f1a871553b7992f044e2f83f70ae5fb8576c14da)) - Samuel-J

<!-- generated by git-cliff -->
