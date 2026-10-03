//! `workbook-digest/v2` 的目录摘要单元（存储合同 §5.1）。
//!
//! 精确字节流：域前缀、BE64 文件数、按规范 UTF-8 相对路径字节序排序的每个普通文件
//! `BE64(路径长) || 路径 || BE64(内容长) || 内容`，最后只做一次 SHA256。流式读取并
//! 准确计数，读取间的增长或收缩都会被拒，不能用更大的内容绕过限额。
//!
//! 生产调用方（`WorkbookRepo::digest_dir` 等 schema 1 路径）按 C002 计划在 T07 一次
//! 切换到本单元；切换前旧算法不与新算法并存于同一字段。

use std::io::Read;

use sha2::Digest as _;
use sheltie_core::digest::{Sha256Hex, WORKBOOK_DIGEST_V2_PREFIX, be64, digest_v2_file_frame};
use sheltie_core::flow::{ResourceIndex, ResourceMeta};
use sheltie_core::path::{AbsPath, RelPath};

use crate::error::{Error, Result};
use crate::fsx::{ExternalReadTree, MAX_FILE_BYTES};

/// 目录摘要 `workbook-digest/v2`。
pub fn digest_dir_v2(dir: &AbsPath) -> Result<Sha256Hex> {
    let tree = ExternalReadTree::open(dir)?;
    digest_tree_v2(&tree)
}

pub(crate) fn digest_managed_dir_v2(home: &crate::home::Home, dir: &AbsPath) -> Result<Sha256Hex> {
    let tree = ExternalReadTree::open_managed(home, dir)?;
    digest_tree_v2(&tree)
}

pub(crate) fn digest_tree_v2(tree: &ExternalReadTree) -> Result<Sha256Hex> {
    inspect_tree_v2(tree, &std::collections::BTreeMap::new()).map(|(digest, _, _)| digest)
}

pub(crate) fn inspect_tree_v2(
    tree: &ExternalReadTree,
    captured: &std::collections::BTreeMap<RelPath, Vec<u8>>,
) -> Result<(
    Sha256Hex,
    ResourceIndex,
    std::collections::BTreeMap<RelPath, Sha256Hex>,
)> {
    let files = tree.files();
    // 元数据首轮先核完全部限额，再打开任何正文（storage §5.1）。
    tree.validate_sizes()?;

    let mut hasher = sha2::Sha256::new();
    hasher.update(WORKBOOK_DIGEST_V2_PREFIX);
    hasher.update(be64(files.len() as u64));
    let mut resources = std::collections::BTreeMap::new();
    let mut file_hashes = std::collections::BTreeMap::new();
    for file in files {
        // 相对路径以 UTF-8 字节排序，不做 Unicode/大小写转换。
        let path_bytes = file.relative.as_str().as_bytes();
        hasher.update(digest_v2_file_frame(path_bytes, file.bytes));
        if let Some(content) = captured.get(&file.relative) {
            if content.len() as u64 != file.bytes {
                return Err(Error::InvalidRequest {
                    reason: format!("{} 在读取间改变", file.relative),
                });
            }
            hasher.update(content);
            file_hashes.insert(file.relative.clone(), Sha256Hex::of_bytes(content));
            resources.insert(
                file.relative.clone(),
                ResourceMeta {
                    bytes: file.bytes,
                    is_utf8: std::str::from_utf8(content).is_ok(),
                },
            );
        } else {
            let mut source = tree.open_file(file)?;
            let (is_utf8, file_hash) = stream_file_into(
                &mut source,
                &file.relative.to_string(),
                file.bytes,
                &mut hasher,
            )?;
            file_hashes.insert(file.relative.clone(), file_hash);
            resources.insert(
                file.relative.clone(),
                ResourceMeta {
                    bytes: file.bytes,
                    is_utf8,
                },
            );
        }
    }
    tree.validate_unchanged()?;
    // 单次 SHA256 收口：finalize 的输出直接转十六进制，不再二次哈希。
    Ok((
        Sha256Hex::from_sha256(hasher.finalize()),
        ResourceIndex { files: resources },
        file_hashes,
    ))
}

