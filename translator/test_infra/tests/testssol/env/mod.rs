#![allow(dead_code)]
#![cfg(test)]

pub mod generator;
pub mod revm;

pub fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
