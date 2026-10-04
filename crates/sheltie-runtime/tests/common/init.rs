use super::runtime;

pub fn temporary_home() -> (tempfile::TempDir, runtime::Home) {
    let directory = tempfile::tempdir().unwrap();
    let home = runtime::Home::resolve(Some(directory.path().to_str().unwrap())).unwrap();
    (directory, home)
}

#[cfg(feature = "failpoint")]
pub fn concurrent_store_init(
    home: &runtime::Home,
    first_action: impl FnOnce(runtime::Home) -> Result<(), runtime::Error> + Send + 'static,
    second_action: impl FnOnce(runtime::Home) -> Result<(), runtime::Error> + Send + 'static,
) {
    use std::time::{Duration, Instant};

    struct Guard(Vec<std::path::PathBuf>);
    impl Drop for Guard {
        fn drop(&mut self) {
            for release in &self.0 {
                let _ = std::fs::write(release, b"release");
            }
            let _ = runtime::failpoint::disarm_rendezvous();
        }
    }

    let first_sync = tempfile::tempdir().unwrap();
    let second_sync = tempfile::tempdir().unwrap();
    std::thread::scope(|scope| {
        let _guard = Guard(vec![
            first_sync.path().join("release"),
            second_sync.path().join("release"),
        ]);
        runtime::failpoint::arm_rendezvous(
            "write_session_after_store_create",
            home.root().as_str(),
            first_sync.path(),
        )
        .unwrap();
        let first_home = home.clone();
        let first = scope.spawn(move || first_action(first_home));

        let deadline = Instant::now() + Duration::from_secs(10);
        while !first_sync.path().join("reached").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        if !first_sync.path().join("reached").exists() {
            let _ = std::fs::write(first_sync.path().join("release"), b"release");
            let _ = first.join();
            panic!("首个真实写入口未停在Store初始化窗口");
        }

        runtime::failpoint::disarm_rendezvous().unwrap();
        runtime::failpoint::arm_rendezvous(
            "home_lock_waiting",
            home.lock_path().as_str(),
            second_sync.path(),
        )
        .unwrap();
        let second_home = home.clone();
        let second = scope.spawn(move || second_action(second_home));

        let deadline = Instant::now() + Duration::from_secs(10);
        while !second_sync.path().join("reached").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        if !second_sync.path().join("reached").exists() {
            let _ = std::fs::write(first_sync.path().join("release"), b"release");
            let _ = std::fs::write(second_sync.path().join("release"), b"release");
            let _ = first.join();
            let _ = second.join();
            panic!("第二个真实写入口没有等待初始化者持有的同一把锁");
        }

        std::fs::write(first_sync.path().join("release"), b"release").unwrap();
        first.join().unwrap().unwrap();
        std::fs::write(second_sync.path().join("release"), b"release").unwrap();
        second.join().unwrap().unwrap();
    });
}
