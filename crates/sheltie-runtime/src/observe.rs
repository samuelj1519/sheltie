//! Runtime obtains the actual principal and timestamp, then passes Context to core.

use sheltie_core::work::{Principal, Timestamp};

/// Current principal: initiating process OS identity (D-036); on Unix use the effective UID's
/// account name, falling back to uid:<number> for missing or non-UTF-8 names without guesses.
/// The dependency is the maintained, API-compatible uzers fork (D-036 erratum).
/// Never read USER/USERNAME, which callers can set arbitrarily.
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

/// Current UTC time at second precision; core Timestamp::from_unix_secs formats it.
pub fn now() -> Timestamp {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Timestamp::from_unix_secs(secs)
}
