# 安装、更新、回滚与卸载引擎

[English](../../en/how-to/manage-installation.md) | 简体中文

用于管理已经取得的可信 `sheltie` 二进制。安装前核二进制来源、版本和目标管理根；支持范围见[限制](../reference/limitations.md)。只想从源码学习时使用[构建指南](build-from-source.md)，无需安装。

命令中的 `engine_binary` 是已核验的绝对二进制路径，`management_root` 是本次明确选择的绝对管理根。每条命令非零或 `ok=false` 时停止，保存完整响应与错误，不串行执行后续动作。

## 1. 选择相匹配的版本与管理根

```bash
"$engine_binary" --home "$management_root" --json self version
```

核 `data.version`、`data.platform`、`data.home`、`data.schema_version`。正式发布及其二进制实物见[发布记录](../reference/releases/README.md)；开发源码使用 Store schema 4，正式 v0.2.0 使用 schema 2。两者不能共用业务 Store；旧记录使用对应旧二进制和旧根查询。

新安装选择新根。`self install` 安装的是正在执行的二进制，不会自动取得正式版；从开发源码执行它就安装开发候选。路径确认无误后：

```bash
"$engine_binary" --home "$management_root" --json self install
installed_binary="$management_root/bin/sheltie"
"$installed_binary" --home "$management_root" --json self version
```

成功返回 `installed_to`；相同字节已安装时 `already_installed=true`。安装只写管理根，不写 shell 配置。需要短命令时，在当前 shell 临时设置：

```bash
export PATH="$management_root/bin:$PATH"
```

后续仍显式带 `--home`，避免 PATH 指向一个版本而业务操作落在另一个根。协调者 skill 是独立交付物；核对[skill 说明](../../../skills/sheltie/SKILL.zh-CN.md)，在宿主按实际授权配置，引擎不会代为安装。

## 2. 更新已发布版本

更新从正式发布渠道取得包。先核目标版本、平台资产、Store 格式及实际 `SHELTIE_RELEASE_BASE`；未设置时使用本项目 GitHub Releases。不要用 schema 4 开发根试装 schema 2 正式版来验证业务兼容性。

在与发布版匹配的已安装根中，明确选择版本后执行；下例仅适用于目标确为 `0.2.0`：

```bash
"$installed_binary" --home "$management_root" --json self update --version 0.2.0
"$installed_binary" --home "$management_root" --json self version
```

版本值不带 `v`。省略 `--version` 会选择 latest，不适合固定候选的复现。成功核 `from`、`to`、`up_to_date` 和更新后的真实版本。新字节经过发布清单摘要核验；网络、平台或摘要失败按准确错误停止，不跳过校验。

## 3. 回滚一级二进制

确认根与 `bin/sheltie.prev` 属于本次安装，并且已经授权回退后：

```bash
"$installed_binary" --home "$management_root" --json self rollback
"$installed_binary" --home "$management_root" --json self version
```

成功后只有恢复的二进制，prev 被移回。回滚不恢复 Work、不降级 Store，也不证明恢复版能打开当前库。若更新中断造成主 binary 缺失但 prev 存在，用该根下已核验的 prev 调用相同 `self rollback`；不能下载一个猜测版本替代原回滚实物。

## 4. 卸载并保留数据

只移除引擎安装时执行：

```bash
"$installed_binary" --home "$management_root" --json self uninstall
```

核 `data.kept`。默认删除 `bin/`，保留 Store、Workbook、Work 和其他管理数据；后续查询仍需要相匹配的二进制。卸载不删除宿主配置或独立安装的 skill。

## 5. 明确清空管理数据

purge 会删除本根的运行数据、方法、pending、暂存和二进制，不能再继续查询这些 Work。先记录真实根与完整删除范围，保存需要保留的原件，并取得清空该根的明确授权；不要把 purge 当作排障或格式升级步骤。

文本模式运行 `self uninstall --purge` 并输入 `yes`；自动化 JSON 模式必须显式确认：

```bash
"$installed_binary" --home "$management_root" --json self uninstall --purge --yes
```

成功保留空根与原 `.lock`，不是删除整个根目录。失败可能已部分清理；保留准确错误，核剩余对象与原意图后再处理，不将部分删除解释为“什么都没发生”。全部 self 命令都不接受 `--request-id`。
