# C008 外部预检设计

状态：`proposed`；全部脚本与实验路径为拟新增，脚本尚未实现。无新 Rust crate，无核心状态变化。

## 1. 真实入口与实验文件

| 现有路径 / 符号 | 用途 |
| --- | --- |
| `crates/sheltie-core/src/workbook/manifest.rs` / `HostRequire`、`parse_manifest` | 理解声明字段与严格校验；实验不修改解析器 |
| `crates/sheltie-core/src/flow/parse.rs`、`compile.rs` | 理解 Node requires 与 manifest 对应规则 |
| `crates/sheltie-core/src/work/render.rs` / `render_brief` | 对照引擎已提供的宿主资源说明 |
| `specs/contracts/workbook.md` / requires 表 | 实验字段事实来源 |
| `examples/`、`workbooks/spec-dev/` | 只用于盘点已有显式声明；无声明不能推断为必需 |

拟新增文件均位于采用后的 `specs/changes/active/C008-dependency-readiness/experiment/`：

| 文件 | 责任 |
| --- | --- |
| `probe.py` | 读取严格实验配置，单宿主只读观察，输出 JSON；无 Sheltie 写命令 |
| `test_probe.py` | 机制 oracle，正反例和 no-write 检查 |
| `probe.toml` | 冻结真实宿主、声明、路径范围与摘要口径；不放凭据 |
| `fixtures/` | 合成缺失、同名遮蔽、摘要不符、未知；与真实样本明确分开 |
| `README.md` | 固定启动命令、输入字段、观察范围、限制和实验说明 |

实际证据放 package 的 `evidence/`，记录输入闭包、命令、原始观察与脱敏日志。脚本只输出到 stdout；证据文件由执行者定向保存，不写 Store、宿主目录或工作区。

## 2. 小模块边界

```text
显式 requires + Node 引用 + 实验配置
    → 严格读取与对象选择
    → 固定版本的单宿主解析观察
    → 分别比较存在、选择和声明身份
    → JSON 观察，unknown 保持 unknown
    → 用户 / 协调者决定下一步
```

强模型在 T01 固定函数签名、配置结构与手写 JSON oracle。建议边界为「读取配置」「枚举获准位置」「解释已知宿主优先级」「比较可确认声明」「渲染观察」；不为每个宿主或资源类型建立插件接口。若缺官方解析规则，结果未知，不让简单模型猜。

Python 版本在 T01 固定；使用标准库能力可满足时不增加依赖。若解析当前 TOML 必须新工具，由强模型先核公开 API、锁定版本并说明成本；不临时增加 crate。shell 调用真实宿主必须是明确的只读信息命令，且不能读取凭据、启动模型任务或安装扩展。

## 3. 身份与摘要

kind/name 只是查找键。明确缺失要求完整宿主搜索位置都已核实并在授权观察范围内；否则只能说范围内未发现，存在/选择保持unknown。source、version、digest 是声明的约束。宿主实际选出的对象路径与元数据是观察事实；「查到了同名资源」不足以声称来源相同。

实验优先检查可直接验证的缺失与固定内容摘要。对于带脚本的 skill，摘要须涵盖作者确认的完整内容闭包；不能默默退化为单文件摘要。宿主选择规则、内容闭包或版本约束的语义缺失时，把对应项记 unknown，保留解释，不改 Workbook 制造更易通过的约束。

## 4. 对照和边界

人工对照与脚本使用同一份 requires 和同一宿主状态。人工核对先进行时会帮助第二次核对，因此记录顺序与熟悉度；可选不同真实任务交替顺序，但不为扩大样本制造业务需求。

合成 fixtures 只测机制：修改的是临时 fixture，不删真实 skill、不改用户搜索目录、不动宿主配置。真实任务按原环境运行；明确缺失时不强行执行失败来做对照，只记录当前人工核对如何处理，并标注这种样本不能精确量化「避免失败次数」。

检查后文件或宿主变化只能被再次检查发现。没有有效期、准入缓存、绑定版本或可靠性保证。若真实任务仅需要冻结的 resource 文件，停止本方向，继续使用 Workbook 提供的材料。
