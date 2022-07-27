use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

mod convert;
use crate::convert::Convert;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Path to the file. Specify the path to sol file or abi|bin.
    #[clap(value_parser)]
    path: PathBuf,

    /// Where to save the converted move binary code
    #[clap(short, long = "output", display_order = 3, value_parser)]
    output_path: Option<PathBuf>,

    /// The name of the Move module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 4, value_parser)]
    move_module_name: Option<String>,

    /// The address of the Move module.
    #[clap(
        long = "address",
        display_order = 5,
        default_value = "0x1",
        value_parser
    )]
    move_module_address: String,

    /// Math backend.
    #[clap(long = "math", short = 'm', default_value = "u128", value_parser)]
    math_backend: String,

    /// Output of debugging information
    #[clap(short, long, value_parser)]
    trace: Option<bool>,
}

fn main() {
    match run() {
        Ok(path) => {
            println!("Saved in {path:?}");
        }
        Err(err) => {
            println!("Error: {err:?}");
        }
    }
}

fn run() -> Result<String> {
    let args: Args = Args::parse();
    inic_of_log_configs(args.trace)?;

    Convert::try_from(args)?
        .create_mv()
        .map(|path| path.to_string_lossy().to_string())
}

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
