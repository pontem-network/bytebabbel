//! Simple EVM-bytecode disassembler.

use std::collections::HashMap;

use anyhow::Error;
use ethabi::Contract;
use log::{log_enabled, trace};
use primitive_types::U256;

use bytecode::block::BlockIter;
use bytecode::ops::InstructionIter;
pub use bytecode::ops::OpCode;
use bytecode::pre_processing::swarm::remove_swarm_hash;
use program::Program;

use crate::abi::call::FunHash;
use crate::abi::MoveAbi;
use crate::bytecode::block::Offset;
use crate::bytecode::hir::ir::Hir;
use crate::bytecode::hir::HirBuilder;
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::MirTranslator;
use crate::bytecode::types::Function;
use crate::vm::static_initialization;

pub mod abi;
pub mod bytecode;
pub mod compile;
pub mod program;
pub mod vm;

pub fn transpile_program(
    name: &str,
    bytecode_str: &str,
    init_args: &str,
    abi_entries: &Contract,
    contract_addr: U256,
    flags: Flags,
) -> Result<Program, Error> {
    let (contract_code, constructor) =
        static_initialization(bytecode_str, abi_entries, init_args, contract_addr)?;
    if log_enabled!(log::Level::Trace) {
        trace!("Bytecode: {}", &hex::encode(&contract_code));
    }
    let contract_code_len = contract_code.len();

    let abi = MoveAbi::new(name, abi_entries)?;

    let contract = BlockIter::new(InstructionIter::new(contract_code))
        .map(|block| (block.start, block))
        .collect::<HashMap<_, _>>();

    let hir = HirBuilder::new(contract, flags, contract_addr, contract_code_len as u128)?;
    hir.translate_module_base()?;
    let functions = abi
        .functions()
        .iter()
        .map(|(hash, fun)| translate_function(&hir, fun, flags).map(|mir| (*hash, mir)))
        .collect::<Result<HashMap<FunHash, Mir>, _>>()?;
    Program::new(constructor, functions, abi)
}

pub fn translate_function(hir: &HirBuilder, fun: &Function, flags: Flags) -> Result<Mir, Error> {
    let hir = hir.translate_public_fun(fun)?;
    let mut buff = String::new();
    hir.print(&mut buff)?;
    trace!("{}", buff);
    let mir_translator = MirTranslator::new(fun, flags);
    let mir = mir_translator.translate(hir)?;
    mir.print(&fun.name);
    Ok(mir)
}

pub fn parse_bytecode(input: &str) -> Result<Vec<u8>, Error> {
    const HEX_PREFIX: &str = "0x";
    let input = if input[0..2] == *HEX_PREFIX {
        &input[(HEX_PREFIX.len())..]
    } else {
        input
    };
    let mut bytecode = hex::decode(input)?;
    remove_swarm_hash(&mut bytecode);
    Ok(bytecode)
}

#[derive(Copy, Clone, Debug)]
pub struct Flags {
    pub native_input: bool,
    pub native_output: bool,
    pub hidden_output: bool,
    pub u128_io: bool,
    pub package_interface: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for Flags {
    fn default() -> Self {
        Self {
            native_input: false,
            native_output: false,
            hidden_output: false,
            u128_io: false,
            package_interface: false,
        }
    }
}

impl Flags {
    pub fn native_interface() -> Self {
        Self {
            native_input: true,
            native_output: true,
            hidden_output: false,
            u128_io: false,
            package_interface: false,
        }
    }
}
