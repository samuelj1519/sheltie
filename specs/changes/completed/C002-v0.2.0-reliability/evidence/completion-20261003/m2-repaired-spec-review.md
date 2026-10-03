# C002-M2 修后完整Spec独立验收

**PASS，当前C002采用恢复义务的限定最终验收。** 受审候选 `6e33c3714c1c8d563e5559f9be9e6d66786be3e8`，根协议工作树精确SHA `57f1a1b03e07ecf91dc5361faa1a32f8d39273242f3f7ade25b33301c36623e6`。限macOS aarch64、已核APFS Data载体及可构造输入；无剩余必须恢复的当前C002 Spec缺口。Reviewer未编写实现/oracle/准备方法，只读重核根文档、真实调用链、原件及修复；未跑Cargo或改仓库。结论不从前Task或215计数推导，不批准C004–C008使用价值或发布。

## 依据与真实调用链

CONTEXT、constitution、spec、engineering及三合同七份当前文件逐SHA绑定，代码/fixture/config190与新951的134工程子集匹配。下列主链按当前源码与真实消费核查，不只看归属标签：

| 根要求 | 真实实现与验收事实 |
| --- | --- |
| INV1/2/5、GF1–5：业务无关/不读自然语言选路 | CLI解析明确动词→runtime观察→core compile/decide/legal_next；十项图结构/门槛/限额校验，四种边同规则，instruction/summary不参与判断。三类真实CLI场景、Review同时提供main/back、human普通Node等已在951执行 |
| INV3/4：只写管理根、不安装宿主、只有Workbook | Home规范根与受管FD/锁/路径；self使用bin/tmp，tar只将唯一普通sheltie成员取stdout后受管写入，不在根内自由解包，不写shell配置。requires只声明/透传。Cargo依赖/实体词汇未增第二业务Package或安装主体 |
| INV6/7、GF7/10/12/29/32：系统事实/单一权威 | runtime从UID/时钟/同FD fstat+hash观察，core仅接事实；SQLite state_json唯一，冗余列严核。status/stats/result从同一bundle的state/request/audit/effects闭包，owner清理竞态有限重取整bundle，不拼接两版。门槛by记实际OS账户，不声称独立真人认证 |
| GF6/8/9/13/14：任务书/产物/边/计数 | instruction原文与绑定引用→精确历史brief/stats bytes登记，worker目录分离；必需/限额/冻结摘要与同句柄封存；失败次数与创建number/行政superseded分开，门槛/终态/visit/retry由core决定。各合法/拒绝及exact32MiB控制真实通过 |
| GF15/16/30/31：预检/事务/恢复/拒绝 | 意图只含原参数和完整目标，replay先于@file/当前Workbook；纯预检不建Home/锁/业务，锁内重核。decide在事务外，BEGIN IMMEDIATE/CAS后业务+request+audit同事务，COMMIT后效果；schema4形状先只读识别，旧库不迁移不清空，D039控制文件例外明列 |
| publication/delete与历史original资格 | 真实owner先sync再container、NOREPLACE与同步/两端归属、删除无合法marker结果不明；protected pending不按年龄删，未提交归属清理与tmp维护分开。严格Command/data先frozenIO，合法历史next/数据不拿当前状态重写，坏snapshot无original，合法snapshot的effect错误保留original，A/B身份区分 |
| 当前GF11/14/33的已有接口边界 | 行政replace只撤销资格/继承冻结输入，不停进程或认证接手者；最终成果来自具体成功终点的required槽、effects完成才final；原字节读取先revision/资格后同FD校hash，外围副本不改引擎状态。只审这些当前接口一致性，不给其他package真实使用价值授批准 |
| GF18/19：交付和诚实边界 | skill是说明、生成的命令合同与引用自包含；技术测试/原生平台/实际使用/质量成本分别记，usage null。不把API试验当用户收益或未执行的真人trial |

## 两个Spec Finding修后关闭

F-M2-DOC-01：协议4/5准确描述既有Workbook/Flow→输入keys→新请求@file及锁内定义重验，同份输入复用；replayfirst/GF30/序号第6步未变。此协议修正单独M2收尾，不混入T58源码提交。

F-M2-01：读与write/replay共用两私有层。pure在strictCommand/check_data后、frozenIO前核Timestamp/非空principal/ApprovedData.by-at与同audit；旧七Reply execution事实在business绑定后、original生成前共享，七个match谓词逐字保留。合法历史后来状态不被current.updated_at或当前账户替代；newbad audit省original/revision，已有合法effect错误仍走with_original。新3能力含七合法历史/明确跨UTC秒、Gate6case存在/缺freeze优先序、ordinary3case及五表/完整业务原件保全；46个真实受影响消费者包含T46与合法effect/A-B阻断例外。

原native单principal反例用新Cargo冻结engine `7c961834888b085c1bd54a0302b83354c11eed480ee0941e5866b665f273e88c` 复跑：exactrid replay EFFECT_PENDING/committedtrue/causeSTORE_CORRUPT、无original/revision，status同STORE_CORRUPT，五表与全业务tree保持。原32c/engine66aa意外成功和needs_changes原件不覆盖，不称新gate绕过或真实账户动作。

## 215与原生限制不是缺省PASS

T43准确215=185dynamic/21limitedstatic/9结构保持各组实际冻结资格。37曾在T43候选实际重验、148分组冻源+delta，不称6e33全新执行或215旧原生。T58新增两个共享Fn变体另计，分别检测先audit的阶段诊断和错误Cancel event时间实际成功。19原Service目标段与32c同字节，其余196源码未改；更早audit守卫可改变他次Submit/Fail的首错原因，但不扩错误资格。21静态的拒绝集/诊断/资源边界、9pure/wiring/native caller资格前提未改，old9native仍not_run/Caughtfalse。

最新6e33 native probe再次使用新bin及134同源：arm64/Mach-O、实际APFS Data dev16777234、Unicode源/Home add四file独立SHA相同且原件/fixture回收、rawFF argv退出2且不建Home。物理badFF file/dir创建实际errno92/EILSEQ，engine物理遍历仍not_run/environment_blocked，不泛化全mac filesystems。已有951里的非法SHELTIE_HOME/HOME与显式override是真实CLI，pure entry-name bytes不是物理遍历。

旧e3未获最终批准及需修改意见/M1 SK01-SK02暂停/215原stage1Missed、旧compiler/static处分、9oldnative、物理不可构造、UTF8 pending/成本与诊断差异、模型同步/非原生断电、历史工具未知失败及usage未知均保留。不是对抗同账户任意文件篡改的隔离/全安全证明。

本次当前Spec无剩余必修项，可按采用计划登记**C002-M2 scoped PASS**并完成该package的文档/证据收尾。其他平台按用户排除，C004–C008实际使用/质量/成本仍各有独立义务；无push、release、部署或Host安装批准。
