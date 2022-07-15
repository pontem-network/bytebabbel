extern crate core;

use crate::evm::parse_program;
use crate::mv::mvir::MvModule;
use anyhow::Error;
use move_core_types::account_address::AccountAddress;

pub mod evm;
pub mod mv;

pub fn translate(
    addr: AccountAddress,
    name: &str,
    bytecode: &str,
    abi: &str,
    trace: bool,
) -> Result<Vec<u8>, Error> {
    let program = parse_program(name, bytecode, abi, trace)?;
    let module = MvModule::from_evm_program(addr, program)?;
    let compiled_module = module.make_move_module()?;
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode)?;
    Ok(bytecode)
}
