use crate::evm::block::BasicBlock as InstructionBLock;
use crate::evm::instruction::{Instruction, Offset};
use crate::evm::loc::Loc;
use crate::evm::OpCode;
use bigint::U256;
use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub fn mark_stack(block: InstructionBLock) -> Loc<BasicBlock> {
    let mut stack = ExecutionStack::default();
    let mut statement_block = block.wrap(BasicBlock::new(block.start));

    for inst in block.inner().into_iter() {
        let pops = inst.pops();
        let pushes = inst.pushes();
        let mut st = Statement::new(inst, stack.pop(pops));
        let to_push = st.perform();
        assert_eq!(to_push.len(), pushes);
        stack.push(to_push);
        statement_block.statements.push(st);
    }
    statement_block.in_stack_items = stack.negative_stack.into_iter().collect();
    statement_block.out_stack_items = stack.stack;

    statement_block
}

pub type BlockId = usize;

#[derive(Debug)]
pub struct BasicBlock {
    id: BlockId,
    in_stack_items: Vec<StackItem>,
    out_stack_items: Vec<StackItem>,
    statements: Vec<Statement>,
}

impl BasicBlock {
    fn new(id: BlockId) -> BasicBlock {
        BasicBlock {
            id,
            statements: vec![],
            in_stack_items: vec![],
            out_stack_items: vec![],
        }
    }

    pub fn last_jump(&self) -> Option<(Instruction, Offset)> {
        let last = self.statements.last()?;
        if last.inst.is_jump() {
            Some((last.inst.clone(), last.in_items[0].clone().as_usize()))
        } else {
            None
        }
    }

    pub fn id(&self) -> BlockId {
        self.id
    }
}

#[derive(Default)]
struct ExecutionStack {
    negative_stack: VecDeque<StackItem>,
    stack: Vec<StackItem>,
}

impl ExecutionStack {
    pub fn pop(&mut self, count: usize) -> Vec<StackItem> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            } else {
                let item = StackItem::default();
                self.negative_stack.push_front(item.clone());
                res.push(item);
            }
        }
        res
    }

    pub fn push(&mut self, to_push: Vec<StackItem>) {
        self.stack.extend(to_push);
    }
}

#[derive(Clone, Default)]
pub struct StackItem(Rc<Cell<U256>>);

impl StackItem {
    pub fn as_usize(&self) -> usize {
        // note works only on x64
        self.0.get().as_u64() as usize
    }
}

impl Debug for StackItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get().to_hex())
    }
}

impl Display for StackItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get().to_hex())
    }
}

impl From<&[u8]> for StackItem {
    fn from(buf: &[u8]) -> Self {
        let mut val = [0u8; 32];
        val[(32 - buf.len())..32].copy_from_slice(buf);
        StackItem(Rc::new(Cell::new(U256::from(val))))
    }
}

pub struct Statement {
    inst: Instruction,
    in_items: Vec<StackItem>,
    out_items: Vec<StackItem>,
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n     {}        ", self.inst)?;
        if !self.in_items.is_empty() {
            write!(f, "({:?})", self.in_items,)?;
        }
        if !self.out_items.is_empty() {
            write!(f, "->({:?})", self.out_items,)?;
        }
        Ok(())
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Statement {
    pub fn new(inst: Instruction, in_items: Vec<StackItem>) -> Statement {
        Statement {
            inst,
            in_items,
            out_items: vec![],
        }
    }

    pub fn perform(&mut self) -> Vec<StackItem> {
        let out = match &self.inst.1 {
            OpCode::Stop
            | OpCode::CallDataCopy
            | OpCode::CodeCopy
            | OpCode::ExtCodeCopy
            | OpCode::Pop
            | OpCode::MStore
            | OpCode::MStore8
            | OpCode::SStore
            | OpCode::Jump
            | OpCode::JumpIf
            | OpCode::JumpDest
            | OpCode::Log(..)
            | OpCode::Return
            | OpCode::Invalid(_)
            | OpCode::SelfDestruct
            | OpCode::ReturnDataCopy
            | OpCode::Revert => vec![],
            OpCode::Push(val) => vec![StackItem::from(val.as_slice())],
            OpCode::Dup(_) => {
                let mut out = self.in_items.clone();
                let new_item = out[0].clone();
                out.push(new_item);
                out
            }
            OpCode::Swap(_) => {
                let mut out = self.in_items.clone();
                let last_index = out.len() - 1;
                out.swap(0, last_index);
                out
            }
            _ => vec![StackItem::default()],
        };
        self.out_items = out.clone();
        out
    }
}
