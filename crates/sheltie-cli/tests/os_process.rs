//! OS/CLI boundary regression, moved from runtime with original Task ownership/oracles.
#![allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "../../sheltie-runtime/tests/common/mod.rs"]
mod runtime_common;
use runtime_common::*;
fn sheltie_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_sheltie"))
}

// Task: C002-T15
#[test]
fn install_modify_path_flag_is_rejected() {
    // N05: host-rc writing removed (INV-3); --modify-path is no longer valid.
    let (d, home) = temp_home();
    let fake_home = d.path().join("fakehome");
    std::fs::create_dir_all(&fake_home).unwrap();
    let out = std::process::Command::new(sheltie_bin())
        .env("HOME", &fake_home)
        .env("SHELL", "/bin/zsh")
        .args([
            "--home",
            home.root().as_str(),
            "self",
            "install",
            "--modify-path",
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "self install --modify-path unexpectedly succeeded"
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "Unknown parameters yield protocol exit code 2 (clap usage error)"
    );
    assert!(
        !fake_home.join(".zshrc").exists(),
        "Removed entry still wrote shell rc"
    );
    assert!(
        !fake_home.join(".bashrc").exists(),
        "Removed entry still wrote shell rc"
    );
}
fn account_name_oracle() -> String {
    let out = std::process::Command::new("id")
        .arg("-un")
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// Use actual OS identity: start a Work through real CLI with forged USER; audit principal differs from
/// the forgery and matches independent id -un output (D-036 verification).
// Task: C002-T05
#[test]
fn audit_principal_ignores_spoofed_user_env() {
    let (_d, home) = temp_home();
    repo(&home)
        .add(&abs(&example_dir("two-step")), None)
        .unwrap();

    let out = std::process::Command::new(sheltie_bin())
        .env("USER", "spoofed-auditor")
        .env("USERNAME", "spoofed-auditor")
        .env("SHELTIE_HOME", home.root().as_str())
        .args([
            "work",
            "start",
            "--workbook",
            "two-step",
            "--flow",
            "default",
            "--input",
            "topic=t",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "start failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let conn = rusqlite::Connection::open_with_flags(
        home.store_path().as_str(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let principal: String = conn
        .query_row(
            "SELECT principal FROM audit ORDER BY seq DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_ne!(principal, "spoofed-auditor");
    assert_eq!(principal, account_name_oracle());
}

// Task: C002-T33
#[test]
fn home_resolution_rejects_unrepresentable_roots_without_selecting_an_alternative() {
    use std::os::unix::ffi::OsStringExt as _;
    use std::process::Command;
    let directory = tempfile::tempdir().unwrap();
    let invalid = std::ffi::OsString::from_vec(vec![0xff]);
    for case in ["environment", "home_environment", "override"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sheltie"));
        command.args(["--json", "workbook", "list"]);
        match case {
            "environment" => {
                command
                    .env("SHELTIE_HOME", &invalid)
                    .env("HOME", directory.path());
            }
            "home_environment" => {
                command.env_remove("SHELTIE_HOME").env("HOME", &invalid);
            }
            "override" => {
                command
                    .arg("--home")
                    .arg(directory.path().join("valid-root"))
                    .env("SHELTIE_HOME", &invalid);
            }
            _ => unreachable!(),
        }
        let output = command.output().unwrap();
        let reply: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(!output.status.success(), "{case}");
        assert_eq!(
            reply["error"]["code"],
            if case == "override" {
                "NOT_FOUND"
            } else {
                "INVALID_REQUEST"
            },
            "{case}: {reply}"
        );
    }
    assert!(!directory.path().join("valid-root").exists());
    assert!(!directory.path().join(".sheltie").exists());
}
