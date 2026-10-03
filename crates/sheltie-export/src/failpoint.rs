#[derive(Debug, Clone, Copy)]
pub(crate) enum Point {
    StagingCreated,
    DuringReceive,
}

pub(crate) fn checkpoint(point: Point) {
    let name = match point {
        Point::StagingCreated => "staging_created",
        Point::DuringReceive => "during_receive",
    };
    rendezvous(name);
}

pub(crate) fn install_target_hook() {
    #[cfg(feature = "failpoint")]
    {
        crate::target::install_boundary_hook(target_boundary);
        if let Ok(name) = std::env::var("SHELTIE_EXPORT_TEST_SYNC_ERROR") {
            if let Some(point) = target_point(&name) {
                crate::target::arm_sync_error(point);
            }
        }
    }
}

#[cfg(feature = "failpoint")]
fn target_point(name: &str) -> Option<crate::target::TargetPoint> {
    use crate::target::TargetPoint;
    match name {
        "after_file_sync" => Some(TargetPoint::AfterFileSync),
        "before_readback" => Some(TargetPoint::BeforeReadback),
        "after_readback" => Some(TargetPoint::AfterReadback),
        "during_manifest_write" => Some(TargetPoint::DuringManifestWrite),
        "after_manifest_write" => Some(TargetPoint::AfterManifestWrite),
        "before_tree_sync" => Some(TargetPoint::BeforeTreeSync),
        "before_rename" => Some(TargetPoint::BeforeRename),
        "after_rename_before_parent_sync" => Some(TargetPoint::AfterRenameBeforeParentSync),
        "after_parent_sync" => Some(TargetPoint::AfterParentSync),
        _ => None,
    }
}

#[cfg(feature = "failpoint")]
fn target_boundary(point: crate::target::TargetPoint) {
    use crate::target::TargetPoint;
    let name = match point {
        TargetPoint::AfterFileSync => "after_file_sync",
        TargetPoint::BeforeReadback => "before_readback",
        TargetPoint::AfterReadback => "after_readback",
        TargetPoint::DuringManifestWrite => "during_manifest_write",
        TargetPoint::AfterManifestWrite => "after_manifest_write",
        TargetPoint::BeforeTreeSync => "before_tree_sync",
        TargetPoint::BeforeRename => "before_rename",
        TargetPoint::AfterRenameBeforeParentSync => "after_rename_before_parent_sync",
        TargetPoint::AfterParentSync => "after_parent_sync",
    };
    rendezvous(name);
}

#[cfg(feature = "failpoint")]
fn rendezvous(name: &str) {
    if std::env::var("SHELTIE_EXPORT_TEST_POINT").ok().as_deref() != Some(name) {
        return;
    }
    if let Err(error) = wait_at_boundary(name) {
        eprintln!("export test boundary {name} failed: {error}");
        std::process::exit(71);
    }
}

#[cfg(feature = "failpoint")]
fn wait_at_boundary(name: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    let directory = std::env::var_os("SHELTIE_EXPORT_TEST_DIRECTORY")
        .map(PathBuf::from)
        .ok_or_else(|| std::io::Error::other("测试同步点缺少明确目录"))?;
    if !directory.is_absolute()
        || !std::fs::symlink_metadata(&directory)?.is_dir()
        || directory.canonicalize()? != directory
    {
        return Err(std::io::Error::other("测试同步点必须使用真实绝对目录"));
    }
    let mut reached = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("reached"))?;
    reached.write_all(name.as_bytes())?;
    let deadline = Instant::now() + Duration::from_secs(15);
    while !directory.join("release").exists() {
        if Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "测试同步点未释放",
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}

#[cfg(not(feature = "failpoint"))]
fn rendezvous(_name: &str) {}
