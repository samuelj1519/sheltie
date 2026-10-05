//! Output: human-readable text or the protocol §5 JSON response envelope.

use serde::Serialize;
use sheltie_core::ErrorCode;
use sheltie_core::ids::WorkId;
use sheltie_runtime::Response;

/// Success response envelope.
#[derive(Debug, Serialize)]
pub struct OkEnvelope<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
    pub data: T,
    pub next: Vec<serde_json::Value>,
}

/// A command's output, constructed by command modules and printed by `dispatch`.
#[derive(Debug)]
pub struct Outcome {
    /// Content printed in text mode.
    pub text: String,
    /// Complete response envelope for JSON mode.
    pub json: serde_json::Value,
    pub exit_code: i32,
}

/// The caller renders `text`; `next` uses core's protocol representation.
pub fn ok<T: Serialize>(
    text: String,
    request_id: Option<String>,
    revision: Option<u64>,
    data: T,
    next: Vec<serde_json::Value>,
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

/// Work writes render only commit-time snapshots; `next` shares the read-only status card's protocol shape.
pub(crate) fn ok_response(text: String, response: Response, work: &WorkId) -> Outcome {
    let next = response
        .next
        .iter()
        .map(|op| sheltie_core::work::render::next_item_json(work, op))
        .collect();
    ok(
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

/// Wrap an error in both output forms, with exit code 1.
pub fn err(code: ErrorCode, message: String, detail: Option<serde_json::Value>) -> Outcome {
    let mut error = serde_json::Map::new();
    error.insert("code".to_string(), to_value_lossy(code));
    error.insert("message".to_string(), serde_json::json!(message));
    if let Some(d) = detail {
        error.insert("detail".to_string(), d);
    }
    let json = serde_json::json!({
        "ok": false,
        "error": serde_json::Value::Object(error),
        "next": [],
    });
    Outcome {
        text: format!("{message}\n"),
        json,
        exit_code: 1,
    }
}

/// Common parameter-error response: `INVALID_REQUEST` envelope and exit code 2 (protocol §5).
pub(crate) fn param_error(message: String) -> Outcome {
    let mut out = err(ErrorCode::InvalidRequest, message, None);
    out.exit_code = 2;
    out
}

/// Print one-line `json` or human-readable `text`; errors go to stderr.
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

/// These plain-data response types cannot normally fail serialization; return `null` rather than panic if they do.
fn to_value_lossy<T: Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}
