# T43 两项 schema 当前补验独立审查

结论：**PASS，仅限这两项当前候选补验**。Reviewer 未编写 oracle、实现或准备变体；只读检查合同、源文件、215 原清单、fresh T57 inventory、真实 baseline/红日志与 diff，没有运行 Cargo 或修改源码。

本次原目录 `mutation-t43-schema-current` 实际 exit 0、28.897s，2 Caught、0 Missed/timeout/unviable。两变体均 Build Success、Test Failure(100)，不是编译错误或超时捕获。正常基线 run `26b1a907-9365-4d16-8fb8-e0fa11f4eda4` 实际执行两个指定消费者，2/2 通过，433 项由 filter 排除。

| 原/current 精确 ID | 原始失败事实 | 可接受分类 |
| --- | --- | --- |
| `schema.rs:82:5 replace normalize_sql -> String with "xyzzy".into()` | `open_rejects_same_whitespace_different_column_type` 第一次 `ReadOnly` 调用返回 Ok(Store)，`unwrap_err` 失败；合法控制通过 | 同版本错误 `works.state_json BLOB` 表结构被接受；不推定 mutant 的后续 ReadWrite 已执行 |
| `schema.rs:82:28 delete ! in normalize_sql` | 合法手写库被 `StoreSchemaMismatch(workbooks)` 拒绝；坏库也在 workbooks 提前拒绝，错误明细不是期望 works | 合法结构误拒绝，并有准确首错表诊断检测；不是坏结构最终接受 |

依据为当前 `storage.md` §1.1/§1.2：同版本逐表比较忽略空白，合法库可打开，坏结构在写库/WAL 前拒绝。oracle 手写 schema 4 的五张表和 `user_version=4`，只把 state_json TEXT 改为 BLOB；不调用生产 TABLES/create_script/normalize_sql 生成答案。拒绝例比较主库及非空既有 WAL 全 bytes，正常控制实际覆盖 ReadOnly/ReadWrite，并独立 SQL 核 `work_sequence.last=7`。名称 `handwritten_schema2_control...` 与旧 schema3 注释是历史命名，实际输入为 schema4。

两 ID 在原 215 清单中各唯一一次，fresh T57 inventory 的 name/function/span/genre/replacement 及完整 diff 与本次输出逐字对应。原 e54dcd41 schema 文件 SHA 为 `2d6e0e445849adb3aa6f7eb4b567b8331b32389b3859222d0355ab16ae7e9e74`，当前为 `2ca35f4b87adc5c9bb3fdec012b7da7680d3491f9beb88c150934d6c26698b5d`：仅头注释和 SCHEMA_VERSION 2→4 改变；TABLES 起到文件末尾完全相同，包括 normalize_sql。当前真实 schema4 补验另立，旧 stage1 Missed 保留，不从旧样本移植 PASS。

本次 selection 的 190 输入逐 SHA 匹配当前 HEAD `4487ac1eb20672b8136aba186e8392fafa38ebca` 的执行源码/fixtures/config。selection.base=`4bf9f872...` 是提交前运行时 HEAD；其 190 记录已包含 T57 新测试，不能称整个输入是历史 4bf9 的干净提交。runner 显式 empty RUSTC_WRAPPER、relative target、jobs2 与 nextest PATH，实际 phase argv 核 all-features/locked、两消费者 filter/threads4；未声称记录全部继承环境。

无剩余必修 oracle 项。mapping 原件中的 `running_current_candidate...` 是派发前标签，应由 Owner 在最终汇总另列真实终态；`schema production unchanged` 应限定相对 T57 冻结窗口，而不是相对原 e54 的 schema 版本。SHA、原始阶段与限定分类见同名 JSON。本报告不批准剩余 213、T43 总账、M2 或全产品/全平台原生证明。
