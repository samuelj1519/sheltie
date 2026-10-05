//! Strongly typed IDs validate on construction; do not pass bare `String` values across module boundaries.
//!
//! ID syntax: `specs/contracts/workbook.md` §2, `^[a-z0-9]+(-[a-z0-9]+)*$`, at most 64 bytes.
//! For `work_id` and names, see `specs/contracts/protocol.md`, `work start` steps 3 and 4.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Validate a kebab-case ID; `field` is diagnostic context only.
///
/// Return `Error::InvalidId` on failure. Require nonempty text, at most 64 bytes, containing only `a-z0-9-`,
/// without leading, trailing, or consecutive `-`.
pub fn validate_id(value: &str, field: &str) -> Result<()> {
    let reject = |reason: &'static str| Error::InvalidId {
        field: field.to_string(),
        value: value.to_string(),
        reason,
    };
    if value.is_empty() {
        return Err(reject("Must not be empty"));
    }
    if value.len() > 64 {
        return Err(reject("Exceeds 64 bytes"));
    }
    if value.starts_with('-') || value.ends_with('-') {
        return Err(reject("Must not start or end with -"));
    }
    if value.contains("--") {
        return Err(reject("Must not contain consecutive -"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(reject("May contain only a-z, 0-9, and -"));
    }
    Ok(())
}

/// `YYYY-MM-DD` syntax: digit groups and two fixed-position hyphens, without calendar validation.
fn is_day_shape(day: &str) -> bool {
    let b = day.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..].iter().all(u8::is_ascii_digit)
}

/// Canonical decimal: `0` or `1-9` followed by digits; reject leading `+`, leading zeros, empty text, and overflow.
fn parse_u32_canonical(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if s.len() > 1 && s.starts_with('0') {
        return None;
    }
    s.parse().ok()
}

/// The twelve Han ranges in protocol `work start`, step 3; std has no script classifier, so test ranges.
fn is_han(c: char) -> bool {
    matches!(
        c,
        '\u{3400}'..='\u{4DBF}'      // Extension A
        | '\u{4E00}'..='\u{9FFF}'    // Basic range
        | '\u{F900}'..='\u{FAFF}'    // Compatibility
        | '\u{20000}'..='\u{2A6DF}'  // Extension B
        | '\u{2A700}'..='\u{2B73F}'  // Extension C
        | '\u{2B740}'..='\u{2B81F}'  // Extension D
        | '\u{2B820}'..='\u{2CEAF}'  // Extension E
        | '\u{2CEB0}'..='\u{2EBEF}'  // Extension F
        | '\u{2EBF0}'..='\u{2EE5F}'  // Extension I
        | '\u{2F800}'..='\u{2FA1F}'  // Compatibility supplement
        | '\u{30000}'..='\u{3134F}'  // Extension G
        | '\u{31350}'..='\u{323AF}'  // Extension H
    )
}

