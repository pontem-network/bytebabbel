use crate::evm::bytecode::instruction::Offset;
use crate::evm::bytecode::loc::Loc;
use crate::evm::bytecode::statement::{BasicBlock, BlockId};
use crate::evm::OpCode;
use anyhow::{anyhow, Error};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub struct ControlFlowGraph {
    blocks: BTreeMap<BlockId, Vertex>,
    entry_points: BTreeSet<BlockId>,
}

impl ControlFlowGraph {
    pub fn new(
        basic_blocks: &BTreeMap<BlockId, Loc<BasicBlock>>,
        entry_points: impl Iterator<Item = BlockId>,
    ) -> Result<ControlFlowGraph, Error> {
        let mut blocks = BTreeMap::new();
        let entry_points: BTreeSet<_> = entry_points.collect();

        for ep in &entry_points {
            Self::push_child(*ep, None, &mut blocks, basic_blocks)?;
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
    ) -> Result<(), Error> {
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
            Self::push_child(child, Some(id), blocks, basic_blocks)?;
        }
        Ok(())
    }

    fn block_children(block: &Loc<BasicBlock>) -> Vec<BlockId> {
        if let Some((jump, offset)) = block.last_jump() {
            if matches!(jump.1, OpCode::JumpIf) {
                vec![offset, block.next_block_id()]
            } else {
                vec![offset]
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
}
