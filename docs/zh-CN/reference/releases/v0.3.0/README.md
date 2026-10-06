# v0.3.0 发布记录

[English](../../../../en/reference/releases/v0.3.0/README.md) | 简体中文

Status: `released`
版本：`0.3.0`
Git tag：`v0.3.0`
Release commit：`063029a9d9a41ef97ee8be48c12187312e599d1c`
工程验收闭包：`063029a9d9a41ef97ee8be48c12187312e599d1c`
发布时间：`2026-10-06T05:19:15Z`
发布目标：`aarch64-apple-darwin`

## 范围

首个对外支持基线为当前本地 macOS Apple Silicon／APFS 实现。依据 [D-046](../../../explanation/decisions/D-046-first-public-baseline.md)，旧 v0.1.0／v0.2.0 发布及其 Tag、附件已撤回。开发历史和原始验证结论仍可从固定提交查阅。不支持旧版本兼容或数据迁移；格式保留为 schema 4、cli-result/v4、work-result/v1 和 workbook-digest/v2。

发布附件包含引擎、自包含协调者 skill、源码压缩包、安装脚本、清单及校验和。sheltie-export 仅从源码构建，不包含在引擎包中；可视化 Workbook 工具从源码目录运行。Workbook 保留独立版本。源码 Tag 保留发布前状态，实际发布事实在验证后记录于本文。

## 验收

- [发布流水线 37417410662](https://github.com/samuelj1519/sheltie/actions/runs/37417410662) 在精确 Release commit 上通过 quality、plan、原生构建、全局构建、host 和 announce。CI Nextest：961 项通过、0 跳过、1 项 slow；锁定 MSRV 1.85、依赖及治理检查通过。
- 独立 Standards 和 Spec 审查通过。[C013 验证及原始输出](../../../../../specs/changes/completed/C013-first-public-baseline/validation.md) 保留最初环境设置失败、精确输入闭包及实际发布证据。
- 实际下载公开附件，核验 API 大小、SHA256 及发行清单摘要，并确认 Mach-O arm64。公开安装脚本只安装到临时 unmanaged 目录；得到的二进制与公开压缩包内字节完全一致。
- 新的独立管理根完成安装及完整两步 Work。通过默认 GitHub 来源，从未发布的 0.3.0-rc.1／schema-4 源码二进制固定更新至公开 0.3.0，再回滚；原 Store 和二进制字节完全一致。这不证明 schema 降级或旧发布格式迁移。
- 下载的 skill 通过同源码自包含交付验证。远端复核确认只剩 v0.3.0 发布及 Tag。

| 附件 | SHA256 |
| --- | --- |
| [macOS aarch64 包](https://github.com/samuelj1519/sheltie/releases/download/v0.3.0/sheltie-cli-aarch64-apple-darwin.tar.xz)，1833540 字节 | `1d4e79b4bbf0ac4d0e563bb0dc0cd3dad8579a84a2bc2a865b1f9a37f77cc900` |
| 包内 `sheltie` 二进制 | `219b884870d6a1aa824b3481df8065dee49753206284e0e2996e75586bc3cf79` |
| [dist-manifest.json](https://github.com/samuelj1519/sheltie/releases/download/v0.3.0/dist-manifest.json) | `96db972900ef4494a857d1f0f0fd920249c2f6f0570222edd3ebbe98bcc55281` |
| [协调者 skill](https://github.com/samuelj1519/sheltie/releases/download/v0.3.0/sheltie-skill.tar.gz) | `6613c9655142d63d7abeecc47318098712a6e0de3846826ee56639c6eaa2eb89` |

## 已知限制

只支持 macOS aarch64／APFS。Linux、x86_64、非 APFS 和外部物理设备认证不在范围内。历史原生变异、安全／Spec 批准、LEAK 归因、usage、费用及公平比较缺口按原记录保留；新发布检查不把这些缺口改为 PASS。Gate 只标识实际 OS 账户，不提供独立真人认证。执行者替换不会停止或隔离进程。内容质量、宿主就绪和可衡量收益与技术发布分别判断；详见[限制](../../limitations.md)及[验收边界](../../acceptance.md)。
