#[macro_use]
extern crate log;
extern crate clap;
extern crate eth2move;

use std::fs::File;
use std::path::PathBuf;

use eth2move::*;
use clap::Parser;
use eth2move::cfg::Cfg;
use eth2move::cfg::CfgOverride;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    input: PathBuf,
    #[clap(long, value_parser)]
    abi: Option<PathBuf>,

    #[clap(short, long, value_parser)]
    output: PathBuf,

    #[clap(short, long, value_parser)]
    config: PathBuf,

    #[clap(flatten)]
    overrides: CfgOverride,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    run(args)
        .map_err(|err| {
            error!("{}", err);
            err
        })
        .ok();
}

fn run(mut args: Args) -> Result<(), error::Error> {
    env_logger::init();

	 let config = File::open(args.config)?;


    // overrides -> args.config
    {
        // if let
    }

    let input = File::open(args.input)?;
    let abi = if let Some(file) = args.abi.map(File::open) {
        Some(file?)
    } else {
        None
    };

    let output = File::create(args.output)?;

    translate(input, output, abi, config)?;

    Ok(())
}
