mod cases;
mod common;
mod evm;
mod mv;

pub fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
