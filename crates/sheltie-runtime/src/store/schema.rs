//! Sole table-structure authority; schema changes must increment SCHEMA_VERSION.
//!
//! Schema 4 (D-041) uses complete sequence/replacement/bounded-reason payloads, retaining C002's
//! request intents, response snapshots, and effect registration; directory digests remain workbook-digest/v2.
//! Preserve and reject older databases without migrating or clearing them.

/// PRAGMA user_version value.
pub const SCHEMA_VERSION: i64 = 4;

/// All table DDL (name, SQL); validation compares sqlite_master.sql table-by-table after removing whitespace.
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

/// Initialize DDL and PRAGMA user_version within one transaction (storage §1.1),
/// without partial schemas; the root lock serializes concurrent initial creation.
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

/// Remove all whitespace for comparison.
pub fn normalize_sql(sql: &str) -> String {
    sql.chars().filter(|c| !c.is_whitespace()).collect()
}
