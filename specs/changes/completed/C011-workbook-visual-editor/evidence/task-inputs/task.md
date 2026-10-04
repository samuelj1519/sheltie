# 真实任务：Workbook可视化作者工具

用户需要简单易用的本地网页Workbook创建/编辑界面，采用类似ComfyUI的节点画布。本轮只验收可用性，不作公平两臂或省时/费用收益结论。

在project的独立仓库中按active C011完整README/plan/design、规格GF-34和Workbook合同实现完整工具。显式边与input绑定分开；Node界面至少支持入口、id/title/executor、说明file/text、输入from/required/result、输出name/path/required/max_bytes/result及高级tier/gate/max_visits/max_retries/requires保真。Manifest/已有多Flow/资源和未编辑文件保留，未知字段改另一字段后仍拒绝。不从show有限投影重建原定义。

目录File.size/数量和路径先核再读取；无ZIP导入；保存是新完整ZIP下载，用户解压后重新打开。只检查和导出字节，不写源目录、正式Home或Work冻结副本。Browser端FileMap/Map保留字节，HTTP传输逐项[[path,base64],...]。16MiB/1024文件；HTTP流式24MiB。重复/ASCII大小写/NFC/目录别名/文件前缀/路径逃逸/严格base64拒绝；实际物化再核文件集合/原字节。

最终结构检查在本次自有0700新临时根写文件、用可信sheltie直接argv和新Home调用公开workbook add。单CLI30秒，各流1MiB；超时/输出超限/断开先TERM1秒→KILL，观察child退出/流关闭后才清自有根，否则失败保留准确残留。只127.0.0.1、严格Host/POST Origin/token。无任意路径、shell或任意binary调用。导入说明/标题/路径/错误显示为文本，不执行HTML。

Node工具完整可运行，npm test非零 meaningful cases，真实CLI合法/未知字段/高级声明/回环/同字节ZIP解压add/show/verify；真实HTTP拒绝矩阵/路径/数量/base64/超限和child结束归属；model编辑/布局分离/输入与边独立。测试期望来自手写合同/源字节，不能被测helper算自己的答案。界面不要静态假按钮，后续交实际浏览器和真人使用。

画布深灰蓝，左方法/Flow，中间可平移缩放节点/连线，右属性，高级项折叠，底部可定位错误。中文文案，清晰键盘焦点，窄屏折叠，减弱动画。允许额外只读样例按钮，只能使用固定code-change/spec-dev目录；无任意宿主路径读取。

用户个人活动每任务120分钟、整个任务等待60小时，费用unknown；达到预算/权限/输入不符或不可交付时停止并保原件。方法implement/review/deliver访问3/3/1、每到达失败重试1；自然review需修改走back，不伪造事件。实现结束保检查/候选/change.md/checks.md，日常review与真人接受分列。
