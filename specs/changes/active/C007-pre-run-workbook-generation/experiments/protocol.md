# C007 准备协议与正式准入

当前状态：技术资产准备；正式实验未冻结、六次运行not_run。原要求见[validation](../validation.md)，本轮范围见[adoption](../adoption.md)。方法作者是本次agent，不能把自测算成未参与编写的首次读者。

## 固定技术范围

使用C006限定验收后的0.3.0-rc.1/schema4/cli-resultv4源码。实际source SHA、Cargo JSON binary/SHA、专用Home及每条CLI原件存[preparation](preparation/README.md)；不猜target路径。当前组合含C004–C006，不能把未来观察分别归因某增量。试验路径只按当前CLI推进，不自动replace或export。

一份[confirmed-input](confirmed-input/README.md)与[共同方法](shared-method.md)供两组。Native组读取相同三instruction文件、task/project、阶段、返工与上限，并用宿主已有的认真配置保存状态；Sheltie组使用同一文字的Workbook。默认deliver无gate，外部用户接受与独立最终质量评审分别记录。

技术CLI单条30秒、资产准备20分钟墙钟反馈预算；这些是工程反馈上限，不是实验人工活动/费用预算。超时保留原件、停止相应命令；零测试或编译失败不是行为成功。无重复正式记录摩擦，不制作collect_run.py或第二套parser/Store/runner。

## 正式运行前必须补全

| 输入 | 当前值 | 补全要求 |
| --- | --- | --- |
| 三项近期尚未解决任务 | null | 小bug、小功能、自然返工情境；真实目标/验收/初始candidate和两个独立副本 |
| 同一连续使用者及首次读者 | null | 稳定匿名actor ID，每组各三次；至少一人未写方法，说明所有助手 |
| actor × arm真实使用历史 | null | 包含练习/失败/停止的实际使用，不从slot编号推first_use |
| 模型/宿主/环境/权限 | null | 两组同版本/配置/能力；Root方法作者不能补成真实参与者 |
| 真正关闭重开/自然返工 | null | 冻结阶段与实会话材料；同会话继续或人为缺陷不计自然样本 |
| 活动/墙钟/费用/准备回收预算 | null | 实际任务提供者在看到结果前固定；达到上限停止、失败成本保留 |
| 逐任务质量oracle/接受条件 | null | 预先声明正反行为、现有检查、完整patch和重大缺陷标准 |
| 有意义改善阈值 | null | 质量先达标，再看含准备维护成本的累计改善；不能事后挑阈值 |
| 配对顺序/配置/初始副本身份 | null | 在sample/actor确定后冻结顺序，披露跨组熟悉和顺序偏差 |

以上缺任一项不开始正式计时或run，不分配正式run_id。只读技术材料审查不是原M1正式试用准入。补齐后冻结protocol/method/oracle/inputs/binary标识，独立审准入再按first-use运行；不覆盖此前准备原件。

## 记录和计数

格式见[record-template](preparation/record-template.md)。每次真正方法求解开始前据实际prior_uses计算本组序号与exposure；同一run内节点重试/返工/续接不增加方法使用次数，not_run不计。准备作者/助手已有接触必须披露，不能暗记为0。

每人活动区间不重叠重复计时，phase/cost_bucket唯一归类；tool/model等待与人工活动分开。P、S、M、R、E及两组累计/实验总投入按[完整成本公式](../validation.md#3-分析口径)复算：公共P在每组比较同口径包含、实验实际支出只计一次。未知token/费用/分钟null，不用work stats反推。

最终盲审给中性编号candidate/patch/task标准/必要检查；原件留在run，组别映射仅分析者持有。移除提示标签不能改代码/检查内容；盲审无法遮蔽则披露。用户接受和质量独审均在方法之外单独记录，不由Work终态/gate/报告自称PASS推断。
