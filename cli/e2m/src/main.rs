use anyhow::Result;
use clap::Parser;
use test_infra::init_log;

pub mod convert;

pub mod profile;

use crate::convert::CmdConvert;

pub trait Cmd {
    fn execute(&self) -> Result<String>;
}

#[derive(Parser, Debug)]
#[clap(version, about)]
pub enum Args {
    /// Converting a sol script to move binary code.
    Convert(CmdConvert),

    /// @todo Call a Move function
    Call,

    /// @todo Command to list resources, modules, or other items owned by an address
    Resources,
}

impl Cmd for Args {
    fn execute(&self) -> Result<String> {
        match self {
            Args::Convert(data) => data.execute(),
            Args::Call => {
                todo!()
            }
            Args::Resources => {
                todo!()
            }
        }
    }
}

fn main() {
    init_log();

    match Args::parse().execute() {
        Ok(result) => {
            println!("{result}");
        }
        Err(err) => {
            println!("Error: {err:?}");
        }
    }
}
