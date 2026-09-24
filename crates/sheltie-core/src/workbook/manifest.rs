//! `workbook.toml` 的 DTO 与领域类型。

use serde::{Deserialize, Serialize};

use crate::digest::Sha256Hex;
use crate::error::{Error, Result};
use crate::ids::WorkbookId;
use crate::path::RelPath;

/// 宿主资源类型（合同 §2 `requires[].kind`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequireKind {
    Skill,
    Agent,
    Mcp,
}

impl RequireKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Agent => "agent",
            Self::Mcp => "mcp",
        }
    }

    /// 解析 `skill` / `agent` / `mcp`。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "skill" => Some(Self::Skill),
            "agent" => Some(Self::Agent),
            "mcp" => Some(Self::Mcp),
            _ => None,
        }
    }
}

/// 一条宿主资源声明。身份是 `kind + name`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRequire {
    pub kind: RequireKind,
    pub name: String,
    pub version: Option<String>,
    pub digest: Option<Sha256Hex>,
    pub source: Option<String>,
}

/// 校验过的 manifest。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub id: WorkbookId,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub flows: Vec<RelPath>,
    pub requires: Vec<HostRequire>,
}

impl Manifest {
    /// 按 `kind:name` 查一条声明。
    pub fn find_require(&self, kind: RequireKind, name: &str) -> Option<&HostRequire> {
        self.requires
            .iter()
            .find(|r| r.kind == kind && r.name == name)
    }
}

/// 版本串允许的字符：`[0-9A-Za-z.+-]`，≤ 32 字节。
pub const VERSION_MAX_BYTES: usize = 32;
/// `name` ≤ 128 字节。
pub const NAME_MAX_BYTES: usize = 128;
/// `description` ≤ 2 KiB。
pub const DESCRIPTION_MAX_BYTES: usize = 2048;
/// `requires` ≤ 32 项。
pub const REQUIRES_MAX: usize = 32;
/// `source` ≤ 512 字节。
pub const SOURCE_MAX_BYTES: usize = 512;

/// 原始 TOML 形状。未知字段拒绝。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDto {
    schema: String,
    id: String,
    version: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    flows: Vec<String>,
    #[serde(default)]
    requires: Vec<RequireDto>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequireDto {
    kind: String,
    name: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    digest: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

/// 解析并校验 `workbook.toml`。
///
/// 任何 TOML 语法错误或未知字段返回 `Error::WorkbookInvalid { field: "toml", .. }`；
/// 字段级错误的 `field` 用路径写法，如 `flows[1]`、`requires[0].digest`。
/// 规则见合同 §2 两张表：`schema` 必须是 `workbook/v1`；`flows` 非空；
/// `requires` 的 `(kind, name)` 唯一，`digest` 必须是 `sha256:` 加 64 位小写十六进制。
pub fn parse_manifest(toml_text: &str) -> Result<Manifest> {
    let dto: ManifestDto = toml::from_str(toml_text).map_err(|e| Error::WorkbookInvalid {
        field: "toml".to_string(),
        reason: e.to_string(),
    })?;
    convert(dto)
}

/// DTO 到领域类型的逐字段转换。这是 T03 要填的函数。
#[allow(unused_variables)]
fn convert(dto: ManifestDto) -> Result<Manifest> {
    todo!("T03")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"
schema = "workbook/v1"
id = "two-step"
version = "1.0.0"
name = "两步"
flows = ["flows/default.toml"]
"#;

    fn with(extra: &str) -> String {
        format!("{MINIMAL}\n{extra}")
    }

    #[test]
    #[ignore = "T03"]
    fn t03_parses_minimal_manifest() {
        let m = parse_manifest(MINIMAL).unwrap();
        assert_eq!(m.id.as_str(), "two-step");
        assert_eq!(m.version, "1.0.0");
        assert_eq!(m.flows.len(), 1);
        assert!(m.requires.is_empty());
        assert_eq!(m.description, None);
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_unknown_field() {
        let err = parse_manifest(&with("author = \"x\"")).unwrap_err();
        assert!(matches!(err, Error::WorkbookInvalid { .. }));
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_wrong_schema_string() {
        let text = MINIMAL.replace("workbook/v1", "workbook/v2");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "schema")
        );
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_empty_flows() {
        let text = MINIMAL.replace("[\"flows/default.toml\"]", "[]");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "flows")
        );
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_flow_path_with_dotdot() {
        let text = MINIMAL.replace("flows/default.toml", "../x.toml");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "flows[0]")
        );
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_description_over_2kib() {
        let text = with(&format!("description = \"{}\"", "x".repeat(2049)));
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "description")
        );
    }

    #[test]
    #[ignore = "T03"]
    fn t03_parses_requires_with_optional_fields() {
        let text = with(
            r#"
[[requires]]
kind = "skill"
name = "company-api"
version = "^1"
digest = "sha256:5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
source = "https://example.com/skills/company-api"

[[requires]]
kind = "mcp"
name = "db"
"#,
        );
        let m = parse_manifest(&text).unwrap();
        assert_eq!(m.requires.len(), 2);
        assert_eq!(m.requires[0].kind, RequireKind::Skill);
        assert_eq!(m.requires[0].version.as_deref(), Some("^1"));
        assert!(m.requires[0].digest.is_some());
        assert_eq!(m.requires[1].kind, RequireKind::Mcp);
        assert_eq!(m.requires[1].digest, None);
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_duplicate_require_kind_name() {
        let text = with(
            "[[requires]]\nkind = \"skill\"\nname = \"a\"\n[[requires]]\nkind = \"skill\"\nname = \"a\"\n",
        );
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "requires[1]")
        );
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_unknown_require_kind() {
        let text = with("[[requires]]\nkind = \"plugin\"\nname = \"a\"\n");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "requires[0].kind")
        );
    }

    #[test]
    #[ignore = "T03"]
    fn t03_rejects_require_digest_without_sha256_prefix() {
        let text = with(
            "[[requires]]\nkind = \"skill\"\nname = \"a\"\ndigest = \"5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03\"\n",
        );
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "requires[0].digest")
        );
    }
}
