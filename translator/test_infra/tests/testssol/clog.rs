use crate::testssol::color::font_red;
use lazy_static::lazy_static;
use log::{Metadata, Record};
use std::collections::HashMap;
use std::sync::Mutex;

pub static CUST_LOGGER: CustLogger = CustLogger;
lazy_static! {
    static ref LOG_BUFF: Mutex<HashMap<std::thread::ThreadId, Vec<String>>> =
        Mutex::new(HashMap::new());
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
            log::Level::Error => Some(font_red("ERROR")),
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

pub fn log_init() {
    log::set_logger(&CUST_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}
