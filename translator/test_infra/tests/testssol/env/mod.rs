#![allow(dead_code)]
pub mod executor;
pub mod generator;
#[cfg(test)]
pub mod revm;
pub mod stdlib;

pub fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
