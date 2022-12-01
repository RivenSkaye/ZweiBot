use log::{error, trace};
use serenity::prelude::TypeMapKey;
pub use sqlx::{
    pool::PoolConnection as Connection, query, query_as, Error as SQLerr, Sqlite,
    SqlitePool as Pool,
};
use std::collections::HashMap;

pub struct ZweiDbConn;
impl TypeMapKey for ZweiDbConn {
    type Value = Pool;
}

pub(crate) type ZweiDbRes<T> = Result<T, SQLerr>;

pub async fn get_all_prefixes(mut conn: Connection<Sqlite>) -> HashMap<u64, String> {
    trace!("Getting all prefixes from the database");
    query!("SELECT `server`, `prefix` FROM `prefixes`")
        .fetch_all(&mut conn)
        .await
        .unwrap_or_else(|e| {
            error!("Proceeding without content from the prefixes table!\n\t{e:#?}");
            Vec::new()
        })
        .iter()
        .map(|row| (row.server as u64, row.prefix.to_owned()))
        .collect()
}

macro_rules! rowcount {
    ($res:expr, $trace:literal, $err:literal, $( $args:expr ),*) => {
        match $res.await {
            Ok(r) => {
                trace!($trace, $($args, )*);
                Ok(r.rows_affected())
            },
            Err(e) => {
                error!(concat!($err, "\n\t{}"), $($args, )* e);
                Err(e)
            }
        }
    };
}

pub async fn set_prefix(conn: &Pool, guild: u64, pfx: &str) -> ZweiDbRes<u64> {
    let g = guild as i64;
    rowcount!(
        query!("INSERT OR REPLACE INTO `prefixes` VALUES(?, ?)", g, pfx).execute(conn),
        "Setting custom prefix {} for guild ID {}",
        "Failed to set custom prefix {} for guild ID {}",
        pfx,
        guild
    )
}

pub async fn remove_prefix(conn: &Pool, guild: u64) -> ZweiDbRes<u64> {
    let g = guild as i64;
    rowcount!(
        query!("DELETE FROM `prefixes` WHERE `server` = ?", g).execute(conn),
        "Removing custom prefix for guild ID {}",
        "Failed to remove custom prefix for guild ID {}",
        guild
    )
}

pub async fn get_server_tags(conn: &Pool, guild: u64) -> ZweiDbRes<Vec<String>> {
    let g = guild as i64;
    query!("SELECT `tagname` FROM `servertags` WHERE `serverid` = ?", g)
        .fetch_all(conn)
        .await
        .map(|r| r.iter().map(|row| row.tagname.to_owned()).collect())
}

pub async fn get_subbers(conn: &Pool, guild: u64, tag: &String) -> ZweiDbRes<Vec<u64>> {
    let g = guild as i64;
    query!("SELECT `userid` FROM `tagsubs` WHERE `tagid` = (SELECT `tagid` FROM `servertags` WHERE `serverid` = ? AND `tagname` = ?)", g, tag)
    .fetch_all(conn)
    .await
    .map(|ids|ids.iter().map(|id| id.userid as u64).collect())
}

pub async fn filter_tags(conn: &Pool, guild: u64, tagvec: Vec<String>) -> ZweiDbRes<Vec<String>> {
    get_server_tags(conn, guild).await.map(|tags| {
        tagvec
            .iter()
            .filter(|t| tags.contains(t))
            .cloned()
            .collect()
    })
}
