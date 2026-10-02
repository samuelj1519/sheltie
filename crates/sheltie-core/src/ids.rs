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
pub fn validate_id(value: &str, field: &str) -> Result<()> {
    let reject = |reason: &'static str| Error::InvalidId {
        field: field.to_string(),
        value: value.to_string(),
        reason,
    };
    if value.is_empty() {
        return Err(reject("不能为空"));
    }
    if value.len() > 64 {
        return Err(reject("超过 64 字节"));
    }
    if value.starts_with('-') || value.ends_with('-') {
        return Err(reject("不能以 - 开头或结尾"));
    }
    if value.contains("--") {
        return Err(reject("不能有连续 -"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(reject("只能含 a-z、0-9 与 -"));
    }
    Ok(())
}

/// `YYYY-MM-DD` 的格式：四段数字与两个固定位置的连字符。不做日历校验。
fn is_day_shape(day: &str) -> bool {
    let b = day.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..].iter().all(u8::is_ascii_digit)
}

/// 规范十进制：`0`，或 `1-9` 后跟任意位。拒绝前导 `+`、前导零、空串与溢出。
fn parse_u32_canonical(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if s.len() > 1 && s.starts_with('0') {
        return None;
    }
    s.parse().ok()
}

/// 协议 `work start` 第 3 步列出的十二个汉字区间。std 没有 Unicode 脚本分类，按区段判断。
fn is_han(c: char) -> bool {
    matches!(
        c,
        '\u{3400}'..='\u{4DBF}'      // 扩展 A
        | '\u{4E00}'..='\u{9FFF}'    // 基本区
        | '\u{F900}'..='\u{FAFF}'    // 兼容
        | '\u{20000}'..='\u{2A6DF}'  // 扩展 B
        | '\u{2A700}'..='\u{2B73F}'  // 扩展 C
        | '\u{2B740}'..='\u{2B81F}'  // 扩展 D
        | '\u{2B820}'..='\u{2CEAF}'  // 扩展 E
        | '\u{2CEB0}'..='\u{2EBEF}'  // 扩展 F
        | '\u{2EBF0}'..='\u{2EE5F}'  // 扩展 I
        | '\u{2F800}'..='\u{2FA1F}'  // 兼容补充
        | '\u{30000}'..='\u{3134F}'  // 扩展 G
        | '\u{31350}'..='\u{323AF}'  // 扩展 H
    )
}

