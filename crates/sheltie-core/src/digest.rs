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

    /// 把 sha2 的摘要输出直接转成本类型。流式摘要（`workbook-digest/v2`）的收口：
    /// 摘出的是字节流的**单次** SHA256，不得再经过 `of_bytes` 二次哈希。
    pub fn from_sha256(output: sha2::digest::Output<Sha256>) -> Self {
        Self(format!("{output:x}"))
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

/// `workbook-digest/v2` 的域前缀（存储合同 §5.1）。目录摘要字节流以它开头，
/// 之后是 BE64 文件数与逐文件帧。
pub const WORKBOOK_DIGEST_V2_PREFIX: &[u8] = b"sheltie-workbook-digest/v2\0";

/// BE64：8 字节大端无符号整数。数量与长度都入流，文件边界无歧义（O07）。
pub fn be64(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

/// 一个文件的帧头：`BE64(路径字节数) || 路径 || BE64(内容字节数)`，内容字节紧随其后。
/// 路径按字节序排序、内容以流式供给由 runtime 完成；core 只做纯字节组装。
pub fn digest_v2_file_frame(path: &[u8], content_len: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + path.len());
    out.extend_from_slice(&be64(path.len() as u64));
    out.extend_from_slice(path);
    out.extend_from_slice(&be64(content_len));
    out
}

impl From<Sha256Hex> for String {
    fn from(value: Sha256Hex) -> String {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task: T02
    #[test]
    fn sha256_hex_requires_64_lowercase_hex() {
        let ok = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
        assert_eq!(Sha256Hex::new(ok).unwrap().as_str(), ok);
        assert!(Sha256Hex::new(ok.to_uppercase()).is_err());
        assert!(Sha256Hex::new(&ok[..63]).is_err());
        assert!(Sha256Hex::new(format!("{}g", &ok[..63])).is_err());
    }

    // Task: T01
    #[test]
    fn of_bytes_matches_known_vector() {
        assert_eq!(
            Sha256Hex::of_bytes(b"hello\n").as_str(),
            "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
    }

    // Task: T02
    #[test]
    fn sha256_hex_short_display_and_into_string() {
        let hex = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
        let h = Sha256Hex::new(hex).unwrap();
        assert_eq!(h.short(), "5891");
        assert_eq!(h.to_string(), hex);
        assert_eq!(String::from(h), hex);
    }

    // 期望帧字节全部手写，不经过被测函数。

    // Task: C002-T09
    #[test]
    fn digest_v2_prefix_is_domain_string_with_nul() {
        // 域前缀逐字节手写：ASCII 串加一个 NUL。
        let expected: &[u8] = &[
            b's', b'h', b'e', b'l', b't', b'i', b'e', b'-', b'w', b'o', b'r', b'k', b'b', b'o',
            b'o', b'k', b'-', b'd', b'i', b'g', b'e', b's', b't', b'/', b'v', b'2', 0,
        ];
        assert_eq!(WORKBOOK_DIGEST_V2_PREFIX, expected);
        assert_eq!(WORKBOOK_DIGEST_V2_PREFIX.len(), 27);
    }

    // Task: C002-T09
    #[test]
    fn be64_encodes_big_endian_bytes() {
        assert_eq!(be64(0), [0; 8]);
        assert_eq!(be64(1), [0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(be64(256), [0, 0, 0, 0, 0, 0, 1, 0]);
        assert_eq!(
            be64(u64::MAX),
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    // Task: C002-T09
    #[test]
    fn digest_v2_file_frame_is_length_prefixed_path_and_content() {
        // path = "za"（2 字节）、content 3 字节：帧头逐字节手写。
        assert_eq!(
            digest_v2_file_frame(b"za", 3),
            vec![
                0, 0, 0, 0, 0, 0, 0, 2, // BE64(2)
                b'z', b'a', //
                0, 0, 0, 0, 0, 0, 0, 3, // BE64(3)
            ]
        );
        // 空路径零长度内容也在帧里显式编码，边界无歧义。
        assert_eq!(
            digest_v2_file_frame(b"", 0),
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }
}
