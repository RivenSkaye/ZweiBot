use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

/// # Conf
/// The basic bot configuration for Zwei, powered by Serde and JSON!
/// Almost all fields have defaults configured that should work out of the box.
/// However, the `token` and `owners` fields cannor be filled out, as every
/// running bot instance will have different values for this data.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Conf {
    /// The bot token to connect to Discord with.
    pub(crate) token: String,
    /// A list of users that the bot should consider owners
    pub(crate) owners: HashSet<u64>,
    /// The database file to read.
    #[serde(default = "default_db")]
    pub(crate) database: PathBuf,
    /// Database password (if relevant). Currently unused.
    #[serde(default)]
    pub(crate) db_pass: String,
    /// The color to use for error messages.
    #[serde(default = "default_err_color", deserialize_with = "strip_hex")]
    pub(crate) err_color: String,
    /// The color to use for success messages.
    #[serde(default = "default_ok_color", deserialize_with = "strip_hex")]
    pub(crate) ok_color: String,
    /// The log level to configure the env logger with.
    #[serde(default = "default_loglevel")]
    pub(crate) loglevel: String,
}

/// Default log level to use, "warn"
fn default_loglevel() -> String {
    "warn".to_owned()
}

/// Default error color to use, 0x9A48C9
fn default_err_color() -> String {
    "9A48C9".to_string()
}

/// Deefault success color to use, 0xB82748
fn default_ok_color() -> String {
    "B82748".to_string()
}

/// Default database path, computed as <[`DATADIR`]>/Zwei.sdb
fn default_db() -> PathBuf {
    DATADIR.join("Zwei.sdb")
}

/// The directory to search for data files. See [`get_data_dir`] for the paths
/// Zwei searches in to find the stuff she needs to run.
/// Used to define the [`default_db`] and to fetch the config file.
pub(crate) static DATADIR: Lazy<PathBuf> = Lazy::new(get_data_dir);
/// once_cell Lazy config powered by [`read_conf`].
pub(crate) static CONF: Lazy<Conf> = Lazy::new(|| read_conf().unwrap());

/// # get_data_dir
/// Function that searches several places on the system in an attempt to find
/// its config files. The places searched are
/// - a `data` folder living next to the executable
/// - a `data` folder in the current working directory
/// If neither of these places contain this folder, the bot will attempt to
/// create this directory instead. Make sure that Zwei has write permissions
/// if you plan to use this mechanism to generate the data folder.
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

/// # create_default_conf
/// A simple function that creates a default config file if possible.
/// This function writes the file as well, allowing Zwei to populate her own
/// data directory for first boot situations and such.
fn create_default_conf<P: AsRef<Path>>(path: P) -> io::Result<Conf> {
    let conf = Conf::default();
    let f = fs::File::create(path)?;
    serde_json::to_writer_pretty(f, &conf)?;
    Ok(conf)
}

/// # read_conf
/// Reads the config file into a [`Conf`] object.
/// Initializes a default config in the event it can't parse the existing file.
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

/// # strip_hex
/// Strips the leading # from
fn strip_hex<'d, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'d>,
{
    let val: &str = Deserialize::deserialize(deserializer)?;
    if let Some(s) = val.strip_prefix('#') {
        Ok(s.to_owned())
    } else {
        Ok(val.to_owned())
    }
}
