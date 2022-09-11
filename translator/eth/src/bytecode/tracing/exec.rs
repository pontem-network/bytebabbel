use crate::bytecode::block::InstructionBlock;
use crate::bytecode::instruction::Offset;
use crate::{BlockId, OpCode, U256};
use anyhow::{anyhow, Error};
use std::mem;

#[derive(Debug, Clone, Default)]
pub struct Executor {
    call_stack: Vec<StackItem>,
    pub path: Vec<BlockId>,
    negative_stack_seq: usize,
    negative_item_used: Vec<StackItem>,
}

impl Executor {
    fn pop_stack(&mut self, count: usize, offset: Offset) -> Vec<StackItem> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            res.push(self.call_stack.pop().unwrap_or_else(|| {
                self.negative_stack_seq += 1;
                StackItem::Negative(self.negative_stack_seq, offset)
            }));
        }
        res
    }

    fn push_stack(&mut self, to_push: Vec<StackItem>) {
        self.call_stack.extend(to_push.into_iter().rev());
    }

    pub fn exec_one(&mut self, block: &InstructionBlock) -> BlockResult {
        let next = self.exec(block);
        BlockResult {
            next,
            input: self.negative_item_used.clone(),
            output: mem::take(&mut self.call_stack),
        }
    }

    pub fn exec(&mut self, block: &InstructionBlock) -> Next {
        if let Some(inst) = block.first() {
            self.path.push(BlockId::from(inst.0));
        } else {
            return Next::Stop;
        };

        for inst in block.iter() {
            let mut ops = self.pop_stack(inst.pops(), inst.offset());
            match &inst.1 {
                OpCode::Jump => {
                    return Next::Jmp(ops.remove(0));
                }
                OpCode::JumpIf => {
                    let jmp = ops.remove(0);
                    return Next::Cnd(
                        jmp,
                        StackItem::Positive(BlockId(inst.next()), inst.offset()),
                    );
                }
                OpCode::Return | OpCode::Stop | OpCode::Revert | OpCode::SelfDestruct => {
                    return Next::Stop;
                }
                OpCode::Dup(_) => {
                    let new_item = ops[ops.len() - 1];
                    ops.insert(0, new_item);
                    self.push_stack(ops);
                }
                OpCode::Swap(_) => {
                    let last_index = ops.len() - 1;
                    ops.swap(0, last_index);
                    self.push_stack(ops);
                }
                OpCode::Push(val) => {
                    let val = U256::from(val.as_slice());
                    if val <= U256::from(u32::MAX) {
                        self.push_stack(vec![StackItem::Positive(
                            BlockId::from(val.as_usize()),
                            inst.offset(),
                        )]);
                    } else {
                        self.push_stack(vec![StackItem::Positive(
                            BlockId::default(),
                            inst.offset(),
                        )]);
                    }
                }
                _ => {
                    for op in ops {
                        if op.is_negative() {
                            self.negative_item_used.push(op);
                        }
                    }

                    let pushes = inst.pushes();
                    if pushes > 0 {
                        self.push_stack(
                            (0..pushes)
                                .map(|_| StackItem::Positive(BlockId::default(), inst.offset()))
                                .collect(),
                        );
                    }
                }
            }
        }
        block
            .last()
            .map(|last| Next::Jmp(StackItem::Positive(BlockId(last.next()), last.offset())))
            .unwrap_or(Next::Stop)
    }

    pub fn in_path(&self, id: BlockId) -> bool {
        self.path.contains(&id)
    }

    pub fn take_while(&self, id: BlockId) -> Vec<BlockId> {
        self.path
            .iter()
            .rev()
            .take_while(|&x| x != &id)
            .cloned()
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StackItem {
    Negative(usize, Offset),
    Positive(BlockId, Offset),
    Calc(usize, Offset),
}

impl StackItem {
    pub fn is_negative(&self) -> bool {
        match self {
            StackItem::Negative(_, _) => true,
            _ => false,
        }
    }

    pub fn is_positive(&self) -> bool {
        !self.is_negative()
    }

    pub fn as_positive(&self) -> Result<BlockId, Error> {
        match self {
            StackItem::Positive(id, _) => Ok(*id),
            StackItem::Negative(id, _) => Err(anyhow!("Negative stack item: {} as jump", id)),
            StackItem::Calc(_, _) => Err(anyhow!("Calc stack item as jump")),
        }
    }

    pub fn as_negative(&self) -> Option<usize> {
        match self {
            StackItem::Negative(id, _) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Next {
    Jmp(StackItem),
    Stop,
    Cnd(StackItem, StackItem),
}

#[derive(Clone, Debug)]
pub struct BlockResult {
    pub next: Next,
    pub input: Vec<StackItem>,
    pub output: Vec<StackItem>,
}
