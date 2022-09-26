use std::future::Future;

use anyhow::Result;
use clap::Parser;
use test_infra::init_log;

pub mod convert;
pub mod profile;

#[cfg(feature = "deploy")]
pub mod call;
#[cfg(feature = "deploy")]
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

    /// Run a Move function
    #[cfg(feature = "deploy")]
    Call(crate::call::CmdCall),
}

impl Cmd for Args {
    fn execute(&self) -> Result<String> {
        match self {
            Args::Convert(data) => data.execute(),

            #[cfg(feature = "deploy")]
            Args::Call(data) => data.execute(),
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

pub fn wait<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
