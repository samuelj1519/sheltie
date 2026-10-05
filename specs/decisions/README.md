# Decision records

MVP 的 D-01–D-31、里程碑复核与首次真实运行保存在 [v0.1.0 legacy decision log](../releases/v0.1.0/decisions.md)。该文件已关闭，不再追加。

MVP 之后，一个重要决定使用一个 Markdown 文件。旧决定被取代时保留原文，状态改为 `superseded by D-nnn`，并与新决定互相链接。

| ID | 状态 | 决定 |
| --- | --- | --- |
| D-032 | accepted | [使用 change package 管理版本迭代](D-032-use-change-packages.md) |
| D-033 | accepted | [Store schema 2 明确拒旧](D-033-store-schema-2.md) |
| D-034 | accepted | [workbook-digest/v2 的带长度编码](D-034-workbook-digest-v2.md) |
| D-035 | accepted | [管理根写锁用 fs4](D-035-root-write-lock-fs4.md) |
| D-036 | accepted | [操作主体取真实 OS 进程身份](D-036-os-principal-from-uid.md) |
| D-037 | accepted | [受管文件使用目录句柄和安全 API](D-037-managed-file-handles.md) |
| D-038 | accepted | [purge清除数据并保留根锁](D-038-purge-lock-lifecycle.md) |
| D-039 | accepted | [只读SQLite允许共享内存控制文件](D-039-sqlite-read-control-files.md) |
| D-040 | accepted | [成果与接续采用单一 Store 格式](D-040-result-resume-format.md) |
| D-041 | accepted | [创建顺序号与行政替换](D-041-attempt-number-and-replacement.md) |
| D-042 | accepted | [最终成果原字节与外围副本](D-042-final-artifact-copy.md) |
| D-043 | accepted | [未发布候选的开发目标权威](D-043-development-target-authority.md) |
| D-044 | accepted | [当前产品定位为macOS／APFS本地场景](D-044-macos-apfs-product-scope.md) |

模板字段：状态、日期、关联 change、背景、选择、否决方案、后果、确认方式。
