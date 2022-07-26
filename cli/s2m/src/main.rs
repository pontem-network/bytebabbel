use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod convert;
use crate::convert::Convert;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Path to the sol file
    #[clap(short, long, display_order = 1)]
    path: PathBuf,

    /// Where to save the converted binary move file
    #[clap(short, long = "output", display_order = 2)]
    output_path: Option<PathBuf>,

    /// The name of the Move module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 3)]
    module_name: Option<String>,

    /// The address of the Move module.
    #[clap(long = "address", display_order = 4, default_value = "0x1")]
    module_address: String,

    /// Math backend.
    #[clap(long = "math", short = 'm', display_order = 5, default_value = "u128")]
    math_backend: String,

    /// Output of debugging information
    #[clap(short, long, display_order = 6)]
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
    inic_log::inic_of_log_configs(args.trace)?;

    Convert::try_from(args)?
        .create_mv()
        .map(|path| path.to_string_lossy().to_string())
}
