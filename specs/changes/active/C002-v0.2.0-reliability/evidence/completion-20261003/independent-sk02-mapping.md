独立 SK02 映射审查：**215 项均对应仍存在的当前行为；映射无剩余歧义，0 项退役。** 本结论只是源码对应核验，不是执行、等价或验收 PASS。

旧候选 `e3eea899877165f8573befee3774555598ec92bd`；现行 HEAD `8167165a5a707bbb51a29fd1aa1047f387c9e9d3` 与实际工作区源码。原输入 current_base `8167165` 保留，准确源码/输入/原diff/当前diff SHA256 见同名 JSON。只读审查者未修改仓库、测试或原件，未编写新 oracle。

200 项原唯一候选均核原 genre/replacement、旧diff SHA 与旧/当前完整原hunk，15 项按上下文独立消歧。旧hunk逐项来自 git show e3eea89；当前hunk逐项存在于实际源码。215旧ID无漏无重，215当前ID无重复。源码位置变化或包装删除不代表机制退役。

| 旧位置/行为 | 准确当前 ID | 独立理由 |
| --- | --- | --- |
| `crates/sheltie-runtime/src/effects.rs:1153:13: replace || with && in delete_dir` | `crates/sheltie-runtime/src/effects.rs:1159:13: replace || with && in delete_dir` | 已存在删除完成marker：sync_deleted_marker后复核final/pending不存在；不是删树后或新写marker后的检查。 |
| `crates/sheltie-runtime/src/effects.rs:1203:9: replace || with && in delete_dir` | `crates/sheltie-runtime/src/effects.rs:1209:9: replace || with && in delete_dir` | remove_managed_tree和delete_after_tree_removed_before_marker后，在write_deleted_marker前复核final/pending不存在。 |
| `crates/sheltie-runtime/src/effects.rs:1212:9: replace || with && in delete_dir` | `crates/sheltie-runtime/src/effects.rs:1218:9: replace || with && in delete_dir` | write_deleted_marker和delete_marker_synced_before_mark后复核final/pending不存在；marker已写，故窗口不同。 |
| `crates/sheltie-runtime/src/fsx.rs:1369:33: replace match guard (stat.st_dev as u64, stat.st_ino) == locked_ident with true in ManagedFs::purge_contents` | `crates/sheltie-runtime/src/fsx.rs:1457:33: replace match guard (stat.st_dev as u64, stat.st_ino) == locked_ident with true in ManagedFs::purge_contents` | purge_contents最后remaining复扫的filter_map保留锁inode；不是初始names或三次extras扫描的相同守卫。 |
| `crates/sheltie-runtime/src/fsx.rs:3085:45: replace || with && in set_dir_tree_mode` | `crates/sheltie-runtime/src/fsx.rs:3271:45: replace || with && in set_dir_tree_mode` | 目录分支：OFlags::DIRECTORY打开child，fstat目录身份，后接递归set_dir_tree_mode及directory_mode。 |
| `crates/sheltie-runtime/src/fsx.rs:3113:45: replace || with && in set_dir_tree_mode` | `crates/sheltie-runtime/src/fsx.rs:3299:45: replace || with && in set_dir_tree_mode` | 普通文件分支：NOFOLLOW/NONBLOCK打开fd，check_regular_stat后核dev/ino，后接file_mode封存。 |
| `crates/sheltie-runtime/src/service.rs:601:51: replace match guard !location.pending_publish with true in WorkService::load_row` | `crates/sheltie-runtime/src/service.rs:843:51: replace match guard !location.pending_publish with true in WorkService::load_row_at` | load_row的冻结Workbook装入块迁到load_row_at，NotFound且已发布时映STORE_CORRUPT的相同守卫；load_row仍有真实caller并转发该块。 |
| `crates/sheltie-runtime/src/service.rs:601:51: replace match guard !location.pending_publish with false in WorkService::load_row` | `crates/sheltie-runtime/src/service.rs:843:51: replace match guard !location.pending_publish with false in WorkService::load_row_at` | 同一load_row→load_row_at块的false守卫替换，故对应同一当前位置的false变体。 |
| `crates/sheltie-runtime/src/service.rs:601:51: delete ! in WorkService::load_row` | `crates/sheltie-runtime/src/service.rs:843:51: delete ! in WorkService::load_row_at` | 同一load_row→load_row_at块删除pending_publish否定，故对应同一当前位置的UnOp变体。 |
| `crates/sheltie-runtime/src/service.rs:1191:17: replace && with || in validate_command_owner` | `crates/sheltie-runtime/src/service.rs:1533:17: replace && with || in validate_command_owner_data` | 原validate_command_owner的Start业务绑定迁到validate_command_owner_data：start_matches与冻结graph.requires共同约束；data改为借用参数不是退役。 |
| `crates/sheltie-runtime/src/service.rs:1241:17: replace && with || in validate_command_owner` | `crates/sheltie-runtime/src/service.rs:1663:17: replace && with || in validate_command_owner_data` | 原Submitted分支迁到validate_command_owner_data：最后一个&&核提交outputs等于该Succeeded Attempt的record.outputs。 |
| `crates/sheltie-runtime/src/service.rs:1240:17: replace && with || in validate_command_owner` | `crates/sheltie-runtime/src/service.rs:1662:17: replace && with || in validate_command_owner_data` | 原Submitted分支迁到validate_command_owner_data：第一个&&连接reply_attempt身份与Succeeded资格；与outputs的&&不同。 |
| `crates/sheltie-runtime/src/service.rs:1251:17: replace && with || in validate_command_owner` | `crates/sheltie-runtime/src/service.rs:1673:17: replace && with || in validate_command_owner_data` | 原Failed分支迁到validate_command_owner_data：reply_attempt身份与state中Failed资格同时成立。 |
| `crates/sheltie-runtime/src/workbook_repo.rs:692:29: replace || with && in WorkbookRepo::publication_for_row` | `crates/sheltie-runtime/src/workbook_repo.rs:692:29: replace || with && in WorkbookRepo::publication_for_row` | publication_for_row的Add审计分支，id/version匹配或request_id属于publisher_ids；下一个相同语句属于Remove。 |
| `crates/sheltie-runtime/src/workbook_repo.rs:698:25: replace || with && in WorkbookRepo::publication_for_row` | `crates/sheltie-runtime/src/workbook_repo.rs:698:25: replace || with && in WorkbookRepo::publication_for_row` | publication_for_row的Remove审计分支，target匹配或request_id属于publisher_ids；上一个相同语句属于Add。 |

12 项完整 hunk 正文一致；其余 3 项存在已核邻接差异：set_dir_tree_mode 的 Mode::from_raw_mode 增加 `as _`，Start绑定的 data 从局部值改为借用参数。对应条件、分支和操作位置分别核实，不将文本相似度当动态等价。

仍需为映射后的每个实际缺失义务执行对应 oracle 并处置，保留旧 MissedMutant/暂停/timeout 原文。主 agent 的 schema 三个当前 caught 中仅两项对应历史215，第三个是额外样本；此报告不把这些样本扩展到所有ID，也不改原执行状态。
