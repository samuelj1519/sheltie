#![allow(clippy::unwrap_used, clippy::expect_used)]
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sheltie_core::ids::WorkId;
use sheltie_export::model::{Artifact, MAX_FILE_BYTES, MAX_METADATA_BYTES, SelectedResult};
use sheltie_export::source::Source;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn work() -> WorkId {
    WorkId::parse("2026-09-24-001-export").unwrap()
}
fn metadata(bytes: &[u8]) -> Value {
    json!({"ok":true,"data":{
    "format":"work-result/v1","work_id":work().as_str(),"revision":3,
    "workbook":{"id":"method","version":"1.0.0","digest":"0".repeat(64)},"flow":"default",
    "status":{"kind":"succeeded"},"effects_pending":false,"final":true,
    "artifacts":[{"key":"payload","path":"/original/payload.bin","sha256":format!("{:x}",Sha256::digest(bytes)),"bytes":bytes.len(),"source":{"attempt":"finish#1.0","kind":"output","name":"payload"}}]
},"next":[]})
}

struct Fixture {
    _directory: tempfile::TempDir,
    source: Source,
    metadata: PathBuf,
    bytes: PathBuf,
    args: PathBuf,
}
fn fixture(bytes: &[u8], exit: i32) -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let metadata_path = root.join("result.json");
    let bytes_path = root.join("bytes.bin");
    let args = root.join("argv.json");
    let executable = root.join("source.py");
    std::fs::write(&metadata_path, metadata(bytes).to_string()).unwrap();
    std::fs::write(&bytes_path, bytes).unwrap();
    std::fs::write(&executable,format!("#!/usr/bin/python3\nimport json,sys\nfrom pathlib import Path\nPath({args:?}).write_text(json.dumps(sys.argv[1:]))\nsys.stdout.buffer.write(Path({metadata:?} if '--json' in sys.argv else {bytes:?}).read_bytes())\nsys.stdout.buffer.flush()\nsys.exit({exit})\n",args=args.to_str().unwrap(),metadata=metadata_path.to_str().unwrap(),bytes=bytes_path.to_str().unwrap())).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let source = Source::new(&executable, &root, &work()).unwrap();
    Fixture {
        _directory: directory,
        source,
        metadata: metadata_path,
        bytes: bytes_path,
        args,
    }
}

// Task: C006-T01
#[test]
fn source_strictly_reads_metadata_and_preserves_binary_bytes_with_direct_arguments() {
    let bytes = b"\0\xff\xfe\nraw\0";
    let fixture = fixture(bytes, 0);
    let result = fixture.source.result().unwrap();
    assert_eq!(result.revision, 3);
    let mut received = Vec::new();
    fixture
        .source
        .receive(3, &result.artifacts[0], &mut received)
        .unwrap();
    assert_eq!(received, bytes);
    let args: Vec<String> = serde_json::from_slice(&std::fs::read(&fixture.args).unwrap()).unwrap();
    assert_eq!(
        &args[2..],
        [
            "work",
            "result",
            work().as_str(),
            "--artifact=payload",
            "--revision",
            "3"
        ]
    );
    assert!(!args.iter().any(|arg| arg == "--json"));
}

// Task: C006-T01
#[test]
fn metadata_unknown_fields_identity_order_and_bounds_are_rejected() {
    for field in [
        "envelope",
        "result",
        "artifact",
        "source",
        "work",
        "pending",
        "nonempty",
        "duplicate",
        "nul",
    ] {
        let fixture = fixture(b"bytes", 0);
        let mut data = metadata(b"bytes");
        match field {
            "envelope" => data["unknown"] = json!(true),
            "result" => data["data"]["unknown"] = json!(true),
            "artifact" => data["data"]["artifacts"][0]["unknown"] = json!(true),
            "source" => data["data"]["artifacts"][0]["source"]["unknown"] = json!(true),
            "work" => data["data"]["work_id"] = json!("2026-09-24-002-other"),
            "pending" => data["data"]["effects_pending"] = json!(true),
            "nonempty" => data["data"]["artifacts"] = json!([]),
            "duplicate" => {
                let artifact = data["data"]["artifacts"][0].clone();
                data["data"]["artifacts"]
                    .as_array_mut()
                    .unwrap()
                    .push(artifact);
            }
            "nul" => data["data"]["artifacts"][0]["key"] = json!("bad\0key"),
            _ => unreachable!(),
        }
        std::fs::write(&fixture.metadata, data.to_string()).unwrap();
        assert!(fixture.source.result().is_err(), "{field}");
    }
}

