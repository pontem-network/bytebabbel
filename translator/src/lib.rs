use anyhow::{bail, Error};
use evm::bytecode::types::U256;
use evm::transpile_program;
use move_core_types::account_address::AccountAddress;
use mv::translator::MvIrTranslator;
use std::str::FromStr;

pub const MAX_MEMORY: u64 = 1024 * 32;

pub fn translate(
    addr: AccountAddress,
    name: &str,
    bytecode: &str,
    abi: &str,
    _model: Math,
) -> Result<Vec<u8>, Error> {
    let program = transpile_program(name, bytecode, abi, U256::from(addr.as_slice()))?;
    let mvir = MvIrTranslator::new(addr, program.name());
    let module = mvir.translate(MAX_MEMORY, program)?;
    let compiled_module = module.make_move_module()?;
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode)?;
    Ok(bytecode)
}

#[derive(Debug, Clone, Copy)]
pub enum Math {
    U128,
    U256,
}

impl FromStr for Math {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "u128" => Math::U128,
            "u256" => Math::U256,
            _ => bail!("Unsupported math backend."),
        })
    }
}
