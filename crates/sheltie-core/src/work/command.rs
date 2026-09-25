//! 命令、上下文、效果与回复。`decide` 的输入输出类型。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::state::{ArtifactRef, Principal, Timestamp, WorkState, WorkbookRef};
use crate::digest::Sha256Hex;
use crate::ids::{AttemptId, FlowId, NodeId, WorkId, WorkName};
use crate::path::AbsPath;
use crate::workbook::HostRequire;

/// runtime 对一个文件的只读观察。core 拿它对照合同，不自己读文件。
///
/// 只有 runtime 的 `observe_file` 会构造它；这是 `INV-6` 的落点：摘要不由模型报。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedFile {
    pub path: AbsPath,
    pub sha256: Sha256Hex,
    pub bytes: u64,
}

impl ObservedFile {
    /// 仅供 runtime 调用。
    #[doc(hidden)]
    pub fn new(path: AbsPath, sha256: Sha256Hex, bytes: u64) -> Self {
        Self {
            path,
            sha256,
            bytes,
        }
    }

    /// 测试用：把一个已有引用当作「当前观察」。
    pub fn for_test(path: AbsPath, sha256: Sha256Hex, bytes: u64) -> Self {
        Self {
            path,
            sha256,
            bytes,
        }
    }

    pub fn into_ref(self) -> ArtifactRef {
        ArtifactRef {
            path: self.path,
            sha256: self.sha256,
            bytes: self.bytes,
        }
    }

    pub fn matches(&self, r: &ArtifactRef) -> bool {
        self.sha256 == r.sha256
    }
}

/// 协调者能做的全部写操作。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "command")]
pub enum Command {
    /// 创建 Work。`inputs` 已由 runtime 写成文件并算好摘要。
    Start {
        work_id: WorkId,
        name: WorkName,
        workbook: WorkbookRef,
        flow: FlowId,
        work_dir: AbsPath,
        inputs: BTreeMap<String, ArtifactRef>,
    },
    /// 进入节点并开始一次尝试。`observed_inputs` 由 runtime 按 `input_paths_for` 观察；
    /// `instruction_text` 是说明书原文（`File` 由 runtime 从冻结副本读出，`Text` 直接取值），core 用它渲染任务书。
    BeginAttempt {
        node: NodeId,
        observed_inputs: BTreeMap<String, Option<ObservedFile>>,
        instruction_text: String,
    },
    /// 提交尝试。`observed_outputs` 由 runtime 按 `output_paths_for` 观察，缺文件为 `None`。
    SubmitAttempt {
        attempt: AttemptId,
        summary: String,
        observed_outputs: BTreeMap<String, Option<ObservedFile>>,
    },
    FailAttempt {
        attempt: AttemptId,
        reason: String,
    },
    ApproveGate {
        node: NodeId,
    },
    Cancel,
}

impl Command {
    /// 命令名，用于审计与错误信息。
    pub fn name(&self) -> &'static str {
        match self {
            Self::Start { .. } => "start",
            Self::BeginAttempt { .. } => "attempt begin",
            Self::SubmitAttempt { .. } => "attempt submit",
            Self::FailAttempt { .. } => "attempt fail",
            Self::ApproveGate { .. } => "gate approve",
            Self::Cancel => "work cancel",
        }
    }
}

/// 一次决定的不可变输入：时间与操作者。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub now: Timestamp,
    pub principal: Principal,
}

/// core 要求 runtime 在提交后做的事。全部幂等。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "effect")]
pub enum Effect {
    WriteBrief {
        path: AbsPath,
        content: String,
    },
    /// 引擎生成的输入文件（目前只有 `engine.stats` 的 `stats.json`）。内容在 core 里算好并已记摘要。
    WriteFile {
        path: AbsPath,
        content: String,
    },
    SealOutputs {
        paths: Vec<AbsPath>,
    },
    RefreshStatusCard,
}

/// 命令成功后的回复数据，对应协议 §3 各操作的返回。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reply")]
pub enum Reply {
    Started {
        work_id: WorkId,
        work_dir: AbsPath,
        requires: Vec<HostRequire>,
    },
    AttemptBegun {
        attempt: AttemptId,
        brief_path: AbsPath,
        output_dir: AbsPath,
        inputs: BTreeMap<String, Option<AbsPath>>,
        outputs: BTreeMap<String, AbsPath>,
        requires: Vec<HostRequire>,
    },
    AttemptSubmitted {
        attempt: AttemptId,
        outputs: BTreeMap<String, ArtifactRef>,
    },
    AttemptFailed {
        attempt: AttemptId,
    },
    GateApproved {
        node: NodeId,
        occurrence: u32,
    },
    Cancelled,
}

/// `decide` 的输出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub state: WorkState,
    pub effects: Vec<Effect>,
    pub reply: Reply,
}
