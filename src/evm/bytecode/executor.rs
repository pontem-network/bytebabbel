use crate::evm::bytecode::block::InstructionBlock;
use crate::evm::bytecode::instruction::{Instruction, Offset};
use crate::evm::bytecode::loc::Loc;
use crate::evm::OpCode;
use bigint::U256;
use itertools::Itertools;
use std::cell::Cell;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub fn exec(blocks: BTreeMap<BlockId, InstructionBlock>) -> BTreeMap<BlockId, Loc<ExecutedBlock>> {
    let mut exec_blocks = BTreeMap::new();
    let executor = Executor::default();
    mark(0.into(), &blocks, &mut exec_blocks, executor);
    exec_blocks
}

fn mark(
    block_id: BlockId,
    blocks: &BTreeMap<BlockId, InstructionBlock>,
    exec_blocks: &mut BTreeMap<BlockId, Loc<ExecutedBlock>>,
    mut executor: Executor,
) {
    let parent = executor.parent().clone();
    if let Some(block) = blocks.get(&block_id) {
        if let Entry::Vacant(e) = exec_blocks.entry(block_id) {
            let new_block = executor.exec(block);
            let jmp = new_block.last_jump(&parent);
            e.insert(new_block);
            if let Some(jmp) = jmp {
                for jmp in jmp.jumps() {
                    mark(jmp, blocks, exec_blocks, executor.clone());
                }
            }
        } else {
            let exec_block = exec_blocks.get_mut(&block_id).unwrap();
            if !exec_block.has_parent(&parent) {
                exec_block.merge(executor.exec(block).inner());
                if let Some(jmp) = exec_block.last_jump(&parent) {
                    for jmp in jmp.jumps() {
                        mark(jmp, blocks, exec_blocks, executor.clone());
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BlockId(pub usize);

impl BlockId {
    pub fn hex(x: &str) -> BlockId {
        let mut buf = 0_usize.to_be_bytes();
        let f = hex::decode(x).unwrap();
        let start_idx = buf.len() - f.len();
        buf[start_idx..].copy_from_slice(&f);
        BlockId(usize::from_be_bytes(buf))
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

impl From<BlockId> for usize {
    fn from(id: BlockId) -> Self {
        id.0
    }
}

impl From<usize> for BlockId {
    fn from(val: usize) -> Self {
        BlockId(val)
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct Chain(Vec<BlockId>);

impl Chain {
    pub fn new(block_id: BlockId) -> Chain {
        Chain(vec![block_id])
    }

    pub fn join(&mut self, block_id: BlockId) {
        self.0.push(block_id);
    }

    pub fn last(&self) -> Option<BlockId> {
        self.0.last().cloned()
    }
}

impl Debug for Chain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|i| i.to_string()).join("->"))
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Clone)]
pub struct ExecutionStack {
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
        self.stack.extend(to_push.into_iter().rev());
    }
}

#[derive(Clone, Default)]
pub struct StackItem(Rc<Cell<U256>>);

impl StackItem {
    pub fn as_usize(&self) -> usize {
        self.0.get().as_u64() as usize
    }

    pub fn as_block_id(&self) -> BlockId {
        self.as_usize().into()
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

#[derive(Debug, Clone, Copy)]
pub struct CodeCopy {
    pub new_offset: Offset,
    pub old_offset: Offset,
    pub len: Offset,
}

#[derive(Default, Clone)]
pub struct Executor {
    stack: ExecutionStack,
    parent: Chain,
}

impl Executor {
    pub fn with_parent(parent: BlockId) -> Executor {
        Executor {
            stack: Default::default(),
            parent: Chain::new(parent),
        }
    }

    pub fn exec(&mut self, block: &InstructionBlock) -> Loc<ExecutedBlock> {
        let mut executed_block = block.wrap(ExecutedBlock::new(block.start.into()));
        let mut execution = Execution::default();

        let input_stack = self.stack.stack.clone();
        for inst in block.iter() {
            let pops = inst.pops();
            let pushes = inst.pushes();
            let mut st = Statement::new(self.stack.pop(pops));
            let to_push = st.perform(inst);
            assert_eq!(to_push.len(), pushes);
            self.stack.push(to_push);
            executed_block.instructions.push(inst.clone());
            execution.state.push(st);
        }
        execution.in_stack_items = self.stack.negative_stack.iter().cloned().collect();
        execution.in_stack_items.extend(input_stack);
        execution.out_stack_items = self.stack.stack.clone();

        executed_block
            .executions
            .insert(self.parent.clone(), execution);
        self.parent.join(block.start.into());
        executed_block
    }

    pub fn parent(&self) -> &Chain {
        &self.parent
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    in_items: Vec<StackItem>,
    out_items: Vec<StackItem>,
}

impl Statement {
    pub fn new(in_items: Vec<StackItem>) -> Statement {
        Statement {
            in_items,
            out_items: vec![],
        }
    }

    fn perform(&mut self, inst: &Instruction) -> Vec<StackItem> {
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
                let new_item = out[out.len() - 1].clone();
                out.insert(0, new_item);
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

#[derive(Default, Debug, Clone)]
pub struct Execution {
    in_stack_items: Vec<StackItem>,
    out_stack_items: Vec<StackItem>,
    state: Vec<Statement>,
}

#[derive(Clone)]
pub struct ExecutedBlock {
    id: BlockId,
    instructions: Vec<Instruction>,
    executions: BTreeMap<Chain, Execution>,
}

impl ExecutedBlock {
    pub fn new(id: BlockId) -> ExecutedBlock {
        ExecutedBlock {
            id,
            instructions: vec![],
            executions: Default::default(),
        }
    }

    pub fn first_execution(&self) -> Option<&Chain> {
        self.executions.iter().next().map(|(p, _)| p)
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn has_parent(&self, parent: &Chain) -> bool {
        self.executions.contains_key(parent)
    }

    pub fn merge(&mut self, other: ExecutedBlock) -> Option<ExecutedBlock> {
        if self.id == other.id {
            self.executions.extend(other.executions);
            None
        } else {
            Some(other)
        }
    }

    pub fn last_jump(&self, parent: &Chain) -> Option<Jump> {
        let inst = self.instructions.last()?;
        let state = self.executions.get(parent)?.state.last()?;

        if matches!(inst.1, OpCode::Jump) {
            Some(Jump::UnCnd(state.in_items[0].as_block_id()))
        } else if matches!(inst.1, OpCode::JumpIf) {
            Some(Jump::Cnd {
                true_br: state.in_items[0].as_block_id(),
                false_br: self.next_block_id(),
            })
        } else {
            None
        }
    }

    pub fn code_copy(&self, parent: &Chain) -> Option<CodeCopy> {
        let code_copy = self
            .instructions
            .iter()
            .enumerate()
            .find(|i| matches!(i.1 .1, OpCode::CodeCopy))
            .map(|(i, _)| i)?;
        let execution = self.executions.get(parent)?;
        let stack = &execution.state[code_copy];
        Some(CodeCopy {
            new_offset: stack.in_items[0].as_usize(),
            old_offset: stack.in_items[1].as_usize(),
            len: stack.in_items[2].as_usize(),
        })
    }

    pub fn next_block_id(&self) -> BlockId {
        if let Some(last) = self.instructions.last() {
            (last.0 + last.1.size()).into()
        } else {
            (self.id.0 + 1).into()
        }
    }

    pub fn is_invalid(&self) -> bool {
        self.instructions
            .iter()
            .all(|s| matches!(s.1, OpCode::Invalid(_)))
    }

    pub fn id(&self) -> BlockId {
        self.id
    }

    pub fn all_jumps(&self) -> Vec<BlockId> {
        self.executions
            .iter()
            .filter_map(|(ch, _)| self.last_jump(ch).map(|jmp| jmp.jumps()))
            .flatten()
            .collect()
    }
}

impl Debug for ExecutedBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block id: {}", self.id)?;
        writeln!(
            f,
            "==========================[EXECUTIONS]=========================="
        )?;
        for (path, execution) in &self.executions {
            writeln!(f, "Execution in stack:{:?}", execution.in_stack_items)?;
            writeln!(f, "Execution start:{}", path)?;
            for (idx, inst) in self.instructions.iter().enumerate() {
                write!(f, "{inst}")?;
                let state = &execution.state[idx];
                writeln!(f, "({:?}) => ({:?})", state.in_items, state.out_items)?;
            }
            writeln!(f, "Execution end")?;
            writeln!(f, "Execution out stack:{:?}", execution.out_stack_items)?;
        }
        writeln!(
            f,
            "================================================================"
        )
    }
}

#[derive(Debug)]
pub enum Jump {
    Cnd { true_br: BlockId, false_br: BlockId },
    UnCnd(BlockId),
}

impl Jump {
    pub fn jumps(&self) -> Vec<BlockId> {
        match self {
            Jump::Cnd { true_br, false_br } => vec![*true_br, *false_br],
            Jump::UnCnd(id) => vec![*id],
        }
    }

    pub fn as_cnd(&self) -> Option<(BlockId, BlockId)> {
        match self {
            Jump::Cnd { true_br, false_br } => Some((*true_br, *false_br)),
            Jump::UnCnd(_) => None,
        }
    }
}
