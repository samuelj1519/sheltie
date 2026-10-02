# D-036 操作主体取真实 OS 进程身份

状态：`accepted`
日期：2026-09-27
关联 change：[C002](../changes/completed/C002-v0.2.0-reliability/README.md)

## 背景

v0.1.0 把审计与 gate 批准的主体记为 `USER`/`USERNAME` 环境变量的值（N04）。环境变量由调用方任意可设：一条 `USER=alice sheltie gate approve …` 就能以任何名字落批准记录。同时宪章承诺「真人批准」，而同一 OS 账户环境下引擎没有任何独立认证手段——承诺与能力不符。

## 选择

主体取发起进程的真实身份：unix 上通过 `uzers::get_effective_uid()` 取得 effective uid，再用 `uzers::get_user_by_uid()` 取账户名；查不到账户条目或账户名不是 UTF-8 时记 `uid:<数值>`，不落到猜测值。完全不读 `USER`/`USERNAME`。这是记账事实；是否等于某个真人，由宿主与用户的授权机制负责，文档与报告只说「已由该账户批准」，不说「已验证独立真人」（宪章 §5 的边界句）。不做自制认证系统。

API 已核对：[uzers `get_effective_uid`](https://docs.rs/uzers/latest/uzers/fn.get_effective_uid.html) 与 [`get_user_by_uid`](https://docs.rs/uzers/latest/uzers/fn.get_user_by_uid.html) 是安全 Rust 接口，内部分别调用 `geteuid` 与 `getpwuid_r`。项目代码继续遵守 `unsafe_code = "forbid"`；非 unix 平台返回 `unknown`（不在发布目标内）。T05 加依赖前按工程规范核对目标平台与 MSRV，不能用直接 `libc` 调用绕过禁令。

## 否决方案

- 继续用环境变量：可伪造，记录无意义。
- 子进程调 `id -un`：每次调用一个进程，失败模式（PATH、locale）无谓增加；`getpwuid_r` 一步到位。
- 自制口令/审批人认证：超出本地单用户工具的边界，制造假安全感。

## C002-T05 实施勘误

实现时 `cargo deny check` 拒绝了原选的 `users` 0.11.0：RUSTSEC-2023-0040（无人维护）、RUSTSEC-2023-0059（未对齐指针读取）等 advisory 命中，而工程规范禁止把告警藏成通过规则。改用同 API 的维护分支 [`uzers`](https://crates.io/crates/uzers)（`users` 的接续维护版，函数签名一致）；本决策的语义与确认方式不变。

## 后果

测试里不能再靠设 `USER` 伪造主体，改用真实 effective uid 的期望值（本机账户名）或 `uid:<n>` 形式。批准记录的语义收窄为「OS 账户记账」，上游宪章、规格、协议与 skill 已同步这句边界。

## 确认方式

设 `USER=someone-else` 后执行 gate approve，断言批准记录不是 `someone-else`；同一测试断言记录等于当前 uid 对应账户名。
