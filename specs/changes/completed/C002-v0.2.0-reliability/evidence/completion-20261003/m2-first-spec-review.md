# C002-M2 首次完整Spec审查

**NEEDS_CHANGES，候选32c085f53955c5b63a2253eefa57e7504b2cec35不能授M2 PASS。** 未参与源码/方法/oracle准备，Reviewer只读重核根词汇、不变式、规格、合同及CLI→core→runtime→Store/FS/恢复，未运行Cargo或修改源码。正常门禁或215处分资格不能消除此具体现行产品反例。

**F-M2-01：GateApprove审计主体矛盾仍被重放认领为合法成功。** `service.load_checked_request`1143+核请求/audit Work、revision、时间及Command/data/state，却未执行只读路径的`validate_read_audit`1459+；后者明确核批准`by == audit.principal`及主体非空。原响应资格须核唯一audit及业务绑定（protocol§5、GF12/INV6），不能在相同持久闭包上让status拒绝而replay相信历史成功。

当前native engine66aa，真实gated-release add→start→begin→submit→approve全部完成、该request published1；只改批准audit.principal从shushu为different-account。exactrid重放实际exit0/oktrue/replayedtrue/byshushu，status实际exit1/STORE_CORRUPT，明细为audit主体/时间不一致。未新增批准/业务行，也未改写历史或文件；五表与业务树分别在两调用后保持。Reviewer读取完整原件、核唯一差异audit行principal，并只读核保留Home当前五表与changed_rows相同。repro JSON SHA045d97525284001323a58d26991db6004c74ef76802ca6fc04973ae9cafbd8f1。

必修范围是现有写/重放请求的audit execution-facts资格，与读取使用同一规则后才构造可信original；保持T46严格Command/data先于冻结读取及已核合法快照遇effect错误保留original。不引新状态、public getter、观察点或额外业务IO。健康控制及单字段主体/相关事件时间反例、完整原响应/身份、五表与物理原件保全，受影响consumer和新冻结完整门禁/独审必须另验。该问题是历史original资格差异，不称新gate绕过或真人认证失效。

F-M2-DOC-01已局部解决：protocol work start4/5按service203–250准确改为Workbook/Flow→keys→新请求@file内容，锁内重核并复用内容。无源码变动，replayfirst/GF30/序号步骤不变；新protocol SHA57f1a1b03e07ecf91dc5361faa1a32f8d39273242f3f7ade25b33301c36623e6。原首次Finding与候选结论保留，后续修后审查另立，不覆盖旧needs-changes。
