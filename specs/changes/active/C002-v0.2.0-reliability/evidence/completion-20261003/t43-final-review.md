# T43 / 215 项最终独立汇总审查

结论：**PASS，限定当前采用的215项处分资格，macOS aarch64。** Reviewer未参与oracle、实现或准备编写，只读核源码、合同、caller、精确集合、实际日志、输入记录及档案，未运行Cargo或改源码/plan。绑定候选 `4487ac1eb20672b8136aba186e8392fafa38ebca`；本次所审最终账本 `t43-215-dispositions.json` SHA为 `c2beb0898b23013daea56f3eaa4edfd034822b75077bef180a0ed5ff4535e32e`。本结论不授M2或全产品完成。

## 准确集合和资格

原固定Git归档e54dcd41的215个old ID逐一核source/diff与原stage1 outcomes，仍全为MissedMutant；无重、漏、外项。185动态+21限定静态+9结构=215。当前库存206可表达target唯一；9旧表达式移除、target=null。204个可表达target的删除/替换行与原逐字相同，另两个完整Fn删除受directory pure替换或模型sync设施影响，分别由T55/T56真实当前consumer资格覆盖；九个函数名变化按表达式/function/column/genre/replacement审，不猜旧行号。

| 当前处分 | 个数 | 本次验收依据 |
| --- | ---: | --- |
| 当前相同190输入的实际动态执行 | 37 | 本次补30、当前schema2、T57当前5，各自准确target、非零控制、BuildSuccess/TestFailure100、目标实际红、0timeout与输入稳定 |
| 其他分组冻结动态 | 148 | T49–T56各自完整190输入、原log/diff/phase及独审资格，加当前production/caller/fixture/观察点delta复核；不称最新重跑 |
| 限定静态 | 21 | 完整当前producer/caller、同一不可变事实及支配guard仍成立，限定拒绝集/分类或普通观察范围；动态not_run/Caught=false |
| 当前结构替换验证 | 9 | T55 pure关系、3+4接线、真实caller/清理正反例、独立pure-only新9机制变体实际红；原9native not_run/Caught=false |

185不是185次最新候选执行，215不是215次旧原生执行，也不是215个Caught。单mutant的不同失败类型及不变前提按各独审保留。

## F-T43-01 修后资格

原finding保留 `t43-findings.md/json`：早期30项126/127记录仅覆盖crates，缺根Cargo/lock、nextest配置及外部fixtures，无法补造完整执行输入。Root以新当前190补验解决，旧运行/缺失记录另存，不重写历史或增加old ID。

| 新批次 | 实际Captured | 秒数 | 正式baseline | 补充真实control |
| --- | ---: | ---: | ---: | ---: |
| G01-A | 12/12 | 160.646 | runtime3/3 | CLI/runtime共享filter7/7 |
| G01-B | 7/7 | 99.548 | runtime3/3 | 同190/filter的7项控制另列 |
| early11 | 11/11 | 169.107 | runtime31/31 | CLI/runtime43/43 |

三批实际exit0、0Missed/timeout/unviable，完整190逐SHA相同并匹配当前。自动prune后的正式基线不冒称7/43；fail-fast未运行的其他mutant阶段测试不变成PASS。A及较早controls的六项继承binary/fault env未在dispatch记录，保留unknown；B/C实际dispatch JSON六keys全null，env3为empty RUSTC_WRAPPER/relative target/jobs2。A真实CLI目标红直接核，不靠旧T57父环境snapshot推定null。

30项逐目标红已核：Publication7、load binding2、metadata4及asset3是纯合同消费者的拒绝失效，各自真实producer/合法控制范围保留；Start/Fail两项CLI重放确意外成功；Submit两项只错误暴露不合格original，后层仍EffectPending；Start data/Reply两项仍拒但错误借后层冻结副本缺失诊断；child四项raw确unix_wait_status(9)，在显式release前被kill，不是timeout。其余Store四项分别为坏schema后层仍拒但主库bytes已变化、独立PRAGMA仍delete非wal、revision0错报CAS conflict、真实非Constraint INSERT错归WorkbookExists。F-T43-01因此**另列resolved_by_current30**，原finding不覆写。

## 当前源码/证明复核

完整原文见 `t43-current-delta-qualification.md` 和 `t43-static-current-delta-review.json`。从早期执行源码到当前，业务资格/恢复/查询相关生产文件保持或仅追加测试；T53未修改workbook_digest算法。新增feature观察点无匹配时不增加观察文件I/O，原stat/open/锁/EXCHANGE/fsync/rename/unlink顺序与既有pause阶段保持。原rendezvous name包装仍传name bytes，只有新tmp观察点传实际随机路径。T55两个pure判定与同一held fstat、before/after布尔角色同式，IO及错误序未变；T56是实际created FD之后的模型sync故障，T57生产/观察点新增均0。

当前21项静态完整闭包仍成立：T49十二项保留八项观察相同及四项诊断差异；T50只对正规化当前root callers；T53只对固定Read轨迹正常完成的UTF8分类，invalid pending增长/近二次成本与资源失败差异保留；T54 Some路径受同owned row和verify_published=false早退支配；T55四项canonical/shape/分类拒绝及T56两项纯索引/不可达分支保留诊断和工作量差异。没有新增生产caller绕过这些前提。

9结构项的当前pure源码、三处directory及四处sole角色接线、独立真值表、真实caller和OwnedTempDir Drop原生控制到当前未变。原根实际消失，外部root/sentinel原件及metadata保持的cleanup控制仍逐字存在。广新9和pure-only新9是独立机制执行，不给旧9native窗口授执行或等价。

动态分类不合并成“坏对象最终接受”：G04保留fcntl/exec/FIFO-open机制与first-open诊断；G05保留EXCHANGE不确定却报成功和A已chmod/B绑定失败的阶段边界；G06copy raw只证明API后私有data存在、prefix/late-refusal为源码推导，frame只证明特定Read来源优先序；G07限定API/rootepoch和准确32MiB；G09 cache先healthyA publishedfalse、UUID先warning缺失，删除只是推导；G13 env红仅early child finish/unreached，未打印status/stdout仍unknown。纯helper、模型sync、诊断检测均不扩为整链产品或原生断电证明。

## 原件与工程边界

185正式原红逐log/diff SHA及Build/Test phase核；11既有task档案2142成员全SHA回读一致。T49–T57二十份最终selection均完整190，所有到当前差异只在已审Rust源/测试；Cargo/lock/config/外部fixtures共同输入未变。当前补30原件另立SHA；不把旧不完整schema sample移植，也不复用T50未知五个旧exporter空失败或早期截断/fixtures编译失败。

当前同134/190源码工程资格沿用已独审T57实际门禁：run `fca0838a-afa4-436c-af23-0403e735b3b9`，948/948、0skip/LEAK、5compilefail，fmt/check/clippy/MSRV/default实际0；当前字节未漂移，本Reviewer未重复运行。T43最终文档/Task/check-task、追加全部报告/current30/schema原件的新archive逐SHA回读与提交由Owner收尾；本报告不预判未组装档案。

**无剩余必修oracle或mutation补验项。** 原stage1/SK02暂停事实、旧候选、其他平台、人类trial、完整错误JSON/资源/全安全等价均不被升级；M2另需自身最终Spec/产品闭环审查。本次仅允许按采用计划登记T43当前215处分资格完成。
