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
    query!("SELECT server, prefix FROM prefixes")
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

/// # rowcount!
/// Small macro that expands to executing a query, checking the rowcount and handling any errors.
/// The two main points for this code were learning macro basics and not repeating the same
/// check _every time_ I write a query.
///
/// ## usage
/// ```rs
/// rowcount!(
///     sqlx::macros::query("DELETE FROM users WHERE users.name IS NOT NULL")
///         .execute(),
///     "Trace logging message, yeeting all users!\nCustom text: {}",
///     "Error message! This one's bad, boss!\nAdditional info: {}",
///     "You can format in whatever you want!"
/// )
/// ```
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
        query!("INSERT OR REPLACE INTO prefixes VALUES(?, ?)", g, pfx).execute(conn),
        "Setting custom prefix {} for guild ID {}",
        "Failed to set custom prefix {} for guild ID {}",
        pfx,
        guild
    )
}

pub async fn remove_prefix(conn: &Pool, guild: u64) -> ZweiDbRes<u64> {
    let g = guild as i64;
    rowcount!(
        query!("DELETE FROM prefixes WHERE server = ?", g).execute(conn),
        "Removing custom prefix for guild ID {}",
        "Failed to remove custom prefix for guild ID {}",
        guild
    )
}

pub async fn get_server_tags(conn: &Pool, guild: u64) -> ZweiDbRes<Vec<String>> {
    let g = guild as i64;
    query!("SELECT tagname FROM servertags WHERE serverid = ?", g)
        .fetch_all(conn)
        .await
        .map(|r| r.iter().map(|row| row.tagname.to_owned()).collect())
}

pub async fn get_subbers(conn: &Pool, guild: u64, tag: &String) -> ZweiDbRes<Vec<u64>> {
    let g = guild as i64;
    query!("SELECT userid FROM tagsubs WHERE tagid = (SELECT tagid FROM servertags WHERE serverid = ? AND tagname = ?)", g, tag)
    .fetch_all(conn)
    .await
    .map(|ids|ids.iter().map(|id| id.userid as u64).collect())
}

pub async fn add_tag(conn: &Pool, guild: u64, tag: &String) -> ZweiDbRes<u64> {
    let g = guild as i64;
    rowcount!(
        query!(
            "INSERT INTO servertags (serverid, tagname) VALUES (?, ?)",
            g,
            tag
        )
        .execute(conn),
        "Adding tag {} to guild ID {}",
        "INSERT failed with tag {} for guild ID {}",
        tag,
        g
    )
}

pub async fn remove_tag(conn: &Pool, guild: u64, tag: &String) -> ZweiDbRes<u64> {
    let g = guild as i64;
    rowcount!(
        query!(
            "DELETE FROM servertags WHERE tagname = ? AND serverid = ?",
            tag,
            g
        )
        .execute(conn),
        "Deleting {} for guild ID {}",
        "Something went wrong deleting tag {} for guild ID {}!",
        tag,
        g
    )
}

async fn get_tag_id(conn: &Pool, guild: u64, tag: &String) -> ZweiDbRes<i64> {
    let g = guild as i64;
    query!(
        "SELECT tagid FROM servertags WHERE serverid = ? AND tagname = ?",
        g,
        tag
    )
    .fetch_one(conn)
    .await
    .map(|res| res.tagid)
}

pub async fn sub_to(conn: &Pool, guild: u64, tag: &String, uid: u64) -> ZweiDbRes<u64> {
    let u = uid as i64;
    let t = get_tag_id(conn, guild, tag).await?;
    rowcount!(
        query!("INSERT INTO tagsubs VALUES (?, ?)", t, u).execute(conn),
        "Subscribing user ID {} to tag ID {}",
        "Something went wrong, failed to subscribe user {} to {}",
        u,
        t
    )
}

pub async fn unsub(conn: &Pool, guild: u64, tag: &String, uid: u64) -> ZweiDbRes<u64> {
    let u = uid as i64;
    let t = get_tag_id(conn, guild, tag).await?;
    rowcount!(
        query!("DELETE FROM tagsubs WHERE tagid = ? AND userid = ?", t, u).execute(conn),
        "Unsubscribing user ID {} from tag ID {}",
        "Something went wrong, failed to unsubscribe user {} from {}",
        u,
        t
    )
}

pub async fn usersubs(conn: &Pool, guild: u64, uid: u64) -> ZweiDbRes<Vec<String>> {
    let g = guild as i64;
    let u = uid as i64;
    query!(
        "SELECT tagname FROM servertags WHERE serverid = ? AND tagid IN (SELECT tagid FROM tagsubs WHERE userid = ?)",
        g, u
    )
    .fetch_all(conn)
    .await
    .map(|res| res.iter().map(|row| row.tagname.to_owned()).collect())
}

pub async fn are_tags_in_server(conn: &Pool, guild: u64, tagvec: &Vec<String>) -> ZweiDbRes<usize> {
    get_server_tags(conn, guild)
        .await
        .map(|res| res.iter().filter(|tag| tagvec.contains(tag)).count())
}
