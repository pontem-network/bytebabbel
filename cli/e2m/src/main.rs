use anyhow::Result;
use clap::Parser;

pub mod aptos_commands;
pub mod convert;

#[derive(Parser)]
#[clap(version, about)]
pub enum Args {
    Translator(convert::Converting),

    #[clap(subcommand)]
    Aptos(Box<aptos_commands::Tools>),
}

fn main() {
    match run() {
        Ok(result) => {
            println!("{result}");
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
        Args::Translator(data) => data.execute().map(|path| format!("Saved in: {path:?}")),
        Args::Aptos(data) => data.execute(),
    }
}
