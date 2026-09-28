use std::os::unix::fs::{symlink, PermissionsExt};
use sheltie_runtime::{Home, fsx::SafeFile};
fn main() {
    let dir = format!("/private/tmp/sheltie-safe-seal-probe-{}",std::process::id());
    std::fs::create_dir(&dir).unwrap();
    let outer=Home::resolve(Some(&dir)).unwrap();
    let managed=format!("{dir}/home");
    std::fs::create_dir(&managed).unwrap();
    let home=Home::resolve(Some(&managed)).unwrap();
    let path=home.root().join_segment("output");
    let original=home.root().join_segment("original-renamed");
    let sentinel=outer.root().join_segment("outside-sentinel");
    std::fs::write(path.as_path(), b"observed-artifact").unwrap();
    std::fs::write(sentinel.as_path(), b"sentinel").unwrap();
    std::fs::set_permissions(sentinel.as_path(),std::fs::Permissions::from_mode(0o600)).unwrap();
    let safe=SafeFile::open_regular(&path).unwrap();
    let observed=safe.sha256_bounded(1024).unwrap();
    std::fs::rename(path.as_path(), original.as_path()).unwrap();
    symlink(sentinel.as_path(),path.as_path()).unwrap();
    let result=safe.set_readonly();
    println!("path={path}; observed_bytes={}; seal_result={result:?}; sentinel_mode={:o}; original_mode={:o}", observed.1,std::fs::metadata(sentinel.as_path()).unwrap().permissions().mode()&0o777,std::fs::metadata(original.as_path()).unwrap().permissions().mode()&0o777);
}
