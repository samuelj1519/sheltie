//! 输出：给人读的文本，或协议 §5 的 JSON 响应封装。

use serde::Serialize;
use sheltie_core::ErrorCode;
use sheltie_core::work::NextOp;

/// 成功响应封装。
#[derive(Debug, Serialize)]
pub struct OkEnvelope<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
    pub data: T,
    pub next: Vec<NextOp>,
}

/// 失败响应封装。
#[derive(Debug, Serialize)]
pub struct ErrEnvelope {
    pub ok: bool,
    pub error: ErrorBody,
    pub next: Vec<NextOp>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

/// 一次命令的输出，由命令模块构造，`dispatch` 决定怎么打印。
#[derive(Debug)]
pub struct Outcome {
    /// 文本模式下打印的内容。
    pub text: String,
    /// JSON 模式下的完整响应封装。
    pub json: serde_json::Value,
    pub exit_code: i32,
}

/// 把成功结果包成两种形式。`text` 由调用方渲染。
#[allow(unused_variables)]
pub fn ok<T: Serialize>(
    text: String,
    request_id: Option<String>,
    revision: Option<u64>,
    data: T,
    next: Vec<NextOp>,
) -> Outcome {
    todo!("T17")
}

/// 把错误包成两种形式。退出码 1。
#[allow(unused_variables)]
pub fn err(
    code: ErrorCode,
    message: String,
    detail: Option<serde_json::Value>,
    next: Vec<NextOp>,
) -> Outcome {
    todo!("T17")
}

/// 打印。JSON 模式打一行 `json`；文本模式打 `text`。错误走 stderr。
#[allow(unused_variables)]
pub fn print(outcome: &Outcome, json_mode: bool) {
    todo!("T17")
}
