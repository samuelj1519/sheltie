# D-043：未发布候选的开发目标权威

状态：`accepted`
日期：2026-10-03
关联 change：[C006](../changes/active/C006-result-delivery/README.md)

产品实施完成后仍可保留未发布RC，随后执行不进入产品release的实验。现有check-specs只从active目标取基础版本，归档产品package后误拒绝同一RC。

开发目标在specs/README首屏以唯一`开发目标：`字段指定数值基础版本。它描述当前源码候选的版本目标；active plan描述实施进度，两者职责不同。数值产品active目标必须与开发目标一致；明确非产品实验或没有active时仍核Cargo基础版本与同一目标，允许目标本身或其RC。未知/缺失/重复/非法权威不放行。

已发布Cargo版本继续逐项核release record、tag、历史和CHANGELOG，不以开发目标替代。产品package completed、检查通过或实验采用均不表示发布，不创建tag或上传。治理修复在C006-T05独立验证；具体目标只见specs入口字段，不从completed版本大小推断新的目标。
