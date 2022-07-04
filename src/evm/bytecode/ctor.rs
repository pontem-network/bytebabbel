use crate::evm::bytecode::block::InstructionBlock;
use crate::evm::bytecode::loc::{Loc, Move};
use crate::evm::bytecode::statement::{BasicBlock, BlockId, Executor, Statement};
use crate::evm::OpCode;
use std::collections::BTreeMap;

type Blocks = BTreeMap<BlockId, Loc<BasicBlock>>;

pub fn split(blocks: Blocks) -> (Blocks, Option<Blocks>) {
    let next_entry_point = blocks
        .iter()
        .find_map(|(_, block)| block.statements().iter().find_map(next_entry_point));

    if let Some(next_entry_point) = next_entry_point {
        let (main, ctor) = blocks.into_iter().fold(
            (Blocks::new(), Blocks::new()),
            |(mut main, mut ctor), (block_id, mut block)| {
                if block_id >= next_entry_point {
                    block.move_back(next_entry_point);
                    main.insert(block_id - next_entry_point, block);
                } else {
                    ctor.insert(block_id, block);
                }
                (main, ctor)
            },
        );
        (main, Some(ctor))
    } else {
        (blocks, None)
    }
}

fn next_entry_point(statement: &Statement) -> Option<BlockId> {
    if let Some(code_copy) = statement.as_code_copy() {
        if code_copy.new_offset == 0 {
            Some(code_copy.old_offset)
        } else {
            None
        }
    } else {
        None
    }
}

// pub fn split_1(
//     blocks: BTreeMap<BlockId, InstructionBlock>,
// ) -> (
//     BTreeMap<BlockId, InstructionBlock>,
//     Option<BTreeMap<BlockId, InstructionBlock>>,
// ) {
//     let code_reallocation = blocks
//         .iter()
//         .any(|(id, block)| block.iter().any(|i| i.1 == OpCode::CodeCopy));
//     if code_reallocation {
//         let mut executor = Executor::default();
//         if let Some(block) = blocks.get(&0) {
//             executor.exec(block)(blocks, None)
//         } else {
//             (blocks, None)
//         }
//     } else {
//         (blocks, None)
//     }
// }
