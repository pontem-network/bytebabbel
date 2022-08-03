use anyhow::Result;
use clap::Parser;

pub mod convert;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub enum Args {
    Translator(convert::Converting),
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

    match args {
        Args::Translator(data) => data.execute(),
        _ => todo!(),
    }
}