macro_rules! kebab_id {
    ($name:ident, $field:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// Validate on construction; return `Error::InvalidId` on failure.
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

kebab_id!(
    WorkbookId,
    "workbook_id",
    "Stable, globally unique Workbook identity."
);
kebab_id!(FlowId, "flow_id", "The ID of a graph within a Workbook.");
kebab_id!(
    NodeId,
    "node_id",
    "The ID of a node within a Flow; compile rule 1 rejects reserved `start` and `resource`."
);

/// The Work name, forming the suffix of `work_id`.
///
/// Normalization (protocol `work start`, step 3): trim, collapse whitespace to one `-`, and lowercase;
/// then allow lowercase letters, digits, Han, and single interior `-`, at most 48 bytes. Han means the twelve specified ranges.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WorkName(String);

impl WorkName {
    /// Maximum byte length.
    pub const MAX_BYTES: usize = 48;

    /// Normalize and validate; return `Error::InvalidId { field: "work_name", .. }` on failure.
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
            return Err(reject("Must not be empty"));
        }
        if value.len() > Self::MAX_BYTES {
            return Err(reject("Exceeds 48 bytes"));
        }
        if value.starts_with('-') || value.ends_with('-') {
            return Err(reject("Must not start or end with -"));
        }
        if value.contains("--") {
            return Err(reject("Must not contain consecutive -"));
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || is_han(c))
        {
            return Err(reject("May contain only a-z, 0-9, Han characters, and -"));
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
        // Reads (serde or storage roundtrips) require canonical names, without silently renaming corrupt data; same as `WorkId::parse`.
        let name = Self::normalize(&value)?;
        if name.0 != value {
            return Err(Error::InvalidId {
                field: "work_name".to_string(),
                value,
                reason: "Not in canonical form",
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

/// `work_id`, such as `2026-09-24-003-article-draft`: UTC date, three-digit daily sequence, and name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WorkId(String);

impl WorkId {
    /// Construct `work_id` from the date, sequence, and name.
    ///
    /// Require `YYYY-MM-DD` and `seq` in 1..=999; otherwise return `Error::InvalidId { field: "work_id", .. }`.
    pub fn new(day: &str, seq: u32, name: &WorkName) -> Result<Self> {
        let reject = |value: &str, reason: &'static str| Error::InvalidId {
            field: "work_id".to_string(),
            value: value.to_string(),
            reason,
        };
        if !is_day_shape(day) {
            return Err(reject(day, "Date must be YYYY-MM-DD"));
        }
        if !(1..=999).contains(&seq) {
            return Err(reject(&seq.to_string(), "Sequence must be in 1..=999"));
        }
        Ok(Self(format!("{day}-{seq:03}-{}", name.as_str())))
    }

    /// Parse a complete `work_id` string, including database reads.
    ///
    /// Require `YYYY-MM-DD-NNN-<name>` with a valid, already canonical `WorkName`.
    pub fn parse(value: &str) -> Result<Self> {
        let reject = |reason: &'static str| Error::InvalidId {
            field: "work_id".to_string(),
            value: value.to_string(),
            reason,
        };
        let day = value
            .get(..10)
            .ok_or_else(|| reject("Format must be YYYY-MM-DD-NNN-<name>"))?;
        let seq_str = value
            .get(11..14)
            .ok_or_else(|| reject("Format must be YYYY-MM-DD-NNN-<name>"))?;
        let name_str = value
            .get(15..)
            .ok_or_else(|| reject("Format must be YYYY-MM-DD-NNN-<name>"))?;
        if value.as_bytes().get(10) != Some(&b'-') || value.as_bytes().get(14) != Some(&b'-') {
            return Err(reject("Format must be YYYY-MM-DD-NNN-<name>"));
        }
        if !is_day_shape(day) {
            return Err(reject("Date must be YYYY-MM-DD"));
        }
        if !seq_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(reject("Sequence must contain exactly three digits"));
        }
        let seq: u32 = seq_str
            .parse()
            .map_err(|_| reject("Sequence must contain exactly three digits"))?;
        let name = WorkName::normalize(name_str)?;
        if name.as_str() != name_str {
            return Err(reject("Name is not in canonical form"));
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

/// Attempt ID: node, Occurrence number, and zero-based Attempt sequence; displayed as `draft#1.0`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptId {
    pub node: NodeId,
    pub occurrence: u32,
    pub number: u32,
}

impl AttemptId {
    pub fn new(node: NodeId, occurrence: u32, number: u32) -> Self {
        Self {
            node,
            occurrence,
            number,
        }
    }

    /// Parse `node#n.number`; return `Error::InvalidId { field: "attempt_id", .. }` on failure.
    pub fn parse(value: &str) -> Result<Self> {
        let reject = |reason: &'static str| Error::InvalidId {
            field: "attempt_id".to_string(),
            value: value.to_string(),
            reason,
        };
        let (node_str, rest) = value
            .split_once('#')
            .ok_or_else(|| reject("Format must be node#n.number"))?;
        let (occ_str, number_str) = rest
            .split_once('.')
            .ok_or_else(|| reject("Format must be node#n.number"))?;
        let node = NodeId::new(node_str).map_err(|e| match e {
            Error::InvalidId { reason, .. } => reject(reason),
            e => e,
        })?;
        let occurrence: u32 = parse_u32_canonical(occ_str)
            .ok_or_else(|| reject("Occurrence number must be canonical decimal"))?;
        let number: u32 = parse_u32_canonical(number_str)
            .ok_or_else(|| reject("Attempt sequence must be canonical decimal"))?;
        Ok(Self::new(node, occurrence, number))
    }
}

impl fmt::Display for AttemptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}.{}", self.node, self.occurrence, self.number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task: C005-T01
    #[test]
    fn attempt_id_requires_number_and_rejects_retired_retry_field() {
        let id: AttemptId =
            serde_json::from_str(r#"{"node":"draft","occurrence":2,"number":3}"#).unwrap();
        assert_eq!(id.to_string(), "draft#2.3");
        assert_eq!(serde_json::to_value(&id).unwrap()["number"], 3);
        assert!(
            serde_json::from_str::<AttemptId>(r#"{"node":"draft","occurrence":2,"retry":3}"#)
                .is_err()
        );
    }

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
        assert!(WorkName::normalize(&"文".repeat(17)).is_err(), "51 bytes");
        assert!(WorkName::normalize("a/b").is_err());
        assert!(WorkName::normalize("").is_err());
        assert!(WorkName::normalize("--").is_err());
    }

    // Task: T02
    #[test]
    fn work_id_parse_rejects_plus_sign_in_seq() {
        // str::parse::<u32> accepts leading +; reject it so parsing and rendering cannot change the original text.
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
        // "01".parse::<u32>() succeeds and would render draft#01.0 as draft#1.0; require canonical decimal.
        assert!(AttemptId::parse("draft#01.0").is_err());
        assert!(AttemptId::parse("draft#1.00").is_err());
        assert!(AttemptId::parse("draft#1.0").is_ok());
        assert!(
            AttemptId::parse("draft#0.0").is_ok(),
            "Occurrence and Attempt numbers both accept 0"
        );
        // The work_id sequence requires exactly three digits.
        assert!(WorkId::parse("2026-09-24-0001-x").is_err());
        assert!(WorkId::parse("2026-09-24-01-x").is_err());
    }

    // Task: T02
    #[test]
    fn work_name_accepts_han_extension_f_h_i_and_compat_supplement() {
        // Test one code point from every range in protocol work start, step 3.
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
                "{c:?} should be valid"
            );
        }
        // The radicals supplement range is not allowed.
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
            other => panic!("Expected InvalidId{{ field: attempt_id }}, got {other:?}"),
        }
    }

    // Task: T02
    #[test]
    fn work_id_rejects_seq_zero_or_over_999() {
        assert!(WorkId::new("2026-09-24", 0, &name("x")).is_err());
        assert!(WorkId::new("2026-09-24", 1000, &name("x")).is_err());
        assert!(WorkId::new("20260924", 1, &name("x")).is_err());
    }

    // ── M1 additional boundary and format coverage for surviving mutants ─────────────────────

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
        // Reject noncanonical forms on read rather than silently renaming them.
        assert!(WorkName::try_from("Hello".to_string()).is_err());
        assert!(WorkName::try_from("two  words".to_string()).is_err());
        assert!(serde_json::from_str::<WorkName>("\"文章 初稿\"").is_err());
        assert!(serde_json::from_str::<WorkName>("\"文章-初稿\"").is_ok());
    }

    // Task: C002-T39
    #[test]
    fn attempt_id_json_keeps_its_shape_and_rejects_unknown_fields_in_any_position() {
        let valid = r#"{"node":"draft","occurrence":2,"number":1}"#;
        let attempt: AttemptId = serde_json::from_str(valid).unwrap();
        assert_eq!(attempt, AttemptId::new(NodeId::new("draft").unwrap(), 2, 1));
        assert_eq!(serde_json::to_string(&attempt).unwrap(), valid);
        for invalid in [
            r#"{"unexpected":true,"node":"draft","occurrence":2,"number":1}"#,
            r#"{"node":"draft","unexpected":true,"occurrence":2,"number":1}"#,
            r#"{"node":"draft","occurrence":2,"number":1,"unexpected":true}"#,
        ] {
            assert!(
                serde_json::from_str::<AttemptId>(invalid).is_err(),
                "{invalid}"
            );
        }
    }
}
