#![allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "../../sheltie-export/tests/common/mod.rs"]
mod common;
use common::*;
use std::process::Command;

fn raw(fixture: &Fixture, key: &str, revision: &str) -> std::process::Output {
    Command::new(&binaries().0)
        .args([
            "--home",
            fixture.home.to_str().unwrap(),
            "work",
            "result",
            &fixture.work,
            "--artifact",
            key,
            "--revision",
            revision,
        ])
        .output()
        .unwrap()
}

// Task: C006-T02
#[test]
fn result_artifact_stdout_preserves_binary_empty_and_text_bytes_without_json_or_extra_newline() {
    let fixture = Fixture::complete();
    for (key, bytes) in [
        ("binary", fixture.binary.as_slice()),
        ("empty", &[][..]),
        ("task", "原目标".as_bytes()),
    ] {
        let out = raw(&fixture, key, &fixture.revision.to_string());
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, bytes);
    }
}

// Task: C006-T02
#[test]
fn result_artifact_rejects_json_request_ids_and_unpaired_parameters_before_creating_a_home() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("absent");
    for args in [
        vec!["--json", "--artifact", "binary", "--revision", "3"],
        vec![
            "--request-id",
            "readonly-id",
            "--artifact",
            "binary",
            "--revision",
            "3",
        ],
        vec!["--artifact", "binary"],
        vec!["--revision", "3"],
        vec!["--artifact", "binary", "--revision", "0"],
    ] {
        let out = Command::new(&binaries().0)
            .args([
                "--home",
                home.to_str().unwrap(),
                "work",
                "result",
                "2026-10-03-001-test",
            ])
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!home.exists());
    }
}

// Task: C006-T02
#[test]
fn result_artifact_rejects_revision_key_and_source_modification_without_business_writes() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::complete();
    let before = ok(&fixture.home, &["work", "status", &fixture.work]);
    for (key, revision) in [
        ("binary", fixture.revision + 1),
        ("missing", fixture.revision),
    ] {
        let out = raw(&fixture, key, &revision.to_string());
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
        assert_eq!(
            ok(&fixture.home, &["work", "status", &fixture.work]),
            before
        );
    }
    let source = fixture.result["artifacts"][0]["path"].as_str().unwrap();
    std::fs::set_permissions(source, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(source, b"changed").unwrap();
    let out = raw(&fixture, "binary", &fixture.revision.to_string());
    assert!(!out.status.success());
    assert!(!out.stderr.is_empty());
    assert_eq!(
        ok(&fixture.home, &["work", "status", &fixture.work]),
        before
    );
}

// Task: C006-T02
#[test]
fn result_artifact_accepts_empty_hyphen_unicode_and_literal_metacharacter_slot_keys() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path().canonicalize().unwrap();
    let home = base.join("home");
    let workbook = base.join("keys");
    std::fs::create_dir(&workbook).unwrap();
    std::fs::create_dir(workbook.join("flows")).unwrap();
    std::fs::create_dir(workbook.join("resources")).unwrap();
    std::fs::write(workbook.join("workbook.toml"), "schema='workbook/v1'\nid='keys'\nversion='1.0.0'\nname='合法槽名'\nflows=['flows/default.toml']\n").unwrap();
    let keys = ["", "-name", "--json", "中文", "a$(echo literal)"];
    let declarations: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(index, key)| {
            format!(
                "{{name={},from='resource.resources/file{index}.bin',result=true}}",
                serde_json::to_string(key).unwrap()
            )
        })
        .collect();
    std::fs::write(workbook.join("flows/default.toml"), format!("schema='flow/v1'\nid='default'\nentry='make'\n[[nodes]]\nid='make'\ntitle='键'\nexecutor='agent'\ninstruction={{text='消费冻结文件'}}\ninputs=[{}]\n", declarations.join(","))).unwrap();
    for (index, _) in keys.iter().enumerate() {
        std::fs::write(
            workbook.join("resources").join(format!("file{index}.bin")),
            [index as u8, 0, 255],
        )
        .unwrap();
    }
    ok(&home, &["workbook", "add", workbook.to_str().unwrap()]);
    let work = ok(
        &home,
        &["work", "start", "--workbook", "keys", "--flow", "default"],
    )["data"]["work_id"]
        .as_str()
        .unwrap()
        .to_string();
    ok(&home, &["attempt", "begin", &work, "--node", "make"]);
    ok(
        &home,
        &[
            "attempt",
            "submit",
            &work,
            "--attempt",
            "make#1.0",
            "--summary",
            "完成",
        ],
    );
    let revision = ok(&home, &["work", "result", &work])["data"]["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    for (index, key) in keys.iter().enumerate() {
        let out = Command::new(&binaries().0)
            .args(["--home", home.to_str().unwrap(), "work", "result", &work])
            .arg(format!("--artifact={key}"))
            .args(["--revision", &revision])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "key={key:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, [index as u8, 0, 255]);
    }
}
