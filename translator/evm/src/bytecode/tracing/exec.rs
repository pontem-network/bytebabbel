use crate::bytecode::block::InstructionBlock;
use crate::{BlockId, OpCode, U256};

#[derive(Debug, Clone, Default)]
pub struct Executor {
    call_stack: Vec<BlockId>,
    path: Vec<BlockId>,
}

impl Executor {
    fn pop_stack(&mut self, count: usize) -> Vec<BlockId> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            res.push(self.call_stack.pop().unwrap_or_default());
        }
        res
    }

    fn push_stack(&mut self, to_push: Vec<BlockId>) {
        self.call_stack.extend(to_push.into_iter().rev());
    }

    pub fn exec_block(&mut self, block: &InstructionBlock) -> Next {
        if let Some(inst) = block.first() {
            self.path.push(BlockId::from(inst.0));
        } else {
            return Next::Stop;
        };

        for inst in block.iter() {
            let mut ops = self.pop_stack(inst.pops());
            match &inst.1 {
                OpCode::Jump => return Next::Jmp(ops.remove(0)),
                OpCode::JumpIf => {
                    let jmp = ops.remove(0);
                    return Next::Cnd(jmp, BlockId(inst.next()));
                }
                OpCode::Return | OpCode::Stop | OpCode::Revert | OpCode::SelfDestruct => {
                    return Next::Stop
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
                        self.push_stack(vec![BlockId::from(val.as_usize())]);
                    } else {
                        self.push_stack(vec![BlockId::default()]);
                    }
                }
                _ => {
                    let pushes = inst.pushes();
                    if pushes > 0 {
                        self.push_stack((0..pushes).map(|_| BlockId::default()).collect());
                    }
                }
            }
        }
        block
            .last()
            .map(|last| Next::Jmp(BlockId(last.next())))
            .unwrap_or(Next::Stop)
    }
    
    pub fn in_path(&self, id: BlockId) -> bool {
        self.path.contains(&id)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Next {
    Jmp(BlockId),
    Stop,
    Cnd(BlockId, BlockId),
}
