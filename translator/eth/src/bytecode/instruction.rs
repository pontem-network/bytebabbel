use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::OpCode;

pub type Offset = u64;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Instruction(pub Offset, pub OpCode);

impl Instruction {
    pub fn new(offset: Offset, code: OpCode) -> Self {
        Self(offset, code)
    }

    pub fn offset(&self) -> Offset {
        self.0
    }

    pub fn next(&self) -> Offset {
        self.0 + self.size() as Offset
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
