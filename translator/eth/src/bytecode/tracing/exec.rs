use std::mem;

use anyhow::{anyhow, Error};

use crate::bytecode::block::InstructionBlock;
use crate::{Offset, OpCode, U256};

#[derive(Debug, Clone, Default)]
pub struct Executor {
    call_stack: Vec<StackItem>,
    pub path: Vec<Offset>,
    negative_stack_seq: usize,
    negative_stack: Vec<StackItem>,
}

impl Executor {
    fn pop_stack(&mut self, count: usize, offset: Offset) -> Vec<StackItem> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            res.push(self.call_stack.pop().unwrap_or_else(|| {
                self.negative_stack_seq += 1;
                let item = StackItem::Negative {
                    id: self.negative_stack_seq,
                    offset,
                };
                self.negative_stack.push(item.clone());
                item
            }));
        }
        res
    }

    fn push_stack(&mut self, to_push: Vec<StackItem>) {
        self.call_stack.extend(to_push.into_iter().rev());
    }

    pub fn negative_stack(&self) -> &[StackItem] {
        &self.negative_stack
    }

    pub fn call_stack(&self) -> &[StackItem] {
        &self.call_stack
    }

    pub fn into_io(self) -> (Vec<StackItem>, Vec<StackItem>) {
        (self.negative_stack, self.call_stack)
    }

    pub fn exec(&mut self, block: &InstructionBlock) -> Next {
        if let Some(inst) = block.first() {
            self.path.push(inst.0);
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
                        StackItem::Positive {
                            value: inst.next(),
                            offset: inst.offset(),
                        },
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
                        self.push_stack(vec![StackItem::Positive {
                            value: Offset::from(val),
                            offset: inst.offset(),
                        }]);
                    } else {
                        self.push_stack(vec![StackItem::Calc(inst.offset())]);
                    }
                }
                OpCode::Pop => {}
                _ => {
                    let pushes = inst.pushes();
                    if pushes > 0 {
                        self.push_stack(
                            (0..pushes)
                                .map(|_| StackItem::Calc(inst.offset()))
                                .collect(),
                        );
                    }
                }
            }
        }
        block
            .last()
            .map(|last| {
                Next::Jmp(StackItem::Positive {
                    value: last.next(),
                    offset: last.offset(),
                })
            })
            .unwrap_or(Next::Stop)
    }

    pub fn in_path(&self, id: Offset) -> bool {
        self.path.contains(&id)
    }

    pub fn take_while(&self, id: Offset) -> Vec<Offset> {
        self.path
            .iter()
            .rev()
            .take_while(|&x| x != &id)
            .cloned()
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum StackItem {
    Negative { id: usize, offset: Offset },
    Positive { value: Offset, offset: Offset },
    Calc(Offset),
}

impl StackItem {
    pub fn is_negative(&self) -> bool {
        matches!(self, StackItem::Negative { .. })
    }

    pub fn is_positive(&self) -> bool {
        !self.is_negative()
    }

    pub fn as_positive(&self) -> Result<Offset, Error> {
        match self {
            StackItem::Positive { value, offset: _ } => Ok(*value),
            StackItem::Negative { id, offset: _ } => {
                Err(anyhow!("Negative stack item: {} as jump", id))
            }
            StackItem::Calc(_) => Err(anyhow!("Calc stack item as jump")),
        }
    }

    pub fn as_negative(&self) -> Option<(usize, Offset)> {
        match self {
            StackItem::Negative { id, offset } => Some((*id, *offset)),
            _ => None,
        }
    }

    pub fn offset(&self) -> Offset {
        match self {
            StackItem::Negative { offset, .. } => *offset,
            StackItem::Positive { offset, .. } => *offset,
            StackItem::Calc(offset) => *offset,
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
