use crate::evm::bytecode::executor::block::{BlockId, Chain, ExecutedBlock};
use crate::evm::bytecode::loc::Loc;
use std::collections::BTreeMap;

pub struct FlowGraph {
    blocks: BTreeMap<BlockId, Loc<ExecutedBlock>>,
}

impl FlowGraph {
    pub fn new(blocks: BTreeMap<BlockId, Loc<ExecutedBlock>>) -> FlowGraph {
        FlowGraph { blocks }
    }

    pub fn root_chain(&self, block_id: BlockId) -> Option<&Chain> {
        let block = self.blocks.get(&block_id)?;
        block.shortest_root_execution()
    }

    pub fn block(&self, id: &BlockId) -> Option<&Loc<ExecutedBlock>> {
        self.blocks.get(id)
    }

    pub fn blocks(&self) -> &BTreeMap<BlockId, Loc<ExecutedBlock>> {
        &self.blocks
    }
}
