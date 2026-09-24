//! 对文件系统的只读观察。这是 core 拿到「事实」的唯一通道。

use sheltie_core::flow::ResourceIndex;
use sheltie_core::path::AbsPath;
use sheltie_core::work::{ObservedFile, Principal, Timestamp};

use crate::error::Result;

/// 观察一个文件：必须是普通文件（`symlink_metadata`，不跟随软链）；算 sha256 与字节数。
/// 不存在返回 `Error::NotFound`；是软链或目录返回 `Error::InvalidRequest`。
#[allow(unused_variables)]
pub fn observe_file(path: &AbsPath) -> Result<ObservedFile> {
    todo!("T12")
}

/// 观察一个文件，不存在时返回 `Ok(None)`，其他错误照常返回。
pub fn observe_optional(path: &AbsPath) -> Result<Option<ObservedFile>> {
    match observe_file(path) {
        Ok(f) => Ok(Some(f)),
        Err(crate::Error::NotFound { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

/// 递归列出目录下全部普通文件，建 `ResourceIndex`。跳过目录，拒绝软链（返回错误）。
/// 键是相对 `dir` 的路径。`is_utf8` 通过读全文判断。
#[allow(unused_variables)]
pub fn build_resource_index(dir: &AbsPath) -> Result<ResourceIndex> {
    todo!("T12")
}

/// 当前操作系统用户名，取 `USER` 或 `USERNAME`，都没有则 `unknown`。
pub fn principal() -> Principal {
    let name = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    Principal(name)
}

/// 当前 UTC 时间，RFC 3339，秒精度。
pub fn now() -> Timestamp {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Timestamp(format_rfc3339(secs))
}

/// 把 Unix 秒数格式化成 `YYYY-MM-DDTHH:MM:SSZ`。纯算法，不依赖时区库。
pub fn format_rfc3339(secs: u64) -> String {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

// Howard Hinnant 的 days_from_civil 逆运算。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t01_format_rfc3339_epoch_and_known_date() {
        assert_eq!(format_rfc3339(0), "1970-01-01T00:00:00Z");
        // 2026-09-24T03:00:00Z
        assert_eq!(format_rfc3339(1_790_218_800), "2026-09-24T03:00:00Z");
    }
}
