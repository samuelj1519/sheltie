# D-038 purge清除数据并保留根锁

[English](../../../en/explanation/decisions/D-038-purge-lock-lifecycle.md) | 简体中文

状态：`accepted`
日期：`2026-09-28`
关联 change：[C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## 背景

M1确认含0555冻结Workbook的正常purge会EACCES；原实现会先删`store.db`与`.lock`，再留下其他树。移除锁后才删除根会令等待者建新锁并写入，同时purge仍持有旧inode锁。

## 选择

`self uninstall --purge --yes`删除管理根下的用户数据、SQLite、workbooks、works、pending、tmp和bin，但保留空管理根和原`.lock`。purge在同一HomeLock内核将删对象及权限，按数据树→pending/tmp/bin→数据库次序删除；SQLite最后，`.lock`不删除。失败保留根/锁，准确报告部分清理，不能自动重建Store或Work；重复purge可继续清理。

排队者沿同一根锁继续：合法`workbook add`或`self install`可初始化空Store；等待中的旧Work写命令若Store/Work行已删除则返回`NOT_FOUND`且不建库、不恢复旧Work。卸载协议明确根/.lock保留。

## 否决方案

- 先删`.lock`再删根，存在重新创建锁的并发窗口。
- 把锁搬到根外，违反INV-3并新增第二管理位置。
- 复制数据再补偿删除，扩大写入面且无法可靠恢复部分删除的SQLite唯一状态。

## 后果

purge清掉全部用户工作数据与二进制，不再用“目录不存在”表示成功；根路径和锁对象保持稳定。

## 确认方式

对含冻结树/Store的临时Home运行正常清理、部分失败重试、install/add等待者与旧Work等待者；以同步点确认等待者已进入锁等待。
