use std::collections::HashMap;
use std::io::Write;
use std::iter::Map;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

mod convert;
use crate::convert::Convert;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Path to the abi file
    #[clap(short, long = "abi", display_order = 1)]
    abi_path: PathBuf,

    /// Path to the bin file
    #[clap(short, long = "bin", display_order = 2)]
    bin_path: PathBuf,

    /// Where to save the converted file
    #[clap(short, long = "output", display_order = 3)]
    output_path: Option<PathBuf>,

    /// The name of the module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 4)]
    module_name: Option<String>,

    /// The address of the module.
    #[clap(long = "address", display_order = 5, default_value = "0x1")]
    module_address: String,

    /// Math backend.
    #[clap(long = "math", short = 'm', display_order = 6, default_value = "u128")]
    math_backend: String,

    #[clap(short, long, display_order = 6, value_parser)]
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
    inic_of_log_configs(&args)?;

    Convert::try_from(args)?
        .create_mv()
        .map(|path| path.to_string_lossy().to_string())
}

fn inic_of_log_configs(args: &Args) -> Result<()> {
    let mut conf = if let Some(is_trace) = args.trace {
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
        if !std::env::var(name).is_ok() {
            continue;
        }
        builder.parse_env(name);
        return Ok(builder);
    }
    Ok(builder)
}
