use log::LevelFilter;

mod cases;
mod common;
mod evm;
mod mv;

pub fn log_init() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .is_test(true)
        .try_init();
}
