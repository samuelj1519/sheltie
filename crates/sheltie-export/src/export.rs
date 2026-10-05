use std::io::{self, Write};
use std::path::Path;

use sheltie_core::ids::WorkId;

use crate::failpoint::{self, Point};
use crate::output::Report;
use crate::source::Source;
use crate::target::Destination;

pub fn copy(binary: &Path, home: &Path, work: &WorkId, parent: &Path) -> Report {
    failpoint::install_target_hook();
    let destination = match Destination::open(home, parent) {
        Ok(destination) => destination,
        Err(error) => return Report::failure(Some(work.clone()), None, None, error),
    };
    let source = match Source::new(binary, home, work) {
        Ok(source) => source,
        Err(error) => return Report::failure(Some(work.clone()), None, None, error),
    };
    let result = match source.result() {
        Ok(result) => result,
        Err(error) => return Report::failure(Some(work.clone()), None, None, error),
    };
    let revision = Some(result.revision);
    let mut staging = match destination.stage(work) {
        Ok(staging) => staging,
        Err(error) => return Report::failure(Some(work.clone()), revision, None, error),
    };
    failpoint::checkpoint(Point::StagingCreated);
    let outcome = (|| {
        let mut files = Vec::with_capacity(result.artifacts.len());
        for (index, artifact) in result.artifacts.iter().enumerate() {
            let mut writer = staging.create_artifact(index, artifact)?;
            source.receive(
                result.revision,
                artifact,
                &mut ReceivingWriter {
                    writer: &mut writer,
                    notified: false,
                },
            )?;
            files.push(staging.finish_artifact(writer, artifact)?);
        }
        staging.publish(&result, &files)
    })();
    match outcome {
        Ok(target) => Report::complete(work.clone(), result.revision, target),
        Err(error) => Report::failure(Some(work.clone()), revision, staging.staging_path(), error),
    }
}

struct ReceivingWriter<'a, W> {
    writer: &'a mut W,
    notified: bool,
}

impl<W: Write> Write for ReceivingWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(bytes)?;
        if written > 0 && !self.notified {
            self.notified = true;
            failpoint::checkpoint(Point::DuringReceive);
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::output::Status;
    use std::os::unix::fs::PermissionsExt;

    // Task: C006-T01
    #[test]
    fn directory_authority_is_validated_before_the_source_binary_can_execute() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let home = root.join("home");
        let parent = root.join("copies");
        std::fs::create_dir(&home).unwrap();
        std::fs::create_dir(&parent).unwrap();
        let home_alias = root.join("home-alias");
        let parent_alias = root.join("parent-alias");
        std::os::unix::fs::symlink(&home, &home_alias).unwrap();
        std::os::unix::fs::symlink(&parent, &parent_alias).unwrap();
        let marker = root.join("source-executed");
        let binary = root.join("source.py");
        std::fs::write(
            &binary,
            format!(
                "#!/usr/bin/python3\nfrom pathlib import Path\nPath({:?}).write_bytes(b'executed')\nprint('{{}}')\n",
                marker.to_str().unwrap()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let work = WorkId::parse("2026-09-24-001-export").unwrap();

        let control = copy(&binary, &home, &work, &parent);
        assert_eq!(control.error.unwrap().code, "INVALID_RESULT");
        assert_eq!(std::fs::read(&marker).unwrap(), b"executed");
        std::fs::remove_file(&marker).unwrap();

        for (tested_home, tested_parent) in [
            (home_alias, parent.clone()),
            (home.clone(), parent_alias),
            (home.join("."), parent.clone()),
            (home.clone(), parent.join(".")),
            (home.clone(), home.clone()),
            (home.clone(), root.clone()),
        ] {
            let report = copy(&binary, &tested_home, &work, &tested_parent);
            assert!(
                !marker.exists(),
                "Source binary executed before directory validation: home={tested_home:?}, parent={tested_parent:?}"
            );
            assert_eq!(report.status, Status::Rejected);
            assert_eq!(report.exit_code(), 2);
            assert_eq!(report.error.unwrap().code, "INVALID_TARGET");
            assert_eq!(report.revision, None);
            assert_eq!(report.target_path, None);
            assert_eq!(report.staging_path, None);
            assert_eq!(std::fs::read_dir(&parent).unwrap().count(), 0);
        }
    }
}
