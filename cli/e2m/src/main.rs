#[cfg(feature = "deploy")]
use std::future::Future;

use anyhow::Result;
use clap::Parser;

use test_infra::color::font_red;
use test_infra::init_log;

use crate::convert::CmdConvert;

pub mod convert;
pub mod profile;

#[cfg(feature = "deploy")]
pub mod call;
#[cfg(feature = "deploy")]
pub mod resources;
#[cfg(feature = "deploy")]
pub mod txflags;

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

    /// Command to list resources, modules, or other items owned by an address
    #[cfg(feature = "deploy")]
    Resources(crate::resources::CmdResources),
}

impl Cmd for Args {
    fn execute(&self) -> Result<String> {
        match self {
            Args::Convert(data) => data.execute(),

            #[cfg(feature = "deploy")]
            Args::Call(data) => data.execute(),

            #[cfg(feature = "deploy")]
            Args::Resources(data) => data.execute(),
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
            println!("[{}]: {err:?}", font_red("Error"));
        }
    }
}

#[cfg(feature = "deploy")]
pub fn wait<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
