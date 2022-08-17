use crate::bytecode::block::{BlockId, InstructionBlock};
use crate::bytecode::loc::Move;
use crate::bytecode::pre_processing::code_copy::find_entry_points;
use crate::OpCode;
use anyhow::Error;
use std::collections::HashMap;

type Blocks = HashMap<BlockId, InstructionBlock>;

pub fn split(
    blocks: HashMap<BlockId, InstructionBlock>,
) -> Result<(Blocks, BlockId, Option<Blocks>), Error> {
    let code_reallocation = blocks
        .iter()
        .any(|(_, block)| block.iter().any(|i| i.1 == OpCode::CodeCopy));

    if !code_reallocation {
        return Ok((blocks, BlockId::default(), None));
    }

    if let Some(code_copy) = find_entry_points(&blocks)? {
        let (main, ctor) = blocks.into_iter().fold(
            (HashMap::new(), HashMap::new()),
            |(mut main, mut ctor), (block_id, mut block)| {
                if block_id >= code_copy {
                    block.move_back(code_copy.0);
                    main.insert((block_id.0 - code_copy.0).into(), block);
                } else {
                    ctor.insert(block_id, block);
                }
                (main, ctor)
            },
        );
        Ok((main, code_copy, Some(ctor)))
    } else {
        Ok((blocks, BlockId::default(), None))
    }
}
