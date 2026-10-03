# C006 采用记录

2026-10-03，用户明确采用C004–C008的开发全范围并依次执行，授权计划人工动作与必要调整。C004 work-result/v1和C005机械实现已限定独立验收；当前只激活C006。开发需求支持结果选定后的可编辑副本，不虚构已发生复制摩擦、真实用户用途或净收益。

总体root；可信源与目标原语分作者，独立Reviewer不参与设计/代码/oracle。限定macOS arm64实际安全API，无unsafe或覆盖fallback；engine仍仅写管理根，外围未发布sheltie-export仅写明确父目录，不读Store、不回写生命周期。publish=false/dist=false，发行和外部安装不随完成发生。

目标root与父目录明确采用真实无链接绝对路径；key沿C004字符串语义，不冒充NodeId，NUL不可传argv时明确拒绝。完成只表示全字节核验与规定OS sync成功；不声称断电物理持久或同权限安全隔离。T01实测safeNOREPLACE/目录sync/身份竞态并冻结原语。

用户授权不可执行项记录后延期。原nextest0.9.145、onlinefresh和其他平台边界保持；仅有0.9.140override实际运行不称原版本已执行。真实副本用途、人工对照成本或用户接受缺失时not_run，不以fixture或工程PASS替代。C005原raw/原失败/LEAK unknown/whitespace例外保留。
