use crate::evm::bytecode::executor::{BasicBlock, BlockId, Executor};
use crate::evm::bytecode::instruction::Offset;
use crate::evm::bytecode::loc::Loc;
use crate::evm::OpCode;
use anyhow::{anyhow, Error};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{Debug, Display, Formatter};

pub struct ControlFlowGraph {
    blocks: BTreeMap<BlockId, Vertex>,
    entry_points: BTreeSet<BlockId>,
}

impl ControlFlowGraph {
    pub fn new(
        basic_blocks: &BTreeMap<BlockId, Loc<BasicBlock>>,
        entry_points_iter: impl Iterator<Item = (BlockId, usize)>,
    ) -> Result<ControlFlowGraph, Error> {
        let mut blocks = BTreeMap::new();
        let mut entry_points = BTreeSet::new();

        for (ep, input_size) in entry_points_iter {
            entry_points.insert(ep);
            let mut executor = Executor::default();
            Self::push_child(ep, None, &mut blocks, basic_blocks, executor)?;
        }

        Ok(ControlFlowGraph {
            blocks,
            entry_points,
        })
    }

    fn push_child(
        id: BlockId,
        parent: Option<BlockId>,
        blocks: &mut BTreeMap<BlockId, Vertex>,
        basic_blocks: &BTreeMap<BlockId, Loc<BasicBlock>>,
        mut executor: Executor,
    ) -> Result<Executor, Error> {
        let block = basic_blocks
            .get(&id)
            .ok_or_else(|| anyhow!("Failed to make flow graph. Block with id {id} not found"))?;
        let children = Self::block_children(block);

        let vertex = blocks
            .entry(id)
            .or_insert_with(|| Vertex::new(block.start, block.end));
        if let Some(parent) = parent {
            vertex.add_parent(parent);
        }

        let children_to_hndl = children
            .into_iter()
            .filter(|id| vertex.add_child(*id))
            .collect::<Vec<_>>();

        for child in children_to_hndl {
            Self::push_child(child, Some(id), blocks, basic_blocks, executor.clone())?;
        }
        Ok(executor)
    }

    fn block_children(block: &Loc<BasicBlock>) -> Vec<BlockId> {
        if let Some((jump, offset)) = block.last_jump() {
            if matches!(jump.1, OpCode::JumpIf) {
                vec![offset.into(), block.next_block_id()]
            } else {
                vec![offset.into()]
            }
        } else {
            vec![]
        }
    }

    pub fn entry_points(&self) -> &BTreeSet<BlockId> {
        &self.entry_points
    }

    pub fn blocks(&self) -> &BTreeMap<BlockId, Vertex> {
        &self.blocks
    }

    pub fn vertex(&self, id: BlockId) -> Result<&Vertex, Error> {
        self.blocks
            .get(&id)
            .ok_or_else(|| anyhow!("Vertex {id} not found."))
    }

    pub fn build_flow(&self, ep: BlockId) -> Result<Flow, Error> {
        let successor = self.vertex(ep)?.successor();
        Ok(if successor.is_empty() {
            Flow::Ln(ep)
        } else if successor.len() == 1 {
            let jmp = *successor.iter().next().unwrap();
            // may be fn call
            println!("{} -> jmp: {}", ep, jmp);
            let flow = self.build_flow(jmp)?;
            Flow::Call(vec![Flow::Ln(ep), flow])
        } else {
            let mut iter = successor.iter();
            let true_br = *iter.next().unwrap();
            let false_br = *iter.next().unwrap();
            // if else
            // loop
            println!("{} -> jmp_if({},{})", ep, true_br, false_br);
            Flow::If {
                true_branch: Box::new(self.build_flow(true_br)?),
                false_branch: Box::new(self.build_flow(false_br)?),
            }
        })
    }
}

#[derive(Debug)]
pub enum Flow {
    If {
        true_branch: Box<Flow>,
        false_branch: Box<Flow>,
    },
    Loop(Vec<Flow>),
    Call(Vec<Flow>),
    Ln(BlockId),
}

#[derive(Debug)]
pub struct Vertex {
    entry: Offset,
    exit: Offset,
    parents: HashSet<BlockId>,
    successor: HashSet<BlockId>,
}

impl Vertex {
    fn new(entry: Offset, exit: Offset) -> Vertex {
        Vertex {
            entry,
            exit,
            parents: Default::default(),
            successor: Default::default(),
        }
    }

    fn add_parent(&mut self, block_id: BlockId) -> bool {
        self.parents.insert(block_id)
    }

    fn add_child(&mut self, block_id: BlockId) -> bool {
        self.successor.insert(block_id)
    }

    pub fn contains(&self, offset: Offset) -> bool {
        self.entry <= offset && self.exit >= offset
    }

    pub fn has_no_parents(&self) -> bool {
        self.parents.is_empty()
    }

    pub fn successor(&self) -> &HashSet<BlockId> {
        &self.successor
    }
}

impl Display for ControlFlowGraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for ep in &self.entry_points {
            writeln!(f, "{}", ep)?;
            let vp = &self.blocks[ep];
            // todo
            writeln!(f, "{:?}", vp.successor)?;
        }

        Ok(())
    }
}
