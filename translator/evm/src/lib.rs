//! Simple EVM-bytecode disassembler.

use crate::abi::{Abi, FunHash};
use crate::bytecode::block::BlockId;
use crate::bytecode::ctor;
use crate::bytecode::executor::execution::FunctionFlow;
use crate::bytecode::executor::StaticExecutor;
use crate::bytecode::flow_graph::FlowBuilder;
use crate::bytecode::llir::ir::Ir;
use crate::bytecode::llir::Translator;
use crate::bytecode::types::{Function, U256};
use crate::program::Program;
use anyhow::Error;
use bytecode::block::BlockIter;
use bytecode::ops::InstructionIter;
pub use bytecode::ops::OpCode;
use bytecode::swarm::remove_swarm_hash;
use std::collections::HashMap;

pub mod abi;
pub mod bytecode;
pub mod function;
pub mod program;

pub fn parse_program(
    name: &str,
    bytecode: &str,
    abi: &str,
    contract_addr: U256,
) -> Result<Program, Error> {
    let abi = Abi::try_from(abi)?;
    let bytecode = parse_bytecode(bytecode)?;

    let blocks = BlockIter::new(InstructionIter::new(bytecode))
        .map(|block| (BlockId::from(block.start), block))
        .collect::<HashMap<_, _>>();
    let (contract, ctor) = ctor::split(blocks)?;

    let contract_flow = FlowBuilder::new(&contract).make_flow();
    let llir = Translator::new(&contract, contract_flow);

    let mut old_executor = StaticExecutor::new(&contract);

    let functions = abi
        .fun_hashes()
        .filter_map(|h| abi.entry(&h).map(|e| (h, e)))
        .map(|(h, entry)| {
            Function::try_from((h, entry))
                .and_then(|f| {
                    translate_function(&llir, f.clone(), contract_addr).unwrap();
                    old_executor.exec(f)
                })
                .map(|res| (h, res))
        })
        .collect::<Result<HashMap<FunHash, FunctionFlow>, _>>()?;
    Program::new(name, functions, ctor, abi)
}

pub fn translate_function(
    llir: &Translator,
    fun: Function,
    contract_addr: U256,
) -> Result<Ir, Error> {
    llir.translate(fun, contract_addr)
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
