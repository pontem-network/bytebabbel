use crate::bytecode::block::{BlockId, InstructionBlock};
use crate::bytecode::executor::StaticExecutor;
use crate::bytecode::loc::Move;
use crate::OpCode;
use anyhow::Error;
use std::collections::BTreeMap;

type Blocks = BTreeMap<BlockId, InstructionBlock>;

pub fn split(
    blocks: BTreeMap<BlockId, InstructionBlock>,
) -> Result<(Blocks, Option<Blocks>), Error> {
    let code_reallocation = blocks
        .iter()
        .any(|(_, block)| block.iter().any(|i| i.1 == OpCode::CodeCopy));

    if !code_reallocation {
        return Ok((blocks, None));
    }
    if let Some(code_copy) = StaticExecutor::new(&blocks).find_next_entry_point()? {
        let (main, ctor) = blocks.into_iter().fold(
            (BTreeMap::new(), BTreeMap::new()),
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
        Ok((main, Some(ctor)))
    } else {
        Ok((blocks, None))
    }
}
