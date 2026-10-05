# D-039 只读SQLite的控制文件边界

状态：`accepted`
日期：`2026-09-28`
关联 change：[C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## 背景

M1发现WAL下SQLite只读连接在`-shm`缺失时可于已存在的可写目录创建wal-index。不能以错误副作用或`immutable=1`逃避；更不能静默checkpoint/migrate，或让schema 1拒绝改动旧数据。

## 选择

只读SQLite连接可在已存在、Home身份已核验的管理根中创建/维护`store.db-shm`共享内存控制文件；已有Store处于WAL模式且WAL文件缺失时，可由SQLite只读连接创建零字节`store.db-wal`控制载体。不能给空WAL写入header/frame或改已有WAL字节。不能创建Home、`.lock`、Store、WAL数据记录或业务文件。schema 1拒绝前`store.db`和已有WAL逐字节不变，不checkpoint旧库。不能仅以main文件长度拒绝：若短main对应的WAL有效且包含完整已提交数据库快照，SQLite可从WAL提供逻辑数据库视图。若WAL header损坏但main自身有效，bundled SQLite可忽略WAL并从main返回旧快照；运行时接受SQLite给出的可查询视图，不另行猜测“被忽略”状态或解析WAL格式，且main/WAL字节仍必须不变。查询失败时准确拒绝，不能修复或删除旧文件。

schema识别连接在第一次查询前使用安全rusqlite `Connection::set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)`。拒绝叶/sidecar链接与特殊对象后才打开SQLite；不自动降级RW。`SQLITE_OPEN_NOFOLLOW`强度不等同rustix目录fd锚定；不得使用`immutable=1`、自动RW fallback或自写SQLite VFS。

## 后果

只读不改Work/Workbook/audit/request/main数据库/WAL业务状态，不取引擎HomeLock，不创建Home或`.lock`。允许的`-shm`变化和缺失WAL的零字节创建单独记录，不作为第二状态源；schema和数据格式不变。

## 确认方式

以Cargo.lock对应的bundled SQLite版本测试WAL已存在、缺shm、活动写者、schema 1前后main/WAL字节、sidecar链接、连接close后的字节。macOS探针已运行；Linux运行依用户明确指示豁免，记`not_run`，不写成Linux/跨平台PASS。API文档不代替已执行平台的实测。新增确认项：原WAL缺失时，核合法只读预检创建的WAL长度为零、主库字节不变。purge在删除Store后复扫时，只接受合法单链接SHM或零字节WAL，保留非空晚到WAL及其他异常对象并报部分清理。历史运行与审查原文从[历史查阅指南](../../how-to/maintain-docs.md#查阅历史原件)的固定快照读取。
