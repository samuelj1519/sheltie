//! 管理根 `~/.sheltie` 与路径约束。见 `specs/contracts/storage.md` §6、`specs/architecture.md` §5。

use sheltie_core::ids::WorkId;
use sheltie_core::path::{AbsPath, RelPath};

use crate::error::{Error, Result};

/// 把路径规范化到「最深已存在祖先的真实位置 + 余下原样段」。
/// 全路径已存在时等价于 `canonicalize`；尚不存在的尾部保持词法形式。
fn canonicalize_deepest(path: &camino::Utf8Path) -> Result<camino::Utf8PathBuf> {
    let mut probe = path.to_path_buf();
    let mut tail: Vec<String> = Vec::new();
    loop {
        match std::fs::canonicalize(&probe) {
            Ok(real) => {
                let mut out = camino::Utf8PathBuf::from_path_buf(real).map_err(|path| {
                    Error::InvalidRequest {
                        reason: format!("管理根的真实祖先 {} 不是UTF-8路径", path.display()),
                    }
                })?;
                for seg in tail.iter().rev() {
                    out = out.join(seg);
                }
                return Ok(out);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let Some(name) = probe.file_name().map(|n| n.to_string()) else {
                    return Err(Error::io(probe.as_str(), e));
                };
                tail.push(name);
                let Some(parent) = probe.parent().map(|p| p.to_path_buf()) else {
                    return Err(Error::io(probe.as_str(), e));
                };
                probe = parent;
            }
            Err(error) => return Err(Error::io(probe.as_str(), error)),
        }
    }
}

/// 管理根。只有 runtime 能在它下面写东西。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    root: AbsPath,
}

impl Home {
    /// 解析管理根：`cli` 参数 > 环境变量 `SHELTIE_HOME` > `$HOME/.sheltie`。
    /// 相对路径按当前目录转成绝对路径。不建目录。
    ///
    /// 根在入口一次规范化：对最深已存在祖先做 `canonicalize`，再拼回余下段。
    /// 之后所有 managed 路径都从这个规范根派生，祖先软链（如 `/tmp` 一类）不会
    /// 让派生路径与 `confine` 的前缀比较失真（架构 §5）。
    pub fn resolve(cli: Option<&str>) -> Result<Self> {
        let configured = |name: &str| -> Result<Option<String>> {
            std::env::var_os(name)
                .map(|value| {
                    value.into_string().map_err(|_| Error::InvalidRequest {
                        reason: format!("{name} 不是UTF-8文本"),
                    })
                })
                .transpose()
        };
        let given = match cli {
            Some(path) => path.to_string(),
            None => match configured("SHELTIE_HOME")? {
                Some(path) => path,
                None => format!(
                    "{}/.sheltie",
                    configured("HOME")?.ok_or_else(|| Error::InvalidRequest {
                        reason: "既没有 --home 与 SHELTIE_HOME，也取不到 $HOME".to_string(),
                    })?
                ),
            },
        };
        let root = if camino::Utf8Path::new(&given).is_absolute() {
            camino::Utf8PathBuf::from(given)
        } else {
            let cwd = std::env::current_dir().map_err(|e| Error::io(".", e))?;
            camino::Utf8PathBuf::from_path_buf(cwd.join(&given)).map_err(|path| {
                Error::InvalidRequest {
                    reason: format!("管理根的当前目录 {} 不是UTF-8路径", path.display()),
                }
            })?
        };
        Ok(Self {
            root: AbsPath::new(canonicalize_deepest(&root)?.to_string())?,
        })
    }

    pub fn root(&self) -> &AbsPath {
        &self.root
    }

    pub fn store_path(&self) -> AbsPath {
        self.root.join_segment("store.db")
    }

    pub fn workbooks_dir(&self) -> AbsPath {
        self.root.join_segment("workbooks")
    }

    pub fn staging_dir(&self) -> AbsPath {
        self.workbooks_dir().join_segment(".staging")
    }

    pub fn works_dir(&self) -> AbsPath {
        self.root.join_segment("works")
    }

    pub fn tmp_dir(&self) -> AbsPath {
        self.root.join_segment("tmp")
    }

