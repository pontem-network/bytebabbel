use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::lazy_static;
use log::{Metadata, Record};

pub mod color;
pub mod env;

pub static CUST_LOGGER: CustLogger = CustLogger;
lazy_static! {
    static ref REG_FOR_NAME: regex::Regex = regex::Regex::new(r#"(?i)[^a-z\d]+"#).unwrap();
    static ref LOG_BUFF: Mutex<HashMap<String, (ThreadSettings, Vec<String>)>> =
        Mutex::new(HashMap::new());
}

pub struct CustLogger;

impl CustLogger {
    fn set_setting(settings: ThreadSettings) {
        match LOG_BUFF.lock().as_deref_mut() {
            Ok(logbuff_mut) => {
                logbuff_mut
                    .entry(thread_name_id())
                    .and_modify(|item| *item = (settings.clone(), Vec::new()))
                    .or_insert((settings, Vec::new()));
            }
            Err(err) => output_error(err),
        };
    }

    fn settings() -> ThreadSettings {
        LOG_BUFF
            .lock()
            .ok()
            .and_then(|buff| buff.get(&thread_name_id()).cloned().map(|sett| sett.0))
            .unwrap_or_else(|| {
                output_error(format!("Settings not found. {}", thread_name_id()));
                ThreadSettings::default()
            })
    }

    fn write(content: String) {
        let settings = Self::settings();

        if let Some(path) = &settings.save_path {
            // file
            Self::write_to_file(path, &content);
        } else if settings.buff {
            // buffer
            CustLogger::write_to_buff(content)
        } else {
            // output
            println!("{content}");
        }
    }

    fn write_to_file(file_path: &PathBuf, content: &str) {
        if let Err(err) = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(file_path)
            .and_then(|mut open| open.write(format!("{content}\n").as_bytes()))
        {
            output_error(err);
        }
    }

    fn write_to_buff(content: String) {
        match LOG_BUFF.lock().as_mut() {
            Ok(buff) => buff.get_mut(&thread_name_id()).map_or_else(
                || output_error(format!("Not found buff. {}", thread_name_id())),
                |item| item.1.push(content),
            ),
            Err(err) => output_error(err),
        }
    }

    pub fn flushbuff_and_get() -> String {
        match LOG_BUFF.lock().as_mut() {
            Ok(buff) => {
                let data = buff
                    .get_mut(&thread_name_id())
                    .map(|item| {
                        let list = item.1.to_owned();
                        item.1 = Vec::new();
                        list
                    })
                    .unwrap_or_default();
                let output: String = data.join("\n");
                output
            }
            Err(err) => {
                output_error(err);
                String::new()
            }
        }
    }
}

impl log::Log for CustLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let content = record.args().to_string();

        let content = match record.level() {
            log::Level::Error => format!("[{pref}] {content}", pref = color::font_red("ERROR")),
            _ => content,
        };

        CustLogger::write(content);
    }

    fn flush(&self) {
        let output = CustLogger::flushbuff_and_get();
        println!("{output}");
    }
}

#[derive(Clone, Debug, Default)]
struct ThreadSettings {
    buff: bool,
    save_path: Option<PathBuf>,
}

impl ThreadSettings {
    fn init(test_name: Option<&str>) -> ThreadSettings {
        let mut settings = ThreadSettings::default();

        let save_path = env::log_save()
            .unwrap_or_else(|err| {
                output_error(err);
                None
            })
            .map(|path| {
                let mut file_name = test_name.map_or_else(thread_name, |name| {
                    REG_FOR_NAME.replace_all(name, "_").to_string()
                });

                file_name = format!(
                    "{}_{}",
                    chrono::Local::now().format("%y%m%d_%H%M%S"),
                    file_name
                );

                let mut suff = 0;
                let mut next = file_name.clone();
                let mut save_path;
                loop {
                    save_path = path.join(format!("{next}.log"));
                    if !save_path.exists() {
                        break;
                    }
                    next = format!("{file_name}_{suff}");
                    suff += 1;
                }

                if let Err(err) = fs::write(&save_path, "") {
                    output_error(err);
                }

                save_path
            });

        settings.save_path = save_path;

        settings
    }
}

pub fn init_log() {
    _log_init(false, None);
}

pub fn init_log_with_buff_and_name(name: &str) {
    _log_init(true, Some(name));
}

fn _log_init(write_to_buff: bool, file_name: Option<&str>) {
    if log::set_logger(&CUST_LOGGER).is_ok() {
        let filter = env::log_level_filter().unwrap_or_else(|err| {
            output_error(err);
            log::LevelFilter::Off
        });
        log::set_max_level(filter);
    }

    let mut settings = ThreadSettings::init(file_name);
    settings.buff = write_to_buff;

    CustLogger::set_setting(settings);
}

// ===

fn thread_id() -> String {
    format!("{:?}", std::thread::current().id())
}

fn thread_name() -> String {
    let name_id = std::thread::current()
        .name()
        .map_or_else(thread_id, |name| name.to_string());
    REG_FOR_NAME.replace_all(&name_id, "_").to_string()
}

fn thread_name_id() -> String {
    let name_id = format!(
        "{}_{}",
        std::thread::current()
            .name()
            .map_or_else(|| "", |name| name),
        thread_id()
    );
    REG_FOR_NAME.replace_all(&name_id, "_").to_string()
}

fn output_error<T>(err: T)
where
    T: std::fmt::Debug + std::fmt::Display,
{
    println!("[{}] {err}", color::font_red("ERROR"));
}
