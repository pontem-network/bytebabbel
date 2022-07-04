//! Simple EVM-bytecode disassembler.

use crate::evm::abi::Abi;
use crate::evm::bytecode::ctor;
use crate::evm::bytecode::executor::BlockId;
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
pub mod function;
pub mod program;

pub fn parse_program(name: &str, bytecode: &str, abi: &str) -> Result<Program, Error> {
    let abi = Abi::try_from(abi)?;
    let bytecode = parse_bytecode(bytecode)?;

    let blocks = BlockIter::new(InstructionIter::new(bytecode))
        .map(|block| (BlockId::from(block.start), block))
        .collect::<BTreeMap<_, _>>();
    let (blocks, ctor) = ctor::split_1(blocks);
    let blocks = executor::mark_stack_1(blocks);

    // let blocks = BlockIter::new(InstructionIter::new(bytecode))
    //     .map(executor::mark_stack)
    //     .map(|block| (block.id(), block))
    //     .collect::<BTreeMap<_, _>>();
    // let (blocks, ctor) = ctor::split(blocks);
    // Program::new(name, blocks, ctor, abi)
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
