use std::collections::HashMap;

use crate::bytecode::block::{InstructionBlock, Offset};
use crate::bytecode::loc::Move;

type Blocks = HashMap<Offset, InstructionBlock>;

pub fn replace(blocks: Blocks, entry_point: Offset) -> Blocks {
    blocks
        .into_iter()
        .filter(|(id, _)| *id >= entry_point)
        .map(|(block_id, mut block)| {
            block.move_back(entry_point);
            ((block_id.0 - entry_point.0).into(), block)
        })
        .collect()
}
