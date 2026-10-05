# Workbook 作者工具设计

作者工具编辑 Workbook 草稿，以可信 `sheltie` 的公开 CLI 判断结构，再交付完整 ZIP 新副本。操作方法见[工具 README](../../tools/workbook-editor/README.md)；精确方法格式见[Workbook 合同](../contracts/workbook.md)。工具不维护 Work 状态，不直接修改作者目录、正式 Home 或 Work 冻结副本。

## 数据来源与操作接口

浏览器使用文件 Map 保存原字节。UI 解析 manifest／Flow 只用于表单和画布，完整原对象及未编辑文件仍是数据来源；不能从 `workbook show` 的有限投影重建定义。拖动位置保存在 localStorage，不进入方法语义。

服务端仅提供 check 和 export 两种草稿操作。传输使用逐项数组 `[[relative_path, base64], ...]`，先核重复与路径别名，不通过 JSON 对象或 ZIP 覆盖同名条目。export 重新核它收到的确切文件集合，不消费 UI 上一次绿色标记。

目录选择先按 File 清单核大小、数量和全部相对路径，再读取内容，读取后另核实际集合和大小。第一版不导入 ZIP；下载后先解压，再选择目录，避免新增解压输入路径。

## 文件限额与物化

| 对象 | 限额与规则 |
| --- | --- |
| 文件集合 | 至多 16 MiB、1024 个文件；不支持或超限时整组拒绝 |
| HTTP 请求体 | 流式累计至多 24 MiB，不先拼接全部字符串 |
| 相对路径 | UTF-8 至多 4096 字节；拒绝绝对路径、盘符、反斜杠、NUL、冒号、空段、`.` 和 `..` |
| 路径关系 | 拒绝重复、ASCII 大小写／NFC 别名、目录拼写别名与文件／目录前缀冲突 |
| 编码 | 逐项核类型、数量、编码长度与严格 base64，再核实际累计字节 |

全部前检通过后，在工具自有、权限 0700 的新临时目录独占创建文件。物化后的文件集合与字节必须和输入逐项一致；不能规范化后丢条目，也不能让文件系统别名掩盖差异。

## 可信 CLI 与失败清理

启动时显式指定可信 `sheltie`。服务端以直接 argv、新的临时 Home 调用 `workbook add`；引擎结果才授予结构判定。非零退出、不完整 JSON、超时和输出超限保持失败。

单次 CLI 限 30 秒，stdout／stderr 各限 1 MiB。超时、输出超限或请求断开时先 TERM，1 秒后仍未关闭则 KILL。只有观察到直接 child 退出与流关闭后才清理本次自有根；无法确认退出时保留根并报告准确残留原因，不删除父目录或声称整棵进程树已停止。

## 编辑保真

未编辑的 Flow、TOML 和二进制 resource 保持原字节。已编辑 TOML 可以重排格式，但保留完整原始对象及未知字段，只更新实际编辑字段，不用表单字段重建整个对象。

未知字段在修改其他合法字段后仍须被真实引擎拒绝。无法解析或表单不能表示的文档保留原字节并拒绝相应编辑，不替换模板或静默省略。删除引用造成的非法草稿由检查定位，工具不猜新入口或新来源。Markdown 说明保留原文件关联。

方法格式保持 workbook/v1 与 flow/v1。表单基础检查改善反馈，不替代引擎校验。TOML／ZIP 使用公开库，实际版本以[package-lock.json](../../tools/workbook-editor/package-lock.json)为准；参考接口为 [smol-toml](https://github.com/squirrelchat/smol-toml) 与 [fflate ZIP](https://github.com/101arrowz/fflate/blob/master/docs/functions/zipSync.md)。

## 本地服务的信任边界

只监听 `127.0.0.1`。Host 必须为实际 `127.0.0.1:port`；POST 的 Origin 必须精确等于该 HTTP 来源，并携带本次启动的随机 token。缺失、null、其他来源或错误 token 均拒绝。

静态资源路由使用固定白名单及依赖目录，拒绝路径逃逸。服务不提供任意宿主路径读取、shell 或任意二进制调用。标题、说明、ID、路径与错误作为文本显示，不把导入内容插入为可执行 HTML／脚本。相应拒绝与转义需真实测试覆盖。

这些措施不提供同一 OS 账户下的真人认证或整棵进程树隔离，边界见[当前限制](../limitations.md)。

## 界面与验证入口

图占主要区域，左侧为方法／Flow 导航，右侧为属性，底部为可展开检查错误。节点选择高亮，边使用真实 kind；键盘焦点清晰，表单及节点列表均可选择。窄屏面板折叠，尊重减少动画设置。空画布引导增加节点，错误定位文件、字段与下一步。

代码入口是 [server.mjs](../../tools/workbook-editor/server.mjs)、[文件边界](../../tools/workbook-editor/lib/files.mjs)、[CLI 边界](../../tools/workbook-editor/lib/engine.mjs) 与[浏览器模型](../../tools/workbook-editor/public/model.mjs)。[测试](../../tools/workbook-editor/test/)覆盖独立模型期望、路径拒绝、真实 CLI、完整 ZIP 字节、HTTP 和直接 child 失败路径。具体执行命令见工具 README；依赖、文件模型或服务边界变化时沿这些真实消费者验证。
