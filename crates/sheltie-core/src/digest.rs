//! Hexadecimal SHA-256 digests for artifact sealing and Workbook integrity.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// A SHA-256 digest represented by 64 lowercase hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha256Hex(String);

impl Sha256Hex {
    /// Validate a hexadecimal string; return `Error::InvalidDigest` on failure.
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

    /// Digest bytes with pure computation suitable for core.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        Self(format!("{digest:x}"))
    }

    /// Convert sha2 output directly; finalize streaming digests (`workbook-digest/v2`)
    /// with one SHA256 of the byte stream; do not hash it again through `of_bytes`.
    pub fn from_sha256(output: sha2::digest::Output<Sha256>) -> Self {
        Self(format!("{output:x}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The first four digits, for human-readable display.
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

/// Domain prefix for `workbook-digest/v2` (storage contract §5.1), followed by
/// the BE64 file count and per-file frames in the directory digest stream.
pub const WORKBOOK_DIGEST_V2_PREFIX: &[u8] = b"sheltie-workbook-digest/v2\0";

/// BE64: an eight-byte unsigned big-endian integer; counts and lengths make file boundaries unambiguous (O07).
pub fn be64(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

/// File frame header: `BE64(path byte length) || path || BE64(content byte length)`, followed by content.
/// Runtime sorts paths bytewise and streams contents; core only assembles bytes.
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

    // Expected frame bytes are handwritten independently of the implementation.

    // Task: C002-T09
    #[test]
    fn digest_v2_prefix_is_domain_string_with_nul() {
        // Handwritten domain-prefix bytes: ASCII plus one NUL.
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
        // path = "za" (two bytes), content length three: a handwritten frame header.
        assert_eq!(
            digest_v2_file_frame(b"za", 3),
            vec![
                0, 0, 0, 0, 0, 0, 0, 2, // BE64(2)
                b'z', b'a', //
                0, 0, 0, 0, 0, 0, 0, 3, // BE64(3)
            ]
        );
        // Empty paths and content lengths are explicitly framed to avoid boundary ambiguity.
        assert_eq!(
            digest_v2_file_frame(b"", 0),
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }
}
