#![forbid(unsafe_code)]
use rusqlite::{config::DbConfig, Connection, OpenFlags};
use rustix::fs::{
    fchmod, fstat, fsync, openat, renameat_with, statat, AtFlags, Dir, FileType, Mode, OFlags,
    RenameFlags,
};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn count_rows(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT count(*) FROM probe", [], |row| row.get(0))
}

fn open_regular(dir: &File, name: &str) -> Result<File, Box<dyn Error>> {
    let before = statat(dir, name, AtFlags::SYMLINK_NOFOLLOW)?;
    if FileType::from_raw_mode(before.st_mode) != FileType::RegularFile || before.st_nlink != 1 {
        return Err(format!("{name} is not a regular single-link file").into());
    }
    let fd = openat(
        dir,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    let after = fstat(&fd)?;
    if FileType::from_raw_mode(after.st_mode) != FileType::RegularFile
        || after.st_nlink != 1
        || after.st_dev != before.st_dev
        || after.st_ino != before.st_ino
    {
        return Err(format!("{name} changed identity or type while opening").into());
    }
    Ok(File::from(fd))
}

fn sqlite_probe(root: &Path) -> Result<String, Box<dyn Error>> {
    let db = root.join("store.db");
    let writer = Connection::open(&db)?;
    writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; PRAGMA user_version=1; CREATE TABLE probe(value INTEGER NOT NULL); INSERT INTO probe VALUES(7);")?;
    if !writer.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)? {
        return Err("NO_CKPT_ON_CLOSE was not enabled on the writer".into());
    }
    let sqlite_version =
        writer.query_row("SELECT sqlite_version()", [], |row| row.get::<_, String>(0))?;
    let wal = PathBuf::from(format!("{}-wal", db.display()));
    if !wal.exists() {
        return Err("bundled SQLite did not leave a committed WAL".into());
    }
    let main_before = fs::read(&db)?;
    let wal_before = fs::read(&wal)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW;

    {
        let reader = Connection::open_with_flags(&db, flags)?;
        if !reader.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)? {
            return Err("NO_CKPT_ON_CLOSE was not enabled on the reader".into());
        }
        let version =
            reader.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?;
        if version != 1 || count_rows(&reader)? != 1 {
            return Err("read-only WAL reader saw the wrong schema or committed data".into());
        }
    }
    if fs::read(&db)? != main_before || fs::read(&wal)? != wal_before {
        return Err("read-only close changed main database/WAL bytes".into());
    }

    writer.execute_batch("BEGIN IMMEDIATE; INSERT INTO probe VALUES(8);")?;
    {
        let reader = Connection::open_with_flags(&db, flags)?;
        reader.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)?;
        if count_rows(&reader)? != 1 {
            return Err("reader observed an uncommitted writer row".into());
        }
    }
    writer.execute_batch("ROLLBACK;")?;
    drop(writer);
    if fs::read(&db)? != main_before || fs::read(&wal)? != wal_before {
        return Err("writer close checkpointed despite NO_CKPT_ON_CLOSE".into());
    }

    let cold = root.join("wal-copy-without-shm");
    fs::create_dir(&cold)?;
    let cold_db = cold.join("store.db");
    let cold_wal = PathBuf::from(format!("{}-wal", cold_db.display()));
    fs::copy(&db, &cold_db)?;
    fs::copy(&wal, &cold_wal)?;
    let cold_main_before = fs::read(&cold_db)?;
    let cold_wal_before = fs::read(&cold_wal)?;
    let shm = PathBuf::from(format!("{}-shm", cold_db.display()));
    if shm.exists() {
        return Err("missing-shm fixture unexpectedly contains a wal-index".into());
    }
    {
        let reader = Connection::open_with_flags(&cold_db, flags)?;
        reader.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)?;
        if count_rows(&reader)? != 1 {
            return Err("missing-shm reader lost a committed row".into());
        }
    }
    if !shm.exists() {
        return Err("bundled SQLite did not build the permitted -shm control file".into());
    }
    if fs::read(&cold_db)? != cold_main_before || fs::read(&cold_wal)? != cold_wal_before {
        return Err("missing-shm read changed main/WAL bytes".into());
    }

    let sidecar_links = root.join("sidecar-links");
    fs::create_dir(&sidecar_links)?;
    let sidecar_dir = File::open(&sidecar_links)?;
    for (name, target) in [
        ("store.db", &db),
        ("store.db-wal", &wal),
        ("store.db-shm", &shm),
    ] {
        symlink(target, sidecar_links.join(name))?;
        let kind = statat(&sidecar_dir, name, AtFlags::SYMLINK_NOFOLLOW)?;
        if FileType::from_raw_mode(kind.st_mode) != FileType::Symlink
            || openat(
                &sidecar_dir,
                name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .is_ok()
        {
            return Err(format!(
                "SQLite sidecar symlink {name} was not rejected by the nofollow preflight"
            )
            .into());
        }
        fs::hard_link(target, sidecar_links.join(format!("{name}.hard")))?;
        let hardlink = statat(
            &sidecar_dir,
            format!("{name}.hard"),
            AtFlags::SYMLINK_NOFOLLOW,
        )?;
        if FileType::from_raw_mode(hardlink.st_mode) != FileType::RegularFile
            || hardlink.st_nlink != 2
        {
            return Err(
                format!("SQLite sidecar hard link {name} was not identified by nlink").into(),
            );
        }
    }
    let main_link = sidecar_links.join("linked-main.db");
    symlink(&db, &main_link)?;
    if Connection::open_with_flags(&main_link, flags).is_ok() {
        return Err("SQLITE_OPEN_NOFOLLOW accepted a symlink database leaf".into());
    }

    let legacy = root.join("legacy-v1.db");
    {
        let conn = Connection::open(&legacy)?;
        conn.execute_batch("PRAGMA user_version=1; CREATE TABLE legacy(v INTEGER NOT NULL);")?;
    }
    let legacy_before = fs::read(&legacy)?;
    let conn = Connection::open_with_flags(&legacy, flags)?;
    conn.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)?;
    if conn.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))? != 1 {
        return Err("legacy user_version was misread".into());
    }
    drop(conn);
    if fs::read(&legacy)? != legacy_before
        || PathBuf::from(format!("{}-wal", legacy.display())).exists()
    {
        return Err("legacy read-only rejection changed main bytes or created a WAL".into());
    }

    let truncated = root.join("truncated-main-with-valid-wal");
    fs::create_dir(&truncated)?;
    let truncated_db = truncated.join("store.db");
    let truncated_wal = PathBuf::from(format!("{}-wal", truncated_db.display()));
    let good_bytes = fs::read(&db)?;
    fs::write(&truncated_db, &good_bytes[..64])?;
    fs::copy(&wal, &truncated_wal)?;
    let truncated_main_before = fs::read(&truncated_db)?;
    let truncated_wal_before = fs::read(&truncated_wal)?;
    {
        let reader = Connection::open_with_flags(&truncated_db, flags)?;
        if !reader.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)? {
            return Err("NO_CKPT_ON_CLOSE was not enabled on truncated-main reader".into());
        }
        if count_rows(&reader)? != 1 {
            return Err(
                "valid committed WAL did not provide the truncated main database snapshot".into(),
            );
        }
    }
    if fs::read(&truncated_db)? != truncated_main_before
        || fs::read(&truncated_wal)? != truncated_wal_before
    {
        return Err("valid WAL read repaired or checkpointed the truncated main file".into());
    }

    let malformed = root.join("malformed-wal-with-valid-main");
    fs::create_dir(&malformed)?;
    let malformed_db = malformed.join("store.db");
    let malformed_wal = PathBuf::from(format!("{}-wal", malformed_db.display()));
    let malformed_writer = Connection::open(&malformed_db)?;
    malformed_writer.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; CREATE TABLE probe(value INTEGER NOT NULL); INSERT INTO probe VALUES(7); PRAGMA wal_checkpoint(TRUNCATE); INSERT INTO probe VALUES(8);",
    )?;
    if !malformed_writer.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)? {
        return Err("NO_CKPT_ON_CLOSE was not enabled on malformed-WAL fixture writer".into());
    }
    let mut malformed_wal_bytes = fs::read(&malformed_wal)?;
    if malformed_wal_bytes.len() < 32 {
        return Err("malformed-WAL fixture lacks a committed WAL frame".into());
    }
    malformed_wal_bytes[0] ^= 0xff;
    let malformed_main_copy = fs::read(&malformed_db)?;
    drop(malformed_writer);
    fs::write(&malformed_db, malformed_main_copy)?;
    fs::write(&malformed_wal, &malformed_wal_bytes)?;
    let malformed_main_before = fs::read(&malformed_db)?;
    let malformed_wal_before = fs::read(&malformed_wal)?;
    let mut malformed_visible_rows = None;
    if let Ok(reader) = Connection::open_with_flags(&malformed_db, flags) {
        reader.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)?;
        malformed_visible_rows = count_rows(&reader).ok();
    }
    if malformed_visible_rows != Some(1) {
        return Err(
            "valid main fallback was not visible after SQLite ignored the corrupt WAL".into(),
        );
    }
    if fs::read(&malformed_db)? != malformed_main_before
        || fs::read(&malformed_wal)? != malformed_wal_before
    {
        return Err("SQLite's corrupt-WAL fallback changed main/WAL bytes".into());
    }
    Ok(sqlite_version)
}