    /// 成功CLI写后的独立维护；不创建根、锁或tmp，不参与请求恢复与业务提交。
    pub fn cleanup_tmp(&self) -> Result<()> {
        let lock = match self.acquire_existing_lock() {
            Ok(Some(lock)) => lock,
            Ok(None) | Err(Error::NotFound { .. }) => return Ok(()),
            Err(error) => return Err(error),
        };
        crate::fsx::ManagedFs::open_existing(self)?
            .cleanup_expired_tmp(&lock, std::time::SystemTime::now())
    }

    /// 管理根写锁 `<root>/.lock`（存储合同 §2.2，D-035 的 `fs4`）。
    pub fn lock_path(&self) -> AbsPath {
        self.root.join_segment(".lock")
    }

    /// `pending/`：引擎持锁创建的私有暂存（未提交准备区、已提交未发布原件、待删除目录）。
    pub fn pending_dir(&self) -> AbsPath {
        self.root.join_segment("pending")
    }

    /// 把经过校验的根内路径拼回绝对路径。非法登记路径必须报错，不能退回管理根。
    pub fn rel(&self, rel: &str) -> Result<AbsPath> {
        let rel = crate::fsx::ManagedRelPath::new(rel)?;
        AbsPath::new(format!(
            "{}/{}",
            self.root.as_str().trim_end_matches('/'),
            rel.as_str()
        ))
        .map_err(Error::Core)
    }

    /// 绝对路径相对管理根的形式；不在根内时报错。
    pub fn to_rel(&self, path: &AbsPath) -> Result<String> {
        path.as_path()
            .strip_prefix(self.root.as_path())
            .map(|p| p.to_string())
            .map_err(|_| Error::InvalidRequest {
                reason: format!("{path} 不在管理根 {} 之内", self.root),
            })
    }

    pub fn bin_dir(&self) -> AbsPath {
        self.root.join_segment("bin")
    }

    pub fn work_dir(&self, id: &WorkId) -> AbsPath {
        self.works_dir().join_segment(id.as_str())
    }

    pub fn workbook_dir(&self, id: &str, version: &str) -> AbsPath {
        self.workbooks_dir().join_segment(id).join_segment(version)
    }

    /// 把外部给的相对路径限制在 `base` 之下。
    ///
    /// 拒绝：绝对路径、含 `..`、空段。若拼出的路径已存在，`canonicalize` 后必须仍以 `base` 的
    /// 规范形式为前缀。路径不合法返回 `Error::Core(InvalidPath)`，I/O 错误保留路径与原因。
    pub fn confine(base: &AbsPath, rel: &str) -> Result<AbsPath> {
        // 先按写法拒绝：`RelPath` 的构造就是这套检查。
        let rel = RelPath::new(rel)?;
        let joined = base.join(&rel);
        // base 不存在时其下不可能有已存在路径，写法检查已足够。
        let base_canon = match std::fs::canonicalize(base.as_path()) {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(joined),
            Err(error) => return Err(Error::io(base.as_str(), error)),
        };
        crate::failpoint::rendezvous("confine_after_base_canonicalize", base.as_str())
            .map_err(|error| Error::io(base.as_str(), error))?;
        // 找最近的存在祖先（叶与中间段可能还没建出来），对它 canonicalize 比前缀。
        let mut probe = joined.as_path().to_path_buf();
        loop {
            match std::fs::symlink_metadata(probe.as_std_path()) {
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(Error::io(probe.as_str(), e)),
            }
            match probe.parent() {
                Some(p) if p != base.as_path() => probe = p.to_path_buf(),
                _ => return Ok(joined),
            }
        }
        let canon =
            std::fs::canonicalize(probe.as_std_path()).map_err(|e| Error::io(probe.as_str(), e))?;
        if !canon.starts_with(&base_canon) {
            return Err(Error::Core(sheltie_core::Error::InvalidPath {
                path: joined.to_string(),
                reason: "经符号链接逃出了根",
            }));
        }
        Ok(joined)
    }
}

/// 管理根写锁的守卫（存储合同 §2.2，D-035 的 `fs4`）。取得锁前只创建管理根与
/// `.lock` 本身；进程退出由 OS 释放。drop 时解锁。守卫记下取得锁时的管理根与
/// 锁对象身份（dev/inode），供获锁后的 §2.2 复核用。
#[derive(Debug)]
pub struct HomeLock {
    _file: std::fs::File,
    root: AbsPath,
    lock_path: AbsPath,
    locked_ident: (u64, u64),
    root_ident: (u64, u64),
}

