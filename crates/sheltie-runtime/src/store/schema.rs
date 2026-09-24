//! 表结构。这里是唯一定义；改结构必须同时升 `SCHEMA_VERSION`。

/// `PRAGMA user_version` 的值。
pub const SCHEMA_VERSION: i64 = 1;

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
  work_id      TEXT,
  payload_hash TEXT NOT NULL,
  reply_json   TEXT NOT NULL,
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

/// 去掉全部空白，用于比对。
pub fn normalize_sql(sql: &str) -> String {
    sql.chars().filter(|c| !c.is_whitespace()).collect()
}
