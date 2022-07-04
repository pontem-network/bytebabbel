use crate::evm::bytecode::instruction::Instruction;
use crate::evm::bytecode::loc::Loc;
use crate::evm::OpCode;

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
