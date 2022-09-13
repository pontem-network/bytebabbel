//! Simple EVM-bytecode disassembler.

use crate::abi::entries::{AbiEntries, FunHash};
use crate::abi::MoveAbi;
use crate::bytecode::block::BlockId;
use crate::bytecode::flow_graph::FlowBuilder;
use crate::bytecode::hir::ir::Hir;
use crate::bytecode::hir::HirTranslator;
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::MirTranslator;
use crate::bytecode::types::Function;
use crate::vm::static_initialization;
use anyhow::Error;
use bytecode::block::BlockIter;
use bytecode::ops::InstructionIter;
pub use bytecode::ops::OpCode;
use bytecode::pre_processing::swarm::remove_swarm_hash;
use log::{log_enabled, trace};
use primitive_types::U256;
use program::Program;
use std::collections::HashMap;

pub mod abi;
pub mod bytecode;
pub mod program;
pub mod vm;

pub fn transpile_program(
    name: &str,
    bytecode_str: &str,
    init_args: &str,
    abi_entries: &AbiEntries,
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
        .map(|block| (BlockId::from(block.start), block))
        .collect::<HashMap<_, _>>();

    let mut flow_builder = FlowBuilder::new(&contract)?;
    let contract_flow = flow_builder.make_flow();
    let block_io = flow_builder.block_io();
    let hir = HirTranslator::new(&contract, contract_flow, block_io, flags);

    let functions = abi
        .functions()
        .iter()
        .map(|(hash, fun)| {
            translate_function(&hir, fun, contract_addr, contract_code_len as u128, flags)
                .map(|mir| (*hash, mir))
        })
        .collect::<Result<HashMap<FunHash, Mir>, _>>()?;
    Program::new(constructor, functions, abi)
}

pub fn translate_function(
    hir_translator: &HirTranslator,
    fun: &Function,
    contract_addr: U256,
    code_size: u128,
    flags: Flags,
) -> Result<Mir, Error> {
    let hir = hir_translator.translate_fun(fun, contract_addr, code_size)?;
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
