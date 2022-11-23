use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, SubAssign};

use primitive_types::U256;

use crate::bytecode::instruction::Instruction;
use crate::bytecode::loc::Loc;
use crate::OpCode;

pub type InstructionBlock = Loc<Vec<Instruction>>;

pub struct BlockIter<'a, I: Iterator<Item = Instruction>> {
    inst: &'a mut I,
    next: Option<Instruction>,
}

impl<'a, I: Iterator<Item = Instruction>> BlockIter<'a, I> {
    pub fn new(iter: &'a mut I) -> Self {
        Self {
            inst: iter,
            next: None,
        }
    }
}

impl<'a, I: Iterator<Item = Instruction>> Iterator for BlockIter<'a, I> {
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
pub struct Offset(pub u128);

impl Offset {
    pub fn hex(x: &str) -> Offset {
        let mut buf = 0_u128.to_be_bytes();
        let f = hex::decode(x).unwrap();
        let start_idx = buf.len() - f.len();
        buf[start_idx..].copy_from_slice(&f);
        Offset(u128::from_be_bytes(buf))
    }
}

impl Debug for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0.to_be_bytes()[14..]))
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Offset> for u128 {
    fn from(id: Offset) -> Self {
        id.0
    }
}

impl From<u128> for Offset {
    fn from(val: u128) -> Self {
        Offset(val)
    }
}

impl From<U256> for Offset {
    fn from(val: U256) -> Self {
        Offset(val.as_u128())
    }
}

impl From<usize> for Offset {
    fn from(val: usize) -> Self {
        Offset(val as u128)
    }
}

impl From<i32> for Offset {
    fn from(val: i32) -> Self {
        Offset(val as u128)
    }
}

impl Add for Offset {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Offset(self.0 + rhs.0)
    }
}

impl Add<u128> for Offset {
    type Output = Self;

    fn add(self, rhs: u128) -> Self::Output {
        Offset(self.0 + rhs)
    }
}

impl Add<usize> for Offset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Offset(self.0 + rhs as u128)
    }
}

impl AddAssign for Offset {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Offset {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
