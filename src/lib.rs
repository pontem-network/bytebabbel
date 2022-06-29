extern crate core;

use anyhow::Error;
use move_binary_format::CompiledModule;

pub mod evm;

pub fn translate(_bytecode: &str) -> Result<CompiledModule, Error> {
    let module = CompiledModule::default();

    Ok(module)
}
