# C006 构建、复制与现场核查

## 1. 固定输入与阶段入口

测试名见 [plan](../plan.md) 的T01/T02“测试”清单。当前开发目标是work-result/v1及协议原字节模式，外围工具publish=false/dist=false；不安装、不上传release。T01交付Source、Target及必要耦合library编排、Report与fault边界；T02只开放公开参数/入口，不改变高风险原语或oracle。

[commands.sh](../verification/commands.sh) 以Cargo JSON的真实executable取得两份binary路径并记录SHA；不猜target目录。明确使用隔离target和空RUSTC_WRAPPER。消费者收到SHELTIE_TEST_ENGINE_BINARY/SHELTIE_TEST_EXPORT_BINARY；helper只检查路径存在，SHA由冻结命令记录，不能说helper验过代码身份。

```bash
bash specs/changes/completed/C006-result-delivery/verification/commands.sh primitives
bash specs/changes/completed/C006-result-delivery/verification/commands.sh future-red
bash specs/changes/completed/C006-result-delivery/verification/commands.sh feature
bash specs/changes/completed/C006-result-delivery/verification/commands.sh regression
bash specs/changes/completed/C006-result-delivery/verification/commands.sh dist-plan
```

原配置要求nextest0.9.145，本机0.9.140。原命令exit92且测试not_run；已授权延期后可显式SHELTIE_NEXTEST_VERSION_OVERRIDE=1执行补充验证，版本与原run分记。原语/feature各5分钟，完整回归/工程门禁12分钟，dist-plan3分钟；超时保存原输出，停止相应run交作者，不当PASS。原本实验预算与真实用途仍需明确提供。

T01原语至少一项实际green；未来公开caller在T01显式运行得到实际red（未知raw/copy参数），不是编译失败或零测试。T02使用M1骨架或最新独立测试修订完整SHA，所有期望、helpers、fixtures/snapshots冻结，只删归属ignore。语义缺口交明确修复任务、独立复核、新完整基准，不在实现中修改答案。

## 2. 已冻结接口

`WorkService::write_result_artifact(work,key,revision,writer)` 从一个可信结果读取核资格及引用，用受限同FD流式证明；CLI直接stdout bytes，错误stderr，不生成普通Outcome的JSON/newline，也不恢复/清理。

Source::new(binary,home,work)、result、receive(revision,artifact,writer)负责严格CLI payload、1MiB/32MiB/256MiB、独立摘要/大小/exit及直接child回收。使用单argv `--artifact=<literal key>`，保持空、前导hyphen、Unicode及元字符，不经shell。

Destination::open(home,parent)、stage(work)负责实际对象授权与不重叠。Staging.create_artifact按零起index生成0001起目录；finish_artifact保留FD及CopiedFile。publish完整核集合/原字节读回/manifest/文件目录sync/NOREPLACE/最终对象及父sync。stage路径只报告可核所属对象；rename后确认失败为unconfirmed，不回滚删除。

`export::copy(binary,home,work,parent) -> Report` 已组织以上步骤与四种稳定状态。main接参数并调用copy、Report.write及exit_code即可；Target/Source/error/model/fault政策不由T02重写。

## 3. 正式接线后的一次副本

先从Cargo JSON固定当前两binary及其SHA、same Home、完整WorkId、已存在的真实无链接目标父目录。Home与父目录不能相互包含，管理根与目标仅共享更上层祖先不构成重叠。没有最终非空选择就停止，查询不变为推进或批准。

```bash
"$engine_binary" --home "$management_root" --json work result "$work_id"
"$export_binary" --sheltie "$engine_binary" --home "$management_root" --work "$work_id" --to "$destination_parent" --json
```

这些值在真实用途确认后填入唯一记录；不是示例值冒充真实用户。complete/exit0才使用target_path；校验manifest的result、files及实际字节，副本随后可以编辑，原件与Store不改变。手工复制与工具复制的找结果/接收/核对/失败处理成本分别记录，不把C004找指针收益混算为C006增量。

## 4. 失败与残留

rejected/2表示参数或确定性资格不合格；failed_before_publish/1表示source/target/完整性/IO失败。可能有工具自有私有暂存，不能当完成目录。publication_unconfirmed/3表示rename已经发生，但最后确认不足；核真实对象与manifest/bytes，不删目录作补偿。

kill可能没有响应，按实机现场核查；不据名字猜归属，不接管旧暂存或竞争者，不自动删除不明对象。重跑始终创建新副本，不覆盖旧副本或恢复UUID。记录所有原stdout/stderr/exit、实际目录/权限/哨兵、source/target身份及Store业务前后；SQLite控制文件维护例外单列，不虚称零物理写。

## 5. 真实用途与交接

当前只具备开发实施授权，没有提供实际副本用途、任务使用者、人工对照成本及接受原件，真实价值not_run。机制可以在临时目录执行，不能以fixture称净收益。补全真实用途和质量/成本标准后冻结路径/候选、执行同一手册；未知usage与成本为null，不当零。T03无新增Rust场景，不以零测试task.sh验收。
