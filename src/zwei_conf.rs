use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Conf {
    pub(crate) token: String,
    pub(crate) owners: HashSet<u64>,
    #[serde(default = "default_db")]
    pub(crate) database: PathBuf,
    #[serde(default)]
    pub(crate) db_pass: String,
    #[serde(default = "default_err_color")]
    pub(crate) err_color: String,
    #[serde(default = "default_ok_color")]
    pub(crate) ok_color: String,
}

fn default_err_color() -> String {
    "9A48C9".to_string()
}

fn default_ok_color() -> String {
    "B82748".to_string()
}

fn default_db() -> PathBuf {
    DATADIR.join("Zwei.sdb")
}

pub(crate) static DATADIR: Lazy<PathBuf> = Lazy::new(get_data_dir);
pub(crate) static CONF: Lazy<Conf> = Lazy::new(|| read_conf().unwrap());

fn get_data_dir() -> PathBuf {
    // exe relative
    let preferred = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("data");
    if preferred.exists() {
        return preferred.canonicalize().unwrap();
    }

    // cwd relative
    let fallback = std::env::current_dir().unwrap().join("data");
    if fallback.exists() {
        return fallback.canonicalize().unwrap();
    }

    if fs::create_dir_all(&preferred).is_ok() {
        return preferred;
    } else if fs::create_dir_all(&fallback).is_ok() {
        return fallback;
    } else {
        panic!(
            "Can't create {} or {}. Please create a data folder yourself!",
            preferred.display(),
            fallback.display(),
        )
    }
}

fn create_default_conf<P: AsRef<Path>>(path: P) -> io::Result<Conf> {
    let conf = Conf::default();
    let f = fs::File::create(path)?;
    serde_json::to_writer_pretty(f, &conf)?;
    Ok(conf)
}

fn read_conf() -> io::Result<Conf> {
    let path = DATADIR.join("config.json");

    let conf = if path.exists() {
        let data = fs::read(path)?;
        serde_json::from_slice(&data)?
    } else {
        create_default_conf(path)?
    };

    Ok(conf)
}
