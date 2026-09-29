T30 Standards：通过。

独立Reviewer /root/standards_review；只读复核，未参与实现，未自行运行测试。最终确认冷读历史显式更正、骨架模板移除编号测试名前缀、escalate直接绑定spec/approval与精确格式、human只从自身输入取得路径和摘要。此前指出的继承、模板、原始证据与失败报告缺字段问题已闭合。无新增阻断项。最终门禁须绑定新增human输入后的稳定源码；Linux not_run。

后续独立增量复核通过：原runtime crash 9条拆为CLI子进程7条+runtime直接API2条，OS参数与OS身份CLI用例完整迁移；正文/Owner保留。全crates无测试内Cargo构建入口。marker同步例仅failpoint启用，状态卡API保持默认覆盖。最终门禁绑定全部迁移后的候选。
