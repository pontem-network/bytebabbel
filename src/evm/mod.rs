//! Simple EVM-bytecode disassembler.

use crate::evm::abi::Abi;
use crate::evm::bytecode::ctor;
use crate::evm::bytecode::executor::block::BlockId;
use crate::evm::flow_graph::FlowGraph;
use crate::evm::program::Program;
use anyhow::Error;
use bytecode::block::BlockIter;
use bytecode::executor;
use bytecode::ops::InstructionIter;
pub use bytecode::ops::OpCode;
use bytecode::swarm::remove_swarm_hash;
use std::collections::BTreeMap;

pub mod abi;
pub mod bytecode;
pub mod flow_graph;
pub mod function;
pub mod program;

pub fn parse_program(name: &str, bytecode: &str, abi: &str) -> Result<Program, Error> {
    let abi = Abi::try_from(abi)?;
    let bytecode = parse_bytecode(bytecode)?;

    let blocks = BlockIter::new(InstructionIter::new(bytecode))
        .map(|block| (BlockId::from(block.start), block))
        .collect::<BTreeMap<_, _>>();
    let (blocks, ctor) = ctor::split(blocks);

    let functions_graph = abi
        .fun_hashes()
        .filter_map(|h| abi.entry(&h).map(|e| (h, e.inputs.len())))
        .map(|(h, input_size)| (h, FlowGraph::new(executor::exec(&blocks, h, input_size))))
        .collect();

    Program::new(name, functions_graph, ctor, abi)
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