/// 把一个文件的内容流式喂进哈希器，并核对实际字节数与帧头声明一致。
/// 读取间增长或收缩都拒绝：帧头的长度是摘要流的一部分，事后无法补写。
fn stream_file_into(
    file: &mut impl Read,
    path: &str,
    declared: u64,
    hasher: &mut sha2::Sha256,
) -> Result<(bool, Sha256Hex)> {
    let mut actual = 0u64;
    let mut utf8 = Utf8State::default();
    let mut file_hasher = sha2::Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| Error::io(path, e))?;
        if n == 0 {
            break;
        }
        actual = actual
            .checked_add(n as u64)
            .ok_or_else(|| Error::InvalidRequest {
                reason: format!("{path} 读取字节数溢出"),
            })?;
        if actual > declared || actual > MAX_FILE_BYTES {
            return Err(Error::InvalidRequest {
                reason: format!("{path} 在读取间变大，与帧头不符"),
            });
        }
        hasher.update(&buf[..n]);
        file_hasher.update(&buf[..n]);
        utf8.push(&buf[..n]);
    }
    if actual != declared {
        return Err(Error::InvalidRequest {
            reason: format!("{path} 实际 {actual} 字节与帧头声明 {declared} 不符（读取间变化）"),
        });
    }
    Ok((
        utf8.is_valid(),
        Sha256Hex::from_sha256(file_hasher.finalize()),
    ))
}

struct Utf8State {
    pending: Vec<u8>,
    valid: bool,
}

impl Default for Utf8State {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            valid: true,
        }
    }
}

impl Utf8State {
    fn push(&mut self, chunk: &[u8]) {
        if !self.valid {
            return;
        }
        let mut combined = std::mem::take(&mut self.pending);
        combined.extend_from_slice(chunk);
        match std::str::from_utf8(&combined) {
            Ok(_) => self.valid = true,
            Err(error) if error.error_len().is_none() => {
                self.valid = true;
                self.pending
                    .extend_from_slice(&combined[error.valid_up_to()..]);
            }
            Err(_) => {
                self.valid = false;
                self.pending.clear();
            }
        }
    }

    fn is_valid(&self) -> bool {
        self.valid && self.pending.is_empty()
    }
}

#[cfg(test)]
mod tree_snapshot_tests {
    use super::*;
    use crate::fsx::ExternalReadTree;
    use std::os::unix::fs::MetadataExt as _;

    // Task: C002-T21
    #[test]
    fn digest_and_resource_facts_use_the_bytes_consumed_by_each_reader() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.toml"), b"abc").unwrap();
        std::fs::write(dir.path().join("asset.bin"), b"ok").unwrap();
        let tree =
            ExternalReadTree::open(&AbsPath::new(dir.path().to_str().unwrap()).unwrap()).unwrap();
        let path = RelPath::new("a.toml").unwrap();
        let captured = tree.read_file(&path, MAX_FILE_BYTES).unwrap();
        let before = std::fs::metadata(dir.path().join("a.toml")).unwrap();
        std::fs::write(dir.path().join("a.toml"), b"xyz").unwrap();
        std::fs::write(dir.path().join("asset.bin"), [0xff, 0xfe]).unwrap();
        let after = std::fs::metadata(dir.path().join("a.toml")).unwrap();
        assert_eq!(before.ino(), after.ino(), "反例保持同一inode");
        assert_eq!(before.len(), after.len(), "反例保持同一长度");

