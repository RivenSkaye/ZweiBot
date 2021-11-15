use std::io;
use std::path::PathBuf;
use std::fs;
use fs::File;
use std::io::Write;
use serde_json as sj;
use serde::{Serialize, Deserialize};

#[macro_use]
extern crate lazy_static;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Conf {
    token: String,
    owners: Vec<i64>,
    database: String
}

fn get_data_dir() -> PathBuf{
    std::env::current_exe().unwrap()
    .parent().unwrap().join("data").canonicalize()
    .or_else(|_| PathBuf::from("./data").canonicalize())
    .unwrap()
}

fn create_default_conf(pth: PathBuf) -> Result<File, io::Error> {
    let mut dconf = Conf::default();
    dconf.database = String::from("Zwei.sdb");
    let mut f = File::create(pth).unwrap();
    f.write(sj::ser::to_string_pretty(&dconf)?.as_bytes())?;
    Ok(f)
}

fn read_conf() -> Result<Conf, io::Error> {
    let pth = &DATADIR;
    let conf_file = File::open(pth.join("config.json"))
        .or_else(|_| File::open("./data/config.json"))
        .or_else(|_| create_default_conf(DATADIR.join("config.json")))
        .expect(&format!("Couldn't open or create either {:}/config.json or ./data/config.json. PANIC!", pth.display()));
    let reader = io::BufReader::new(conf_file);
    let conf = sj::from_reader(reader)?;
    Ok(conf)
}

lazy_static! {
    static ref DATADIR: PathBuf = get_data_dir();
    static ref CONF: Conf = read_conf().unwrap();
}

fn main() {
    // do code and get rekt
}
