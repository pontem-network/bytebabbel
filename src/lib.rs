extern crate core;

use anyhow::Error;

pub mod evm;
pub mod mv;

pub fn translate(_bytecode: &str) -> Result<(), Error> {
    Ok(())
}
