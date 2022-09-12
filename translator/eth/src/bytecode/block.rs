use crate::bytecode::instruction::{Instruction, Offset};
use crate::bytecode::loc::Loc;
use crate::OpCode;
use primitive_types::U256;
use std::fmt::{Debug, Display, Formatter};

pub type InstructionBlock = Loc<Vec<Instruction>>;

pub struct BlockIter<I: Iterator<Item = Instruction>> {
    inst: I,
    next: Option<Instruction>,
}

impl<I: Iterator<Item = Instruction>> BlockIter<I> {
    pub fn new(iter: I) -> Self {
        Self {
            inst: iter,
            next: None,
        }
    }
}

impl<I: Iterator<Item = Instruction>> Iterator for BlockIter<I> {
    type Item = InstructionBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let mut inst = self.next.take().or_else(|| self.inst.next())?;

        let start_index = inst.offset();
        let mut block = vec![];

        loop {
            if inst.1 == OpCode::JumpDest {
                if !block.is_empty() {
                    self.next = Some(inst);
                    let end = block.last().map(|i: &Instruction| i.0).unwrap_or_default();
                    return Some(Loc::new(start_index, end, block));
                } else {
                    block.push(inst);
                }
            } else {
                let current_index = inst.0;

                let end = inst.1.ends_basic_block();
                block.push(inst);
                if end {
                    return Some(Loc::new(start_index, current_index, block));
                }
            }

            inst = if let Some(inst) = self.inst.next() {
                inst
            } else {
                let end = block.last().map(|i| i.0).unwrap_or_default();
                return Some(Loc::new(start_index, end, block));
            }
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct BlockId(pub u128);

impl BlockId {
    pub fn hex(x: &str) -> BlockId {
        let mut buf = 0_u128.to_be_bytes();
        let f = hex::decode(x).unwrap();
        let start_idx = buf.len() - f.len();
        buf[start_idx..].copy_from_slice(&f);
        BlockId(u128::from_be_bytes(buf))
    }
}

impl Debug for BlockId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0.to_be_bytes()[6..]))
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<BlockId> for u128 {
    fn from(id: BlockId) -> Self {
        id.0
    }
}

impl From<u128> for BlockId {
    fn from(val: u128) -> Self {
        BlockId(val)
    }
}

impl From<U256> for BlockId {
    fn from(val: U256) -> Self {
        BlockId(val.as_u128())
    }
}

impl From<Offset> for BlockId {
    fn from(val: Offset) -> Self {
        BlockId::from(val as u128)
    }
}
