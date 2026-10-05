# Workbook 作者工具参考

[English](workbook-editor.md) | 简体中文

作者工具是 `tools/workbook-editor` 中未发布的本地网页，用于编辑草稿并下载完整 ZIP。使用步骤见[画布编辑](../how-to/edit-workbook.zh-CN.md)，设计理由见[作者工具原理](../explanation/workbook-editor.zh-CN.md)。方法格式仍由[Workbook 合同](../../specs/contracts/workbook.zh-CN.md)定义。

## 启动与运行环境

需要 Node.js ≥ 22、工具的锁定依赖和可信 `sheltie` 绝对路径。

```text
node tools/workbook-editor/server.mjs --sheltie <absolute-binary> --port <port>
```

启动参数按上述顺序传入。端口为 0–65535 的整数；`0` 由 OS 分配。只监听 `127.0.0.1`，以打印的实际 URL 为准；不能将地址替换成 `localhost`。Ctrl+C 停止服务。

## 界面和保存行为

| 功能 | 行为与边界 |
| --- | --- |
| 新建方法、打开目录、固定样例 | 载入浏览器草稿；第一版不导入 ZIP，下载后解压再打开目录 |
| Node 与 Edge 编辑 | 修改显式定义；自环和重复端点的新增或编辑被拒绝，已有非法导入交 CLI 判断 |
| 清晰流程、完整连线、节点聚焦 | 只改变展示；清晰模式保留所有定义，隐藏边仍可从列表选择 |
| 输入／输出页签与搜索 | 只影响浏览；打开单项并修改才写声明 |
| 选择或输入资料 | 可多选前置输出、沿用前置输入原来源，或输入文件位置／URL |
| 固定外部引用 | 把位置存成 Workbook 内参考文本，不读取宿主文件或访问网址 |
| 检查结构 | 对当前完整字节集合调用公开 CLI；不是内容质量判定 |
| 保存新副本 ZIP | 对当前草稿重新检查后下载，不覆盖作者目录 |
| 拖动、平移、缩放、整理布局 | 只改视图；布局存 localStorage，草稿内容仅在页面内存 |

输入绑定与连线分别编辑。前置输入沿用的是原 `from` 声明，不表示取得前次 Attempt 的冻结文件。已有来源的勾选表示使用状态，移除须从已选列表明确操作，不隐式删同来源的多个别名。完整表单行为见[工具 README](../../tools/workbook-editor/README.zh-CN.md)。

## 限额与文件边界

| 对象 | 限额或规则 |
| --- | --- |
| 文件集合 | ≤ 16 MiB、≤ 1024 个文件；超限整组拒绝 |
| HTTP 请求体 | 流式累计 ≤ 24 MiB |
| 相对路径 | UTF-8 ≤ 4096 字节；拒绝绝对路径、盘符、反斜杠、NUL、冒号、空段、`.`、`..` |
| 路径集合 | 拒绝重复、ASCII 大小写／NFC 别名、目录拼写别名和文件／目录前缀冲突 |
| 单次 CLI | 30 秒，stdout／stderr 各 ≤ 1 MiB |
| CLI 终止 | 超时、输出超限或请求断开先 TERM，1 秒后仍未关闭则 KILL |

输入先核清单再读取，物化后核同一完整集合。只有直接 child 已退出且流已关闭，才清理本次自有临时根；无法确认时失败并报告 `residualRoot`，不宣称整棵进程树已停止。

## HTTP 接口

| 方法与路径 | 结果 |
| --- | --- |
| `GET /api/session` | 获取本次页面会话 token |
| `GET /api/sample/code-change` | 固定 code-change 文件集合 |
| `GET /api/sample/spec-dev` | 固定 spec-dev 文件集合 |
| `POST /api/check` | 当前集合的结构检查结果，内含引擎结果 |
| `POST /api/export` | 重新检查后返回 `application/zip` |

POST 使用 `Content-Type: application/json`、`X-Editor-Token`，Origin 必须精确等于本次打印的 HTTP 来源。正文为逐项数组 `[[relative_path, base64], ...]`，不接受宿主路径或任意命令。所有请求的 Host 必须等于实际 `127.0.0.1:port`。

结构成功返回 HTTP 200；检查失败为 422；Host／Origin／token 不符为 403；Content-Type 不符为 415；请求体超限为 413；未知路由为 404。其他输入拒绝保留返回诊断，不只根据 HTTP 状态猜原因。

## 保真与信任边界

未编辑文件保留原 bytes。编辑 TOML 允许重排格式，但保留完整原对象、未知字段和未操作字段；未知字段仍由引擎拒绝，不能经表单静默清洗。无法解析或无法表达的文档保留原 bytes，并拒绝对应编辑。

标题、说明和错误按文本显示，Markdown 不渲染 HTML；没有任意宿主路径、shell、二进制选择或 Work 操作接口。临时结构通过不证明用户接受、报告质量或实际运行成功。实现和测试入口见[实现定位](implementation.zh-CN.md)。