        let captured = std::collections::BTreeMap::from([(path, captured)]);
        let (digest, resources, _) = inspect_tree_v2(&tree, &captured).unwrap();
        assert_eq!(
            digest.as_str(),
            "a0f849d6ac09cc3a1fdcf37a5f494c6c3dd881aba1ac5f54ce1a67bc8944bd01"
        );
        assert!(
            resources
                .get(&RelPath::new("a.toml").unwrap())
                .unwrap()
                .is_utf8
        );
        assert!(
            !resources
                .get(&RelPath::new("asset.bin").unwrap())
                .unwrap()
                .is_utf8
        );
    }

    // Task: C002-T21
    #[test]
    fn utf8_checker_handles_chunk_boundaries_and_rejects_bad_or_truncated_sequences() {
        let mut valid = Utf8State::default();
        let mut first = vec![b'a'; 65_535];
        first.push(0xe4);
        valid.push(&first);
        assert!(!valid.is_valid(), "尚未收到续字节时暂不算完整UTF-8");
        valid.push(&[0xb8, 0xad]);
        assert!(valid.is_valid(), "跨64KiB边界的字符应当接受");

        let mut bad_continuation = Utf8State::default();
        bad_continuation.push(&[0xe4, b'A']);
        assert!(!bad_continuation.is_valid(), "错误续字节应拒绝");

        let mut truncated = Utf8State::default();
        truncated.push(&[0xe4, 0xb8]);
        assert!(!truncated.is_valid(), "EOF残尾应拒绝");
    }
}

#[cfg(test)]
mod bounded_stream_contract_tests {
    use super::*;

    struct FailOnFurtherRead {
        file: std::fs::File,
        first_read: bool,
    }

    impl Read for FailOnFurtherRead {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if !self.first_read {
                return Err(std::io::Error::other("later external Read failed"));
            }
            self.first_read = false;
            self.file.read(buffer)
        }
    }

    // Task: C002-T53
    #[test]
    fn streamed_file_hashes_and_utf8_facts_match_independent_complete_bytes() {
        let mut boundary = vec![b'a'; 65_535];
        boundary.extend_from_slice(&[0xe4, 0xb8, 0xad]);
        let mut invalid_prefix = vec![0xff];
        invalid_prefix.extend_from_slice(&boundary);
        for bytes in [
            b"data".to_vec(),
            vec![0xe4, 0xb8, 0xad],
            vec![0xe4, 0xb8],
            boundary,
            invalid_prefix,
        ] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("data");
            std::fs::write(&path, &bytes).unwrap();
            let mut file = std::fs::File::open(&path).unwrap();
            let declared = file.metadata().unwrap().len();
            let header = b"independent preceding frame bytes";
            let mut hasher = sha2::Sha256::new();
            hasher.update(header);
            let (utf8, hash) = stream_file_into(&mut file, "data", declared, &mut hasher).unwrap();
            assert_eq!(utf8, std::str::from_utf8(&bytes).is_ok());
            assert_eq!(hash.as_str(), format!("{:x}", sha2::Sha256::digest(&bytes)));
            let expected = [header.as_slice(), bytes.as_slice()].concat();
            assert_eq!(
                format!("{:x}", hasher.finalize()),
                format!("{:x}", sha2::Sha256::digest(&expected))
            );
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
    }

    // Task: C002-T53
    #[test]
    fn an_observed_frame_growth_is_refused_before_a_later_external_read_error() {
        use std::io::Write as _;
        for grows in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("data");
            std::fs::write(&path, b"data").unwrap();
            let file = std::fs::File::open(&path).unwrap();
            let declared = file.metadata().unwrap().len();
            assert_eq!(declared, 4);
            if grows {
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&path)
                    .unwrap()
                    .write_all(b"+")
                    .unwrap();
            }
            let mut source = FailOnFurtherRead {
                file,
                first_read: true,
            };
            let mut hasher = sha2::Sha256::new();
            let error = stream_file_into(&mut source, "data", declared, &mut hasher).unwrap_err();
            if grows {
                let Error::InvalidRequest { reason } = error else {
                    panic!("the already observed invalid frame must be refused before another Read")
                };
                assert_eq!(reason, "data 在读取间变大，与帧头不符");
                assert_eq!(std::fs::read(&path).unwrap(), b"data+");
            } else {
                let Error::Io {
                    path: reported_path,
                    source,
                } = error
                else {
                    panic!("a valid first frame followed by an external IO error must remain IO")
                };
                assert_eq!(reported_path, "data");
                assert_eq!(source.to_string(), "later external Read failed");
                assert_eq!(std::fs::read(&path).unwrap(), b"data");
            }
        }
    }
}
