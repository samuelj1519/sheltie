//! 两种路径类型。`RelPath` 是 Workbook 内或 Attempt 目录内的相对路径；`AbsPath` 是绝对路径。
//! 把两者分开是为了让「这个路径能不能拼到根下面」在类型上就有答案。

use std::fmt;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// 受限相对路径：不含 `..`、不是绝对路径、没有空段、UTF-8。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelPath(Utf8PathBuf);

impl RelPath {
    /// 校验后构造。失败返回 `Error::InvalidPath`。
    ///
    /// 拒绝：空字符串；以 `/` 开头；任一段为 `..`；任一段为空（如 `a//b`）；段为 `.`。
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value: String = value.into();
        let reject = |reason: &'static str| Error::InvalidPath {
            path: value.clone(),
            reason,
        };
        if value.is_empty() {
            return Err(reject("不能为空"));
        }
        if value.starts_with('/') {
            return Err(reject("不能是绝对路径"));
        }
        for seg in value.split('/') {
            if seg.is_empty() {
                return Err(reject("不能有空段"));
            }
            if seg == ".." {
                return Err(reject("不能含 .."));
            }
            if seg == "." {
                return Err(reject("段不能是 ."));
            }
        }
        Ok(Self(Utf8PathBuf::from(value)))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_path(&self) -> &Utf8Path {
        &self.0
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl TryFrom<String> for RelPath {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<RelPath> for String {
    fn from(value: RelPath) -> String {
        value.0.into_string()
    }
}

/// 绝对路径。只保证「是绝对的」，不保证存在。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AbsPath(Utf8PathBuf);

impl AbsPath {
    /// 校验后构造。不是绝对路径返回 `Error::InvalidPath`。
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value: String = value.into();
        let path = Utf8PathBuf::from(&value);
        if !path.is_absolute() {
            return Err(Error::InvalidPath {
                path: value,
                reason: "不是绝对路径",
            });
        }
        Ok(Self(path))
    }

    /// 在本路径下拼一个相对路径。`RelPath` 已保证不会逃出去。
    pub fn join(&self, rel: &RelPath) -> AbsPath {
        AbsPath(self.0.join(rel.as_path()))
    }

    /// 拼一个已知安全的字面段（如 `"attempts"`）。调用方保证不含分隔符。
    pub fn join_segment(&self, segment: &str) -> AbsPath {
        AbsPath(self.0.join(segment))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_path(&self) -> &Utf8Path {
        &self.0
    }
}

impl fmt::Display for AbsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl TryFrom<String> for AbsPath {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<AbsPath> for String {
    fn from(value: AbsPath) -> String {
        value.0.into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t02_rel_path_rejects_dotdot_and_absolute() {
        assert!(RelPath::new("instructions/draft.md").is_ok());
        for bad in ["../x", "a/../b", "/abs", "", "a//b", "./a"] {
            assert!(
                matches!(RelPath::new(bad), Err(Error::InvalidPath { .. })),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn t01_abs_path_requires_absolute() {
        assert!(AbsPath::new("/tmp/x").is_ok());
        assert!(AbsPath::new("tmp/x").is_err());
        let root = AbsPath::new("/root").unwrap();
        assert_eq!(
            root.join_segment("attempts").join_segment("draft").as_str(),
            "/root/attempts/draft"
        );
    }
}
