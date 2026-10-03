use serde::{Deserialize, Serialize};
use sheltie_core::digest::Sha256Hex;
use sheltie_core::ids::{AttemptId, FlowId, WorkId};
use sheltie_core::path::AbsPath;
use sheltie_core::work::{WorkStatus, WorkbookRef};

use crate::{Error, Result};

pub const MAX_METADATA_BYTES: u64 = 1_048_576;
pub const MAX_FILE_BYTES: u64 = 33_554_432;
pub const MAX_TOTAL_BYTES: u64 = 268_435_456;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ResultKey(String);

impl TryFrom<String> for ResultKey {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        if value.as_bytes().contains(&0) {
            return Err("result key包含argv不能表达的NUL".to_string());
        }
        Ok(Self(value))
    }
}

impl ResultKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotKind {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSource {
    pub attempt: String,
    pub kind: SlotKind,
    pub name: ResultKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub key: ResultKey,
    pub path: AbsPath,
    pub sha256: Sha256Hex,
    pub bytes: u64,
    pub source: ArtifactSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedResult {
    pub format: String,
    pub work_id: WorkId,
    pub revision: u64,
    pub workbook: WorkbookRef,
    pub flow: FlowId,
    #[serde(deserialize_with = "strict_result_status")]
    pub status: WorkStatus,
    pub effects_pending: bool,
    pub r#final: bool,
    pub artifacts: Vec<Artifact>,
}

impl SelectedResult {
    pub fn validate(&self, expected_work: &WorkId) -> Result<()> {
        let reject = |message: &str| Error::Rejected {
            code: "INVALID_RESULT",
            message: message.to_string(),
        };
        if self.format != "work-result/v1" || self.work_id != *expected_work || self.revision == 0 {
            return Err(reject("结果格式、WorkId或revision不符"));
        }
        if self.status != WorkStatus::Succeeded
            || !self.r#final
            || self.effects_pending
            || self.artifacts.is_empty()
        {
            return Err(reject("需要无待完成效果且非空的明确最终成果"));
        }
        let version = &self.workbook.version;
        if version.is_empty()
            || version.len() > 32
            || matches!(version.as_str(), "." | ".." | ".staging")
            || version
                .bytes()
                .any(|byte| !byte.is_ascii_alphanumeric() && !b".+-".contains(&byte))
        {
            return Err(reject("Workbook版本不是合同规定的安全目录段"));
        }
        if self
            .artifacts
            .windows(2)
            .any(|pair| pair[0].key >= pair[1].key)
        {
            return Err(reject("Artifact key必须严格排序且唯一"));
        }
        let mut total = 0_u64;
        let terminal = &self.artifacts[0].source.attempt;
        for artifact in &self.artifacts {
            let leaf = std::path::Path::new(artifact.path.as_str())
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| reject("Artifact没有安全文件叶名"))?;
            if leaf.is_empty() || matches!(leaf, "." | "..") || leaf.contains(['\0', '/', '\\']) {
                return Err(reject("Artifact文件叶名不安全"));
            }
            let attempt = AttemptId::parse(&artifact.source.attempt)
                .map_err(|_| reject("来源AttemptId非法"))?;
            if attempt.occurrence == 0
                || artifact.source.attempt != *terminal
                || attempt.to_string() != artifact.source.attempt
                || artifact.source.name != artifact.key
            {
                return Err(reject("来源槽或AttemptId非规范"));
            }
            if artifact.bytes > MAX_FILE_BYTES {
                return Err(reject("选定单文件超过32MiB"));
            }
            total = total
                .checked_add(artifact.bytes)
                .ok_or_else(|| reject("成果总量溢出"))?;
            if total > MAX_TOTAL_BYTES {
                return Err(reject("选定成果总量超过256MiB"));
            }
        }
        Ok(())
    }
}

fn strict_result_status<'de, D>(deserializer: D) -> std::result::Result<WorkStatus, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct StatusDto {
        kind: String,
        #[serde(default, deserialize_with = "present_reason")]
        reason: Option<Option<sheltie_core::work::BlockedReason>>,
    }
    let dto = StatusDto::deserialize(deserializer)?;
    match (dto.kind.as_str(), dto.reason) {
        ("active", None) => Ok(WorkStatus::Active),
        ("succeeded", None) => Ok(WorkStatus::Succeeded),
        ("cancelled", None) => Ok(WorkStatus::Cancelled),
        ("blocked", Some(Some(reason))) => Ok(WorkStatus::Blocked(reason)),
        _ => Err(D::Error::custom("status字段与variant不相符")),
    }
}

fn present_reason<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Option<sheltie_core::work::BlockedReason>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopiedFile {
    pub key: ResultKey,
    pub path: sheltie_core::path::RelPath,
    pub sha256: Sha256Hex,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub result: SelectedResult,
    pub files: Vec<CopiedFile>,
}
