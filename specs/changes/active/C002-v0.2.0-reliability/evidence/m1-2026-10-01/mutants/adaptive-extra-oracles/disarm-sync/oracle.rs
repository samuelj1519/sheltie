#![allow(clippy::unwrap_used, clippy::expect_used)]
use sheltie_runtime::fsx::{ManagedFs, ManagedRelPath};
use sheltie_runtime::{Home, failpoint};
#[test]
fn disarmed_parent_sync_fault_does_not_poison_real_atomic_write() {
    let directory = tempfile::tempdir_in("/private/tmp").unwrap();
    let home = Home::resolve(Some(directory.path().to_str().unwrap())).unwrap();
    let lock = home.acquire_lock().unwrap();
    let filesystem = ManagedFs::open_existing(&home).unwrap();
    failpoint::arm_sync_error(home.root().as_str(), "managed_file_parent_sync").unwrap();
    failpoint::disarm_sync_error().unwrap();
    let result = filesystem.write_atomic(&lock, &ManagedRelPath::new("independent.txt").unwrap(), b"independent bytes");
    println!("SYNC_ORACLE {}", serde_json::json!({"result":format!("{result:?}"),"bytes":std::fs::read(directory.path().join("independent.txt")).ok()}));
    failpoint::disarm_sync_error().unwrap();
    assert!(result.is_ok(), "SEMANTIC explicit disarm left the named sync failure active: {result:?}");
    assert_eq!(std::fs::read(directory.path().join("independent.txt")).unwrap(), b"independent bytes");
}
