use std::os::unix::fs::PermissionsExt;
fn main(){
 let p=format!("/private/tmp/sheltie-read-handle-permissions-{}",std::process::id());
 std::fs::write(&p,b"read-only").unwrap();
 std::fs::set_permissions(&p,std::fs::Permissions::from_mode(0o444)).unwrap();
 let f=std::fs::File::open(&p).unwrap();
 println!("read_open_readonly_fchmod={:?}",f.set_permissions(std::fs::Permissions::from_mode(0o444)));
}
