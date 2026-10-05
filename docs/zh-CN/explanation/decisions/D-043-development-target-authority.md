# D-043：未发布候选的开发目标权威

[English](../../../en/explanation/decisions/D-043-development-target-authority.md) | 简体中文

状态：`accepted`
日期：2026-10-03
关联 change：[C006](../../history/changes/C006-result-delivery/README.md)

产品实施完成后仍可保留未发布 RC，随后执行不进入产品 release 的实验。采用此决定前，check-specs 只从 active 目标取基础版本，归档产品 package 后会误拒绝同一 RC。

开发目标在 specs/README 首屏以唯一 `开发目标：` 字段指定数值基础版本。它描述源码候选的版本目标，active plan 描述实施进度。数值产品 active 目标必须与开发目标一致；非产品实验或没有 active 时，仍核 Cargo 基础版本与同一目标，允许目标本身或其 RC。未知、缺失、重复或非法的权威字段不放行。

已发布 Cargo 版本继续逐项核 release record、tag、历史和 CHANGELOG，不以开发目标替代。产品 package completed、检查通过或实验采用均不表示发布，也不创建 tag 或上传。治理修复在 C006-T05 独立验证；具体目标只查[规范入口](../../../../specs/README.md)的字段，不从 completed 版本大小推断新目标。
