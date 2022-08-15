use anyhow::{bail, Error};
use evm::bytecode::types::U256;
use evm::transpile_program;
use move_core_types::account_address::AccountAddress;
use mv::mvir::MvModule;
use std::str::FromStr;

pub fn translate(
    addr: AccountAddress,
    name: &str,
    bytecode: &str,
    abi: &str,
    model: Math,
) -> Result<Vec<u8>, Error> {
    let program = transpile_program(name, bytecode, abi, U256::from(addr.as_slice()))?;
    let module = match model {
        Math::U128 => MvModule::from_evm_program(addr, program)?,
        Math::U256 => MvModule::from_evm_program(addr, program)?,
    };
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
