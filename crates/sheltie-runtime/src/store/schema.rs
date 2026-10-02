//! 表结构。这里是唯一定义；改结构必须同时升 `SCHEMA_VERSION`。
//!
//! schema 4（D-041）采用顺序号、替换状态与有界理由的完整载荷；表结构沿用 C002 的
//! 请求意图、响应快照与效果登记，目录摘要仍为 `workbook-digest/v2`。
//! 较早 schema 的原库保留并拒绝，不迁移、不清空。

/// `PRAGMA user_version` 的值。
pub const SCHEMA_VERSION: i64 = 4;

/// 全部建表语句 `(表名, SQL)`。结构校验把 `sqlite_master.sql` 与这里逐表比对（去掉全部空白）。
pub const TABLES: &[(&str, &str)] = &[
    (
        "workbooks",
        "CREATE TABLE workbooks (
  id          TEXT NOT NULL,
  version     TEXT NOT NULL,
  digest      TEXT NOT NULL,
  dir         TEXT NOT NULL,
  added_at    TEXT NOT NULL,
  PRIMARY KEY (id, version)
)",
    ),
    (
        "works",
        "CREATE TABLE works (
  work_id     TEXT PRIMARY KEY,
  revision    INTEGER NOT NULL,
  status      TEXT NOT NULL,
  state_json  TEXT NOT NULL,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
)",
    ),
    (
        "work_sequence",
        "CREATE TABLE work_sequence (
  day          TEXT PRIMARY KEY,
  last         INTEGER NOT NULL
)",
    ),
    (
        "requests",
        "CREATE TABLE requests (
  request_id   TEXT PRIMARY KEY,
  intent_hash  TEXT NOT NULL,
  work_id      TEXT,
  reply_json   TEXT NOT NULL,
  effects_json TEXT NOT NULL,
  published    INTEGER NOT NULL,
  at           TEXT NOT NULL
)",
    ),
    (
        "audit",
        "CREATE TABLE audit (
  seq          INTEGER PRIMARY KEY AUTOINCREMENT,
  work_id      TEXT NOT NULL,
  revision     INTEGER NOT NULL,
  request_id   TEXT NOT NULL,
  principal    TEXT NOT NULL,
  command_json TEXT NOT NULL,
  at           TEXT NOT NULL
)",
    ),
];

/// 建库脚本：DDL 与 `PRAGMA user_version` 在**同一个事务**里执行（存储合同 §1.1），
/// 不留半结构库；首次并发建库由管理根写锁串行化。
pub fn create_script() -> String {
    let mut script = String::from("BEGIN;\n");
    for (_, sql) in TABLES {
        script.push_str(sql);
        script.push_str(";\n");
    }
    script.push_str(&format!("PRAGMA user_version = {SCHEMA_VERSION};\n"));
    script.push_str("COMMIT;\n");
    script
}

/// 去掉全部空白，用于比对。
pub fn normalize_sql(sql: &str) -> String {
    sql.chars().filter(|c| !c.is_whitespace()).collect()
}
