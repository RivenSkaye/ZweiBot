use std::io;
use std::fs::File;
use serde_json as sj;
use serde::{Serialize, Deserialize};

#[macro_use]
extern crate lazy_static;

#[derive(Serialize, Deserialize, Debug)]
struct Conf {
    token: String,
    owners: Vec<i64>,
    database: String
}

fn read_conf() -> Result<Conf, io::Error> {
    let conf_file = File::open("./data/config.json")?;
    let reader = io::BufReader::new(conf_file);
    let conf = sj::from_reader(reader)?;
    Ok(conf)
}

lazy_static! {
    static ref CONF: Conf = read_conf().unwrap();
}

fn main() {
    // do code and get rekt
}
