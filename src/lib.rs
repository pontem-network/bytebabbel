extern crate core;

use crate::evm::parse_program;
use crate::mv::function::code::intrinsic::math::u128_model::U128MathModel;
use crate::mv::function::code::intrinsic::math::u256_model::U256MathModel;
use crate::mv::mvir::MvModule;
use anyhow::{bail, Error};
use move_core_types::account_address::AccountAddress;
use std::str::FromStr;

pub mod evm;
pub mod mv;

pub fn translate(
    addr: AccountAddress,
    name: &str,
    bytecode: &str,
    abi: &str,
    trace: bool,
    model: Math,
) -> Result<Vec<u8>, Error> {
    let program = parse_program(name, bytecode, abi, trace)?;
    let module = match model {
        Math::U128 => MvModule::from_evm_program(addr, U128MathModel::default(), program)?,
        Math::U256 => MvModule::from_evm_program(addr, U256MathModel::default(), program)?,
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
