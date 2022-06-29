//! Simple EVM-bytecode disassembler.

use crate::evm::ops::InstructionIter;
use crate::evm::swarm::remove_swarm_hash;
use anyhow::Error;
pub use ops::OpCode;

mod block;
mod flow_graph;
pub mod ops;
mod swarm;

pub fn parse_bytecode(input: &str) -> Result<InstructionIter, Error> {
    const HEX_PREFIX: &str = "0x";
    let input = if input[0..2] == *HEX_PREFIX {
        &input[(HEX_PREFIX.len())..]
    } else {
        input
    };
    let mut bytecode = hex::decode(input)?;
    remove_swarm_hash(&mut bytecode);
    Ok(InstructionIter::new(bytecode))
}
