//! Simple EVM-bytecode disassembler.

use crate::abi::{Abi, FunHash};
use crate::bytecode::block::BlockId;
use crate::bytecode::executor::execution::FunctionFlow;
use crate::bytecode::executor::StaticExecutor;
use bytecode::pre_processing::ctor;
//use crate::bytecode::flow_graph::FlowBuilder;
use crate::bytecode::flow_graph::FlowBuilder;
use crate::bytecode::hir::ir::Hir;
use crate::bytecode::hir::HirTranslator;
use crate::bytecode::mir::translation::MirTranslator;
use crate::bytecode::types::{Function, U256};
use crate::program::Program;
use anyhow::Error;
use bytecode::block::BlockIter;
use bytecode::ops::InstructionIter;
pub use bytecode::ops::OpCode;
use bytecode::pre_processing::swarm::remove_swarm_hash;
use std::collections::HashMap;

pub mod abi;
pub mod bytecode;
pub mod function;
pub mod program;

pub fn transpile_program(
    name: &str,
    bytecode: &str,
    abi: &str,
    _contract_addr: U256,
) -> Result<Program, Error> {
    let abi = Abi::try_from(abi)?;
    let bytecode = parse_bytecode(bytecode)?;
    let blocks = BlockIter::new(InstructionIter::new(bytecode))
        .map(|block| (BlockId::from(block.start), block))
        .collect::<HashMap<_, _>>();
    let (contract, ctor) = ctor::split(blocks)?;

    let contract_flow = FlowBuilder::new(&contract).make_flow();

    let _hir = HirTranslator::new(&contract, contract_flow);
    let mut old_executor = StaticExecutor::new(&contract);

    let functions = abi
        .fun_hashes()
        .filter_map(|h| abi.entry(&h).map(|e| (h, e)))
        .map(|(h, entry)| {
            Function::try_from((h, entry))
                .and_then(|f| {
                    //todo new translator
                    // translate_function(&hir, f.clone(), contract_addr).unwrap();
                    old_executor.exec(f)
                })
                .map(|res| (h, res))
        })
        .collect::<Result<HashMap<FunHash, FunctionFlow>, _>>()?;
    Program::new(name, functions, ctor, abi)
}

pub fn translate_function(
    hir_translator: &HirTranslator,
    fun: Function,
    contract_addr: U256,
) -> Result<Hir, Error> {
    let hir = hir_translator.translate(fun.clone(), contract_addr)?;
    let mir_translator = MirTranslator::new(fun);
    let _mir = mir_translator.translate_hir(hir)?;
    todo!()
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
