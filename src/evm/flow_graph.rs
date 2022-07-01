use crate::evm::instruction::Offset;
use crate::evm::statement::BlockId;
use std::collections::BTreeMap;

pub struct ControlFlowGraph {
    blocks: BTreeMap<BlockId, Vertex>,
}

#[derive(Debug)]
pub struct Vertex {
    entry: Offset,
    exit: Offset,
    parents: Vec<BlockId>,
    successor: Option<BlockId>,
}