fn file_ident(file: &std::fs::File) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt as _;
    file.metadata().ok().map(|m| (m.dev(), m.ino()))
}

impl HomeLock {
    pub(crate) fn locked_identity(&self) -> (u64, u64) {
        self.locked_ident
    }

    /// §2.2 复核：管理根与 `.lock` 路径仍存在，且与取得锁时是同一对象（同 dev/inode）。
    /// 根/锁若被锁外因素替换，调用方必须释放旧锁并整体重试；正常purge保留根锁。
    pub fn identity_still_valid(&self) -> bool {
        use std::os::unix::fs::MetadataExt as _;
        let Ok(lock) = std::fs::symlink_metadata(self.lock_path.as_path()) else {
            return false;
        };
        let Ok(root) = std::fs::symlink_metadata(self.root.as_path()) else {
            return false;
        };
        lock.file_type().is_file()
            && lock.nlink() == 1
            && (lock.dev(), lock.ino()) == self.locked_ident
            && root.file_type().is_dir()
            && (root.dev(), root.ino()) == self.root_ident
    }

    pub(crate) fn matches_root(&self, root: &AbsPath, ident: (u64, u64)) -> bool {
        self.root == *root && self.root_ident == ident && self.identity_still_valid()
    }
}

impl Home {
    /// 排他取得管理根写锁（阻塞等待本地协作进程）。根不存在时只在锁前创建根与`.lock`。
    /// 获锁后复核根与锁对象身份（§2.2），不沿锁外替换后的旧inode继续写。
    pub fn acquire_lock(&self) -> Result<HomeLock> {
        const RETRY_LIMIT: u32 = 16;
        let mut last_miss = None;
        for _ in 0..RETRY_LIMIT {
            match self.acquire_lock_once() {
                Ok(guard) => {
                    if guard.identity_still_valid() {
                        return Ok(guard);
                    }
                    // drop(guard)：释放落在旧 inode 上的锁，下一轮在新根上重建 .lock。
                }
                Err(e) => {
                    // 建根/建 .lock 的窗口里根被锁外移除同样需要整体重试；
                    // 不把锁外替换/移除误报成普通 I/O 失败。
                    let gone = self.is_lock_setup_path_missing(&e);
                    if !gone {
                        return Err(e);
                    }
                    last_miss = Some(e);
                }
            }
        }
        Err(Error::io(
            self.lock_path().as_str(),
            std::io::Error::other(match last_miss {
                Some(e) => format!("取管理根写锁连续被锁外替换打断（{e}）"),
                None => "复核管理根写锁身份连续失败（根或.lock可能正被锁外替换）".to_string(),
            }),
        ))
    }

    /// Wait on an existing managed-root lock without creating the root or `.lock` file.
    pub(crate) fn acquire_existing_lock(&self) -> Result<Option<HomeLock>> {
        let managed = crate::fsx::ManagedFs::open_existing(self)?;
        let Some(file) = managed.open_existing_lock_file()? else {
            return Ok(None);
        };
        let lock_path = self.lock_path();
        crate::failpoint::rendezvous("home_existing_lock_opened", lock_path.as_str())
            .map_err(|error| Error::io(lock_path.as_str(), error))?;
        let Some(locked_ident) = file_ident(&file) else {
            return Err(Error::io(
                lock_path.as_str(),
                std::io::Error::other("无法读取已存在.lock的文件身份"),
            ));
        };
        let root_ident = managed.identity();
        match fs4::fs_std::FileExt::try_lock_exclusive(&file) {
            Ok(true) => {}
            Ok(false) => {
                crate::failpoint::rendezvous("home_lock_waiting", lock_path.as_str())
                    .map_err(|error| Error::io(lock_path.as_str(), error))?;
                fs4::fs_std::FileExt::lock_exclusive(&file)
                    .map_err(|error| Error::io(lock_path.as_str(), error))?;
            }
            Err(error) => return Err(Error::io(lock_path.as_str(), error)),
        }
        let guard = HomeLock {
            _file: file,
            root: self.root.clone(),
            lock_path,
            locked_ident,
            root_ident,
        };
        if !guard.identity_still_valid() {
            return Err(Error::InvalidRequest {
                reason: "等待期间管理根或.lock对象被替换；拒绝创建或继续访问Store".into(),
            });
        }
        Ok(Some(guard))
    }

