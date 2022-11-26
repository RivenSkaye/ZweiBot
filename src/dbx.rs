use serenity::prelude::TypeMapKey;
pub use sqlx::{
    pool::PoolConnection as Connection, query, query_as, Result as SQLRes, Sqlite,
    SqlitePool as Pool,
};
use std::collections::HashMap;

pub struct ZweiDbConn;
impl TypeMapKey for ZweiDbConn {
    type Value = Pool;
}

pub async fn get_all_prefixes(mut conn: Connection<Sqlite>) -> HashMap<u64, String> {
    query!("SELECT `server`, `prefix` FROM `prefixes`")
        .fetch_all(&mut conn)
        .await
        .unwrap_or(Vec::new())
        .iter()
        .map(|row| (row.server as u64, row.prefix.to_owned()))
        .collect()
}
