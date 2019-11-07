use std::io;
use csv::Writer;
use std::collections::HashMap;
use std::path::Path;
use std::fs::{File, create_dir_all};
use serde::Serialize;
use failure::{Error, Fail};
use std::fmt::Debug;
use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard};
use rand::prelude::{SeedableRng, Rng};
use rand::distributions::Alphanumeric;

lazy_static! {
    static ref REC: Mutex<Recorder> = Mutex::new(Recorder::new());
}

pub fn init_recorder(run_name: impl Into<String>, over_write: bool) {
    let run_name = run_name.into();
    let path = if over_write {
        "./data/".to_string() + &run_name + "/"
    } else {
        let rand = rand::rngs::SmallRng::from_entropy()
            .sample_iter(&Alphanumeric)
            .take(3)
            .collect::<String>();
        "./data/".to_string() + &run_name + "_" + &rand + "/"
    };
    dbg!(&path);
    create_dir_all(&path).unwrap();
    REC.lock().unwrap().directory = path;
}

pub fn register(name: impl AsRef<str>, col_names: &[&str]) {
    REC.lock().unwrap().register(name, col_names).unwrap();
}

pub fn add(name: &str, blob: impl Serialize + Debug) {
    REC.lock().unwrap().add(name, blob).unwrap();
}

pub fn flush() {
    REC.lock().unwrap().flush();
}

pub fn set_tick(i: u16) {
    REC.lock().unwrap().tick = i;
}

struct Recorder {
    pub directory: String,
    pub tick: u16,
    pub files: HashMap<String, Writer<File>>,
}

impl Recorder {
    pub fn new() -> Self {
        Recorder { directory: "./data/".to_string(), files: HashMap::new(), tick: 0 }
    }

    pub fn flush(&mut self) {
        for f in self.files.values_mut() {
            f.flush().unwrap();
        }
    }

    pub fn register(&mut self,
                    name: impl AsRef<str>,
                    col_names: &[&str]) -> Result<(), Error> {
        let p = self.directory.clone() + name.as_ref() + ".csv";
        let p = Path::new(&p);
        let mut w = csv::Writer::from_path(p)?;
        w.write_field("tick")?;
        w.write_record(col_names)?;
        w.flush()?;
        self.files.insert(name.as_ref().to_owned(), w);
        Ok(())
    }

    pub fn add(&mut self, name: &str, blob: impl Serialize + Debug) -> Result<(), impl Fail> {
        let w = self.files.get_mut(name).unwrap();
        w.write_field(self.tick.to_string())?;
        w.serialize(blob)
    }
}
