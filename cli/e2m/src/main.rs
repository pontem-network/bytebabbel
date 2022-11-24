use std::future::Future;
use std::process::exit;

use anyhow::Result;
use clap::Parser;

use test_infra::color::font_red;
use test_infra::init_log;

use crate::convert::CmdConvert;

pub mod convert;
pub mod profile;

pub mod call;
pub mod resources;
pub mod txflags;

pub trait Cmd {
    fn execute(&mut self) -> Result<String>;
}

#[derive(Parser, Debug)]
#[clap(version, about)]
pub enum Args {
    /// Converting a sol script to move binary code.
    Convert(CmdConvert),

    /// Run a Move function
    Call(crate::call::CmdCall),

    /// Command to list resources, modules, or other items owned by an address
    Resources(crate::resources::CmdResources),
}

impl Cmd for Args {
    fn execute(&mut self) -> Result<String> {
        match self {
            Args::Convert(data) => data.execute(),
            Args::Call(data) => data.execute(),
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
            exit(1)
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
