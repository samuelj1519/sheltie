# 修复独立项目

仓库 /private/tmp/sheltie-c011-edge-repair-20261004
基线 HEAD dfe3846db852e31501b74f277025d4f9020c8919
允许写 tools/workbook-editor/public/model.mjs、public/app.mjs、test/model.test.mjs，以及当前C011 evidence/edge-repair/implementation 下原始记录。不是唯一agent，不回退别人的修改。不要改Root、旧独立候选或旧Work输出。
可信engine /private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie
engine SHA256 04dba0274004bd46b35e68a2e5fc8f26387cf009b07c33f2e38e3caf75fb45d8
检查工具目录 SHELTIE_EDITOR_ENGINE=上述路径 npm test；保存命令、stdout/stderr、exit和实际count；必要的原有测试与真实CLI复查。根docs/specs/diff。Rust/格式无变不跑无关948。输出完整相对基线patch且包含所有允许文件，无push/发布/全局安装。新修复Work与旧Work不同，不伪造原Work返工；报告输出只能声明位置。
