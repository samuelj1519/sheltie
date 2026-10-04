#![allow(dead_code)]

use std::collections::BTreeMap;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

pub type StoreRows = Vec<Vec<Vec<rusqlite::types::Value>>>;

pub fn store_rows(connection: &rusqlite::Connection) -> StoreRows {
    ["workbooks", "works", "work_sequence", "requests", "audit"]
        .into_iter()
        .map(|table| {
            let mut query = connection
                .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
                .unwrap();
            let columns = query.column_count();
            query
                .query_map([], |row| {
                    (0..columns).map(|column| row.get(column)).collect()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        })
        .collect()
}

pub fn identity_mode(metadata: &std::fs::Metadata) -> (u64, u64, u32) {
    (metadata.dev(), metadata.ino(), metadata.mode())
}

pub fn identity_mode_links(metadata: &std::fs::Metadata) -> (u64, u64, u32, u64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.mode(),
        metadata.nlink(),
    )
}

pub type TreeSnapshot = BTreeMap<PathBuf, (u64, u64, u32, u64, Vec<u8>)>;

pub fn tree_snapshot(root: &Path) -> TreeSnapshot {
    fn visit(base: &Path, path: &Path, result: &mut TreeSnapshot) {
        let metadata = std::fs::symlink_metadata(path).unwrap();
        let bytes = if metadata.is_file() {
            std::fs::read(path).unwrap()
        } else if metadata.file_type().is_symlink() {
            std::fs::read_link(path)
                .unwrap()
                .as_os_str()
                .as_encoded_bytes()
                .to_vec()
        } else {
            Vec::new()
        };
        result.insert(
            path.strip_prefix(base).unwrap().to_owned(),
            (
                metadata.dev(),
                metadata.ino(),
                metadata.mode(),
                metadata.nlink(),
                bytes,
            ),
        );
        if metadata.is_dir() {
            for child in std::fs::read_dir(path).unwrap() {
                visit(base, &child.unwrap().path(), result);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}
