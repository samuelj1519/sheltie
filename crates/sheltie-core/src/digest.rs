//! SHA-256 摘要的十六进制表示。产物冻结与 Workbook 完整性都靠它。

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// 64 位小写十六进制的 SHA-256。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha256Hex(String);

impl Sha256Hex {
    /// 校验一个已有的十六进制串。失败返回 `Error::InvalidDigest`。
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let ok = value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !ok {
            return Err(Error::InvalidDigest { value });
        }
        Ok(Self(value))
    }

    /// 对字节算摘要。纯计算，core 可以做。
    pub fn of_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        Self(format!("{digest:x}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 前 4 位，给人看的短形式。
    pub fn short(&self) -> &str {
        &self.0[..4]
    }
}

impl fmt::Display for Sha256Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Sha256Hex {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<Sha256Hex> for String {
    fn from(value: Sha256Hex) -> String {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t02_sha256_hex_requires_64_lowercase_hex() {
        let ok = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
        assert_eq!(Sha256Hex::new(ok).unwrap().as_str(), ok);
        assert!(Sha256Hex::new(ok.to_uppercase()).is_err());
        assert!(Sha256Hex::new(&ok[..63]).is_err());
        assert!(Sha256Hex::new(format!("{}g", &ok[..63])).is_err());
    }

    #[test]
    fn t01_of_bytes_matches_known_vector() {
        assert_eq!(
            Sha256Hex::of_bytes(b"hello\n").as_str(),
            "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
    }

    #[test]
    fn t02_sha256_hex_short_display_and_into_string() {
        let hex = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
        let h = Sha256Hex::new(hex).unwrap();
        assert_eq!(h.short(), "5891");
        assert_eq!(h.to_string(), hex);
        assert_eq!(String::from(h), hex);
    }
}
