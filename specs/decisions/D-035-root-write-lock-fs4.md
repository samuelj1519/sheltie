# D-035 管理根写锁用 fs4

状态：`accepted`
日期：2026-09-27
关联 change：[C002](../changes/completed/C002-v0.2.0-reliability/README.md)

## 背景

`work start` / `workbook add` 的发布顺序是「事务提交 → rename pending 到最终目录」。两个本地写进程交错时，一个进程的清理可能删掉另一个进程已提交未发布的原件；若锁外因素替换管理根，等待者可能沿陈旧inode继续写。需要一个管理根级的文件生命周期串行化，且只读操作不能为它建任何文件（GF-30）。C002采用purge保留根与原锁，见D-038。

## 选择

`<管理根>/.lock` 上的排他文件锁，用 `fs4`（MIT OR Apache-2.0）的 `fs_std::FileExt`：`lock_exclusive` 阻塞等待，进程退出由 OS 释放；无需轮询与超时框架。锁内依次做重核 schema、查重放、恢复未完成效果、准备、事务、发布与完成标记。`self uninstall --purge`持锁删除用户数据和二进制，保留空管理根与同一个`.lock`，不释放/删除锁文件后再建新锁。

只读操作不取HomeLock；它们依D-039执行只读SQLite识别与受管对象核验。写操作取得锁后复核`.lock`与根的对象身份。正常purge不会移除根或锁，排队者沿同一锁对象继续：合法add/install可在锁内初始化；旧Work命令在Store/Work行已被purge删除后返回NOT_FOUND，不得建库重建旧Work。若锁外因素意外删除或替换根/锁，等待者不得沿陈旧身份写入，应释放并按受管根重试。API已核对（engineering §1.2）：std `File` 加锁、`try_lock_exclusive` 可探测、跨平台（unix fcntl/flock、Windows LockFileEx），满足上述每一步。D-038记录purge范围与正常同锁等待语义。

锁只串行化本地协作进程，不声称约束同用户手工改文件；SQLite 的 revision CAS 保留为事务边界校验，不做自动业务重试。

## 否决方案

- 只靠 SQLite `BEGIN IMMEDIATE`：它串行化事务，不覆盖事务后的 rename/删除窗口，也管不到 `bin/`、`pending/` 这些库外路径。
- `fd-lock`：API 返回借用句柄的 guard，与「锁文件长期持有 + inode 复核」的用法绕；维护活跃度相当。
- 自制锁（存在性标志文件）：崩溃后残留死锁，OS 不自动释放，还要手工恢复。

## 后果

写操作之间在单机上完全串行；引擎只读操作不创建`.lock`。WAL读取可能维护SQLite共享内存控制文件，见D-039。purge成功后保留空管理根及原锁。新增一个传递依赖很小的直接依赖（`fs4`）。

## 确认方式

并发测试先批量启动再用同步点制造交错再 join。purge等待者测试断言purge保留并复用同一个根/.lock、旧Work不复活；另用独立外部替换测试验证陈旧锁身份被识破且不会误写新根。
