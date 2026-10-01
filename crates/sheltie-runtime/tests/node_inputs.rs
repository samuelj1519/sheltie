//! C002-T31：节点输入只能绑定声明来源的成功产物。
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::*;
use rusqlite::Connection;
use serde_json::json;
use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_runtime::request::InputValue;
use std::path::PathBuf;

fn lit(text: &str) -> InputValue {
    InputValue::Literal { text: text.into() }
}

// Task: C002-T31
#[test]
fn node_input_reference_cannot_drift_to_another_managed_path_digest_or_size() {
    let (_dir, home, svc) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&svc));
    let begun = svc
        .begin(&work, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output = PathBuf::from(output_dir_of(&begun).as_str()).join("outline.md");
    std::fs::write(&output, b"# committed outline\n").unwrap();
    svc.submit(
        &work,
        &AttemptId::parse("outline#1.0").unwrap(),
        &lit("done"),
        None,
    )
    .unwrap();
    let summary = svc
        .begin(&work, &NodeId::new("summary").unwrap(), None)
        .unwrap();
    assert_eq!(
        summary.data["inputs"]["outline"],
        json!(output.to_str().unwrap())
    );
    assert!(svc.status(&work).is_ok() && svc.stats(&work).is_ok());
    assert_eq!(svc.list().unwrap().len(), 1);
    let conn = Connection::open(home.store_path().as_str()).unwrap();
    let raw: String = conn
        .query_row(
            "SELECT state_json FROM works WHERE work_id=?1",
            [work.as_str()],
            |r| r.get(0),
        )
        .unwrap();
    let pristine: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(pristine["attempts"][1]["id"]["node"], "summary");
    assert_ne!(
        pristine["attempts"][1]["inputs"]["outline"]["sha256"],
        json!("0".repeat(64))
    );
    let substitute = PathBuf::from(home.work_dir(&work).as_str()).join("substitute.md");
    std::fs::write(&substitute, b"# committed outline\n").unwrap();
    for field in ["path", "sha256", "bytes"] {
        let mut changed = pristine.clone();
        let reference = &mut changed["attempts"][1]["inputs"]["outline"];
        match field {
            "path" => reference[field] = json!(substitute.to_str().unwrap()),
            "sha256" => reference[field] = json!("0".repeat(64)),
            "bytes" => reference[field] = json!(reference[field].as_u64().unwrap() + 1),
            _ => unreachable!(),
        }
        assert_ne!(changed, pristine);
        let raw = changed.to_string();
        conn.execute(
            "UPDATE works SET state_json=?1 WHERE work_id=?2",
            rusqlite::params![raw, work.as_str()],
        )
        .unwrap();
        let before:(i64,i64,i64)=conn.query_row("SELECT (SELECT revision FROM works),(SELECT COUNT(*) FROM requests),(SELECT COUNT(*) FROM audit)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        for error in [
            svc.status(&work).unwrap_err(),
            svc.stats(&work).unwrap_err(),
            svc.list().unwrap_err(),
            svc.cancel(&work, Some("binding-must-stop".into()))
                .unwrap_err(),
        ] {
            assert_eq!(
                error.code(),
                sheltie_core::ErrorCode::StoreCorrupt,
                "{field}: {error:?}"
            );
        }
        let after:(i64,i64,i64)=conn.query_row("SELECT (SELECT revision FROM works),(SELECT COUNT(*) FROM requests),(SELECT COUNT(*) FROM audit)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(after, before);
        let after_raw: String = conn
            .query_row("SELECT state_json FROM works", [], |r| r.get(0))
            .unwrap();
        assert_eq!(after_raw, raw);
        assert_eq!(std::fs::read(&output).unwrap(), b"# committed outline\n");
        assert_eq!(
            std::fs::read(&substitute).unwrap(),
            b"# committed outline\n"
        );
    }
}

// Task: C002-T31
#[test]
fn node_input_uses_its_declared_source_even_when_a_later_node_has_the_same_output_name() {
    let (dir, home) = temp_home();
    let source = copy_example("two-step", dir.path());
    let flow = source.join("flows/default.toml");
    let text = std::fs::read_to_string(&flow).unwrap();
    assert!(text.contains("to = \"summary\""));
    let text = text.replace("to = \"summary\"", "to = \"middle\"")
        + r#"
[[nodes]]
id = "middle"
title = "Another successful source"
executor = "agent"
instruction = { file = "instructions/outline.md" }
inputs = [{ name = "outline", from = "outline.outline" }]
outputs = [{ name = "outline", path = "outline.md", max_bytes = 65536 }]
[[edges]]
from = "middle"
to = "summary"
kind = "main"
"#;
    std::fs::write(flow, text).unwrap();
    repo(&home).add(&abs(&source), None).unwrap();
    let svc = service(&home);
    let work = work_id_of(&start_two_step(&svc));
    let mut declared_output = None;
    for (node, bytes) in [
        ("outline", b"declared source".as_slice()),
        ("middle", b"later source".as_slice()),
    ] {
        let begun = svc.begin(&work, &NodeId::new(node).unwrap(), None).unwrap();
        let output = PathBuf::from(output_dir_of(&begun).as_str()).join("outline.md");
        std::fs::write(&output, bytes).unwrap();
        if node == "outline" {
            declared_output = Some(output);
        }
        svc.submit(
            &work,
            &AttemptId::parse(&format!("{node}#1.0")).unwrap(),
            &lit("done"),
            None,
        )
        .unwrap();
    }
    let begun = svc
        .begin(
            &work,
            &NodeId::new("summary").unwrap(),
            Some("declared-source".into()),
        )
        .unwrap();
    let output = declared_output.unwrap();
    assert_eq!(
        begun.data["inputs"]["outline"],
        json!(output.to_str().unwrap())
    );
    assert_eq!(std::fs::read(output).unwrap(), b"declared source");
    assert!(svc.status(&work).is_ok());
    assert!(svc.stats(&work).is_ok());
    assert_eq!(svc.list().unwrap().len(), 1);
}
