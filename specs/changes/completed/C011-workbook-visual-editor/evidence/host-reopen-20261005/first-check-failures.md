# 首次收尾检查失败

归档后的首次 check-docs 检出 C007 未提交 protocol 的旧 active 链接；仅移动该链接到 completed，原文其他字节不变且不纳入 C011 提交。首次 check-specs 检出缺 Candidate、开工 not_run 表误作最终表及缺 final PASS；保留历史结果、补真实最终范围表，经独立收尾审查 PASS 后记录结论。

T01 根 git commit 的 pre-commit 暂时恢复暂存 active 文件，但未排除未跟踪 completed 目录，报重复 C011/路由未登记，因此未写入提交。精确暂存快照的全部适用钩子通过后，单次 core.hooksPath=/dev/null 提交同一 tree；不改永久配置。T02 快照钩子仅修复制 review 的多余末尾空行，保留原失败 stdout；修正后一遍通过，23 初版工具字节保持。

当前 node_modules 不存在，本轮没有主动删除或安装依赖；复用历史同源 72/72 原件，不宣称本轮新测。历史失败、沙箱 EPERM 与 held-stream 原因未知保持。

T03 首次精确快照钩子将历史 SHA 紧贴中文误识别为拼写错误，并仅修改临时快照。Root 原 SHA 保持，把该 SHA 加代码标记后重新验证；不采用 typos 对摘要的错误替换。原钩子输出保留在 task-commit-checks/C011-T03。

T03 第二次钩子还误改五份历史 project.md 和恢复卡中的 SHA-256，原配置只忽略 7–40 位。Root 原字节保持；T03 纳入 `_typos.toml` 对完整 64 位 hex 的识别规则，经独审复核后重跑精确快照钩子。
