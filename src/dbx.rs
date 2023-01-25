use log::{error, trace};
use serenity::prelude::TypeMapKey;
pub use sqlx::{
    pool::PoolConnection as Connection, query, query_as, Error as SQLerr, Sqlite,
    SqlitePool as Pool,
};
use std::collections::HashMap;

///# ZweiDbConn
/// A simple `TypeMapKey` implementation that helps access and use Database connections.
/// Currently only contains a `sqlx::SqlitePool`, but might be extended in due time to
/// allow for other DB types supported by SQLx to be used.
pub struct ZweiDbConn;
impl TypeMapKey for ZweiDbConn {
    type Value = Pool;
}

/// # ZweiDbRes
/// Result type for Zwei's database operations.
/// The `Ok` type is function-defined, but all errors MUST be `sqlx::Error`.
pub(crate) type ZweiDbRes<T> = Result<T, SQLerr>;

/// # get_all_prefixes
/// Exactly what it says on the tin, retrieves all known prefixes from the DB.
/// Upon failure, this will log a message, but not terminate with an error.
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
///     sqlx::macros::query!("DELETE FROM users WHERE users.name IS NOT NULL")
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

/// # set_prefix
/// Sets a prefix for a guild to be used to invoke commands. On success, returns
/// the amount of altered rows in the database. This should only ever be one row.
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

/// # remove_prefix
/// Unsets the custom prefix for the specified guild, returning the amount of
/// affected rows upon success. Should only ever return 0 or 1
pub async fn remove_prefix(conn: &Pool, guild: u64) -> ZweiDbRes<u64> {
    let g = guild as i64;
    rowcount!(
        query!("DELETE FROM prefixes WHERE server = ?", g).execute(conn),
        "Removing custom prefix for guild ID {}",
        "Failed to remove custom prefix for guild ID {}",
        guild
    )
}

/// # get_server_tags
/// Selects all registered tags for a guild, returning a Vec.
/// This Vec might be empty if the result set is.
pub async fn get_server_tags(conn: &Pool, guild: u64) -> ZweiDbRes<Vec<String>> {
    let g = guild as i64;
    query!("SELECT tagname FROM servertags WHERE serverid = ?", g)
        .fetch_all(conn)
        .await
        .map(|r| r.iter().map(|row| row.tagname.to_owned()).collect())
}

/// # get_subbers
/// Get all users subscribed to a tag in this guild. Returned vec might be empty.
pub async fn get_subbers(conn: &Pool, guild: u64, tag: &String) -> ZweiDbRes<Vec<u64>> {
    let g = guild as i64;
    query!("SELECT userid FROM tagsubs WHERE tagid = (SELECT tagid FROM servertags WHERE serverid = ? AND tagname = ?)", g, tag)
    .fetch_all(conn)
    .await
    .map(|ids|ids.iter().map(|id| id.userid as u64).collect())
}

/// # add_tag
/// Register a tag for use in this guild.
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

/// # remove_tag
/// Removes a tag registered to this guild, if present.
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

/// # get_tag_id
/// Function for internal use to get the ID of a tag registered for the current guild.
/// This is a helper function to prevent duplicate tags across guilds from becoming
/// a problem or a source of collisions.
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

/// # sub_to
/// Subscribes a user to a specific tag in this guild. Behavior when a user is
/// already subscribed to a tag is up to the sqlite configuration used for the
/// `ON CONFLICT` clause on `INSERT` statements.
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

/// # unsub
/// Removes a subscription to a specific tag in this guild
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

/// # usersubs
/// Fetches a list of all tags a user is subscribed to in the current server.
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

/// # are_tags_in_server
/// Helper function that determines how many of the requested tags are registered
/// for the current guild. Intended to have an early exit method when a provided
/// list of tags has no matches in this guild.
pub async fn are_tags_in_server(conn: &Pool, guild: u64, tagvec: &Vec<String>) -> ZweiDbRes<usize> {
    get_server_tags(conn, guild)
        .await
        .map(|res| res.iter().filter(|tag| tagvec.contains(tag)).count())
}
