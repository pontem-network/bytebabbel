//! Simple EVM-bytecode disassembler.

use crate::abi::entries::{AbiEntries, FunHash};
use crate::bytecode::block::BlockId;
use crate::bytecode::flow_graph::FlowBuilder;
use crate::bytecode::hir::ir::Hir;
use crate::bytecode::hir::HirTranslator;
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::MirTranslator;
use crate::bytecode::pre_processing::ctor;
use crate::bytecode::types::{Function, U256};
use abi::abi::Abi;
use anyhow::{anyhow, Error};
use bytecode::block::BlockIter;
use bytecode::ops::InstructionIter;
pub use bytecode::ops::OpCode;
use bytecode::pre_processing::swarm::remove_swarm_hash;
use log::{log_enabled, trace};
use program::Program;
use std::collections::HashMap;

pub mod abi;
pub mod bytecode;
pub mod program;

pub fn transpile_program(
    name: &str,
    bytecode_str: &str,
    abi: &str,
    contract_addr: U256,
) -> Result<Program, Error> {
    let abi = Abi::new(name, AbiEntries::try_from(abi)?)?;
    let bytecode = parse_bytecode(bytecode_str)?;
    let bytecode_len = bytecode.len() as u128;
    let blocks = BlockIter::new(InstructionIter::new(bytecode))
        .map(|block| (BlockId::from(block.start), block))
        .collect::<HashMap<_, _>>();

    let (constructor, entry_point) =
        translate_constructor(&blocks, &abi, contract_addr, bytecode_len as u128)?;
    let contract = ctor::replace(blocks, entry_point);

    if log_enabled!(log::Level::Trace) {
        trace!("Entry point: {}", entry_point);
        trace!("Bytecode: {}", &bytecode_str[entry_point.0 * 2..]);
    }

    let contract_flow = FlowBuilder::new(&contract).make_flow();
    let hir = HirTranslator::new(&contract, contract_flow);

    let functions = abi
        .functions()
        .iter()
        .map(|(hash, fun)| {
            translate_function(&hir, fun, contract_addr, bytecode_len as u128)
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
) -> Result<Mir, Error> {
    let hir = hir_translator.translate_fun(fun, contract_addr, code_size)?;
    let mir_translator = MirTranslator::new(&fun, false);
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

pub fn translate_constructor(
    contract: &HashMap<BlockId, bytecode::block::InstructionBlock>,
    abi: &Abi,
    contract_addr: U256,
    code_size: u128,
) -> Result<(Mir, BlockId), Error> {
    let contract_flow = FlowBuilder::new(&contract).make_flow();
    let hir = HirTranslator::new(&contract, contract_flow);
    let hir = hir.translate_constractor(abi.constructor(), contract_addr, code_size)?;

    let block = *hir
        .get_code_copy()
        .last()
        .ok_or_else(|| anyhow!("Expected CodeCopy at the end of constructor"))?;

    let constructor = abi.constructor().into();
    let mir_translator = MirTranslator::new(&constructor, true);
    let mir = mir_translator.translate(hir)?;
    mir.print("constructor");
    Ok((mir, block))
}
