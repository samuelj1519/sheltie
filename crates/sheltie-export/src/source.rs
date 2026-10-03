use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use sheltie_core::ids::WorkId;

use crate::model::{Artifact, MAX_FILE_BYTES, MAX_METADATA_BYTES, SelectedResult};
use crate::{Error, Result};

#[derive(Debug)]
pub struct Source {
    binary: PathBuf,
    home: PathBuf,
    work: WorkId,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultEnvelope {
    ok: bool,
    data: SelectedResult,
    next: Vec<serde_json::Value>,
}

struct SourceChild(Child);
impl Drop for SourceChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Source {
    pub fn new(binary: &Path, home: &Path, work: &WorkId) -> Result<Self> {
        if !binary.is_absolute() || !home.is_absolute() {
            return Err(Error::Rejected {
                code: "INVALID_ARGUMENT",
                message: "sheltie和home必须是明确绝对路径".into(),
            });
        }
        Ok(Self {
            binary: binary.into(),
            home: home.into(),
            work: work.clone(),
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.binary);
        command
            .arg("--home")
            .arg(&self.home)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        command
    }

    fn spawn(&self, command: &mut Command) -> Result<SourceChild> {
        command
            .spawn()
            .map(SourceChild)
            .map_err(|source| Error::Io {
                path: self.binary.clone(),
                operation: "spawn source",
                source,
            })
    }

    pub fn result(&self) -> Result<SelectedResult> {
        let mut command = self.command();
        command.args(["--json", "work", "result", self.work.as_str()]);
        let mut child = self.spawn(&mut command)?;
        let stdout = child.0.stdout.take().ok_or_else(|| Error::Source {
            message: "源未提供stdout管道".into(),
            exit_code: None,
        })?;
        let mut bytes = Vec::new();
        stdout
            .take(MAX_METADATA_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|source| Error::Io {
                path: self.binary.clone(),
                operation: "read result metadata",
                source,
            })?;
        if bytes.len() as u64 > MAX_METADATA_BYTES {
            return Err(Error::Rejected {
                code: "INVALID_RESULT",
                message: "结果metadata超过1MiB".into(),
            });
        }
        let status = child.0.wait().map_err(|source| Error::Io {
            path: self.binary.clone(),
            operation: "wait source metadata",
            source,
        })?;
        if !status.success() {
            return Err(Error::Source {
                message: "结果metadata进程非零退出".into(),
                exit_code: status.code(),
            });
        }
        let envelope: ResultEnvelope =
            serde_json::from_slice(&bytes).map_err(|error| Error::Rejected {
                code: "INVALID_RESULT",
                message: format!("结果完整载荷解码失败：{error}"),
            })?;
        if !envelope.ok || !envelope.next.is_empty() {
            return Err(Error::Rejected {
                code: "INVALID_RESULT",
                message: "最终结果的成功封装/next不符".into(),
            });
        }
        envelope.data.validate(&self.work)?;
        Ok(envelope.data)
    }

    pub fn receive(
        &self,
        revision: u64,
        artifact: &Artifact,
        writer: &mut impl Write,
    ) -> Result<()> {
        if revision == 0 || artifact.bytes > MAX_FILE_BYTES {
            return Err(Error::Rejected {
                code: "INVALID_RESULT",
                message: "revision或单文件声明上限不符".into(),
            });
        }
        let mut command = self.command();
        command
            .args(["work", "result", self.work.as_str()])
            .arg(format!("--artifact={}", artifact.key.as_str()))
            .arg("--revision")
            .arg(revision.to_string());
        let mut child = self.spawn(&mut command)?;
        let mut stdout = child.0.stdout.take().ok_or_else(|| Error::Source {
            message: "源未提供stdout管道".into(),
            exit_code: None,
        })?;
        let mut buffer = [0_u8; 65_536];
        let mut digest = Sha256::new();
        let mut bytes = 0_u64;
        loop {
            let read = stdout.read(&mut buffer).map_err(|source| Error::Io {
                path: artifact.path.as_path().into(),
                operation: "read source bytes",
                source,
            })?;
            if read == 0 {
                break;
            }
            bytes = bytes
                .checked_add(read as u64)
                .ok_or_else(|| Error::Integrity {
                    path: artifact.path.as_path().into(),
                    message: "源实际字节数溢出".into(),
                })?;
            if bytes > MAX_FILE_BYTES || bytes > artifact.bytes {
                return Err(Error::Integrity {
                    path: artifact.path.as_path().into(),
                    message: "源实际字节超过冻结声明/32MiB".into(),
                });
            }
            writer
                .write_all(&buffer[..read])
                .map_err(|source| Error::Io {
                    path: artifact.path.as_path().into(),
                    operation: "write received bytes",
                    source,
                })?;
            digest.update(&buffer[..read]);
        }
        let status = child.0.wait().map_err(|source| Error::Io {
            path: self.binary.clone(),
            operation: "wait source bytes",
            source,
        })?;
        if !status.success() {
            return Err(Error::Source {
                message: "源字节进程非零退出".into(),
                exit_code: status.code(),
            });
        }
        if bytes != artifact.bytes || format!("{:x}", digest.finalize()) != artifact.sha256.as_str()
        {
            return Err(Error::Integrity {
                path: artifact.path.as_path().into(),
                message: "源实际大小/sha256与冻结引用不符".into(),
            });
        }
        writer.flush().map_err(|source| Error::Io {
            path: artifact.path.as_path().into(),
            operation: "flush received bytes",
            source,
        })
    }
}
