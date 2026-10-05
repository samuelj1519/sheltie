//! workbook.toml DTO and domain types.

use serde::{Deserialize, Serialize};

use crate::digest::Sha256Hex;
use crate::error::{Error, Result};
use crate::ids::{WorkbookId, validate_id};
use crate::path::RelPath;

/// Host resource kind (contract §2 requires[].kind).
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

    /// Parse skill / agent / mcp.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "skill" => Some(Self::Skill),
            "agent" => Some(Self::Agent),
            "mcp" => Some(Self::Mcp),
            _ => None,
        }
    }
}

/// Host resource declaration, identified by kind + name.
/// Deserialize Reply snapshots while retaining manifest field validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostRequire {
    pub(crate) kind: RequireKind,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) digest: Option<Sha256Hex>,
    pub(crate) source: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostRequireWire {
    kind: RequireKind,
    name: String,
    version: Option<String>,
    digest: Option<Sha256Hex>,
    source: Option<String>,
}

impl<'de> Deserialize<'de> for HostRequire {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = HostRequireWire::deserialize(deserializer)?;
        validate_id(&wire.name, "name").map_err(serde::de::Error::custom)?;
        if wire
            .version
            .as_ref()
            .is_some_and(|value| value.len() > VERSION_MAX_BYTES)
        {
            return Err(serde::de::Error::custom(
                "requires.version exceeds 32 bytes",
            ));
        }
        if wire
            .source
            .as_ref()
            .is_some_and(|value| value.len() > SOURCE_MAX_BYTES)
        {
            return Err(serde::de::Error::custom(
                "requires.source exceeds 512 bytes",
            ));
        }
        Ok(Self {
            kind: wire.kind,
            name: wire.name,
            version: wire.version,
            digest: wire.digest,
            source: wire.source,
        })
    }
}

impl HostRequire {
    pub fn kind(&self) -> RequireKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Validated manifest; external callers can read but not mutate parsed definitions.
///
/// ```compile_fail
/// use sheltie_core::workbook::parse_manifest;
/// let mut manifest = parse_manifest("schema = \"workbook/v1\"\nid = \"x\"\nversion = \"1.0.0\"\nname = \"x\"\nflows = [\"flows/f.toml\"]").unwrap();
/// manifest.flows.clear();
/// ```
///
/// ```compile_fail
/// use sheltie_core::workbook::Manifest;
/// let _: Manifest = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Manifest {
    pub(crate) id: WorkbookId,
    pub(crate) version: String,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) flows: Vec<RelPath>,
    pub(crate) requires: Vec<HostRequire>,
}