    fn acquire_lock_once(&self) -> Result<HomeLock> {
        let lock_path = self.lock_path();
        let managed = crate::fsx::ManagedFs::create_root(&self.root)?;
        // 不跟随叶链接、不接受FIFO或多链接；不得截断已存在的锁文件。
        let file = managed.open_lock_file()?;
        // 身份在等锁前记下：复核比对的是「打开的那个对象」与「路径现在指向的对象」。
        // 取不到身份说明根或锁在打开窗口中消失，整体重试（§2.2）。
        let Some(locked_ident) = file_ident(&file) else {
            return Err(Error::io(
                lock_path.as_str(),
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "取不到 .lock 的文件身份（根在记身份时消失？）",
                ),
            ));
        };
        let root_ident = managed.identity();
        match fs4::fs_std::FileExt::try_lock_exclusive(&file) {
            Ok(true) => {}
            Ok(false) => {
                crate::failpoint::rendezvous("home_lock_waiting", lock_path.as_str())
                    .map_err(|error| Error::io(lock_path.as_str(), error))?;
                fs4::fs_std::FileExt::lock_exclusive(&file)
                    .map_err(|error| Error::io(lock_path.as_str(), error))?;
            }
            Err(error) => return Err(Error::io(lock_path.as_str(), error)),
        }
        Ok(HomeLock {
            _file: file,
            root: self.root.clone(),
            lock_path,
            locked_ident,
            root_ident,
        })
    }

    fn is_lock_setup_path_missing(&self, error: &Error) -> bool {
        let under_root =
            |path: &str| std::path::Path::new(path).starts_with(self.root.as_path().as_std_path());
        match error {
            Error::NotFound { what } => under_root(what),
            Error::Io { path, source } => {
                under_root(path) && source.kind() == std::io::ErrorKind::NotFound
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod lock_retry_tests {
    use super::Home;
    use crate::error::Error;
    use sheltie_core::path::AbsPath;

    // Task: C002-T19
    #[test]
    fn lock_setup_retries_structured_missing_path_under_managed_root() {
        let temp = tempfile::tempdir().unwrap();
        let parent = AbsPath::new(
            std::fs::canonicalize(temp.path())
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        )
        .unwrap();
        let home =
            Home::resolve(Some((parent.join_segment("sheltie-test-home")).as_str())).unwrap();
        assert!(home.is_lock_setup_path_missing(&Error::NotFound {
            what: home.lock_path().to_string(),
        }));
        assert!(!home.is_lock_setup_path_missing(&Error::NotFound {
            what: format!("{}-old/.lock", home.root()),
        }));
        assert!(!home.is_lock_setup_path_missing(&Error::InvalidRequest {
            reason: "不是瞬时缺失".to_string(),
        }));
    }

    // Task: C002-T24
    #[cfg(feature = "failpoint")]
    #[test]
    fn existing_lock_removed_after_open_is_never_recreated() {
        use std::time::{Duration, Instant};

        let _serial = crate::failpoint::RENDEZVOUS_TEST_LOCK.lock().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("home");
        std::fs::create_dir(&root).unwrap();
        let home = Home::resolve(Some(
            (AbsPath::new(root.to_string_lossy().into_owned()).unwrap()).as_str(),
        ))
        .unwrap();
        let held = home.acquire_lock().unwrap();
        let rendezvous = tempfile::tempdir().unwrap();
        crate::failpoint::arm_rendezvous(
            "home_existing_lock_opened",
            home.lock_path().as_str(),
            rendezvous.path(),
        )
        .unwrap();
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                crate::failpoint::disarm_rendezvous().unwrap();
            }
        }
        let _guard = Guard;
        let child_home = home.clone();
        let child = std::thread::spawn(move || child_home.acquire_existing_lock());

        let deadline = Instant::now() + Duration::from_secs(10);
        while !rendezvous.path().join("reached").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        if !rendezvous.path().join("reached").exists() {
            let _ = std::fs::write(rendezvous.path().join("release"), b"release");
            drop(held);
            let _ = child.join();
            panic!("既有锁没有在CREATE=false句柄打开后停住");
        }
        std::fs::remove_file(home.lock_path().as_path()).unwrap();
        drop(held);
        std::fs::write(rendezvous.path().join("release"), b"release").unwrap();
        assert!(matches!(
            child.join().unwrap(),
            Err(Error::InvalidRequest { .. })
        ));
        assert!(!home.lock_path().as_path().exists());
        assert!(home.root().as_path().is_dir());
    }
}

#[cfg(test)]
mod setup_error_contract_tests {
    use super::*;
    use crate::fsx::permission_test_support::PermissionRestore;

    // Task: C002-T50
    #[test]
    fn setup_io_retry_is_confined_to_a_missing_management_object() {
        let directory = tempfile::tempdir().unwrap();
        let home = Home::resolve(Some(directory.path().to_str().unwrap())).unwrap();
        let missing_inside = Error::io(
            home.lock_path().as_str(),
            std::io::Error::from(std::io::ErrorKind::NotFound),
        );
        assert!(home.is_lock_setup_path_missing(&missing_inside));
        for (path, kind) in [
            (
                home.lock_path().to_string(),
                std::io::ErrorKind::PermissionDenied,
            ),
            (
                format!("{}-other/.lock", home.root()),
                std::io::ErrorKind::NotFound,
            ),
        ] {
            assert!(!home.is_lock_setup_path_missing(&Error::io(path, std::io::Error::from(kind))));
        }
        let denied = PermissionRestore::deny(home.root().as_path().as_std_path());
        let result = home.acquire_lock();
        drop(denied);
        let Error::Io { path, source } = result.unwrap_err() else {
            panic!("real denied root must return its original I/O error")
        };
        assert_eq!(path, home.root().as_str());
        assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(!home.lock_path().as_path().exists());
        assert!(!home.store_path().as_path().exists());
    }

    // Task: C002-T50
    #[test]
    fn confined_missing_paths_are_lexical_and_do_not_create_ancestors() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let base = AbsPath::new(root.to_str().unwrap()).unwrap();
        let path = Home::confine(&base, "new/leaf.txt").unwrap();
        assert_eq!(path.as_str(), root.join("new/leaf.txt").to_str().unwrap());
        assert!(!root.join("new").exists());
        let absent_base = AbsPath::new(root.join("not-created-base").to_str().unwrap()).unwrap();
        let absent_target = Home::confine(&absent_base, "new/leaf.txt").unwrap();
        assert_eq!(
            absent_target.as_str(),
            root.join("not-created-base/new/leaf.txt").to_str().unwrap()
        );
        assert!(!root.join("not-created-base").exists());
        std::fs::write(root.join("occupied"), b"retain").unwrap();
        let error = Home::confine(&base, "occupied/sub/leaf.txt").unwrap_err();
        let Error::Io { path, source } = error else {
            panic!("non-directory ancestor must retain I/O cause")
        };
        assert_eq!(path, root.join("occupied/sub/leaf.txt").to_str().unwrap());
        assert_eq!(source.kind(), std::io::ErrorKind::NotADirectory);
        assert_eq!(std::fs::read(root.join("occupied")).unwrap(), b"retain");
    }

    // Task: C002-T50
    #[cfg(feature = "failpoint")]
    #[test]
    fn a_confined_missing_target_remains_lexical_when_its_base_disappears() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let original_base = root.join("base");
        std::fs::create_dir(&original_base).unwrap();
        std::fs::write(original_base.join("original"), b"retain base material").unwrap();
        let base = AbsPath::new(original_base.to_str().unwrap()).unwrap();
        let expected = original_base.join("new.txt");
        assert_eq!(
            Home::confine(&base, "new.txt").unwrap().as_str(),
            expected.to_str().unwrap()
        );
        let reader_base = base.clone();
        let moved = root.join("moved-base");
        let result = crate::fsx::controlled_object_tests::observe_change(
            "confine_after_base_canonicalize",
            base.as_str(),
            move || Home::confine(&reader_base, "new.txt"),
            || {
                std::fs::rename(&original_base, &moved).unwrap();
            },
        );
        assert_eq!(result.unwrap().as_str(), expected.to_str().unwrap());
        assert!(!original_base.exists());
        assert_eq!(
            std::fs::read(moved.join("original")).unwrap(),
            b"retain base material"
        );
        assert!(!moved.join("new.txt").exists());
    }
}
