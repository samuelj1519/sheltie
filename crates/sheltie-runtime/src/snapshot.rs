//! 已持久响应的数据形状校验；历史状态与当前业务归属分别校验。

use std::collections::BTreeMap;

use serde::{Deserialize, de::DeserializeOwned};
use sheltie_core::ids::{AttemptId, FlowId, NodeId, WorkId, WorkName};
use sheltie_core::path::AbsPath;
use sheltie_core::work::{
    ArtifactRef, Command, Principal, Reply, Timestamp, WorkState, WorkStatus, WorkbookRef,
};
use sheltie_core::workbook::HostRequire;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedResponse {
    pub(crate) request_id: String,
    pub(crate) revision: u64,
    pub(crate) replayed: bool,
    pub(crate) reply: Reply,
    pub(crate) data: serde_json::Value,
    pub(crate) next: Vec<sheltie_core::work::NextOp>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartedData {
    pub(crate) work_id: WorkId,
    pub(crate) name: WorkName,
    pub(crate) workbook: WorkbookRef,
    pub(crate) flow: FlowId,
    pub(crate) work_dir: AbsPath,
    pub(crate) requires: Vec<HostRequire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BegunData {
    #[serde(deserialize_with = "attempt_from_text")]
    attempt: AttemptId,
    node: NodeId,
    occurrence: u32,
    number: u32,
    brief_path: AbsPath,
    output_dir: AbsPath,
    inputs: BTreeMap<String, Option<AbsPath>>,
    outputs: BTreeMap<String, AbsPath>,
    requires: Vec<HostRequire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplacedData {
    #[serde(deserialize_with = "attempt_from_text")]
    replaced_attempt: AttemptId,
    #[serde(deserialize_with = "attempt_from_text")]
    attempt: AttemptId,
    node: NodeId,
    occurrence: u32,
    number: u32,
    brief_path: AbsPath,
    output_dir: AbsPath,
    inputs: BTreeMap<String, Option<AbsPath>>,
    outputs: BTreeMap<String, AbsPath>,
    requires: Vec<HostRequire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmittedData {
    #[serde(deserialize_with = "attempt_from_text")]
    attempt: AttemptId,
    outputs: BTreeMap<String, ArtifactRef>,
    #[serde(deserialize_with = "canonical_status")]
    work_status: WorkStatus,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FailedData {
    #[serde(deserialize_with = "attempt_from_text")]
    attempt: AttemptId,
    #[serde(deserialize_with = "canonical_status")]
    work_status: WorkStatus,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApprovedData {
    node: NodeId,
    occurrence: u32,
    pub(crate) by: Principal,
    pub(crate) at: Timestamp,
    #[serde(deserialize_with = "canonical_status")]
    pub(crate) work_status: WorkStatus,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CancelledData {
    work_id: WorkId,
    #[serde(deserialize_with = "canonical_status")]
    work_status: WorkStatus,
}

pub(crate) enum CheckedData {
    Started(StartedData),
    Begun,
    Replaced,
    Submitted(WorkStatus),
    Failed(WorkStatus),
    Approved(ApprovedData),
    Cancelled,
}

fn attempt_from_text<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<AttemptId, D::Error> {
    let text = String::deserialize(deserializer)?;
    let attempt = AttemptId::parse(&text).map_err(serde::de::Error::custom)?;
    if attempt.to_string() != text {
        return Err(serde::de::Error::custom("Attempt不是规范表示"));
    }
    Ok(attempt)
}

fn canonical_status<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<WorkStatus, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    let status: WorkStatus =
        serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;
    if serde_json::to_value(status).map_err(serde::de::Error::custom)? != value {
        return Err(serde::de::Error::custom("WorkStatus不是规范表示"));
    }
    Ok(status)
}

fn parse<T: DeserializeOwned>(value: &serde_json::Value) -> Result<T> {
    serde_json::from_value(value.clone()).map_err(|error| Error::StoreCorrupt {
        detail: format!("响应快照data解不开：{error}"),
    })
}

pub(crate) fn check_data(
    reply: &Reply,
    value: &serde_json::Value,
    work: &WorkId,
) -> Result<CheckedData> {
    let checked = match reply {
        Reply::Started {
            work_id,
            work_dir,
            requires,
        } => {
            let data: StartedData = parse(value)?;
            if &data.work_id == work
                && work_id == work
                && &data.work_dir == work_dir
                && work.as_str().get(15..) == Some(data.name.as_str())
                && crate::load::valid_workbook_version(&data.workbook.version)
                && &data.requires == requires
            {
                Some(CheckedData::Started(data))
            } else {
                None
            }
        }
        Reply::AttemptBegun {
            attempt,
            brief_path,
            output_dir,
            inputs,
            outputs,
            requires,
        } => {
            let data: BegunData = parse(value)?;
            (data.attempt == *attempt
                && data.node == attempt.node
                && data.occurrence == attempt.occurrence
                && data.number == attempt.number
                && &data.brief_path == brief_path
                && &data.output_dir == output_dir
                && &data.inputs == inputs
                && &data.outputs == outputs
                && &data.requires == requires)
                .then_some(CheckedData::Begun)
        }
        Reply::AttemptSubmitted { attempt, outputs } => {
            let data: SubmittedData = parse(value)?;
            (data.attempt == *attempt && &data.outputs == outputs)
                .then_some(CheckedData::Submitted(data.work_status))
        }
        Reply::AttemptReplaced {
            replaced_attempt,
            attempt,
            brief_path,
            output_dir,
            inputs,
            outputs,
            requires,
        } => {
            let data: ReplacedData = parse(value)?;
            (data.replaced_attempt == *replaced_attempt
                && data.attempt == *attempt
                && data.node == attempt.node
                && data.occurrence == attempt.occurrence
                && data.number == attempt.number
                && replaced_attempt.node == attempt.node
                && replaced_attempt.occurrence == attempt.occurrence
                && replaced_attempt.number.checked_add(1) == Some(attempt.number)
                && &data.brief_path == brief_path
                && &data.output_dir == output_dir
                && &data.inputs == inputs
                && &data.outputs == outputs
                && &data.requires == requires)
                .then_some(CheckedData::Replaced)
        }
        Reply::AttemptFailed { attempt } => {
            let data: FailedData = parse(value)?;
            (data.attempt == *attempt).then_some(CheckedData::Failed(data.work_status))
        }
        Reply::GateApproved { node, occurrence } => {
            let data: ApprovedData = parse(value)?;
            (data.node == *node && data.occurrence == *occurrence && !data.by.0.is_empty())
                .then_some(CheckedData::Approved(data))
        }
        Reply::Cancelled => {
            let data: CancelledData = parse(value)?;
            (data.work_id == *work && data.work_status == WorkStatus::Cancelled)
                .then_some(CheckedData::Cancelled)
        }
    };
    checked.ok_or_else(|| Error::StoreCorrupt {
        detail: "响应快照data与Reply/Work身份不一致".into(),
    })
}

pub(crate) fn start_matches(
    state: &WorkState,
    command: &Command,
    reply: &Reply,
    data: &CheckedData,
) -> bool {
    match (command, reply, data) {
        (
            Command::Start {
                work_id,
                name,
                workbook,
                flow,
                work_dir,
                inputs,
            },
            Reply::Started {
                work_id: reply_work,
                work_dir: reply_dir,
                ..
            },
            CheckedData::Started(data),
        ) => {
            work_id == &state.work_id
                && reply_work == &state.work_id
                && name == &state.name
                && workbook == &state.workbook
                && flow == &state.flow
                && work_dir == &state.work_dir
                && inputs == &state.inputs
                && reply_dir == &state.work_dir
                && data.workbook == state.workbook
                && data.flow == state.flow
                && data.name == state.name
                && data.work_dir == state.work_dir
        }
        _ => false,
    }
}
