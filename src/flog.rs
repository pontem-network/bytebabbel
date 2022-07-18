use log::{log_enabled, Level};

pub fn is_trace() -> bool {
    log_enabled!(Level::Trace)
}
