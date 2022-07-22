use anyhow::Result;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

/// Initializing the logs display
/// trace:
///     None = get settings from ENV(["RUST_LOG", "LOGS", "LOG"])
///     true = Enable full display
///     false = Do not display logs
pub fn inic_of_log_configs(trace: Option<bool>) -> Result<()> {
    let mut conf = if let Some(is_trace) = trace {
        let mut builder = Builder::new();
        if is_trace {
            builder.filter_level(LevelFilter::Trace);
        } else {
            builder.filter_level(LevelFilter::Off);
        }
        builder
    } else {
        inic_of_log_configs_by_env()?
    };

    conf.format(|buf, record| {
        if record.level() == log::Level::Trace {
            writeln!(buf, "{}", record.args())
        } else {
            writeln!(buf, "[{}]: {}", record.level(), record.args())
        }
    });
    conf.init();
    Ok(())
}

// Unitialize based on data from "env"
// ENV Examples
//  LOG=info
//  LOG=!all
//  LOG=all,!debug,info,!error
fn inic_of_log_configs_by_env() -> Result<env_logger::Builder> {
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Off);

    for name in ["RUST_LOG", "LOGS", "LOG"] {
        if std::env::var(name).is_err() {
            continue;
        }
        builder.parse_env(name);
        return Ok(builder);
    }
    Ok(builder)
}
