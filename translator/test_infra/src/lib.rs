extern crate core;

pub mod executor;
pub mod generator;
pub mod sol;

pub fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
