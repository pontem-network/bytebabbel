//! Simple EVM-bytecode disassembler.

use crate::abi::{Abi, FunHash};
use crate::bytecode::block::BlockId;
use crate::bytecode::ctor;
use crate::bytecode::executor::env::Function;
use crate::bytecode::executor::execution::FunctionFlow;
use crate::bytecode::executor::StaticExecutor;
use crate::program::Program;
use anyhow::Error;
use bytecode::block::BlockIter;
use bytecode::ops::InstructionIter;
pub use bytecode::ops::OpCode;
use bytecode::swarm::remove_swarm_hash;
use log::log_enabled;
use log::Level;
use std::collections::{BTreeMap, HashMap};

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
    let (blocks, ctor) = ctor::split(blocks)?;

    let mut executor = StaticExecutor::new(&blocks);
    let functions = abi
        .fun_hashes()
        .filter_map(|h| abi.entry(&h).map(|e| (h, e)))
        .map(|(h, entry)| {
            Function::try_from((h, entry))
                .and_then(|f| executor.exec(f))
                .map(|res| (h, res))
        })
        .collect::<Result<HashMap<FunHash, FunctionFlow>, _>>()?;
    Program::new(name, functions, ctor, abi)
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

pub fn is_trace() -> bool {
    log_enabled!(Level::Trace)
}