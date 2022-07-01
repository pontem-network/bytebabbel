//! Simple EVM-bytecode disassembler.

use crate::evm::block::BlockIter;
use crate::evm::ops::InstructionIter;
use crate::evm::swarm::remove_swarm_hash;
use anyhow::Error;
pub use ops::OpCode;
use std::collections::BTreeMap;

pub mod block;
pub mod flow_graph;
pub mod instruction;
pub mod loc;
pub mod ops;
pub mod statement;
pub mod swarm;

pub fn parse_bytecode(input: &str) -> Result<(), Error> {
    const HEX_PREFIX: &str = "0x";
    let input = if input[0..2] == *HEX_PREFIX {
        &input[(HEX_PREFIX.len())..]
    } else {
        input
    };
    let mut bytecode = hex::decode(input)?;
    remove_swarm_hash(&mut bytecode);

    let blocks = BlockIter::new(InstructionIter::new(bytecode))
        .map(statement::mark_stack)
        .map(|block| (block.id(), block))
        .collect::<BTreeMap<_, _>>();

    Ok(())
}
