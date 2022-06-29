extern crate core;

use anyhow::Error;
use move_compiler::compiled_unit::NamedCompiledModule;

pub mod evm;
pub mod translator;

pub use translator::{CodeUnit, Translator};


pub fn translate(bytecode: &str) -> Result<NamedCompiledModule, Error> {
    let translator = Translator::new();
    translator.translate(bytecode)
}
