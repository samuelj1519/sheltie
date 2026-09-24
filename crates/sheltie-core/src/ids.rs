//! 强类型 ID。构造即校验，模块边界不传裸 `String`。
//!
//! ID 字符规则见 `specs/contracts/workbook.md` §2：`^[a-z0-9]+(-[a-z0-9]+)*$`，≤ 64 字节。
//! `work_id` 与名字规则见 `specs/contracts/protocol.md` `work start` 第 3、4 步。

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// 校验一个 kebab-case ID。`field` 只用于错误信息。
///
/// 失败返回 `Error::InvalidId`。规则：非空；≤ 64 字节；只含 `a-z0-9-`；
/// 不以 `-` 开头或结尾；没有连续 `-`。
#[allow(unused_variables)]
pub fn validate_id(value: &str, field: &str) -> Result<()> {
    todo!("T02")
}

macro_rules! kebab_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// 校验后构造。失败返回 `Error::InvalidId`。
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                validate_id(&value, stringify!($name))?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;
            fn try_from(value: String) -> Result<Self> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0
            }
        }
    };
}

kebab_id!(WorkbookId, "Workbook 的稳定身份，全局唯一。");
kebab_id!(FlowId, "Workbook 内一张图的 ID。");
kebab_id!(
    NodeId,
    "Flow 内一个节点的 ID。`start` 与 `resource` 是保留字，编译规则 1 拒绝。"
);

/// Work 的名字，`work_id` 的后缀部分。
///
/// 规范化规则（协议 `work start` 第 3 步）：去首尾空白，连续空白替换为一个 `-`，转小写；
/// 之后必须匹配 `^[a-z0-9\p{Han}]+(-[a-z0-9\p{Han}]+)*$` 且 ≤ 48 字节。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WorkName(String);

impl WorkName {
    /// 最大字节数。
    pub const MAX_BYTES: usize = 48;

    /// 规范化并校验。失败返回 `Error::InvalidId { field: "work_name", .. }`。
    #[allow(unused_variables)]
    pub fn normalize(raw: &str) -> Result<Self> {
        todo!("T02")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for WorkName {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Self::normalize(&value)
    }
}

impl From<WorkName> for String {
    fn from(value: WorkName) -> String {
        value.0
    }
}

/// `work_id`，形如 `2026-09-24-003-文章-初稿`：UTC 日期、当日序号（三位补零）、名字。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WorkId(String);

impl WorkId {
    /// 用日期、序号、名字拼出 `work_id`。
    ///
    /// `day` 必须是 `YYYY-MM-DD`；`seq` 必须在 1..=999；否则 `Error::InvalidId { field: "work_id", .. }`。
    #[allow(unused_variables)]
    pub fn new(day: &str, seq: u32, name: &WorkName) -> Result<Self> {
        todo!("T02")
    }

    /// 解析一个完整的 `work_id` 字符串（用于从数据库读回）。
    ///
    /// 形状必须是 `YYYY-MM-DD-NNN-<name>` 且 `<name>` 能通过 `WorkName::normalize` 且规范化后不变。
    #[allow(unused_variables)]
    pub fn parse(value: &str) -> Result<Self> {
        todo!("T02")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for WorkId {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<WorkId> for String {
    fn from(value: WorkId) -> String {
        value.0
    }
}

/// Attempt 的 ID：节点、第几次到达、第几次重试。显示为 `draft#1.0`。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AttemptId {
    pub node: NodeId,
    pub occurrence: u32,
    pub retry: u32,
}

impl AttemptId {
    pub fn new(node: NodeId, occurrence: u32, retry: u32) -> Self {
        Self {
            node,
            occurrence,
            retry,
        }
    }

    /// 解析 `node#n.retry`。失败返回 `Error::InvalidId { field: "attempt_id", .. }`。
    #[allow(unused_variables)]
    pub fn parse(value: &str) -> Result<Self> {
        todo!("T02")
    }
}

impl fmt::Display for AttemptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}.{}", self.node, self.occurrence, self.retry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(s: &str) -> WorkName {
        WorkName::normalize(s).unwrap()
    }

    #[test]
    #[ignore = "T02"]
    fn t02_workbook_id_accepts_kebab_case() {
        assert_eq!(
            WorkbookId::new("article-review").unwrap().as_str(),
            "article-review"
        );
        assert_eq!(WorkbookId::new("a1").unwrap().as_str(), "a1");
    }

    #[test]
    #[ignore = "T02"]
    fn t02_workbook_id_rejects_uppercase_and_double_dash() {
        for bad in ["Article", "a--b", "-a", "a-", "", "a_b", &"a".repeat(65)] {
            assert!(
                matches!(WorkbookId::new(bad), Err(Error::InvalidId { .. })),
                "{bad:?}"
            );
        }
    }

    #[test]
    #[ignore = "T02"]
    fn t02_attempt_id_formats_as_node_hash_n_dot_retry() {
        let id = AttemptId::new(NodeId::new("draft").unwrap(), 1, 0);
        assert_eq!(id.to_string(), "draft#1.0");
        assert_eq!(
            AttemptId::parse("review#2.1").unwrap(),
            AttemptId::new(NodeId::new("review").unwrap(), 2, 1)
        );
        assert!(AttemptId::parse("review#x.1").is_err());
    }

    #[test]
    #[ignore = "T02"]
    fn t02_work_id_builds_from_day_seq_and_name() {
        let id = WorkId::new("2026-09-24", 3, &name("文章-初稿")).unwrap();
        assert_eq!(id.as_str(), "2026-09-24-003-文章-初稿");
        assert_eq!(WorkId::parse("2026-09-24-003-文章-初稿").unwrap(), id);
    }

    #[test]
    #[ignore = "T02"]
    fn t02_work_name_normalizes_whitespace_and_case() {
        assert_eq!(name("  文章  初稿 ").as_str(), "文章-初稿");
        assert_eq!(name("Export CSV").as_str(), "export-csv");
    }

    #[test]
    #[ignore = "T02"]
    fn t02_work_name_rejects_over_48_bytes_and_bad_chars() {
        assert!(WorkName::normalize(&"文".repeat(17)).is_err(), "51 字节");
        assert!(WorkName::normalize("a/b").is_err());
        assert!(WorkName::normalize("").is_err());
        assert!(WorkName::normalize("--").is_err());
    }

    #[test]
    #[ignore = "T02"]
    fn t02_work_id_rejects_seq_zero_or_over_999() {
        assert!(WorkId::new("2026-09-24", 0, &name("x")).is_err());
        assert!(WorkId::new("2026-09-24", 1000, &name("x")).is_err());
        assert!(WorkId::new("20260924", 1, &name("x")).is_err());
    }
}
