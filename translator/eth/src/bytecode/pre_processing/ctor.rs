use crate::bytecode::block::{BlockId, InstructionBlock};
use crate::bytecode::loc::Move;
use std::collections::HashMap;

type Blocks = HashMap<BlockId, InstructionBlock>;

pub fn replace(blocks: Blocks, entry_point: BlockId) -> Blocks {
    blocks
        .into_iter()
        .filter(|(id, _)| *id >= entry_point)
        .map(|(block_id, mut block)| {
            block.move_back(entry_point.0);
            ((block_id.0 - entry_point.0).into(), block)
        })
        .collect()
}
