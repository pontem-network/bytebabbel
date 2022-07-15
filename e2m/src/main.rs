use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

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

    #[clap(short, long, display_order = 7, value_parser, default_value = "false")]
    trace: bool,
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
    Convert::try_from(Args::parse())?
        .create_mv()
        .map(|path| path.to_string_lossy().to_string())
}
