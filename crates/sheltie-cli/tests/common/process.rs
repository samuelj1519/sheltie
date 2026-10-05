use super::Env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

pub struct Process {
    pub(crate) child: Option<Child>,
    pub(crate) release: Option<PathBuf>,
}
impl Process {
    pub fn spawn(env: &Env, args: &[&str], point: Option<(&str, &str, &Path)>) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sheltie"));
        command.args(["--home", &env.home(), "--json"]).args(args);
        command.env_remove("SHELTIE_FAILPOINT");
        for key in [
            "SHELTIE_TEST_RENDEZVOUS_NAME",
            "SHELTIE_TEST_RENDEZVOUS_ID",
            "SHELTIE_TEST_RENDEZVOUS_DIR",
        ] {
            command.env_remove(key);
        }
        if let Some((name, id, directory)) = point {
            command
                .env("SHELTIE_TEST_RENDEZVOUS_NAME", name)
                .env("SHELTIE_TEST_RENDEZVOUS_ID", id)
                .env("SHELTIE_TEST_RENDEZVOUS_DIR", directory);
        }
        Self {
            child: Some(
                command
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            ),
            release: point.map(|(_, _, directory)| directory.join("release")),
        }
    }
    pub fn reached(&mut self, directory: &Path, point: &str) {
        self.reached_with_timeout(directory, point, Duration::from_secs(10));
    }

    pub fn reached_with_timeout(&mut self, directory: &Path, point: &str, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while !directory.join("reached").exists() {
            assert!(
                self.child.as_mut().unwrap().try_wait().unwrap().is_none(),
                "Subprocess exited before {point}"
            );
            assert!(
                Instant::now() < deadline,
                "Did not reach exact synchronization point {point}"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            std::fs::read(directory.join("reached")).unwrap(),
            point.as_bytes()
        );
    }
    pub fn release(&self) {
        std::fs::write(self.release.as_ref().unwrap(), b"release").unwrap();
    }
    pub fn finish(mut self) -> Output {
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.child.as_mut().unwrap().try_wait().unwrap().is_none() {
            assert!(Instant::now() < deadline, "Subprocess completion timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        self.child.take().unwrap().wait_with_output().unwrap()
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            if let Some(release) = &self.release {
                let _ = std::fs::write(release, b"release");
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
