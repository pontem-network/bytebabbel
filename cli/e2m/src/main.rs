use crate::call::CmdCall;
use anyhow::Result;
use clap::Parser;
use test_infra::init_log;

pub mod call;
pub mod convert;
pub mod profile;
pub mod txflags;

use crate::convert::CmdConvert;

pub trait Cmd {
    fn execute(&self) -> Result<String>;
}

#[derive(Parser, Debug)]
#[clap(version, about)]
pub enum Args {
    /// Converting a sol script to move binary code.
    Convert(CmdConvert),

    // "aptos" is used for the call
    #[cfg(feature = "deploy")]
    /// Run a Move function
    Call(CmdCall),

    // "aptos" is used for the view resources
    #[cfg(feature = "deploy")]
    /// @todo Command to list resources, modules, or other items owned by an address
    Resources,
}

impl Cmd for Args {
    fn execute(&self) -> Result<String> {
        match self {
            Args::Convert(data) => data.execute(),
            Args::Call(..) => {
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