impl Manifest {
    pub fn id(&self) -> &WorkbookId {
        &self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn flows(&self) -> &[RelPath] {
        &self.flows
    }

    pub fn requires(&self) -> &[HostRequire] {
        &self.requires
    }

    /// Look up a declaration by kind:name.
    pub fn find_require(&self, kind: RequireKind, name: &str) -> Option<&HostRequire> {
        self.requires
            .iter()
            .find(|r| r.kind == kind && r.name == name)
    }
}

/// Version syntax: [0-9A-Za-z.+-], at most 32 bytes.
pub const VERSION_MAX_BYTES: usize = 32;
/// name is at most 128 bytes.
pub const NAME_MAX_BYTES: usize = 128;
/// `description` ≤ 2 KiB。
pub const DESCRIPTION_MAX_BYTES: usize = 2048;
/// requires has at most 32 entries.
pub const REQUIRES_MAX: usize = 32;
/// source is at most 512 bytes.
pub const SOURCE_MAX_BYTES: usize = 512;
/// Reserved internal version name: engine staging directory segment (storage contract §5.3).
pub const RESERVED_VERSION: &str = ".staging";

/// Raw TOML structure; reject unknown fields.
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

/// Parse and validate workbook.toml.
///
/// TOML syntax errors or unknown fields return Error::WorkbookInvalid { field: "toml", .. };
/// field-level errors use paths such as flows[1] and requires[0].digest.
/// Contract §2 tables require workbook/v1 schema and nonempty flows;
/// unique (kind, name) requires, and digest as sha256: followed by 64 lowercase hexadecimal digits.
pub fn parse_manifest(toml_text: &str) -> Result<Manifest> {
    let dto: ManifestDto = toml::from_str(toml_text).map_err(|e| Error::WorkbookInvalid {
        field: "toml".to_string(),
        reason: e.to_string(),
    })?;
    convert(dto)
}

/// Convert DTO fields into domain types (T03 implementation entry).
///
/// All validation failures use WorkbookInvalid with TOML field paths,
/// so coordinators can locate the workbook.toml line to change.
fn convert(dto: ManifestDto) -> Result<Manifest> {
    let invalid = |field: String, reason: String| Error::WorkbookInvalid { field, reason };

    if dto.schema != "workbook/v1" {
        return Err(invalid(
            "schema".to_string(),
            format!("Must be workbook/v1; actual {:?}", dto.schema),
        ));
    }

    let id = WorkbookId::new(&dto.id).map_err(|e| invalid("id".to_string(), e.to_string()))?;

    if dto.version.is_empty() {
        return Err(invalid(
            "version".to_string(),
            "Must not be empty".to_string(),
        ));
    }
    if dto.version.len() > VERSION_MAX_BYTES {
        return Err(invalid(
            "version".to_string(),
            format!("Exceeds {VERSION_MAX_BYTES} bytes"),
        ));
    }
    if !dto
        .version
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'+' | b'-'))
    {
        return Err(invalid(
            "version".to_string(),
            "May contain only 0-9, A-Z, a-z, ., +, and -".to_string(),
        ));
    }
    // version is one safe directory segment (storage contract §5.3); reject dot segments and reserved names.
    if matches!(dto.version.as_str(), "." | ".." | RESERVED_VERSION) {
        return Err(invalid(
            "version".to_string(),
            format!("Must not be ., .., or reserved name {RESERVED_VERSION}"),
        ));
    }

    if dto.name.is_empty() {
        return Err(invalid("name".to_string(), "Must not be empty".to_string()));
    }
    if dto.name.len() > NAME_MAX_BYTES {
        return Err(invalid(
            "name".to_string(),
            format!("Exceeds {NAME_MAX_BYTES} bytes"),
        ));
    }

    if let Some(description) = &dto.description {
        if description.len() > DESCRIPTION_MAX_BYTES {
            return Err(invalid(
                "description".to_string(),
                format!("Exceeds {DESCRIPTION_MAX_BYTES} bytes"),
            ));
        }
    }

    if dto.flows.is_empty() {
        return Err(invalid(
            "flows".to_string(),
            "Must not be empty".to_string(),
        ));
    }
    let mut flows = Vec::with_capacity(dto.flows.len());
    for (i, raw) in dto.flows.iter().enumerate() {
        let path = RelPath::new(raw).map_err(|e| invalid(format!("flows[{i}]"), e.to_string()))?;
        flows.push(path);
    }

    if dto.requires.len() > REQUIRES_MAX {
        return Err(invalid(
            "requires".to_string(),
            format!("At most {REQUIRES_MAX} entries"),
        ));
    }
    let mut requires = Vec::with_capacity(dto.requires.len());
    for (i, req) in dto.requires.into_iter().enumerate() {
        // Field errors use requires[i].<field>; whole-entry duplicates use requires[i].
        let field = |suffix: &str| format!("requires[{i}].{suffix}");
        let kind = RequireKind::parse(&req.kind).ok_or_else(|| {
            invalid(
                field("kind"),
                format!("{:?} must be skill, agent, or mcp", req.kind),
            )
        })?;
        validate_id(&req.name, "name").map_err(|e| invalid(field("name"), e.to_string()))?;
        if let Some(version) = &req.version {
            if version.len() > VERSION_MAX_BYTES {
                return Err(invalid(
                    field("version"),
                    format!("Exceeds {VERSION_MAX_BYTES} bytes"),
                ));
            }
        }
        let digest = match &req.digest {
            None => None,
            Some(raw) => {
                let hex = raw.strip_prefix("sha256:").ok_or_else(|| {
                    invalid(field("digest"), "Must start with sha256:".to_string())
                })?;
                Some(Sha256Hex::new(hex).map_err(|_| {
                    invalid(
                        field("digest"),
                        "sha256: must be followed by 64 lowercase hexadecimal digits".to_string(),
                    )
                })?)
            }
        };
        if let Some(source) = &req.source {
            if source.len() > SOURCE_MAX_BYTES {
                return Err(invalid(
                    field("source"),
                    format!("Exceeds {SOURCE_MAX_BYTES} bytes"),
                ));
            }
        }
        if requires
            .iter()
            .any(|r: &HostRequire| r.kind == kind && r.name == req.name)
        {
            return Err(invalid(
                format!("requires[{i}]"),
                "Duplicate kind/name pair".to_string(),
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

    // Task: C002-T40
    #[test]
    fn host_require_snapshot_decode_accepts_valid_fields() {
        let valid = serde_json::json!({
            "kind": "skill",
            "name": "company-api",
            "version": "^1",
            "digest": "0".repeat(64),
            "source": "https://example.com/skill"
        });
        let value: HostRequire = serde_json::from_value(valid.clone()).unwrap();
        assert_eq!(value.kind(), RequireKind::Skill);
        assert_eq!(value.name(), "company-api");
        assert_eq!(serde_json::to_value(value).unwrap(), valid);
        for (field, bad) in [
            ("name", serde_json::json!("Bad Name")),
            ("digest", serde_json::json!("not-a-digest")),
        ] {
            let mut changed = valid.clone();
            changed[field] = bad;
            assert!(
                serde_json::from_value::<HostRequire>(changed).is_err(),
                "{field} must be checked"
            );
        }
    }

    const MINIMAL: &str = r#"
schema = "workbook/v1"
id = "two-step"
version = "1.0.0"
name = "Two steps"
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

    // Task: C002-T40
    #[test]
    fn rejects_unknown_field() {
        let text = format!("{}\nauthor = \"x\"\n", crate::testkit::TWO_STEP_MANIFEST);
        let err = parse_manifest(&text).unwrap_err();
        assert_eq!(err.code(), crate::error::ErrorCode::WorkbookInvalid);
        assert!(matches!(err, Error::WorkbookInvalid { field, .. } if field == "toml"));
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
        let at = with(&format!("description = \"{}\"", "a".repeat(2048)));
        assert_eq!(
            parse_manifest(&at).unwrap().description.unwrap().len(),
            2048
        );
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

    // ── M1 additional length/count limits: accept exactly the limit, reject one more ─────

    fn require_toml(kind: &str, name: &str, extra: &str) -> String {
        format!("[[requires]]\nkind = \"{kind}\"\nname = \"{name}\"\n{extra}\n")
    }

    fn field_of(text: &str) -> String {
        match parse_manifest(text) {
            Err(Error::WorkbookInvalid { field, .. }) => field,
            other => panic!("Expected WorkbookInvalid, got {other:?}"),
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
        let at = MINIMAL.replace("\"Two steps\"", &format!("\"{}\"", "a".repeat(128)));
        assert!(parse_manifest(&at).is_ok());
        let over = MINIMAL.replace("\"Two steps\"", &format!("\"{}\"", "a".repeat(129)));
        assert_eq!(field_of(&over), "name");
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

    // Task: C002-T05
    #[test]
    fn version_rejects_dot_segments_and_reserved_name() {
        for bad in [".", "..", ".staging"] {
            let text = MINIMAL.replace("\"1.0.0\"", &format!("\"{bad}\""));
            assert!(
                matches!(parse_manifest(&text), Err(Error::WorkbookInvalid { field, .. }) if field == "version"),
                "{bad:?} should be rejected"
            );
        }
        // One changed condition: a valid dot-prefixed version is still accepted.
        assert!(parse_manifest(&MINIMAL.replace("\"1.0.0\"", "\".1.0\"")).is_ok());
    }
}
