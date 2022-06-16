#[macro_use]
extern crate log;
extern crate clap;
extern crate eth2move;
extern crate serde_json;
use eth2move::cfg::Abi;
use serde::de::Deserialize;
use serde::de::DeserializeOwned;

use std::io::prelude::*;
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
    // read config:
    let mut config: Cfg = {
        let mut buf = String::new();
        let _ = File::open(args.config)?.read_to_string(&mut buf)?;
        serde_json::from_str(&buf)?
    };

    // overrides -> args.config:
    {
        if let Some(address) = args.overrides.address.take() {
            config.address = address;
        }

        if let Some(overs) = args.overrides.mapping {
            config.mapping.extend(overs.mapping.into_iter());
        }
    }

    // open input for feature read in translation:
    let input = File::open(args.input)?;
    // read optional abi:
    let abi: Option<Abi> = if let Some(file) = args.abi.map(File::open) {
        let mut buf = String::new();
        let _ = file?.read_to_string(&mut buf)?;
        Some(serde_json::from_str(&buf)?)
    } else {
        None
    };
    // open output for feature write at translation:
    let output = File::create(args.output)?;

    translate(input, output, abi, config)?;

    Ok(())
}
