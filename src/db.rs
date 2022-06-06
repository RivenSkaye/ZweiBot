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
    while let Some(row) = result.next()? {
        let id: i64 = row.get(0)?;
        let pfx = row.get(1)?;
        pfxs.insert(id as u64, pfx);
    }
    Ok(pfxs)
}

pub fn set_prefix(conn: &Connection, guild: u64, pfx: &str) -> SQLRes<usize> {
    conn.execute(
        "INSERT OR REPLACE INTO prefixes VALUES ($1, $2)",
        params![guild as i64, pfx],
    )
}

pub fn remove_prefix(conn: &Connection, guild: u64) -> SQLRes<usize> {
    conn.execute(
        "DELETE FROM prefixes WHERE server IS $1",
        params![guild as i64],
    )
}

fn get_tag_id(conn: &Connection, tag: &String, guild: u64) -> SQLRes<i64> {
    conn.query_row(
        "SELECT tagid FROM servertags WHERE serverid = $1 AND tagname = $2",
        params![guild as i64, tag],
        |r| r.get(0),
    )
}

pub fn get_server_tags(conn: &Connection, guild: u64) -> SQLRes<Vec<String>> {
    let mut prep = conn.prepare("SELECT tagname FROM servertags WHERE serverid = $1")?;
    let mut result = prep.query(params![guild as i64])?;

    let mut tags: Vec<String> = Vec::new();
    while let Some(row) = result.next()? {
        let mut val: String = row.get(0)?;
        val.push_str("\n");
        tags.push(val);
    }
    Ok(tags)
}

pub fn are_tags_in_server(conn: &Connection, guild: u64, tagvec: &Vec<String>) -> SQLRes<isize> {
    conn.query_row(
        format!(
            "SELECT COUNT(tagid) FROM servertags WHERE serverid = $1 AND tagname IN (\"{}\")",
            tagvec.join("\", \"")
        )
        .as_str(),
        params![guild as i64],
        |row| row.get(0),
    )
}

pub fn add_tag(conn: &Connection, guild: u64, tag: &String) -> SQLRes<usize> {
    conn.execute(
        "INSERT INTO servertags (serverid, tagname) VALUES ($1, $2)",
        params![guild as i64, tag],
    )
}

pub fn remove_tag(conn: &Connection, guild: u64, tag: &String) -> SQLRes<usize> {
    let tag_id = get_tag_id(conn, tag, guild)?;
    conn.execute("DELETE FROM tagsubs WHERE tagid = $1", params![tag_id])?;
    conn.execute(
        "DELETE FROM servertags WHERE serverid = $1 AND tagname = $2",
        params![guild as i64, tag],
    )
}

pub fn sub_to(conn: &Connection, guild: u64, tag: &String, uid: u64) -> SQLRes<usize> {
    let tag_id = get_tag_id(conn, tag, guild)?;
    conn.execute(
        "INSERT INTO tagsubs VALUES ($1, $2)",
        params![tag_id, uid as i64],
    )
}

pub fn unsub(conn: &Connection, guild: u64, tag: &String, uid: u64) -> SQLRes<usize> {
    let tag_id = get_tag_id(conn, tag, guild)?;
    conn.execute(
        "DELETE FROM tagsubs WHERE tagid = $1 AND userid = $2",
        params![tag_id, uid as i64],
    )
}

pub fn usersubs(conn: &Connection, guild: u64, uid: u64) -> SQLRes<Vec<String>> {
    let mut prep = conn.prepare(
        "SELECT tagname FROM servertags, tagsubs WHERE tagsubs.userid = $1 AND tagsubs.tagid = servertags.tagid AND servertags.serverid = $2"
    )?;
    let mut result = prep.query(params![uid as i64, guild as i64])?;

    let mut tags: Vec<String> = Vec::new();
    while let Some(row) = result.next()? {
        let mut val: String = row.get(0)?;
        val.push_str("\n");
        tags.push(val);
    }
    Ok(tags)
}

pub fn get_subbers(conn: &Connection, guild: u64, tag: &String) -> SQLRes<Vec<u64>> {
    let mut prep = conn.prepare(
        "SELECT userid FROM tagsubs WHERE tagid = (SELECT tagid FROM servertags WHERE serverid = $1 AND tagname = $2)"
    )?;
    let mut result = prep.query(params![guild as i64, tag])?;

    let mut users: Vec<u64> = Vec::new();
    while let Some(row) = result.next()? {
        let user: i64 = row.get(0)?;
        users.push(user as u64);
    }
    Ok(users)
}
