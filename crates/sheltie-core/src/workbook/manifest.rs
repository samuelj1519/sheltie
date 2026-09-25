//! `workbook.toml` 的 DTO 与领域类型。

use serde::{Deserialize, Serialize};

use crate::digest::Sha256Hex;
use crate::error::{Error, Result};
use crate::ids::{WorkbookId, validate_id};
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
///
/// 校验失败一律归到 `Error::WorkbookInvalid`，`field` 用 TOML 路径写法，
/// 让协调者能直接指着 `workbook.toml` 的某一行改。
fn convert(dto: ManifestDto) -> Result<Manifest> {
    let invalid = |field: String, reason: String| Error::WorkbookInvalid { field, reason };

    if dto.schema != "workbook/v1" {
        return Err(invalid(
            "schema".to_string(),
            format!("必须是 workbook/v1，实际 {:?}", dto.schema),
        ));
    }

    let id = WorkbookId::new(&dto.id).map_err(|e| invalid("id".to_string(), e.to_string()))?;

    if dto.version.is_empty() {
        return Err(invalid("version".to_string(), "不能为空".to_string()));
    }
    if dto.version.len() > VERSION_MAX_BYTES {
        return Err(invalid(
            "version".to_string(),
            format!("超过 {VERSION_MAX_BYTES} 字节"),
        ));
    }
    if !dto
        .version
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'+' | b'-'))
    {
        return Err(invalid(
            "version".to_string(),
            "只能含 0-9、A-Z、a-z、.、+ 与 -".to_string(),
        ));
    }

    if dto.name.is_empty() {
        return Err(invalid("name".to_string(), "不能为空".to_string()));
    }
    if dto.name.len() > NAME_MAX_BYTES {
        return Err(invalid(
            "name".to_string(),
            format!("超过 {NAME_MAX_BYTES} 字节"),
        ));
    }

    if let Some(description) = &dto.description {
        if description.len() > DESCRIPTION_MAX_BYTES {
            return Err(invalid(
                "description".to_string(),
                format!("超过 {DESCRIPTION_MAX_BYTES} 字节"),
            ));
        }
    }

    if dto.flows.is_empty() {
        return Err(invalid("flows".to_string(), "不能为空".to_string()));
    }
    let mut flows = Vec::with_capacity(dto.flows.len());
    for (i, raw) in dto.flows.iter().enumerate() {
        let path = RelPath::new(raw).map_err(|e| invalid(format!("flows[{i}]"), e.to_string()))?;
        flows.push(path);
    }

    if dto.requires.len() > REQUIRES_MAX {
        return Err(invalid(
            "requires".to_string(),
            format!("最多 {REQUIRES_MAX} 项"),
        ));
    }
    let mut requires = Vec::with_capacity(dto.requires.len());
    for (i, req) in dto.requires.into_iter().enumerate() {
        // 字段级错误用 requires[i].<字段>；整项错误（重复）用 requires[i]。
        let field = |suffix: &str| format!("requires[{i}].{suffix}");
        let kind = RequireKind::parse(&req.kind).ok_or_else(|| {
            invalid(
                field("kind"),
                format!("{:?} 不是 skill、agent 或 mcp", req.kind),
            )
        })?;
        validate_id(&req.name, "name").map_err(|e| invalid(field("name"), e.to_string()))?;
        if let Some(version) = &req.version {
            if version.len() > VERSION_MAX_BYTES {
                return Err(invalid(
                    field("version"),
                    format!("超过 {VERSION_MAX_BYTES} 字节"),
                ));
            }
        }
        let digest = match &req.digest {
            None => None,
            Some(raw) => {
                let hex = raw
                    .strip_prefix("sha256:")
                    .ok_or_else(|| invalid(field("digest"), "必须以 sha256: 开头".to_string()))?;
                Some(Sha256Hex::new(hex).map_err(|_| {
                    invalid(
                        field("digest"),
                        "sha256: 之后必须是 64 位小写十六进制".to_string(),
                    )
                })?)
            }
        };
        if let Some(source) = &req.source {
            if source.len() > SOURCE_MAX_BYTES {
                return Err(invalid(
                    field("source"),
                    format!("超过 {SOURCE_MAX_BYTES} 字节"),
                ));
            }
        }
        if requires
            .iter()
            .any(|r: &HostRequire| r.kind == kind && r.name == req.name)
        {
            return Err(invalid(
                format!("requires[{i}]"),
                "kind 与 name 的组合重复".to_string(),
            ));
        }
        requires.push(HostRequire {
            kind,
            name: req.name,
            version: req.version,
            digest,
            source: req.source,
        });
    }

    Ok(Manifest {
        id,
        version: dto.version,
        name: dto.name,
        description: dto.description,
        flows,
        requires,
    })
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

    // Task: T03
    #[test]
    fn parses_minimal_manifest() {
        let m = parse_manifest(MINIMAL).unwrap();
        assert_eq!(m.id.as_str(), "two-step");
        assert_eq!(m.version, "1.0.0");
        assert_eq!(m.flows.len(), 1);
        assert!(m.requires.is_empty());
        assert_eq!(m.description, None);
    }

    // Task: T03
    #[test]
    fn rejects_unknown_field() {
        let err = parse_manifest(&with("author = \"x\"")).unwrap_err();
        assert!(matches!(err, Error::WorkbookInvalid { .. }));
    }

    // Task: T03
    #[test]
    fn rejects_wrong_schema_string() {
        let text = MINIMAL.replace("workbook/v1", "workbook/v2");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "schema")
        );
    }

    // Task: T03
    #[test]
    fn rejects_empty_flows() {
        let text = MINIMAL.replace("[\"flows/default.toml\"]", "[]");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "flows")
        );
    }

    // Task: T03
    #[test]
    fn rejects_flow_path_with_dotdot() {
        let text = MINIMAL.replace("flows/default.toml", "../x.toml");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "flows[0]")
        );
    }

    // Task: T03
    #[test]
    fn rejects_description_over_2kib() {
        let text = with(&format!("description = \"{}\"", "x".repeat(2049)));
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "description")
        );
    }

    // Task: T03
    #[test]
    fn parses_requires_with_optional_fields() {
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

    // Task: T03
    #[test]
    fn rejects_duplicate_require_kind_name() {
        let text = with(
            "[[requires]]\nkind = \"skill\"\nname = \"a\"\n[[requires]]\nkind = \"skill\"\nname = \"a\"\n",
        );
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "requires[1]")
        );
    }

    // Task: T03
    #[test]
    fn rejects_unknown_require_kind() {
        let text = with("[[requires]]\nkind = \"plugin\"\nname = \"a\"\n");
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "requires[0].kind")
        );
    }

    // Task: T03
    #[test]
    fn rejects_require_digest_without_sha256_prefix() {
        let text = with(
            "[[requires]]\nkind = \"skill\"\nname = \"a\"\ndigest = \"5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03\"\n",
        );
        assert!(
            matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "requires[0].digest")
        );
    }

    // ── M1 补测（长度与个数上限：恰好上限接受，多一个字节拒绝） ─────

    fn require_toml(kind: &str, name: &str, extra: &str) -> String {
        format!("[[requires]]\nkind = \"{kind}\"\nname = \"{name}\"\n{extra}\n")
    }

    fn field_of(text: &str) -> String {
        match parse_manifest(text) {
            Err(Error::WorkbookInvalid { field, .. }) => field,
            other => panic!("应报 WorkbookInvalid：{other:?}"),
        }
    }

    // Task: T03
    #[test]
    fn version_limit_is_32_bytes() {
        let at = MINIMAL.replace("\"1.0.0\"", &format!("\"{}\"", "1".repeat(32)));
        assert!(parse_manifest(&at).is_ok());
        let over = MINIMAL.replace("\"1.0.0\"", &format!("\"{}\"", "1".repeat(33)));
        assert_eq!(field_of(&over), "version");
    }

    // Task: T03
    #[test]
    fn name_limit_is_128_bytes() {
        let at = MINIMAL.replace("\"两步\"", &format!("\"{}\"", "a".repeat(128)));
        assert!(parse_manifest(&at).is_ok());
        let over = MINIMAL.replace("\"两步\"", &format!("\"{}\"", "a".repeat(129)));
        assert_eq!(field_of(&over), "name");
    }

    // Task: T03
    #[test]
    fn description_accepts_exactly_2048_bytes() {
        let at = with(&format!("description = \"{}\"", "a".repeat(2048)));
        assert_eq!(
            parse_manifest(&at).unwrap().description.unwrap().len(),
            2048
        );
    }

    // Task: T03
    #[test]
    fn requires_limit_is_32_items() {
        let reqs = |n: usize| {
            (0..n)
                .map(|i| require_toml("skill", &format!("s{i}"), ""))
                .collect::<String>()
        };
        assert_eq!(parse_manifest(&with(&reqs(32))).unwrap().requires.len(), 32);
        assert_eq!(field_of(&with(&reqs(33))), "requires");
    }

    // Task: T03
    #[test]
    fn require_version_limit_is_32_bytes() {
        let v = |n: usize| format!("version = \"{}\"", "1".repeat(n));
        assert!(parse_manifest(&with(&require_toml("skill", "s", &v(32)))).is_ok());
        assert_eq!(
            field_of(&with(&require_toml("skill", "s", &v(33)))),
            "requires[0].version"
        );
    }

    // Task: T03
    #[test]
    fn require_source_limit_is_512_bytes() {
        let s = |n: usize| format!("source = \"{}\"", "a".repeat(n));
        assert!(parse_manifest(&with(&require_toml("skill", "s", &s(512)))).is_ok());
        assert_eq!(
            field_of(&with(&require_toml("skill", "s", &s(513)))),
            "requires[0].source"
        );
    }

    // Task: T03
    #[test]
    fn same_kind_different_names_are_not_duplicates() {
        let text = with(&format!(
            "{}{}{}",
            require_toml("agent", "a", ""),
            require_toml("agent", "b", ""),
            require_toml("mcp", "a", "")
        ));
        let m = parse_manifest(&text).unwrap();
        assert_eq!(m.requires.len(), 3);
        assert_eq!(m.requires[0].kind, RequireKind::Agent);
        assert!(m.find_require(RequireKind::Agent, "b").is_some());
        assert!(m.find_require(RequireKind::Skill, "a").is_none());
        assert!(m.find_require(RequireKind::Agent, "c").is_none());
    }
}
