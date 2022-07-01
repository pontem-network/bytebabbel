//! Simple EVM-bytecode disassembler.

use crate::evm::ops::InstructionIter;
use crate::evm::swarm::remove_swarm_hash;
use anyhow::Error;
pub use ops::OpCode;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

pub mod block;
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

pub struct Loc<C> {
    pub start: Offset,
    pub end: Offset,
    block: C,
}

impl<C> Loc<C> {
    pub fn new(start: Offset, end: Offset, block: C) -> Loc<C> {
        Loc { start, end, block }
    }

    pub fn map<F: FnOnce(C) -> R, R>(self, f: F) -> Loc<R> {
        Loc::new(self.start, self.end, f(self.block))
    }
}

impl<C> Deref for Loc<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

impl<C> DerefMut for Loc<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block
    }
}

impl<C: Debug> Debug for Loc<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nLoc: [{}; {}]\n{:?}", self.start, self.end, self.block)
    }
}

impl<C: Display> Display for Loc<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nBlock: [{}; {}]\n{}", self.start, self.end, self.block)
    }
}

pub type Offset = usize;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Instruction(Offset, OpCode);

impl Instruction {
    pub fn new(offset: Offset, code: OpCode) -> Self {
        Self(offset, code)
    }

    pub fn offset(&self) -> Offset {
        self.0
    }
}

impl Deref for Instruction {
    type Target = OpCode;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl DerefMut for Instruction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#06x}   ", self.0)?;
        match &self.1 {
            OpCode::Push(data) => write!(f, "Push({})", hex::encode(data)),
            OpCode::Dup(val) => write!(f, "Dup{}", val),
            OpCode::Swap(val) => write!(f, "Swap{}", val),
            OpCode::Log(val) => write!(f, "Log{}", val),
            OpCode::Invalid(opcode) => write!(f, "Invalid(0x{:02x})", opcode),
            _ => write!(f, "{:?}", self.1),
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
