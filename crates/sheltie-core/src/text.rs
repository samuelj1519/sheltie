//! Bounded text. Limits count bytes; reject overflow without truncation (constitution `T-4`).

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A string of at most `N` bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct BoundedText<const N: usize>(String);

impl<const N: usize> BoundedText<N> {
    /// Return `Error::TextTooLong` above `N` bytes; `field` is diagnostic context only.
    pub fn new(value: impl Into<String>, field: &'static str) -> Result<Self> {
        let value = value.into();
        if value.len() > N {
            return Err(Error::TextTooLong {
                field,
                max: N,
                actual: value.len(),
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn max_bytes() -> usize {
        N
    }
}

/// Reads enforce the same limit; reject oversized text rather than silently storing it (constitution `T-4`).
impl<'de, const N: usize> Deserialize<'de> for BoundedText<N> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.len() > N {
            return Err(serde::de::Error::custom(format!(
                "Exceeds {N} bytes (actual {} bytes)",
                value.len()
            )));
        }
        Ok(Self(value))
    }
}

/// Worker reply summary limit (protocol `attempt submit`, step 2).
pub type Summary = BoundedText<4096>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    // Task: T02
    #[test]
    fn bounded_text_rejects_over_limit_bytes() {
        assert!(BoundedText::<4>::new("abcd", "x").is_ok());
        assert!(matches!(
            BoundedText::<4>::new("abcde", "x"),
            Err(Error::TextTooLong {
                max: 4,
                actual: 5,
                ..
            })
        ));
        // Two Han characters occupy six UTF-8 bytes.
        assert!(BoundedText::<5>::new("汉字", "x").is_err());
        assert!(BoundedText::<6>::new("汉字", "x").is_ok());
    }

    // Task: T02
    #[test]
    fn bounded_text_deserialize_rejects_over_limit_bytes() {
        assert!(serde_json::from_str::<BoundedText<4>>("\"abcd\"").is_ok());
        assert!(serde_json::from_str::<BoundedText<4>>("\"abcde\"").is_err());
    }
}
