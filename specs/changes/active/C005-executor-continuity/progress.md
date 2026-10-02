# C005 交接

T01 已完成准备与冻结，M1 独立审定完整骨架 `8d00a29e10810c79bb5b44bdc006dc021e26637b`，下一动作 T02 公开接线。公开 CLI 与正常 next 尚未开放 replace；T02 只在三个生产文件接线，复用已实现的 core Decision、runtime 替换、观察、事务及恢复。

最终工程门禁 exit 0：fmt/check/clippy、764/764 nextest PASS（2 slow、1 LEAK unknown、10 阶段 ignore）、缓存 advisory 快照 deny、docs/specs/tests/diff。原 nextest 0.9.145 环境门禁 exit 92、not_run；实际补充执行为 0.9.140 override。初次全回归 763 PASS / 1 FAIL 及修复原件保留，未改旧 Store 断言。12 个 runtime 原语和 14 个 core 原语都包含在最终普通回归中。

未来 10 项用例在修后候选全部实际 red，编译成功且失败原因吻合；不计功能 PASS。三个历史场景保留原场景、冻结新增 replace 的预期，T02 只删 ignore。M1 core/整体代码独立短审未发现剩余生产必改；正式 M1 已通过修正后的交接与完整 SHA 复核，仅表示阶段二准备就绪。

真实撤销任务、旧执行者处置与宿主记录尚未提供，真实使用 not_run。用户已授权环境不可执行项延期；不会把机制夹具当产品收益。唯一进度来源仍是 plan。
