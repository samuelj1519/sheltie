//! C002-T09：`workbook-digest/v2` 目录摘要的独立向量与拒绝例。
//! 全部期望摘要值由 python3 hashlib 对手工拼出的字节流独立算出，不用生产 helper 生成。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use sheltie_core::digest::Sha256Hex;
use sheltie_runtime::Error;
use sheltie_runtime::workbook_digest::digest_dir_v2;
use tempfile::TempDir;

fn abs(p: &Path) -> sheltie_core::path::AbsPath {
    sheltie_core::path::AbsPath::new(p.to_str().unwrap()).unwrap()
}

fn write(dir: &Path, rel: &str, content: &[u8]) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn digest_of(setup: impl FnOnce(&Path)) -> (TempDir, Sha256Hex) {
    let dir = tempfile::tempdir().unwrap();
    setup(dir.path());
    let d = digest_dir_v2(&abs(dir.path())).unwrap();
    (dir, d)
}

// python3: sha256(b"sheltie-workbook-digest/v2\0" + be64(0)).hexdigest()
const EMPTY: &str = "fcf10ccb454182a0fff58fbc01f0920bd084cf3367c59c2a18aa27d2712a9f36";
// 单文件 a.txt = "hello"
const SINGLE: &str = "c760ce85310cbd4e93450c0133c6d7141fa4216e63194dcf0a920af16b8ab583";
// O07 的旧碰撞对（旧算法摘要相同）：
const ONE_FILE_ZB: &str = "3947b322dce3c579c952a2db5e37ab0c9fb368616f29e232c17b2125b58e0594";
const TWO_FILES_EDGE: &str = "e850aeabe439c8901e0043c289834ecb612b0b45a34b18b70e4f48dbdf5657ae";
// 字节序排序：B.md(0x42) 排在 a.md(0x61) 前
const BYTE_ORDER: &str = "31d5fff6b14c23b7901075b1d61958d2c6cd4ef0eb1ce264ab03b783d0f1e063";
const PATH_CHANGE: &str = "98be5ca87a5fbab4eeab3fa5009ef2811eb2313fd6136bb3a995b9e6a76f88d1";
const CONTENT_CHANGE: &str = "ee29a7d533c38805a80af8523513d3775de7697a36e4f6cafe623c7bb2005c2b";

// Task: C002-T09
#[test]
fn digest_matches_independent_vectors_on_disk() {
    let (_g, d) = digest_of(|_| {});
    assert_eq!(d.as_str(), EMPTY);

    let (_g, d) = digest_of(|dir| write(dir, "a.txt", b"hello"));
    assert_eq!(d.as_str(), SINGLE);

    // 嵌套与多文件；空目录不参与摘要。
    let (_g, d) = digest_of(|dir| {
        write(dir, "flows/default.toml", b"x");
        write(dir, "workbook.toml", b"y");
        write(dir, "z", b"");
        std::fs::create_dir_all(dir.join("empty-dir")).unwrap();
    });
    assert_eq!(
        d.as_str(),
        "fdd25e354069ab8fb9c69bbc4b3a886e62305e466af854e3ab66ea9d1106e176"
    );
}

// Task: C002-T09
#[test]
fn framing_collision_pair_now_yields_two_different_digests() {
    // 旧算法里 za="zb\0X" 与（za=""、zb="X"）碰撞；v2 帧定界后必须分开。
    let (_g, one) = digest_of(|dir| write(dir, "za", b"zb\0X"));
    assert_eq!(one.as_str(), ONE_FILE_ZB);
    let (_g, two) = digest_of(|dir| {
        write(dir, "za", b"");
        write(dir, "zb", b"X");
    });
    assert_eq!(two.as_str(), TWO_FILES_EDGE);
    assert_ne!(one, two);
}

// Task: C002-T09
#[test]
fn insertion_order_is_irrelevant_but_bytes_and_paths_matter() {
    let build = |dir: &Path, order: &[&str]| {
        for name in order {
            write(dir, name, name.as_bytes());
        }
    };
    let (_g, a) = digest_of(|dir| build(dir, &["b.md", "a.md"]));
    let (_g, b) = digest_of(|dir| build(dir, &["a.md", "b.md"]));
    assert_eq!(a, b, "目录枚举顺序不影响摘要（按路径字节序排序）");

    // 只改一个字节的内容。
    let (_g, c) = digest_of(|dir| {
        write(dir, "a.md", b"a.md");
        write(dir, "b.md", b"b.me");
    });
    assert_ne!(a, c);
    // 只改路径一个字符。
    let (_g, p) = digest_of(|dir| {
        write(dir, "a.md", b"a.md");
        write(dir, "c.md", b"b.md");
    });
    assert_ne!(a, p);
    // 对照已知向量：改路径/内容后的摘要等于独立算出的常量。
    let (_g, single_changed_content) = digest_of(|dir| write(dir, "a.txt", b"hellp"));
    assert_eq!(single_changed_content.as_str(), CONTENT_CHANGE);
    let (_g, single_changed_path) = digest_of(|dir| write(dir, "b.txt", b"hello"));
    assert_eq!(single_changed_path.as_str(), PATH_CHANGE);
}

