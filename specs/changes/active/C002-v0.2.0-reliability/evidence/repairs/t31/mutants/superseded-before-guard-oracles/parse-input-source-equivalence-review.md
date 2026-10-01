# parse input source存活体独立语义分析

Reviewer：/root/spec_review；当前精确变异为core/flow/parse.rs:140:32，Node分支rest.is_empty() || rest.contains('.')改为&&。两条件不能同时真，所以提前形状拒绝被移除；后续NodeId::new(head)和validate_id(rest)仍拒绝空串、额外点、非法字符/连字符、Unicode及超过64字节ID。合法两段64字节边界不变；无点与start/resource/engine分支未变。所有拒绝仍是FLOW_INVALID、rule=parse、相同调用path。

自然reason/message确可能改成具体ID错误；两种诊断准确，合同不固定原因字节或校验优先级。因此可在当前完整候选实际workspace复验仍存活时记录contract-equivalent survivor，不是逐字段响应等价，不是caught/PASS。当前证明不复用旧运行，新完整候选须保持所列校验与错误包装，并绑定实际diff/log。
