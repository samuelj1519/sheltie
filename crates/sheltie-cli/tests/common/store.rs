use super::Env;
use rusqlite::{Connection, OpenFlags, types::Value};
use std::collections::BTreeMap;

pub type StoreRows = BTreeMap<&'static str, Vec<Vec<Value>>>;

/// 保留每列的 SQLite 类型与原始内容，不把 JSON 文本重新编码。
pub fn store_rows(env: &Env) -> StoreRows {
    let connection = Connection::open_with_flags(
        env.dir.path().join("store.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let mut rows = BTreeMap::new();
    for (table, order) in [
        ("workbooks", "id, version"),
        ("works", "work_id"),
        ("work_sequence", "day"),
        ("requests", "request_id"),
        ("audit", "seq"),
    ] {
        let mut statement = connection
            .prepare(&format!("SELECT * FROM {table} ORDER BY {order}"))
            .unwrap();
        let columns = statement.column_count();
        let values = statement
            .query_map([], |row| (0..columns).map(|index| row.get(index)).collect())
            .unwrap()
            .collect::<Result<Vec<Vec<Value>>, _>>()
            .unwrap();
        rows.insert(table, values);
    }
    rows
}
