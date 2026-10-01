//! RequestIntent 与意图指纹（存储合同 §2.1、GF-15）。
//!
//! 意图只含操作种类、解析后的完整目标与用户参数：不含观察结果、时钟与模型自报
//! 事实。`intent_hash` 是意图 canonical JSON 的 sha256；观察到的文件摘要或 `@file`
//! 内容变化不改变指纹，同路径同请求重放返回原响应，换字面值或路径才冲突。

use std::collections::BTreeMap;

use serde::Serialize;
use sheltie_core::digest::Sha256Hex;
use sheltie_core::ids::{AttemptId, NodeId, WorkId};

/// 起始输入的值：字面值，或 `@file` 的词法规范化绝对路径（不访问文件系统）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "value")]
pub enum InputValue {
    Literal { text: String },
    AtFile { path: String },
}

/// `--summary` / `--reason` 的值：同 [`InputValue`]。
pub type SummarySource = InputValue;

/// 全部写操作的意图闭集。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "intent")]
pub enum RequestIntent {
    StartWork {
        workbook: String,
        version: Option<String>,
        flow: String,
        name: Option<String>,
        /// 键按 BTreeMap 字节序稳定。
        inputs: BTreeMap<String, InputValue>,
    },
    BeginAttempt {
        work: WorkId,
        node: NodeId,
    },
    SubmitAttempt {
        work: WorkId,
        attempt: AttemptId,
        summary: SummarySource,
    },
    FailAttempt {
        work: WorkId,
        attempt: AttemptId,
        reason: SummarySource,
    },
    ApproveGate {
        work: WorkId,
        node: NodeId,
    },
    CancelWork {
        work: WorkId,
    },
    AddWorkbook {
        /// 源目录参数的词法规范化绝对路径；不做 canonicalize，重放不要求源仍在。
        source: String,
    },
    RemoveWorkbook {
        id: String,
        version: String,
    },
}

impl RequestIntent {
    /// canonical JSON 的 sha256（裸 64 位小写十六进制）。以 `serde_json::to_vec`
    /// 字节为准：枚举固定 tag、字段顺序固定、输入键按字节排序、无浮点数。
    pub fn hash(&self) -> Sha256Hex {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        Sha256Hex::of_bytes(&bytes)
    }
}

/// 词法规范化一个绝对路径：解析 `.` 段并拼接，不访问文件系统、不解析软链
///（重放不要求源路径仍存在，canonicalize 会引入这个依赖）。
pub fn lexical_abs(path: &str) -> String {
    let p = camino::Utf8Path::new(path);
    let base = if p.is_absolute() {
        camino::Utf8PathBuf::from("/")
    } else {
        // 相对路径按当前目录词法展开。
        let cwd = std::env::current_dir().unwrap_or_default();
        camino::Utf8PathBuf::from(cwd.to_string_lossy().into_owned())
    };
    let mut out = base;
    for seg in p.components() {
        match seg {
            camino::Utf8Component::RootDir | camino::Utf8Component::Prefix(_) => {}
            camino::Utf8Component::CurDir => {}
            camino::Utf8Component::ParentDir => {
                out.pop();
            }
            camino::Utf8Component::Normal(name) => out = out.join(name),
        }
    }
    out.to_string()
}
