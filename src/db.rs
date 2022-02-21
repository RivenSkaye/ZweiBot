pub use rusqlite::Connection;
use serenity::prelude::{Mutex, TypeMapKey};
use std::sync::Arc;

pub struct ZweiDBConn;
impl TypeMapKey for ZweiDBConn {
    type Value = Arc<Mutex<Connection>>;
}

async fn get_prefix(conn: Connection, guild: i64) -> String {
    conn.execute("SELECT prefix FROM prefixes WHERE server IS $1", [guild]);
    // perform DB code to get the prefix for <guild>, or return the default.
    return String::from(";");
}
