//! 有界文本。上限按字节计，超限拒绝而不截断（宪章 `T-4`）。

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// 最多 `N` 字节的字符串。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct BoundedText<const N: usize>(String);

impl<const N: usize> BoundedText<N> {
    /// 超过 `N` 字节返回 `Error::TextTooLong`。`field` 只用于错误信息。
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

/// 读回也不豁免上限：超限拒绝，不把超限文本悄悄收进库（宪章 `T-4`）。
impl<'de, const N: usize> Deserialize<'de> for BoundedText<N> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.len() > N {
            return Err(serde::de::Error::custom(format!(
                "超过 {N} 字节（实际 {} 字节）",
                value.len()
            )));
        }
        Ok(Self(value))
    }
}

/// 工作 agent 回复摘要的上限（协议 `attempt submit` 第 2 步）。
pub type Summary = BoundedText<4096>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn t02_bounded_text_rejects_over_limit_bytes() {
        assert!(BoundedText::<4>::new("abcd", "x").is_ok());
        assert!(matches!(
            BoundedText::<4>::new("abcde", "x"),
            Err(Error::TextTooLong {
                max: 4,
                actual: 5,
                ..
            })
        ));
        // 两个汉字六个字节
        assert!(BoundedText::<5>::new("汉字", "x").is_err());
        assert!(BoundedText::<6>::new("汉字", "x").is_ok());
    }

    #[test]
    fn t02_bounded_text_deserialize_rejects_over_limit_bytes() {
        assert!(serde_json::from_str::<BoundedText<4>>("\"abcd\"").is_ok());
        assert!(serde_json::from_str::<BoundedText<4>>("\"abcde\"").is_err());
    }
}
