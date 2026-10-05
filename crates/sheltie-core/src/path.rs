//! Path types: `RelPath` is relative to a Workbook or Attempt directory; `AbsPath` is absolute.
//! Separate types make safe joining beneath a root explicit.

use std::fmt;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A restricted UTF-8 relative path: no `..`, absolute prefix, or empty segments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelPath(Utf8PathBuf);

impl RelPath {
    /// Validate on construction; return `Error::InvalidPath` on failure.
    ///
    /// Reject empty strings, leading `/`, and segments that are `..`, empty (such as `a//b`), or `.`.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value: String = value.into();
        let reject = |reason: &'static str| Error::InvalidPath {
            path: value.clone(),
            reason,
        };
        if value.is_empty() {
            return Err(reject("Must not be empty"));
        }
        if value.starts_with('/') {
            return Err(reject("Must not be absolute"));
        }
        for seg in value.split('/') {
            if seg.is_empty() {
                return Err(reject("Must not contain empty segments"));
            }
            if seg == ".." {
                return Err(reject("Must not contain .."));
            }
            if seg == "." {
                return Err(reject("A segment must not be ."));
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

/// An absolute path; does not guarantee existence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AbsPath(Utf8PathBuf);

impl AbsPath {
    /// Validate on construction; return `Error::InvalidPath` for a nonabsolute path.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value: String = value.into();
        let path = Utf8PathBuf::from(&value);
        if !path.is_absolute() {
            return Err(Error::InvalidPath {
                path: value,
                reason: "Not an absolute path",
            });
        }
        Ok(Self(path))
    }

    /// Join a relative path beneath this path; `RelPath` guarantees confinement.
    pub fn join(&self, rel: &RelPath) -> AbsPath {
        AbsPath(self.0.join(rel.as_path()))
    }

    /// Join a known-safe literal segment (such as `"attempts"`); the caller guarantees no separators.
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

    // Task: T02
    #[test]
    fn rel_path_rejects_dotdot_and_absolute() {
        assert!(RelPath::new("instructions/draft.md").is_ok());
        for bad in ["../x", "a/../b", "/abs", "", "a//b", "./a"] {
            assert!(
                matches!(RelPath::new(bad), Err(Error::InvalidPath { .. })),
                "{bad:?}"
            );
        }
    }

    // Task: T01
    #[test]
    fn abs_path_requires_absolute() {
        assert!(AbsPath::new("/tmp/x").is_ok());
        assert!(AbsPath::new("tmp/x").is_err());
        let root = AbsPath::new("/root").unwrap();
        assert_eq!(
            root.join_segment("attempts").join_segment("draft").as_str(),
            "/root/attempts/draft"
        );
    }

    // Task: T02
    #[test]
    fn paths_display_and_convert_into_string() {
        let rel = RelPath::new("resources/a.md").unwrap();
        assert_eq!(rel.to_string(), "resources/a.md");
        assert_eq!(String::from(rel), "resources/a.md");
        let abs = AbsPath::new("/tmp/x").unwrap();
        assert_eq!(String::from(abs), "/tmp/x");
    }
}
