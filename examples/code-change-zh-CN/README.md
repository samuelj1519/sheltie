# 使用最小代码变更方法

简体中文 | [English](../code-change/README.md)

本方法适用于已经获准修改本地仓库、有明确验收标准和检查命令的代码任务。它提供固定阶段和冻结报告引用；实际代码仍在项目仓库，报告中的 commit、patch 和工具结果由执行者与审阅者核对。

当前源码候选为 0.3.0-rc.1，尚未发布；使用当前源码构建的 sheltie 与新的显式管理根。旧管理根和旧二进制保留，不修改或清空。方法默认没有 gate，不授予部署、合并或发布权限。

## 准备输入

准备 task.md 和 project.md。task 写目标、约束、可接受成果和验收标准；project 写仓库绝对位置、允许范围、当前工作区前提和检查命令。两份材料会按字节冻结；目标改变时另开 Work，运行中不改图或输入。

方法是 implement → review → deliver；review 可经 back 回到 implement。实现和审查各最多到达 3 次、每次到达允许 1 次执行失败重试；deliver 最多到达 1 次。内容返工由显式边处理，执行失败才调用 fail。

## 装入与开工

在仓库根，使用 cargo run 让 Cargo 定位真实二进制，不猜 target 路径。example_home 是本次新的绝对管理根；全部调用保持同一值。

```bash
example_home=/private/tmp/sheltie-code-change-example
cargo run -p sheltie-cli -- --home "$example_home" --json workbook add examples/code-change-zh-CN
cargo run -p sheltie-cli -- --home "$example_home" --json workbook show code-change-zh-cn
cargo run -p sheltie-cli -- --home "$example_home" --json work start --workbook code-change-zh-cn --flow default --input task=@/absolute/task.md --input project=@/absolute/project.md
```

复用已装的同一 Workbook 版本时不用重复 add；只准备本次新 task/project 并新开 Work。show 的 start_inputs 应为 task、project。start 返回完整 work_id；下文的 <work> 替换为该值。给写操作显式 request-id 可以安全重试；只读命令不接受该参数。

## 领取、提交与返工

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json attempt begin <work> --node implement
```

把 brief_path 交给执行者。执行者读取冻结输入，在 task/project 的授权范围内完成代码和检查，按 outputs.change 指定位置写 change.md。报告写清实际候选、检查原文、退出状态、未执行项和剩余问题。

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json attempt submit <work> --attempt implement#1.0 --summary "说明实际完成程度"
cargo run -p sheltie-cli -- --home "$example_home" --json attempt begin <work> --node review
```

审查者不参与被审代码编写，按 brief 和 task 的标准独立核对并写 review.md。完成审查后 submit；协调者读报告和当前 next 选择 back 返工或 main 交付。执行成功仅表示该次任务和输出合同完成，报告说“通过”不会自动选边。

返工产生 implement#2，再独立复核 review#2；每次以 begin 返回的实际 AttemptId 和路径提交。previous-review 是被冻结的具体报告，不根据目录里的最新文件猜来源。执行者崩溃或无法交付时，按 next 和 max_retries 处理 fail；普通内容问题仍走 submit 与显式 back。

## 中断后继续

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json work status <work>
```

先核当前 revision、effects_pending、next 和 resume。resume 给当前 Attempt 的任务书、冻结输入与声明草稿位置；草稿路径不证明文件存在或已经封存。历史 request 重放返回当时的 next，当前操作必须以新查询为准。

确认旧执行者与共享工作区已妥善处理后，继续原 running Attempt。会话重开本身不改变状态，不调用 fail 或 begin 制造一次新尝试。effects_pending=true 时，查询不会自动恢复；通过既有写请求重放处理登记效果，错误持续就停止并保留准确原因。

## 取得成果

独立审查完成且协调者选 deliver 后，执行者核 change/review 指向同一实际候选并写 delivery.md，再提交。

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json work result <work>
```

只有 final=true 才展示明确选择：change、review 是该具体 deliver Attempt 绑定的冻结输入，delivery 是它封存的输出。每项给 path、sha256、bytes 与 source；source.attempt 表示终点绑定或封存者。结果查询列举引用，不重新证明源字节或报告内容。接受、复制、合并和发布按各自实际合同与授权核对。

final=false 时集合为空。final=true 但方法未选择成果会明确为空；本方法已选择三个槽。读取报告内容和实际代码，核未完成义务，不能以 Work succeeded 替代用户接受或独立质量结论。

## 语言版本

此目录是持续维护的简体中文方法；英文为默认语言，保留原 ID 并升级版本。中文方法 ID 追加 `-zh-cn`，可与英文版本并存。新的 Git 提交摘要和正文统一使用英文，中文说明语言不改变这项约定。已有 Work 保留其原始冻结字节；本目录不作为原始历史证据。

语言选择链接用于源码阅读；安装或冻结的说明与参考文件保持所选语言，未包含另一种语言的同级目录。