macro_rules! kebab_id {
    ($name:ident, $field:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// 校验后构造。失败返回 `Error::InvalidId`。
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                validate_id(&value, $field)?;
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

kebab_id!(WorkbookId, "workbook_id", "Workbook 的稳定身份，全局唯一。");
kebab_id!(FlowId, "flow_id", "Workbook 内一张图的 ID。");
kebab_id!(
    NodeId,
    "node_id",
    "Flow 内一个节点的 ID。`start` 与 `resource` 是保留字，编译规则 1 拒绝。"
);

/// Work 的名字，`work_id` 的后缀部分。
///
/// 规范化规则（协议 `work start` 第 3 步）：去首尾空白，连续空白替换为一个 `-`，转小写；
/// 之后只含小写字母、数字、汉字与单个 `-`（不首尾、不连续），≤ 48 字节。「汉字」是协议 `work start` 第 3 步列出的十二个码点区间。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WorkName(String);

impl WorkName {
    /// 最大字节数。
    pub const MAX_BYTES: usize = 48;

    /// 规范化并校验。失败返回 `Error::InvalidId { field: "work_name", .. }`。
    pub fn normalize(raw: &str) -> Result<Self> {
        let value = raw
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
            .to_lowercase();
        let reject = |reason: &'static str| Error::InvalidId {
            field: "work_name".to_string(),
            value: value.clone(),
            reason,
        };
        if value.is_empty() {
            return Err(reject("不能为空"));
        }
        if value.len() > Self::MAX_BYTES {
            return Err(reject("超过 48 字节"));
        }
        if value.starts_with('-') || value.ends_with('-') {
            return Err(reject("不能以 - 开头或结尾"));
        }
        if value.contains("--") {
            return Err(reject("不能有连续 -"));
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || is_han(c))
        {
            return Err(reject("只能含 a-z、0-9、汉字与 -"));
        }
        Ok(Self(value))
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
        // 读取时（serde、存库往返）只收规范化形式，不替脏数据悄悄改名，与 `WorkId::parse` 同规矩。
        let name = Self::normalize(&value)?;
        if name.0 != value {
            return Err(Error::InvalidId {
                field: "work_name".to_string(),
                value,
                reason: "不是规范化形式",
            });
        }
        Ok(name)
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
    pub fn new(day: &str, seq: u32, name: &WorkName) -> Result<Self> {
        let reject = |value: &str, reason: &'static str| Error::InvalidId {
            field: "work_id".to_string(),
            value: value.to_string(),
            reason,
        };
        if !is_day_shape(day) {
            return Err(reject(day, "日期必须是 YYYY-MM-DD"));
        }
        if !(1..=999).contains(&seq) {
            return Err(reject(&seq.to_string(), "序号必须在 1..=999"));
        }
        Ok(Self(format!("{day}-{seq:03}-{}", name.as_str())))
    }

    /// 解析一个完整的 `work_id` 字符串（用于从数据库读出）。
    ///
    /// 格式必须是 `YYYY-MM-DD-NNN-<name>` 且 `<name>` 能通过 `WorkName::normalize` 且规范化后不变。
    pub fn parse(value: &str) -> Result<Self> {
        let reject = |reason: &'static str| Error::InvalidId {
            field: "work_id".to_string(),
            value: value.to_string(),
            reason,
        };
        let day = value
            .get(..10)
            .ok_or_else(|| reject("格式不是 YYYY-MM-DD-NNN-名字"))?;
        let seq_str = value
            .get(11..14)
            .ok_or_else(|| reject("格式不是 YYYY-MM-DD-NNN-名字"))?;
        let name_str = value
            .get(15..)
            .ok_or_else(|| reject("格式不是 YYYY-MM-DD-NNN-名字"))?;
        if value.as_bytes().get(10) != Some(&b'-') || value.as_bytes().get(14) != Some(&b'-') {
            return Err(reject("格式不是 YYYY-MM-DD-NNN-名字"));
        }
        if !is_day_shape(day) {
            return Err(reject("日期必须是 YYYY-MM-DD"));
        }
        if !seq_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(reject("序号必须是三位数字"));
        }
        let seq: u32 = seq_str.parse().map_err(|_| reject("序号必须是三位数字"))?;
        let name = WorkName::normalize(name_str)?;
        if name.as_str() != name_str {
            return Err(reject("名字不是规范化形式"));
        }
        Self::new(day, seq, &name)
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
#[serde(deny_unknown_fields)]
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
    pub fn parse(value: &str) -> Result<Self> {
        let reject = |reason: &'static str| Error::InvalidId {
            field: "attempt_id".to_string(),
            value: value.to_string(),
            reason,
        };
        let (node_str, rest) = value
            .split_once('#')
            .ok_or_else(|| reject("格式不是 节点#n.retry"))?;
        let (occ_str, retry_str) = rest
            .split_once('.')
            .ok_or_else(|| reject("格式不是 节点#n.retry"))?;
        let node = NodeId::new(node_str).map_err(|e| match e {
            Error::InvalidId { reason, .. } => reject(reason),
            e => e,
        })?;
        let occurrence: u32 =
            parse_u32_canonical(occ_str).ok_or_else(|| reject("到达次数不是数字"))?;
        let retry: u32 =
            parse_u32_canonical(retry_str).ok_or_else(|| reject("重试序号不是数字"))?;
        Ok(Self::new(node, occurrence, retry))
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

    // Task: T02
    #[test]
    fn workbook_id_accepts_kebab_case() {
        assert_eq!(
            WorkbookId::new("article-review").unwrap().as_str(),
            "article-review"
        );
        assert_eq!(WorkbookId::new("a1").unwrap().as_str(), "a1");
    }

    // Task: T02
    #[test]
    fn workbook_id_rejects_uppercase_and_double_dash() {
        for bad in ["Article", "a--b", "-a", "a-", "", "a_b", &"a".repeat(65)] {
            assert!(
                matches!(WorkbookId::new(bad), Err(Error::InvalidId { .. })),
                "{bad:?}"
            );
        }
    }

    // Task: T02
    #[test]
    fn attempt_id_formats_as_node_hash_n_dot_retry() {
        let id = AttemptId::new(NodeId::new("draft").unwrap(), 1, 0);
        assert_eq!(id.to_string(), "draft#1.0");
        assert_eq!(
            AttemptId::parse("review#2.1").unwrap(),
            AttemptId::new(NodeId::new("review").unwrap(), 2, 1)
        );
        assert!(AttemptId::parse("review#x.1").is_err());
    }

    // Task: T02
    #[test]
    fn work_id_builds_from_day_seq_and_name() {
        let id = WorkId::new("2026-09-24", 3, &name("文章-初稿")).unwrap();
        assert_eq!(id.as_str(), "2026-09-24-003-文章-初稿");
        assert_eq!(WorkId::parse("2026-09-24-003-文章-初稿").unwrap(), id);
    }

    // Task: T02
    #[test]
    fn work_name_normalizes_whitespace_and_case() {
        assert_eq!(name("  文章  初稿 ").as_str(), "文章-初稿");
        assert_eq!(name("Export CSV").as_str(), "export-csv");
    }

    // Task: T02
    #[test]
    fn work_name_rejects_over_48_bytes_and_bad_chars() {
        assert!(WorkName::normalize(&"文".repeat(17)).is_err(), "51 字节");
        assert!(WorkName::normalize("a/b").is_err());
        assert!(WorkName::normalize("").is_err());
        assert!(WorkName::normalize("--").is_err());
    }

    // Task: T02
    #[test]
    fn work_id_parse_rejects_plus_sign_in_seq() {
        // str::parse::<u32> 接受前导 +，解析层必须自己拒绝，否则解析再拼出会得到不同的串。
        assert!(WorkId::parse("2026-09-24-+12-x").is_err());
        assert!(WorkId::parse("2026-09-24-0x1-x").is_err());
    }

    // Task: T02
    #[test]
    fn attempt_id_parse_rejects_plus_sign() {
        assert!(AttemptId::parse("draft#+1.0").is_err());
        assert!(AttemptId::parse("draft#1.+0").is_err());
        assert!(AttemptId::parse("draft# 1.0").is_err());
    }

    // Task: T02
    #[test]
    fn parse_rejects_leading_zero_in_numeric_segments() {
        // "01".parse::<u32>() 放行，会让 draft#01.0 重排成 draft#1.0；序号段必须是规范十进制。
        assert!(AttemptId::parse("draft#01.0").is_err());
        assert!(AttemptId::parse("draft#1.00").is_err());
        assert!(AttemptId::parse("draft#1.0").is_ok());
        assert!(
            AttemptId::parse("draft#0.0").is_ok(),
            "到达次数与重试都接受 0 本身"
        );
        // work_id 的序号固定三位，恰好三位数字才合法。
        assert!(WorkId::parse("2026-09-24-0001-x").is_err());
        assert!(WorkId::parse("2026-09-24-01-x").is_err());
    }

    // Task: T02
    #[test]
    fn work_name_accepts_han_extension_f_h_i_and_compat_supplement() {
        // 协议 work start 第 3 步列出的全部区段各取一个码点。
        for c in [
            '\u{2CEB0}',
            '\u{31350}',
            '\u{2EBF0}',
            '\u{2F800}',
            '\u{20000}',
            '\u{FA0E}',
        ] {
            assert!(
                WorkName::normalize(&c.to_string()).is_ok(),
                "{c:?} 应当合法"
            );
        }
        // 部首补充区不在允许列表里。
        assert!(WorkName::normalize("\u{2E80}").is_err());
    }

    // Task: T02
    #[test]
    fn kebab_id_error_field_is_snake_case() {
        assert!(
            matches!(WorkbookId::new("Bad"), Err(Error::InvalidId { field, .. }) if field == "workbook_id")
        );
        assert!(
            matches!(FlowId::new("Bad"), Err(Error::InvalidId { field, .. }) if field == "flow_id")
        );
        assert!(
            matches!(NodeId::new("Bad"), Err(Error::InvalidId { field, .. }) if field == "node_id")
        );
    }

    // Task: T02
    #[test]
    fn attempt_id_parse_error_field_is_attempt_id() {
        match AttemptId::parse("Bad#1.0") {
            Err(Error::InvalidId { field, .. }) => assert_eq!(field, "attempt_id"),
            other => panic!("应是 InvalidId{{ field: attempt_id }}，实际 {other:?}"),
        }
    }

    // Task: T02
    #[test]
    fn work_id_rejects_seq_zero_or_over_999() {
        assert!(WorkId::new("2026-09-24", 0, &name("x")).is_err());
        assert!(WorkId::new("2026-09-24", 1000, &name("x")).is_err());
        assert!(WorkId::new("20260924", 1, &name("x")).is_err());
    }

    // ── M1 补测（边界与格式，杀死存活的突变体） ─────────────────────

    // Task: T02
    #[test]
    fn id_accepts_exactly_64_bytes() {
        let at = "a".repeat(64);
        assert_eq!(WorkbookId::new(&at).unwrap().as_str(), at);
    }

    // Task: T02
    #[test]
    fn work_name_accepts_exactly_48_bytes() {
        let at = "a".repeat(48);
        assert_eq!(name(&at).as_str(), at);
    }

    // Task: T02
    #[test]
    fn work_name_rejects_leading_or_trailing_dash_alone() {
        for bad in ["-abc", "abc-"] {
            assert!(
                matches!(WorkName::normalize(bad), Err(Error::InvalidId { .. })),
                "{bad:?}"
            );
        }
    }

    // Task: T02
    #[test]
    fn work_id_parse_rejects_each_misplaced_separator() {
        for bad in [
            "2026x09-24-001-a",
            "2026-09x24-001-a",
            "2026-09-24x001-a",
            "2026-09-24-001xa",
        ] {
            assert!(WorkId::parse(bad).is_err(), "{bad:?}");
        }
    }

    // Task: T02
    #[test]
    fn names_and_work_ids_convert_into_string() {
        assert_eq!(String::from(name("文章-初稿")), "文章-初稿");
        let id = WorkId::new("2026-09-24", 3, &name("a")).unwrap();
        assert_eq!(String::from(id), "2026-09-24-003-a");
    }

    // Task: T02
    #[test]
    fn work_name_try_from_and_serde_reject_non_canonical() {
        assert!(WorkName::try_from("hello".to_string()).is_ok());
        assert!(WorkName::try_from("文章-初稿".to_string()).is_ok());
        // 规范化会改写的形式读取时拒绝，不悄悄改名。
        assert!(WorkName::try_from("Hello".to_string()).is_err());
        assert!(WorkName::try_from("two  words".to_string()).is_err());
        assert!(serde_json::from_str::<WorkName>("\"文章 初稿\"").is_err());
        assert!(serde_json::from_str::<WorkName>("\"文章-初稿\"").is_ok());
    }

    // Task: C002-T39
    #[test]
    fn attempt_id_json_keeps_its_shape_and_rejects_unknown_fields_in_any_position() {
        let valid = r#"{"node":"draft","occurrence":2,"retry":1}"#;
        let attempt: AttemptId = serde_json::from_str(valid).unwrap();
        assert_eq!(attempt, AttemptId::new(NodeId::new("draft").unwrap(), 2, 1));
        assert_eq!(serde_json::to_string(&attempt).unwrap(), valid);
        for invalid in [
            r#"{"unexpected":true,"node":"draft","occurrence":2,"retry":1}"#,
            r#"{"node":"draft","unexpected":true,"occurrence":2,"retry":1}"#,
            r#"{"node":"draft","occurrence":2,"retry":1,"unexpected":true}"#,
        ] {
            assert!(
                serde_json::from_str::<AttemptId>(invalid).is_err(),
                "{invalid}"
            );
        }
    }
}
