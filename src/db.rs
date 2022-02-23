pub use rusqlite::{params, Connection, Result as SQLRes};
use serenity::prelude::{Mutex, TypeMapKey};
use std::{collections::HashMap, sync::Arc};

pub struct ZweiDbConn;
impl TypeMapKey for ZweiDbConn {
    type Value = Arc<Mutex<Connection>>;
}

pub fn get_all_prefixes(conn: &Connection) -> SQLRes<HashMap<u64, String>> {
    let mut prep = conn.prepare("SELECT server, prefix FROM prefixes")?;
    let mut result = prep.query([])?;

    let mut pfxs = HashMap::new();
    while let Some(result_row) = result.next()? {
        let row = result_row;
        let id: i64 = row.get(0)?;
        let pfx = row.get(1)?;
        pfxs.insert(id as u64, pfx);
    }
    Ok(pfxs)
}

pub fn set_prefix(conn: &Connection, guild: u64, pfx: String) -> SQLRes<usize> {
    conn.execute(
        "INSERT OR REPLACE INTO prefixes VALUES ($1, $2)",
        params![guild as i64, pfx],
    )
}

pub fn remove_prefix(conn: &Connection, guild: u64) -> SQLRes<usize> {
    conn.execute("DELETE FROM prefixes WHERE server IS $1", [guild as i64])
}