fn rustix_probe(root: &Path) -> Result<(), Box<dyn Error>> {
    let managed = root.join("managed");
    let outside = root.join("outside");
    fs::create_dir(&managed)?;
    fs::create_dir(&outside)?;
    fs::create_dir(managed.join("nested"))?;
    fs::write(managed.join("nested/output.txt"), b"observed bytes")?;
    let sentinel = outside.join("sentinel.txt");
    fs::write(&sentinel, b"outside sentinel")?;
    fs::set_permissions(&sentinel, fs::Permissions::from_mode(0o600))?;

    let root_fd = File::open(&managed)?;
    symlink(&outside, managed.join("nested-link"))?;
    if openat(
        &root_fd,
        "nested-link",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .is_ok()
    {
        return Err("NOFOLLOW accepted an ancestor symlink".into());
    }
    let dir_fd = openat(
        &root_fd,
        "nested",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    let dir = File::from(dir_fd);
    let entries = Dir::read_from(&dir)?;
    let mut names = Vec::new();
    for entry in entries {
        names.push(entry?.file_name().to_bytes().to_vec());
    }
    if !names.iter().any(|name| name == b"output.txt") {
        return Err("Dir::read_from missed output.txt".into());
    }

    let before = statat(&dir, "output.txt", AtFlags::SYMLINK_NOFOLLOW)?;
    if FileType::from_raw_mode(before.st_mode) != FileType::RegularFile || before.st_nlink != 1 {
        return Err("ordinary file type/nlink check failed".into());
    }
    let fd = openat(
        &dir,
        "output.txt",
        OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    let opened = fstat(&fd)?;
    if opened.st_dev != before.st_dev || opened.st_ino != before.st_ino || opened.st_nlink != 1 {
        return Err("openat identity check failed".into());
    }
    let mut observed = File::from(fd);
    let mut bytes = Vec::new();
    observed.read_to_end(&mut bytes)?;
    if bytes != b"observed bytes" {
        return Err("safe file returned unexpected bytes".into());
    }

    fs::write(managed.join("nested/raced.txt"), b"before replacement")?;
    let raced_before = statat(&dir, "raced.txt", AtFlags::SYMLINK_NOFOLLOW)?;
    if FileType::from_raw_mode(raced_before.st_mode) != FileType::RegularFile {
        return Err("race fixture did not start as a regular file".into());
    }
    fs::remove_file(managed.join("nested/raced.txt"))?;
    symlink(&sentinel, managed.join("nested/raced.txt"))?;
    if openat(
        &dir,
        "raced.txt",
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .is_ok()
    {
        return Err("NOFOLLOW accepted a symlink installed after stat".into());
    }

    for name in ["store.db", "store.db-wal", "store.db-shm"] {
        symlink(&sentinel, managed.join("nested").join(name))?;
        let kind = statat(&dir, name, AtFlags::SYMLINK_NOFOLLOW)?;
        if FileType::from_raw_mode(kind.st_mode) != FileType::Symlink
            || openat(
                &dir,
                name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .is_ok()
        {
            return Err(format!("NOFOLLOW did not reject SQLite-style link {name}").into());
        }
    }

    renameat_with(
        &dir,
        "output.txt",
        &dir,
        "observed.txt",
        RenameFlags::NOREPLACE,
    )?;
    symlink(&sentinel, managed.join("nested/output.txt"))?;
    fchmod(&observed, Mode::from_raw_mode(0o444))?;
    fsync(&observed)?;
    fsync(&dir)?;
    if fs::read(managed.join("nested/observed.txt"))? != b"observed bytes" {
        return Err("opened object changed after its name was replaced".into());
    }
    if fs::metadata(managed.join("nested/observed.txt"))?
        .permissions()
        .mode()
        & 0o777
        != 0o444
    {
        return Err("fchmod did not affect the observed object".into());
    }
    let readonly = openat(
        &dir,
        "observed.txt",
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    fchmod(&readonly, Mode::from_raw_mode(0o400))?;
    fsync(&readonly)?;
    if fs::metadata(managed.join("nested/observed.txt"))?
        .permissions()
        .mode()
        & 0o777
        != 0o400
    {
        return Err("fchmod on a read-only 0444 descriptor did not affect the same object".into());
    }
    if fs::read(&sentinel)? != b"outside sentinel"
        || fs::metadata(&sentinel)?.permissions().mode() & 0o777 != 0o600
    {
        return Err("path replacement affected the outside sentinel".into());
    }
    let (sync_socket, _peer) = std::os::unix::net::UnixStream::pair()?;
    if fsync(&sync_socket).is_ok() {
        return Err("fsync on socket error fixture unexpectedly succeeded".into());
    }
    if openat(
        &dir,
        "output.txt",
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .is_ok()
    {
        return Err("NOFOLLOW accepted a replacement symlink".into());
    }

    fs::hard_link(
        managed.join("nested/observed.txt"),
        managed.join("nested/hardlink.txt"),
    )?;
    let hardlink = statat(&dir, "hardlink.txt", AtFlags::SYMLINK_NOFOLLOW)?;
    if FileType::from_raw_mode(hardlink.st_mode) != FileType::RegularFile || hardlink.st_nlink != 2
    {
        return Err("hard-link fixture was not identified by nlink".into());
    }

    let fifo_path = managed.join("nested/probe.fifo");
    let status = std::process::Command::new("mkfifo")
        .arg(&fifo_path)
        .status()?;
    if !status.success() {
        return Err("host mkfifo fixture setup failed".into());
    }
    let fifo_fd = openat(
        &dir,
        "probe.fifo",
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    if FileType::from_raw_mode(fstat(&fifo_fd)?.st_mode) != FileType::Fifo {
        return Err("NONBLOCK FIFO open did not preserve its type".into());
    }
    if open_regular(&dir, "probe.fifo").is_ok() {
        return Err("regular-file preflight accepted a FIFO".into());
    }

    fs::write(managed.join("nested/source.txt"), b"source")?;
    fs::write(managed.join("nested/existing.txt"), b"destination")?;
    if renameat_with(
        &dir,
        "source.txt",
        &dir,
        "existing.txt",
        RenameFlags::NOREPLACE,
    )
    .is_ok()
    {
        return Err("NOREPLACE overwrote an existing target".into());
    }
    if fs::read(managed.join("nested/source.txt"))? != b"source"
        || fs::read(managed.join("nested/existing.txt"))? != b"destination"
    {
        return Err("failed NOREPLACE changed source/target bytes".into());
    }
    fsync(&dir)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temp = fs::canonicalize(std::env::temp_dir())?;
    let root = temp.join(format!(
        "sheltie-c002-t18-probe-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir(&root)?;
    let sqlite_root = root.join("sqlite");
    fs::create_dir(&sqlite_root)?;
    let sqlite = sqlite_probe(&sqlite_root)?;
    rustix_probe(&root)?;
    println!("T18 macOS API probe PASS; rustix=1.1.4, bundled_sqlite={sqlite}, readonly_WAL_main_and_log_unchanged=true, NO_CKPT_ON_CLOSE=true, missing_shm_rebuilt=true, NOFOLLOW/NONBLOCK/NOREPLACE/fchmod/fsync=true");
    fs::remove_dir_all(root)?;
    Ok(())
}