// Task: C002-T09
#[test]
fn paths_sort_by_raw_bytes_not_by_locale() {
    // B(0x42) < a(0x61)：字节序把 B.md 排在 a.md 前，与区域设置无关。
    let (_g, d) = digest_of(|dir| {
        write(dir, "a.md", b"2");
        write(dir, "B.md", b"1");
    });
    assert_eq!(d.as_str(), BYTE_ORDER);
}

// Task: C002-T09
#[test]
fn symlink_in_tree_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "real.txt", b"x");
    std::os::unix::fs::symlink("real.txt", dir.path().join("link.txt")).unwrap();
    match digest_dir_v2(&abs(dir.path())) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("符号链接"), "{reason}"),
        other => panic!("应当拒绝符号链接：{other:?}"),
    }
}

// Task: C002-T09
#[test]
fn hardlink_in_tree_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "a.txt", b"x");
    std::fs::hard_link(dir.path().join("a.txt"), dir.path().join("b.txt")).unwrap();
    match digest_dir_v2(&abs(dir.path())) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("硬链接"), "{reason}"),
        other => panic!("应当拒绝硬链接：{other:?}"),
    }
}

// Task: C002-T09
#[test]
fn file_size_limit_is_exactly_32_mib() {
    // 恰好 32 MiB 的稀疏文件接受；多一字节拒绝（读取前按元数据核对）。
    let dir = tempfile::tempdir().unwrap();
    let f = std::fs::File::create(dir.path().join("big.bin")).unwrap();
    f.set_len(32 * 1024 * 1024).unwrap();
    drop(f);
    assert!(digest_dir_v2(&abs(dir.path())).is_ok());

    let dir = tempfile::tempdir().unwrap();
    let f = std::fs::File::create(dir.path().join("big.bin")).unwrap();
    f.set_len(32 * 1024 * 1024 + 1).unwrap();
    drop(f);
    match digest_dir_v2(&abs(dir.path())) {
        Err(Error::InvalidRequest { reason }) => {
            assert!(reason.contains("超过"), "{reason}")
        }
        other => panic!("超限文件应当拒绝：{other:?}"),
    }
}

// Task: C002-T09
#[test]
fn total_size_limit_is_exactly_256_mib() {
    // 8 × 32 MiB = 256 MiB 接受；再加一个字节文件即超总量，读取前拒绝。
    let dir = tempfile::tempdir().unwrap();
    for i in 0..8 {
        let f = std::fs::File::create(dir.path().join(format!("p{i}.bin"))).unwrap();
        f.set_len(32 * 1024 * 1024).unwrap();
    }
    assert!(digest_dir_v2(&abs(dir.path())).is_ok());

    write(dir.path(), "one-more", b"x");
    match digest_dir_v2(&abs(dir.path())) {
        Err(Error::InvalidRequest { reason }) => {
            assert!(reason.contains("总量"), "{reason}")
        }
        other => panic!("总量超限应当拒绝：{other:?}"),
    }
}

// Task: C002-T21
#[test]
fn tree_reader_rejects_symlinked_root() {
    let outer = tempfile::tempdir().unwrap();
    let real = outer.path().join("real");
    std::fs::create_dir(&real).unwrap();
    write(&real, "workbook.toml", b"content");
    let root_link = outer.path().join("root-link");
    std::os::unix::fs::symlink(&real, &root_link).unwrap();
    assert!(matches!(
        digest_dir_v2(&abs(&root_link)),
        Err(Error::InvalidRequest { .. })
    ));
}

// Task: C002-T21
#[test]
fn tree_reader_rejects_symlinked_parent_directory() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "sentinel.txt", b"unchanged");
    std::os::unix::fs::symlink(outside.path(), dir.path().join("linked-parent")).unwrap();
    match digest_dir_v2(&abs(dir.path())) {
        Err(Error::InvalidRequest { reason }) => assert!(reason.contains("符号链接"), "{reason}"),
        other => panic!("应当拒绝目录内的父软链：{other:?}"),
    }
    assert_eq!(
        std::fs::read(outside.path().join("sentinel.txt")).unwrap(),
        b"unchanged"
    );
}
