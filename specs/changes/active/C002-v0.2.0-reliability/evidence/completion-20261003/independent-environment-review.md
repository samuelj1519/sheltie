增量最终结论：**T42 限定环境补验 PASS，可按计划记 done；T44 当前受审关键 Spec 合同 PASS。** 候选 `06e5a9108be9afdfb9b5b68fc112ad08a600af8d`。无剩余环境收尾必改。本轮未重跑843项，未编写probe或oracle。初次需补的原意见完整另存 `independent-environment-review-initial.md`，原audit另存initial，不覆写失败历史。

当前 MSRV 缺项已补：`cargo +1.85.0 check --all-targets --all-features --locked` 实际 exit0，独立可写target、RUSTC_WRAPPER为空；130项源码/Cargo输入SHA逐一与现行文件相符，source_unchanged=true，raw SHA与msrv-current.json一致。environment-results补齐dist/fresh-deny/MSRV/tool-version的argv/env/exit/rawSHA、Cargo graph闭包与复用理由；四条原文SHA和全部graph输入独立重算一致，固定06e5a91，清楚区分纯解码修复不会改变dist/deny消费的元数据闭包。旧msrv.txt仍仅作此前运行，未用于冒充本次。

结合初审核验，工具要求0.9.145、完整现行843/843、0skip/0LEAK、五项compile-fail、四门禁、MSRV1.85、fresh原policy公告库及实际dist plan的本机环境义务齐全。probe只说明旧执行器可造成捕获管道报告；历史每次Sheltie LEAK原因仍unknown，不扩大为永久零泄漏、产品微型测试、历史run PASS或全部C001–C008完成。

T44当前受审资格/历史/严格装入/副作用/恢复合同的独立结论不变。旧e3eea89暂停与没有当时最终批准保持历史事实，SK02的215项执行、平台、真实价值、usage仍按实际记录分别闭环。最终提交仍需各自Task门禁、准确进度和证据，不由本报告发起发布。

此前报告原文如下（其“需补”是已被上述增量解决的历史时点结论）：

当前独立结论：**T44 当前受审关键 Spec 合同 PASS；T42 需补当前 MSRV 及运行来源记录后收尾。** 审查候选 `06e5a9108be9afdfb9b5b68fc112ad08a600af8d`。审查者未编写环境 probe、源码或测试；本轮只读核验。准确证据 SHA 和核验项见 `independent-environment-audit.json`。

T42 尚有一项明确收尾缺口：`msrv.txt` 原件在 11:15，早于 T46 的 Command/data 修复；当前 `gate-results.json` 固定修前9ee0f011并于首次nextest失败中止，`t46-final-gates.json` 没有 MSRV argv/结果。现有输出只能证明一个 MSRV check曾完成，不能证明它核过06e5a91同输入闭包。请补当前候选的实际 MSRV check，或提供可核的原完整候选/argv/env/exit/输入元数据。dist、fresh deny和runner比较也应在T42最终记录中保留实际argv/env/退出码/输入或可解释复用边界；不要改写原失败清单。

其他环境内容核验通过：

- nextest官方asset SHA256 `52ecaedb4f5af9267ef7ed02bc937d2a15a94ff96cb663080e81311f798c9905`，dist官方asset SHA256 `aa343b2ff78ec2981f17a65140250c5ad6062c74072163f68c5c2686d94763a7`；尺寸与GitHubasset记录一致。执行二进制逐字节等于各压缩包成员，实际version分别0.9.145/0.32.0。临时使用，不声称改变宿主安装。
- T46固定门禁199输入逐项核SHA一致；三个LICENSE链接分别核Git link字节与解析目标SHA，不错误地以目标正文hash替代link身份。五个门禁输出SHA全相符。Nextest实际843条PASS、0skip、0LEAK、2slow；五项core compile-fail明确通过，其余crate的0doctest不是额外证明。完整四门禁+doctest exit0，提交记录正确，不复用修前842/841/1FAIL结果。
- dist0.32.0实际plan只含sheltie-cli0.3.0-rc.1，六资产仅一个aarch64包及通用source/checksum/installer；exporter publish=false/dist=false，plan无其资产。该运行证明真实plan生成，不证明构建、发布或其他平台。
- fresh deny实际四类advisories/bans/licenses/sources均ok；临时TOML与原政策精确比较，仅advisories.db-path改变，没有新增ignore。新DB HEAD `f8dee89e1b2f2f1eaf548312df7655fe5202a302`，commit时间2026-10-02T22:27:46+02:00，工作树干净，origin为RustSec/advisory-db。许可未使用项与重复winnow仍为原政策warning，不改第三方许可策略。
- runner-pipe-probe有4096个普通libtest函数，无进程spawn或子进程控制；个别函数仅sleep400ms。原源码SHA `5c0d5083bc6a5d10f423dbdc19e3a2c06f32fcfbe9e9994f188b263791d81a92`，Cargo无依赖。旧runner run03175ed4…4096/4096、0skip、3LEAK（process_0772/1819/2149）；要求runner runca6537e7…4096/4096、0skip、0LEAK。raw SHA与comparison相同，新运行复用相同target已有binary。比较说明当前已安装旧runner可产生此类报告，微型probe不计入产品测试数。

[官方 nextest0.9.145发布说明](https://github.com/nextest-rs/nextest/releases/tag/cargo-nextest-0.9.145)当前在线文字也明确Apple等Unix并发spawn可继承兄弟test的捕获管道，修复改由执行器创建/协调这些管道。该官方修复与普通test比较一致；不据此证明历史每一个Sheltie LEAK都来自同一原因。旧Sheltie raw、unknown原因与未追溯的PID/FD仍保留；当前零LEAK不是永久无泄漏保证。首次841PASS/1FAIL的共享target构建冲突另列，不能把定位原因改记原run PASS。

T44当前受审关键合同PASS：T46工作区的独立5/5严格解码与4/4原响应资格消费者、原R22–R24现行5/5CLI及8/8runtime审查报告仍保留各自候选/run。现提交06e5a91源码与独立输入和完整最终gate源码SHA一致，F-SPEC-01的必要修复已提交，original/pending_original、Add目标、历史状态/失败前缀、冻结requires、可信完整装入、副作用/恢复没有剩余受审必改。根工程措辞按protocol/storage分层，效果错误不撤销已核合法快照资格。该当前审查不补造旧e3eea89最终整体批准；215个映射不是执行，平台/真实价值/使用计量需要各自实际验证。
