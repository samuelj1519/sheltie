//! 有界文本。上限按字节计，超限拒绝而不截断（宪章 `T-4`）。

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// 最多 `N` 字节的字符串。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BoundedText<const N: usize>(String);

impl<const N: usize> BoundedText<N> {
    /// 超过 `N` 字节返回 `Error::TextTooLong`。`field` 只用于错误信息。
    #[allow(unused_variables)]
    pub fn new(value: impl Into<String>, field: &'static str) -> Result<Self> {
        todo!("T02")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn max_bytes() -> usize {
        N
    }
}

/// 工作 agent 回复摘要的上限（协议 `attempt submit` 第 2 步）。
pub type Summary = BoundedText<4096>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    #[ignore = "T02"]
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
}