// Task: C006-T01
#[test]
fn metadata_versions_terminal_binders_and_status_shapes_follow_the_full_contract() {
    let fixture = fixture(b"bytes", 0);
    let baseline = metadata(b"bytes");
    assert!(fixture.source.result().is_ok());
    let rejected = |payload: &Value, diagnostic: Option<&str>| {
        std::fs::write(&fixture.metadata, payload.to_string()).unwrap();
        let error = fixture.source.result().unwrap_err();
        assert_eq!(error.code(), "INVALID_RESULT", "{payload}: {error}");
        if let Some(diagnostic) = diagnostic {
            assert!(error.to_string().contains(diagnostic), "{payload}: {error}");
        }
    };

    let mut unicode_leaf = baseline.clone();
    unicode_leaf["data"]["artifacts"][0]["path"] = json!("/original/成果.bin");
    std::fs::write(&fixture.metadata, unicode_leaf.to_string()).unwrap();
    assert!(fixture.source.result().is_ok());
    let mut backslash_leaf = baseline.clone();
    backslash_leaf["data"]["artifacts"][0]["path"] = json!("/original/x\\y");
    rejected(&backslash_leaf, Some("叶名"));

    for version in ["A".repeat(32), "a".into(), ".1+RC-2".into()] {
        let mut valid = baseline.clone();
        valid["data"]["workbook"]["version"] = json!(version);
        std::fs::write(&fixture.metadata, valid.to_string()).unwrap();
        assert!(fixture.source.result().is_ok());
    }
    for version in [
        "A".repeat(33),
        "".into(),
        ".".into(),
        "..".into(),
        ".staging".into(),
        "1/2".into(),
        "1_0".into(),
        "1:0".into(),
        "版本".into(),
        "a\n".into(),
        "a\0".into(),
    ] {
        let mut invalid = baseline.clone();
        invalid["data"]["workbook"]["version"] = json!(version);
        rejected(&invalid, Some("Workbook版本"));
    }

    for attempt in ["finish#0.0", "finish#01.0", "finish#1.00"] {
        let mut invalid = baseline.clone();
        invalid["data"]["artifacts"][0]["source"]["attempt"] = json!(attempt);
        rejected(&invalid, Some("AttemptId"));
    }
    let mut paired = baseline.clone();
    let mut second = paired["data"]["artifacts"][0].clone();
    second["key"] = json!("second");
    second["path"] = json!("/original/second.bin");
    second["source"]["name"] = json!("second");
    paired["data"]["artifacts"]
        .as_array_mut()
        .unwrap()
        .push(second);
    std::fs::write(&fixture.metadata, paired.to_string()).unwrap();
    assert!(fixture.source.result().is_ok());
    for attempt in ["finish#2.0", "other#1.0"] {
        let mut mixed = paired.clone();
        mixed["data"]["artifacts"][1]["source"]["attempt"] = json!(attempt);
        rejected(&mixed, Some("AttemptId"));
    }

    for kind in ["succeeded", "active", "cancelled"] {
        let mut valid = baseline.clone();
        valid["data"]["status"] = json!({"kind":kind});
        if kind == "succeeded" {
            std::fs::write(&fixture.metadata, valid.to_string()).unwrap();
            assert!(fixture.source.result().is_ok());
        } else {
            rejected(&valid, Some("需要无待完成效果"));
        }
        for reason in [Value::Null, json!("gate")] {
            let mut invalid = valid.clone();
            invalid["data"]["status"]["reason"] = reason;
            rejected(&invalid, Some("status字段与variant不相符"));
        }
    }
    let mut blocked = baseline.clone();
    blocked["data"]["status"] = json!({"kind":"blocked","reason":"gate"});
    rejected(&blocked, Some("需要无待完成效果"));
    for (status, diagnostic) in [
        (json!({"kind":"blocked"}), "status字段与variant不相符"),
        (
            json!({"kind":"blocked","reason":null}),
            "status字段与variant不相符",
        ),
        (json!({"kind":"blocked","reason":1}), "expected value"),
        (
            json!({"kind":"blocked","reason":"gate","extra":true}),
            "unknown field",
        ),
    ] {
        let mut invalid = blocked.clone();
        invalid["data"]["status"] = status;
        rejected(&invalid, Some(diagnostic));
    }
    let canonical = baseline.to_string();
    for status in [
        r#""status":{"kind":"succeeded","kind":"succeeded"}"#,
        r#""status":{"kind":"active","kind":"succeeded"}"#,
    ] {
        let raw = canonical.replace(r#""status":{"kind":"succeeded"}"#, status);
        assert_ne!(raw, canonical);
        std::fs::write(&fixture.metadata, raw).unwrap();
        let error = fixture.source.result().unwrap_err();
        assert_eq!(error.code(), "INVALID_RESULT");
        assert!(error.to_string().contains("duplicate field"), "{error}");
    }
}

// Task: C006-T01
#[test]
fn metadata_accepts_exact_one_mib_and_rejects_one_more_byte() {
    let fixture = fixture(b"bytes", 0);
    let mut metadata = metadata(b"bytes").to_string().into_bytes();
    metadata.resize(MAX_METADATA_BYTES as usize, b' ');
    std::fs::write(&fixture.metadata, &metadata).unwrap();
    assert!(fixture.source.result().is_ok());
    metadata.push(b' ');
    std::fs::write(&fixture.metadata, &metadata).unwrap();
    assert!(fixture.source.result().is_err());
}

// Task: C006-T01
#[test]
fn receive_rejects_nonzero_exit_digest_size_and_target_writer_errors() {
    let failed = fixture(b"correct bytes", 7);
    let artifact: Artifact =
        serde_json::from_value(metadata(b"correct bytes")["data"]["artifacts"][0].clone()).unwrap();
    let error = failed
        .source
        .receive(3, &artifact, &mut Vec::new())
        .unwrap_err();
    assert!(matches!(
        error,
        sheltie_export::Error::Source {
            exit_code: Some(7),
            ..
        }
    ));
    assert!(failed.source.result().is_err());
    let fixture = fixture(b"correct bytes", 0);
    let result = fixture.source.result().unwrap();
    for bytes in [
        b"wrong  bytes".as_slice(),
        b"short".as_slice(),
        b"far longer bytes than expected".as_slice(),
    ] {
        std::fs::write(&fixture.bytes, bytes).unwrap();
        assert!(
            fixture
                .source
                .receive(3, &result.artifacts[0], &mut Vec::new())
                .is_err()
        );
    }
    std::fs::write(&fixture.bytes, b"correct bytes").unwrap();
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("target refused"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(
        fixture
            .source
            .receive(3, &result.artifacts[0], &mut Broken)
            .is_err()
    );
}

// Task: C006-T01
#[test]
fn receive_accepts_exact_32_mib_and_stops_growth_beyond_the_limit() {
    let bytes = vec![0; MAX_FILE_BYTES as usize];
    let fixture = fixture(&bytes, 0);
    let result = fixture.source.result().unwrap();
    fixture
        .source
        .receive(3, &result.artifacts[0], &mut std::io::sink())
        .unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(&fixture.bytes)
        .unwrap()
        .write_all(b"x")
        .unwrap();
    assert!(
        fixture
            .source
            .receive(3, &result.artifacts[0], &mut std::io::sink())
            .is_err()
    );
}

// Task: C006-T01
#[test]
fn source_passes_empty_unicode_and_shell_metacharacter_keys_as_literal_argv() {
    for key in ["", "-name", "--json", "中文 $() `literal`"] {
        let fixture = fixture(b"bytes", 0);
        let mut result: SelectedResult =
            serde_json::from_value(metadata(b"bytes")["data"].clone()).unwrap();
        result.artifacts[0].key = key.to_string().try_into().unwrap();
        result.artifacts[0].source.name = key.to_string().try_into().unwrap();
        fixture
            .source
            .receive(3, &result.artifacts[0], &mut Vec::new())
            .unwrap();
        let args: Vec<String> =
            serde_json::from_slice(&std::fs::read(fixture.args).unwrap()).unwrap();
        assert_eq!(args[5], format!("--artifact={key}"));
    }
}

// Task: C006-T01
#[test]
fn receive_stops_a_direct_source_child_instead_of_waiting_for_a_blocked_producer() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let binary = root.join("blocked-source.py");
    std::fs::write(&binary,"#!/usr/bin/python3\nimport sys,time\nsys.stdout.buffer.write(b'xx')\nsys.stdout.buffer.flush()\ntime.sleep(60)\n").unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let source = Source::new(&binary, &root, &work()).unwrap();
    let artifact: Artifact =
        serde_json::from_value(metadata(b"x")["data"]["artifacts"][0].clone()).unwrap();
    let started = std::time::Instant::now();
    assert!(source.receive(3, &artifact, &mut std::io::sink()).is_err());
    assert!(
        started.elapsed() < std::time::Duration::from_secs(3),
        "违规producer的60秒等待必须被终止并回收"
    );
}
