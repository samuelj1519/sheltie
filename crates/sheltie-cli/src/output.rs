//! 输出：给人读的文本，或协议 §5 的 JSON 响应封装。

use serde::Serialize;
use sheltie_core::ErrorCode;
use sheltie_core::ids::WorkId;
use sheltie_core::work::NextOp;
use sheltie_runtime::Response;

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

/// 一次命令的输出，由命令模块构造，`dispatch` 决定怎么打印。
#[derive(Debug)]
pub struct Outcome {
    /// 文本模式下打印的内容。
    pub text: String,
    /// JSON 模式下的完整响应封装。
    pub json: serde_json::Value,
    pub exit_code: i32,
}

/// 把成功结果包成两种形式。`text` 由调用方渲染。`next` 为空的命令用一个；
/// Work 写操作用 [`ok_response`]，只读操作用 [`ok_work_next`]。
pub fn ok<T: Serialize>(
    text: String,
    request_id: Option<String>,
    revision: Option<u64>,
    data: T,
    next: Vec<NextOp>,
) -> Outcome {
    let json = to_value_lossy(OkEnvelope {
        ok: true,
        request_id,
        revision,
        data,
        next,
    });
    Outcome {
        text,
        json,
        exit_code: 0,
    }
}

/// Work 写操作只渲染提交时快照；`next` 与只读状态卡使用同一协议形状。
pub(crate) fn ok_response(text: String, response: Response, work: &WorkId) -> Outcome {
    let next = response
        .next
        .iter()
        .map(|op| sheltie_core::work::render::next_item_json(work, op))
        .collect();
    ok_work_next(
        text,
        Some(response.request_id),
        Some(response.revision),
        replayed_data(response.data, response.replayed),
        next,
    )
}

pub(crate) fn replayed_data(mut data: serde_json::Value, replayed: bool) -> serde_json::Value {
    if let Some(map) = data.as_object_mut() {
        map.insert("replayed".to_string(), serde_json::Value::Bool(replayed));
    }
    data
}

/// `next` 已是协议 §5 形状（core `next_item_json` 生成），
/// 与状态卡 `data.next` 完全同形（O13），不再重复组装。
pub(crate) fn ok_work_next(
    text: String,
    request_id: Option<String>,
    revision: Option<u64>,
    data: serde_json::Value,
    next: Vec<serde_json::Value>,
) -> Outcome {
    let mut root = serde_json::Map::new();
    root.insert("ok".to_string(), serde_json::Value::Bool(true));
    if let Some(id) = request_id {
        root.insert("request_id".to_string(), serde_json::json!(id));
    }
    if let Some(rev) = revision {
        root.insert("revision".to_string(), serde_json::json!(rev));
    }
    root.insert("data".to_string(), data);
    root.insert("next".to_string(), serde_json::Value::Array(next));
    Outcome {
        text,
        json: serde_json::Value::Object(root),
        exit_code: 0,
    }
}

/// 把错误包成两种形式。退出码 1。
pub fn err(
    code: ErrorCode,
    message: String,
    detail: Option<serde_json::Value>,
    next: Vec<NextOp>,
) -> Outcome {
    let mut error = serde_json::Map::new();
    error.insert("code".to_string(), to_value_lossy(code));
    error.insert("message".to_string(), serde_json::json!(message));
    if let Some(d) = detail {
        error.insert("detail".to_string(), d);
    }
    let json = serde_json::json!({
        "ok": false,
        "error": serde_json::Value::Object(error),
        "next": to_value_lossy(next),
    });
    Outcome {
        text: format!("{message}\n"),
        json,
        exit_code: 1,
    }
}

/// 参数格式错误的统一出口：`INVALID_REQUEST` 封装、退出码 2（协议 §5「参数解析错误 2」）。
pub(crate) fn param_error(message: String) -> Outcome {
    let mut out = err(ErrorCode::InvalidRequest, message, None, Vec::new());
    out.exit_code = 2;
    out
}

/// 打印。JSON 模式打一行 `json`；文本模式打 `text`。错误走 stderr。
pub fn print(outcome: &Outcome, json_mode: bool) {
    if json_mode {
        println!("{}", outcome.json);
    } else if outcome.exit_code == 0 {
        print!("{}", outcome.text);
    } else {
        eprint!("{}", outcome.text);
    }
}

pub(crate) fn raw_error(error: &sheltie_runtime::Error) -> i32 {
    let outcome = crate::error_map::to_outcome(error);
    eprint!("{}", outcome.text);
    outcome.exit_code
}

pub(crate) fn raw_param_error(message: &str) -> i32 {
    eprintln!("{message}");
    2
}

/// 这些响应类型全是普通数据，序列化实际不会失败；真失败了给 `null` 也不比 panic 差。
fn to_value_lossy<T: Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}
