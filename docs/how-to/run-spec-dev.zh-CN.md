# 用 spec-dev 推进规格驱动的开发任务

[English](run-spec-dev.md) | 简体中文

适用于需要先形成规格与方案，再逐任务实现、独立验证、整体审查和交付的已授权仓库任务。只需一次有界修改时使用较小的 [code-change](run-code-change.zh-CN.md)。方法的精确说明与选边约定见[spec-dev README](../../workbooks/spec-dev-zh-CN/README.md)；引擎不解读报告来自动选边。

## 1. 准备方法、目标与仓库

按[构建指南](build-from-source.zh-CN.md)取得 `engine_binary`、`source_home` 与 `session_dir`。准备真实 `request_file` 和 `task_repo`：前者写需求、质量标准、范围与预算，后者是已授权业务仓库的绝对路径。先核仓库 HEAD、已有工作区改动和允许动作，保留他人修改。

在源码仓库根装入方法并核结构：

```bash
"$engine_binary" --home "$source_home" --json workbook add workbooks/spec-dev-zh-CN
"$engine_binary" --home "$source_home" --json workbook show spec-dev-zh-cn@0.2.2
"$engine_binary" --home "$source_home" --json workbook verify spec-dev-zh-cn@0.2.2
```

逐条核成功响应、default Flow 的 `request`／`project` 起始键和实际 requires。缺必需宿主资源时先停止；引擎不安装它们。本中文方法 ID 为 `spec-dev-zh-cn`、版本为 `0.2.2`；默认英文方法 `spec-dev` 为 `0.2.3`。方法版本与引擎版本是不同标识。

## 2. 创建 Work

调用前保存 UUID 和完整命令：

```bash
start_request_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
"$engine_binary" --home "$source_home" --json --request-id "$start_request_id"   work start --workbook spec-dev-zh-cn@0.2.2 --flow default --name '规格开发'   --input "request=@$request_file" --input "project=$task_repo"   > "$session_dir/spec-dev-start.json"
cat "$session_dir/spec-dev-start.json"
```

`request` 冻结文件正文，`project` 冻结仓库路径文本，不复制业务仓库。成功后保存实际 `data.work_id`。每次推进前查询当前 status；写重试保持同 ID、同意图，不能因响应未知另开 Work。

## 3. 按任务书组织实现与验证

每次从当前 `next` 选择节点 begin。将真实 `brief_path` 交执行者，要求完整读取绑定输入，并向 `outputs` 声明位置写报告；完成后协调者核实际报告、候选和必需文件，再以该次 Attempt ID submit。

| 阶段 | 协调操作 |
| --- | --- |
| `spec` → `plan` | 形成可验收规格、方案与任务；问题和约束缺失时交回，不在实现中猜测 |
| `plan-review` | 交人审核；写决定并保存被审 plan/tasks 的原字节副本，三份输出都提交 |
| `scaffold` | 按获批版本建立骨架、独立期望与测试；读取冻结 `approval_rules` |
| `implement` → `verify` | 一次一个任务；verify 使用新的独立执行者，自己运行门禁并核候选、范围与批准版本 |
| `fix` | 按实际验证或审查报告修复，再从当前合法边回相应验证者 |
| `review` | 使用未参与实现的执行者审整体原始基线到候选，不用聊天摘要代替差异与证据 |
| `deliver` → `retro` | 整理实际成果、未完成义务与反思；retro 读取冻结统计，提交后等待门槛 |

报告第一行是方法给协调者的约定，选边仍由协调者读取完整必要证据并与 `next` 核对。审查要求返工时提交已完成报告后沿回边；执行者实际无法交付才标 fail。命令模式见[接续 Work](resume-work.zh-CN.md)。

## 4. 卡住或重规划

有授权缺口、修复预算耗尽或方法要求升级时，只有当前 `next` 允许才进入 `escalate`，将实际原因和报告交人。人决定继续、跳过、改方案或止损；选择必须与已声明边、当前合法操作及项目授权一致，不能以“跳过”伪造任务完成。

重规划使用新任务书绑定的 previous_plan、previous_tasks 和已验证前缀。保留整体原始基线、真实验证行和原报告引用；验收条件变更列出需重验事项，并再次审核方案。旧候选的检查不能证明新候选通过，未验证的当前改动不追加到已验证前缀。

`implement` 的到达上限为 24，`verify` 为 32；其他额度以冻结定义为准。外部时间、费用和调用预算另行遵守。额度耗尽或只剩取消时停止，不提高 max_visits 或增加边继续。中断按当前 resume 接续，不自动撤销原执行资格。

## 5. 批准门槛并读取成果

retro 提交后应为 `blocked(gate)`。把它绑定的 delivery 与生成的 lessons 交用户核对；明确批准当前门槛后，才使用当前 `next` 的 work/node 调用：

```bash
"$engine_binary" --home "$source_home" --json gate approve "$work_id" --node retro
"$engine_binary" --home "$source_home" --json work result "$work_id"
```

核成功终态、`final=true` 和 `effects_pending=false`，选集应为 `delivery` 与 `lessons`。前者是 retro 开工时绑定的交付说明，后者是该次封存输出。门槛未批准时选集为空。

成果读取、代码质量、用户接受和对外动作分别判断。交付说明中的 push、PR、合并或发布命令不会由引擎执行，门槛通过也不替代那些动作的实际授权。取得可编辑副本见[导出成果](export-results.zh-CN.md)。
