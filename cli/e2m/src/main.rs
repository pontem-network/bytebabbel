use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod convert;
use crate::convert::Convert;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Path to the solidity abi file
    #[clap(short, long = "abi", display_order = 1)]
    abi_path: PathBuf,

    /// Path to the solidity bin file
    #[clap(short, long = "bin", display_order = 2)]
    bin_path: PathBuf,

    /// Where to save the converted binary move file
    #[clap(short, long = "output", display_order = 3)]
    output_path: Option<PathBuf>,

    /// The name of the Move module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 4)]
    module_name: Option<String>,

    /// The address of the Move module.
    #[clap(long = "address", display_order = 5, default_value = "0x1")]
    module_address: String,

    /// Math backend.
    #[clap(long = "math", short = 'm', default_value = "u128")]
    math_backend: String,

    /// Output of debugging information
    #[clap(short, long)]
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
