//! runtime 取得真实操作主体与当前时间，再作为 Context 交给 core。

use sheltie_core::work::{Principal, Timestamp};

/// 当前操作主体：发起进程的真实 OS 身份（D-036）。unix 上取 effective uid 对应的
/// 账户名；查不到账户条目或名称不是 UTF-8 时记 `uid:<数值>`，不落到猜测值。
/// 依赖是维护中的同 API 分支 `uzers`（见 D-036 勘误）。
/// 完全不读 `USER`/`USERNAME`——环境变量由调用方任意可设。
pub fn principal() -> Principal {
    #[cfg(unix)]
    {
        let uid = uzers::get_effective_uid();
        match uzers::get_user_by_uid(uid) {
            Some(user) => match user.name().to_str() {
                Some(name) => Principal(name.to_string()),
                None => Principal(format!("uid:{uid}")),
            },
            None => Principal(format!("uid:{uid}")),
        }
    }
    #[cfg(not(unix))]
    {
        Principal("unknown".to_string())
    }
}

/// 当前 UTC 时间，秒精度。格式化在 core 的 `Timestamp::from_unix_secs`。
pub fn now() -> Timestamp {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Timestamp::from_unix_secs(secs)
}
