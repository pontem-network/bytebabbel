use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

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
        short = 'a',
        default_value = "0x1",
        value_parser
    )]
    move_module_address: String,

    /// Math backend.
    #[clap(long = "math", short = 'm', default_value = "u128", value_parser)]
    math_backend: String,
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
    env_logger::init();

    Convert::try_from(args)?
        .create_mv()
        .map(|path| path.to_string_lossy().to_string())
}
