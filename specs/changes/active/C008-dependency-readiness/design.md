# C008 设计：一个真实目标的最小只读探针

状态：`active`；原probe机制资产尚未创建（本轮前检文档已准备）。本文字段是实验记录约定，不是 Sheltie 产品合同。

## 1. 最小实现

```text
experiment/
  protocol.md                 对象、人工基线、预算与停止规则
  host-rule.md                单宿主固定版本的搜索/选择/身份依据
  runbook.md                  T02 完整命令、参数、预期、退出与停止交接
  probe.toml                  固定 Workbook、Node、目标及允许读取范围
  probe.py                    一份完整只读探针
  test_probe.py               独立手写机制 oracle
  fixtures/                   仅临时合成环境，不映射真实宿主
  evidence/target/            真实采用前提和声明依据
  evidence/mechanism/         T01 临时正反与 no-write 原件
  evidence/real/<run-id>/     T02 正式真实对照原件
  report.md                   机制、真实收益和保留建议
```

T01 的复杂模型或有经验作者完整写 probe.py、必要 tests/fixtures 和 runbook，并完成临时机制验证；M1 独立核可执行性后，T02 简单模型或初级开发者只按固定协议做真实对照。T03 复杂模型分析，M2 独立核实际结果。不预建 adapter trait、跨宿主注册、状态库、通用版本范围解释器或新摘要标准。选定宿主的文件布局和规则在 T01 固定，探针只实现这一个版本范围；超出范围立即 unknown 或拒绝配置。

Python 使用 T01 确认的现有版本，优先标准库。需要新依赖或安装时停下缩小范围，不能自行扩成环境部署任务。探针不调用 Sheltie 写操作、不读取 Store，不使用 `self`。

## 2. 输入与读取顺序

probe.toml 只允许以下字段；缺必需字段、重复项、未知字段或非法目标返回配置错误，尚未读取宿主。

| 字段 | 类型和要求 |
| --- | --- |
| `workbook_dir` | 绝对路径；用户授权且已由当前真实引擎校验的固定副本 |
| `flow_file` | manifest 的 flows 之一；相对 Workbook 根的普通文件，无 `..`、符号链接或绝对路径 |
| `node` | 选定 Flow 的真实 Node ID |
| `kind` / `name` | manifest 的真实资源身份；Node 必须引用同一个 `kind:name` |
| `host` / `host_version` | T01 固定的单宿主与实际版本；只支持这一个组合 |
| `allowed_roots` | 非空绝对路径数组，只包括核定读取位置；不默认为用户 Home 或整个磁盘 |

实现步骤：先严格解析实验配置，再确认 Workbook 目录与文件归属、读取 manifest 与选定 Flow 中该目标的声明；随后按 host-rule.md 读取获准宿主资料，最后生成报告。只提取已有字段，不复刻引擎图解析或业务状态判断。声明元数据和引用无法唯一定位时返回配置错误，不任选同名条目。

Workbook 原始声明证据和已校验的身份由 T01 记录，必要时先由人对同一个已登记版本运行现有只读 verify。probe 自身不把文件 sha256 当 workbook-digest/v2，不实现新目录摘要。输入在观察前后变化则该观察保持 unknown 并说明漂移，不宣布存在一致快照。

host-rule.md 必须写明固定版本的一手规则来源、实际日期、搜索位置、优先级、scope、选择依据和各身份字段能否观察。不能确认全部位置时 scope 明确为部分；不能可靠观察选择时 selection unknown。允许读取范围小于宿主实际搜索范围时，完整缺失证明不可用。

默认只读文件元数据和必要公开内容。不执行配置内命令或 source URL；确需宿主官方无副作用信息入口时，T01 先核明确规则与用户授权，固定程序、参数及 stdout 使用范围，禁止 shell 拼接、联网、凭据读取和自动安装。不能证明入口无副作用则使用文件观察或 unknown。

## 3. 输出与退出

调用方式为 `python3 -B probe.py --config <absolute-probe.toml>`。有效观察向 stdout 输出一份 JSON，进程退出 0；matches/mismatch/unknown 均属有效观察，业务结论由 JSON 表达。参数或配置非法时退出 2，stderr 写定位信息，不输出有效观察。读取失败属于 unknown，并保留可定位原因；不静默略过。

| 字段 | 类型与含义 |
| --- | --- |
| `run_id` / `observed_at` | 非空观察 ID 和 UTC 时间 |
| `target` | workbook_dir、flow_file、node、kind、name 及原始可选 version/digest/source |
| `host` | name、固定 version、规则证据引用 |
| `scope` | 实际获准读取位置、完整/部分搜索说明及不可观察位置的说明 |
| `checks` | 数组，每项为 field、status、expected、observed、reason、evidence_refs |
| `status` | 按规格聚合的 matches、mismatch 或 unknown |

checks 的 field 为 presence、selection，及实际声明的 version、digest、source；status 仅三值。expected/observed 没有可靠值时为 null，reason 说明未知或不符。只输出比较所需公开身份，不把宿主文件全文、token、凭据或未知字段写入日志。

证据由调用者保存至获准实验目录；probe 不自己创建文件、缓存或锁。每次执行独立保存，不覆盖旧观察。runbook 给实际绝对 Python/脚本/config 路径、唯一证据目录、完整调用和保存步骤、每项预期、0/2 的解释及停止交接；T02 不猜参数或临时修 probe。外部记录者的写证据行为与 probe 的只读义务分开；它也不能写未经授权的宿主路径。

## 4. 身份解释与失败

version 只有在 T01 固定声明范围含义及观测版本来源后比较；否则输出 unknown，不引入猜测性 semver 兼容。digest 只有存在明确的完整内容闭包且所有内容可观察时才比较，独立 oracle 用已知字节或手工摘要，不调用被测 helper 算答案。读取中内容变化、脚本依赖未覆盖、不可读路径或声明无闭包说明均保持 unknown。

source 只有存在可靠来源元数据及明确比较规则时比较；一个安装目录或相同显示名不能证明来源。宿主没有来源元数据是可接受的 unknown，不自行下载声明源核对。

存在但选择不明、部分搜索未发现、权限拒绝、版本解释缺失、内容漂移及宿主版本不符都必须保留。不能因为 unknown 多就递归扫描更大范围或执行真实资源。

## 5. 机制与真实观察

机制 fixtures 与 test_probe.py 归 T01，测试发现清单与实际绿灯原件一并交 M1；不让 T02 改 fixture 或用新测试替代真实观察。机制 fixtures 分别覆盖：明确匹配、完整范围缺失、部分范围未发现、同名遮蔽、版本不明、摘要闭包不足、读取失败、输入漂移、未知配置字段、source 不执行和 no-write。用临时文件前后字节/目录清单核探针自有范围的只读行为，检查代码没有写入或执行通道；有限快照不能证明未观察的全系统安全。

真实观察来自已有资源和真实声明，不扰动宿主。人工核对与脚本核对在同样资料范围内完成；预先记录顺序，第二次核对受第一次结果影响时披露。实际任务正常推进可能写 Store，此行为单列，不混进 probe 调用区间的 no-write 检查。

报告分别给机制结论、逐项可观察范围和真实净成本。单次试用只支持个案；重复需求不足时不建设长期维护工具。
