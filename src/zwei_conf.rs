use serde::{Deserialize, Serialize};
use serde_json as sj;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Conf {
    pub(crate) token: String,
    pub(crate) owners: Vec<i64>,
    pub(crate) database: String,
}

lazy_static! {
    pub(crate) static ref DATADIR: PathBuf = get_data_dir();
    pub(crate) static ref CONF: Conf = read_conf().unwrap();
}

fn get_data_dir() -> PathBuf {
    let pth: PathBuf = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join("data");

    let fallback = PathBuf::from("./data");

    match (pth.exists(), fallback.exists()) {
        (true, _) => pth.canonicalize().unwrap(),
        (false, true) => fallback.canonicalize().unwrap(),
        (false, false) => {
            if let Ok(_) = fs::create_dir_all(&pth) {
                pth.canonicalize().unwrap()
            } else if let Ok(_) = fs::create_dir_all(&fallback) {
                fallback.canonicalize().unwrap()
            } else {
                panic!(
                    "Can't create {:} or {:}, please create a data folder yourself!",
                    pth.display(),
                    fallback.display()
                )
            }
        }
    }
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
        .expect(&format!(
            "Couldn't open or create either {:}/config.json or ./data/config.json. PANIC!",
            pth.display()
        ));
    let reader = io::BufReader::new(conf_file);
    let conf = sj::from_reader(reader)?;
    Ok(conf)
}
