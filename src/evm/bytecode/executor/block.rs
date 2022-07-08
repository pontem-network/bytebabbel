use crate::evm::bytecode::executor::stack::StackFrame;
use crate::evm::bytecode::executor::statement::Statement;
use crate::evm::bytecode::instruction::{Instruction, Offset};
use crate::evm::OpCode;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};

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

    pub fn has_root(&self) -> bool {
        self.0.first().map(|b| b.0 == 0).unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

#[derive(Debug, Clone, Copy)]
pub struct CodeCopy {
    pub new_offset: Offset,
    pub old_offset: Offset,
    pub len: Offset,
}

#[derive(Default, Debug, Clone)]
pub struct Execution {
    pub in_stack_items: Vec<StackFrame>,
    pub out_stack_items: Vec<StackFrame>,
    pub state: Vec<Statement>,
}

#[derive(Clone)]
pub struct ExecutedBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub executions: BTreeMap<Chain, Execution>,
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

    pub fn shortest_root_execution(&self) -> Option<&Chain> {
        let mut min: Option<&Chain> = None;
        for (next, _) in &self.executions {
            if !next.has_root() {
                continue;
            }

            if let Some(old) = min {
                if old.len() > next.len() {
                    min = Some(next);
                }
            } else {
                min = Some(next);
            }
        }
        min
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
            Some(Jump::UnCnd(state.in_items[0].as_block_id()?))
        } else if matches!(inst.1, OpCode::JumpIf) {
            if let Some(static_true_br) = state.in_items[1].as_bool() {
                Some(if static_true_br {
                    Jump::UnCnd(state.in_items[0].as_block_id()?)
                } else {
                    Jump::UnCnd(self.next_block_id())
                })
            } else {
                Some(Jump::Cnd {
                    true_br: state.in_items[0].as_block_id()?,
                    false_br: self.next_block_id(),
                })
            }
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
            new_offset: stack.in_items[0].as_usize()?,
            old_offset: stack.in_items[1].as_usize()?,
            len: stack.in_items[2].as_usize()?,
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
