//! Crash-test fault injection, enabled only by the failpoint feature.
//!
//! Named crash points include before_commit, after_commit_before_effects,
//! update_between_renames, delete_after_first_payload_child, and
//! delete_after_tree_removed_before_marker; configure test rendezvous only with the failpoint feature.
//! The skeleton fixes call sites. maybe_exit was planned for T23, but commit calls it immediately;
//! all-feature tests require it from T13, so implementation moved to T13 (plan.md task card).

/// With the feature enabled and SHELTIE_FAILPOINT equal to name, exit with code 70; otherwise do nothing.
#[cfg(feature = "failpoint")]
pub fn maybe_exit(name: &str) {
    // A subprocess can pause at exactly the same point before the parent sends SIGKILL.
    if let Err(error) = rendezvous(name, name) {
        eprintln!("test checkpoint {name} failed: {error}");
        std::process::exit(71);
    }
    if std::env::var("SHELTIE_FAILPOINT").ok().as_deref() == Some(name) {
        std::process::exit(EXIT_CODE);
    }
}

/// Without the feature, the compiler eliminates this no-op.
#[cfg(not(feature = "failpoint"))]
pub fn maybe_exit(_name: &str) {}

#[cfg(feature = "failpoint")]
#[derive(Clone)]
struct Rendezvous {
    name: String,
    request_id: String,
    directory: std::path::PathBuf,
}

#[cfg(feature = "failpoint")]
static RENDEZVOUS: std::sync::OnceLock<std::sync::Mutex<Option<Rendezvous>>> =
    std::sync::OnceLock::new();

#[cfg(feature = "failpoint")]
static SYNC_ERROR: std::sync::OnceLock<std::sync::Mutex<Option<SyncFailure>>> =
    std::sync::OnceLock::new();

#[cfg(feature = "failpoint")]
#[derive(Clone)]
struct SyncFailure {
    root: String,
    name: String,
}

#[cfg(all(test, feature = "failpoint"))]
pub(crate) static RENDEZVOUS_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Configure a name/scope-matched test synchronization point without changing environment or host configuration.
pub fn arm_rendezvous(
    name: &str,
    request_id: &str,
    directory: &std::path::Path,
) -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    {
        let state = RENDEZVOUS.get_or_init(|| std::sync::Mutex::new(None));
        let mut state = state
            .lock()
            .map_err(|_| std::io::Error::other("rendezvous configuration lock poisoned"))?;
        *state = Some(Rendezvous {
            name: name.to_string(),
            request_id: request_id.to_string(),
            directory: directory.to_path_buf(),
        });
    }
    #[cfg(not(feature = "failpoint"))]
    let _ = (name, request_id, directory);
    Ok(())
}

/// Disarm a test-only rendezvous.
pub fn disarm_rendezvous() -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    {
        let state = RENDEZVOUS.get_or_init(|| std::sync::Mutex::new(None));
        let mut state = state
            .lock()
            .map_err(|_| std::io::Error::other("rendezvous configuration lock poisoned"))?;
        *state = None;
    }
    Ok(())
}

/// Arm one named sync failure for deterministic recovery tests.
pub fn arm_sync_error(root: &str, name: &str) -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    {
        let state = SYNC_ERROR.get_or_init(|| std::sync::Mutex::new(None));
        let mut state = state
            .lock()
            .map_err(|_| std::io::Error::other("sync failure configuration lock poisoned"))?;
        *state = Some(SyncFailure {
            root: root.to_string(),
            name: name.to_string(),
        });
    }
    #[cfg(not(feature = "failpoint"))]
    let _ = (root, name);
    Ok(())
}

/// Disarm the named sync failure.
pub fn disarm_sync_error() -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    {
        let state = SYNC_ERROR.get_or_init(|| std::sync::Mutex::new(None));
        let mut state = state
            .lock()
            .map_err(|_| std::io::Error::other("sync failure configuration lock poisoned"))?;
        *state = None;
    }
    Ok(())
}

/// Return an injected I/O error at a named durable sync boundary.
pub(crate) fn sync_error(root: &str, name: &str) -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    {
        let state = SYNC_ERROR.get_or_init(|| std::sync::Mutex::new(None));
        let mut state = state
            .lock()
            .map_err(|_| std::io::Error::other("sync failure configuration lock poisoned"))?;
        if state
            .as_ref()
            .is_some_and(|failure| failure.root == root && failure.name == name)
        {
            *state = None;
            return Err(std::io::Error::other(format!(
                "injected sync failure: {name}"
            )));
        }
    }
    #[cfg(not(feature = "failpoint"))]
    let _ = (root, name);
    Ok(())
}

/// Inject a recoverable maintenance cleanup failure in subprocess integration tests.
pub(crate) fn cleanup_error(name: &str) -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    if std::env::var("SHELTIE_FAILPOINT").ok().as_deref() == Some(name) {
        return Err(std::io::Error::other(format!(
            "injected cleanup failure: {name}"
        )));
    }
    #[cfg(not(feature = "failpoint"))]
    let _ = name;
    Ok(())
}

/// Match only the specified test name/scope, excluding unrelated I/O in parallel processes.
pub(crate) fn rendezvous(name: &str, request_id: &str) -> std::io::Result<()> {
    rendezvous_payload(name, request_id, name.as_bytes())
}

pub(crate) fn rendezvous_observed_path(
    name: &str,
    request_id: &str,
    observed_path: &str,
) -> std::io::Result<()> {
    rendezvous_payload(name, request_id, observed_path.as_bytes())
}

fn rendezvous_payload(name: &str, request_id: &str, payload: &[u8]) -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    {
        let state = RENDEZVOUS.get_or_init(|| std::sync::Mutex::new(None));
        let configured = state
            .lock()
            .map_err(|_| std::io::Error::other("rendezvous configuration lock poisoned"))?
            .clone();
        let configured = configured
            .filter(|point| point.name == name && point.request_id == request_id)
            .or_else(|| {
                let configured_name = std::env::var("SHELTIE_TEST_RENDEZVOUS_NAME").ok()?;
                let configured_id = std::env::var("SHELTIE_TEST_RENDEZVOUS_ID").ok()?;
                let directory = std::env::var_os("SHELTIE_TEST_RENDEZVOUS_DIR")?;
                (configured_name == name && configured_id == request_id).then(|| Rendezvous {
                    name: configured_name,
                    request_id: configured_id,
                    directory: std::path::PathBuf::from(directory),
                })
            });
        let Some(configured) = configured else {
            return Ok(());
        };
        let temporary = configured
            .directory
            .join(format!("reached-{}.tmp", uuid::Uuid::now_v7().simple()));
        std::fs::write(&temporary, payload)?;
        std::fs::rename(&temporary, configured.directory.join("reached"))?;
        while !configured.directory.join("release").exists() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    #[cfg(not(feature = "failpoint"))]
    let _ = (name, request_id, payload);
    Ok(())
}

/// Fault-injection exit code.
pub const EXIT_CODE: i32 = 70;
