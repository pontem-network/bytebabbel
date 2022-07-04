use crate::evm::bytecode::block::{InstructionBlock as InstructionBLock, InstructionBlock};
use crate::evm::bytecode::instruction::{Instruction, Offset};
use crate::evm::bytecode::loc::{Loc, Move};
use crate::evm::OpCode;
use bigint::U256;
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub fn mark_stack(block: InstructionBlock) -> Loc<BasicBlock> {
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

    pub fn statements(&self) -> &[Statement] {
        self.statements.as_slice()
    }

    pub fn last_jump(&self) -> Option<(Instruction, Offset)> {
        let last = self.statements.last()?;
        if last.inst.is_jump() {
            Some((last.inst.clone(), last.in_items[0].clone().as_usize()))
        } else {
            None
        }
    }

    pub fn next_block_id(&self) -> BlockId {
        if let Some(last) = self.statements.last() {
            last.inst.0 + last.inst.1.size()
        } else {
            self.id + 1
        }
    }

    pub fn is_invalid(&self) -> bool {
        self.statements
            .iter()
            .all(|s| matches!(s.inst.1, OpCode::Invalid(_)))
    }

    pub fn id(&self) -> BlockId {
        self.id
    }
}

impl Move for BasicBlock {
    fn move_forward(&mut self, offset: usize) {
        self.id += offset;
        for statement in self.statements.iter_mut() {
            statement.move_forward(offset);
        }
    }

    fn move_back(&mut self, offset: usize) {
        self.id -= offset;
        for statement in self.statements.iter_mut() {
            statement.move_back(offset);
        }
    }
}

#[derive(Default, Clone)]
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

impl Move for Statement {
    fn move_forward(&mut self, offset: usize) {
        self.inst.0 += offset;
    }

    fn move_back(&mut self, offset: usize) {
        self.inst.0 -= offset;
    }
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

    pub fn as_push(&self) -> Option<&[u8]> {
        if let OpCode::Push(push) = &self.inst.1 {
            Some(push.as_slice())
        } else {
            None
        }
    }

    pub fn as_code_copy(&self) -> Option<CodeCopy> {
        if let OpCode::CodeCopy = &self.inst.1 {
            assert_eq!(self.in_items.len(), 3, "CodeCopy must takes 3 stack items.");
            Some(CodeCopy {
                new_offset: self.in_items[0].as_usize(),
                old_offset: self.in_items[1].as_usize(),
                len: self.in_items[2].as_usize(),
            })
        } else {
            None
        }
    }
}

pub struct CodeCopy {
    pub new_offset: Offset,
    pub old_offset: Offset,
    pub len: Offset,
}

#[derive(Default, Clone)]
pub struct Executor {
    stack: ExecutionStack,
    parent: Option<BlockId>,
}

impl Executor {
    pub fn exec(&mut self, block: &InstructionBlock) -> Loc<ExecutedBlock> {
        let mut executed_block = block.wrap(ExecutedBlock::new(block.start));
        let mut execution = Execution::default();

        for inst in block.iter() {
            let pops = inst.pops();
            let pushes = inst.pushes();
            let mut st = Statement1::new(self.stack.pop(pops));
            let to_push = st.perform(inst);
            assert_eq!(to_push.len(), pushes);
            self.stack.push(to_push);
            executed_block.instructions.push(inst.clone());
            execution.state.push(st);
        }
        execution.in_stack_items = self.stack.negative_stack.iter().cloned().collect();
        execution.out_stack_items = self.stack.stack.clone();

        executed_block.executions.insert(self.parent, execution);
        self.parent = Some(block.start);
        executed_block
    }
}

pub struct Statement1 {
    in_items: Vec<StackItem>,
    out_items: Vec<StackItem>,
}

impl Statement1 {
    pub fn new(in_items: Vec<StackItem>) -> Statement1 {
        Statement1 {
            in_items,
            out_items: vec![],
        }
    }

    pub fn perform(&mut self, inst: &Instruction) -> Vec<StackItem> {
        let out = match &inst.1 {
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

pub struct ExecutedBlock {
    id: BlockId,
    instructions: Vec<Instruction>,
    executions: HashMap<Option<BlockId>, Execution>,
}

#[derive(Default)]
pub struct Execution {
    in_stack_items: Vec<StackItem>,
    out_stack_items: Vec<StackItem>,
    state: Vec<Statement1>,
}

impl ExecutedBlock {
    pub fn new(id: BlockId) -> ExecutedBlock {
        ExecutedBlock {
            id,
            instructions: vec![],
            executions: Default::default(),
        }
    }

    pub fn merge(&mut self, other: ExecutedBlock) -> Option<ExecutedBlock> {
        if self.id == other.id {
            self.executions.extend(other.executions);
            None
        } else {
            Some(other)
        }
    }

    // pub fn last_jump(&self, parent: Option<BlockId>) -> Option<(Instruction, Offset)> {
    //     let last = self.statements.last()?;
    //     if last.inst.is_jump() {
    //         Some((last.inst.clone(), last.in_items[0].clone().as_usize()))
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn next_block_id(&self) -> BlockId {
    //     if let Some(last) = self.statements.last() {
    //         last.inst.0 + last.inst.1.size()
    //     } else {
    //         self.id + 1
    //     }
    // }

    pub fn is_invalid(&self) -> bool {
        self.instructions
            .iter()
            .all(|s| matches!(s.1, OpCode::Invalid(_)))
    }

    pub fn id(&self) -> BlockId {
        self.id
    }
}
