//! 崩溃测试用的故障注入点。只在 `failpoint` 特性下生效。
//!
//! 命名崩溃点包括 `before_commit`、`after_commit_before_effects` 与
//! `update_between_renames`；测试 rendezvous 只在 `failpoint` feature 开启时可配置。
//! 调用处已由骨架放好；`maybe_exit` 原属 T23，因 `commit()` 一开始就调用它，
//! T13 起测试在 `--all-features` 下必经此函数，提前到 T13 填（见 plan.md T13 任务卡）。

/// 若开启特性且环境变量 `SHELTIE_FAILPOINT` 等于 `name`，以退出码 70 结束进程。否则什么都不做。
#[cfg(feature = "failpoint")]
pub fn maybe_exit(name: &str) {
    if std::env::var("SHELTIE_FAILPOINT").ok().as_deref() == Some(name) {
        std::process::exit(EXIT_CODE);
    }
}

/// 没开特性时是空函数，编译器会把它优化掉。
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

/// Arm a test-only COMMIT/effect rendezvous. No environment or host setting is modified.
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

/// Test-only rendezvous used to mutate an observed object after COMMIT and before effects.
/// The request-id guard prevents unrelated commands in a parallel test process from pausing.
pub(crate) fn rendezvous(name: &str, request_id: &str) -> std::io::Result<()> {
    #[cfg(feature = "failpoint")]
    {
        let state = RENDEZVOUS.get_or_init(|| std::sync::Mutex::new(None));
        let configured = state
            .lock()
            .map_err(|_| std::io::Error::other("rendezvous configuration lock poisoned"))?
            .clone();
        let Some(configured) =
            configured.filter(|point| point.name == name && point.request_id == request_id)
        else {
            return Ok(());
        };
        std::fs::write(configured.directory.join("reached"), name.as_bytes())?;
        while !configured.directory.join("release").exists() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    #[cfg(not(feature = "failpoint"))]
    let _ = (name, request_id);
    Ok(())
}

/// 故障注入点的退出码。
pub const EXIT_CODE: i32 = 70;
