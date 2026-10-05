# 当前载体前检

本次只读磁盘和文件系统元数据，不读取卷内目录内容，不挂载、卸载、格式化或写入介质。沙盒 diskutil 无法使用 DiskManagement，首失败保留；相同只读请求在宿主环境成功。当前盘点16个对象，未观察到外置 whole physical 候选，目标路径与授权尚未提供。

实际 argv、退出码、原 stdout/stderr 及字段见 [载体元数据](carrier-metadata-v02.json) 和 [首失败](diskutil-sandbox-failure.json)。这只证明本次可观测挂载状态，不证明所有设备永久缺失，也不证明任意文件系统支持非法 UTF-8 名称。

原 E01 APFS errno 和静态/可构造输入处分保持。若以后提供能创建该名称的真实载体及授权新目录，先核实际路径与对象，再执行物理目录验收。本次实际非法名称构造为 `not_run`；不把内存、虚拟文件系统或字节模拟称为物理目录。C006 当前工具副本在本机文件系统完成，原 E02 外置物理盘仍为 `not_run`。

初次摘要误选了不存在的 `Whole` / `Protocol` 键，当前摘要按实际 `WholeDisk` / `BusProtocol` 字段重新读取原 plist；旧摘要保持。所有本次观察到的外部 whole disk 均明确为 Virtual / Disk Image。该读取修正不新增介质行为证据。
