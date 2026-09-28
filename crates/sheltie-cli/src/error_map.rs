//! runtime 错误到协议错误码、`detail` 与退出码的映射。退出码：成功 0；`ok = false` 1；参数错误 2（clap 自带）。

use serde_json::json;
use sheltie_runtime::Error;

use crate::output::Outcome;

/// 一张 `match`，一次填全。`detail` 按协议 §7 每行的 `detail.*` 字段构造。
pub fn to_outcome(err: &Error) -> Outcome {
    let code = err.code();
    let message = err.to_string();
    let detail = detail_of(err);
    let mut outcome = crate::output::err(code, message, detail, Vec::new());
    if matches!(err, Error::InputFileInvalid { .. }) {
        outcome.exit_code = 2;
    }
    outcome
}

/// 每个变体的定位字段。协议 §7 没写 `detail` 的变体也尽量带上定位信息，方便人看。
fn detail_of(err: &Error) -> Option<serde_json::Value> {
    let detail = match err {
        Error::Core(e) => match e {
            sheltie_core::Error::InvalidId {
                field,
                value,
                reason,
            } => json!({ "field": field, "value": value, "reason": reason }),
            sheltie_core::Error::InvalidPath { path, reason } => {
                json!({ "path": path, "reason": reason })
            }
            sheltie_core::Error::TextTooLong { field, max, actual } => {
                json!({ "field": field, "max": max, "actual": actual })
            }
            sheltie_core::Error::InvalidDigest { value } => json!({ "value": value }),
            sheltie_core::Error::WorkbookInvalid { field, reason } => {
                json!({ "path": field, "reason": reason })
            }
            sheltie_core::Error::FlowInvalid { rule, path, reason } => {
                json!({ "rule": rule, "path": path, "reason": reason })
            }
            sheltie_core::Error::InputMissing { missing, extra } => {
                json!({ "missing": missing, "extra": extra })
            }
            sheltie_core::Error::WorkTerminal { status } => json!({ "status": status }),
            sheltie_core::Error::IllegalNext { requested, next } => {
                json!({ "requested": requested, "next": next })
            }
            sheltie_core::Error::InputUnavailable { input, node } => {
                json!({ "input": input, "node": node.as_str() })
            }
            sheltie_core::Error::ArtifactModified { input, path } => {
                json!({ "input": input, "path": path })
            }
            sheltie_core::Error::AttemptNotRunning { attempt } => {
                json!({ "attempt": attempt.to_string() })
            }
            sheltie_core::Error::SummaryTooLong { max, actual } => {
                json!({ "max": max, "actual": actual })
            }
            sheltie_core::Error::OutputMissing { output, path } => {
                json!({ "output": output, "path": path })
            }
            sheltie_core::Error::OutputTooLarge {
                output,
                max_bytes,
                actual,
            } => json!({ "output": output, "max_bytes": max_bytes, "actual": actual }),
            sheltie_core::Error::InvalidRequest { reason } => json!({ "reason": reason }),
        },
        Error::WorkbookExists { id, version } => json!({ "id": id, "version": version }),
        Error::WorkbookInUse { id, version, works } => json!({
            "id": id,
            "version": version,
            "works": works.iter().map(|w| w.to_string()).collect::<Vec<_>>(),
        }),
        Error::WorkbookTampered { results } => json!({ "results": results }),
        Error::UpdateUnavailable { reason } => json!({ "reason": reason }),
        Error::UpdateChecksumMismatch { expected, actual } => {
            json!({ "expected": expected, "actual": actual })
        }
        Error::NotFound { what } => json!({ "what": what }),
        Error::RequestConflict { request_id } => json!({ "request_id": request_id }),
        Error::RevisionConflict { expected, actual } => {
            json!({ "expected": expected, "actual": actual })
        }
        Error::EffectPending {
            committed,
            request_id,
            pending_request_id,
            detail,
            original,
        } => {
            let mut d = json!({
                "committed": committed,
                "request_id": request_id,
                "cause": detail,
            });
            if let Some(p) = pending_request_id {
                d["pending_request_id"] = json!(p);
            }
            if let Some(o) = original {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(o) {
                    d["pending_original"] = v;
                }
            }
            d
        }
        Error::StoreSchemaMismatch { detail } | Error::StoreCorrupt { detail } => {
            json!({ "detail": detail })
        }
        Error::Io { path, source } => json!({ "path": path, "error": source.to_string() }),
        Error::RecoveryRequired { path, detail } => json!({ "path": path, "error": detail }),
        Error::InputFileInvalid { path, reason } => json!({ "path": path, "reason": reason }),
        Error::InvalidRequest { reason } => json!({ "reason": reason }),
    };
    Some(detail)
}
