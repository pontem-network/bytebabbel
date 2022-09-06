use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use lazy_static::lazy_static;
use log::{Metadata, Record};

use crate::testssol::color;

pub static CUST_LOGGER: CustLogger = CustLogger;
lazy_static! {
    static ref LOG_BUFF: Mutex<HashMap<std::thread::ThreadId, Vec<String>>> =
        Mutex::new(HashMap::new());
    static ref LOG_SAVE: Mutex<Option<PathBuf>> = Mutex::new(None);
}

pub struct CustLogger;
impl CustLogger {
    pub fn flush_and_get() -> String {
        let id = thread_id();
        let mut buff = LOG_BUFF.lock().unwrap();
        if let Some(data) = buff.get_mut(&id) {
            let output: String = data.join("\n");
            *data = Vec::new();
            return output;
        }
        String::new()
    }

    pub fn flush_and_get_or_save(name_file: &str) -> String {
        let output = Self::flush_and_get();

        let save_path = LOG_SAVE.lock().unwrap();
        match save_path.as_ref() {
            Some(path) => {
                let file_path = path.join(format!(
                    "{}-{name_file}.log",
                    chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string()
                ));

                let mut file = match fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(file_path)
                {
                    Ok(file) => file,
                    Err(err) => {
                        println!("[{}] {err}", color::font_red("ERROR"));
                        return String::default();
                    }
                };

                if let Err(err) = file.write(output.as_bytes()) {
                    println!("[{}] {err}", color::font_red("ERROR"));
                }

                String::default()
            }
            None => output,
        }
    }
}

fn thread_id() -> std::thread::ThreadId {
    std::thread::current().id()
}

impl log::Log for CustLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut buff = LOG_BUFF.lock().unwrap();
        let id = thread_id();

        let mut content = record.args().to_string();

        let pref = match record.level() {
            log::Level::Error => Some(color::font_red("ERROR")),
            _ => None,
        }
        .map(|s| format!("[{s}]"));

        if let Some(pref) = pref {
            content = format!("{pref} {content}");
        }

        buff.entry(id).or_insert_with(Vec::new);
        buff.get_mut(&id).unwrap().push(content);
    }

    fn flush(&self) {
        let output = CustLogger::flush_and_get();
        println!("{output}");
    }
}

pub fn my_log_init() {
    log::set_logger(&CUST_LOGGER).unwrap();

    let env_setting = ["RUST_LOG", "LOGS", "LOG"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .next();

    let filter = match env_setting {
        None => log::LevelFilter::Info,
        Some(data) => log::LevelFilter::from_str(&data).unwrap_or(log::LevelFilter::Info),
    };
    log::set_max_level(filter);

    // Save to path
    if let Some(path) = std::env::var("LOG_SAVE")
        .ok()
        .and_then(|path| PathBuf::from(path).canonicalize().ok())
    {
        let mut save_path = LOG_SAVE.lock().unwrap();
        *save_path = Some(path);
    }
}
