use crate::evm::bytecode::block::InstructionBlock;
use crate::evm::bytecode::executor::{BasicBlock, BlockId, CodeCopy, Executor, Statement};
use crate::evm::bytecode::loc::{Loc, Move};
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
                    block.move_back(next_entry_point.into());
                    main.insert((block_id.0 - next_entry_point.0).into(), block);
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
            Some(code_copy.old_offset.into())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn split_1(
    blocks: BTreeMap<BlockId, InstructionBlock>,
) -> (
    BTreeMap<BlockId, InstructionBlock>,
    Option<BTreeMap<BlockId, InstructionBlock>>,
) {
    let code_reallocation = blocks
        .iter()
        .any(|(id, block)| block.iter().any(|i| i.1 == OpCode::CodeCopy));

    if !code_reallocation {
        return (blocks, None);
    }

    if let Some(code_copy) =
        find_code_copy(Executor::with_parent(Some(0.into())), 0.into(), &blocks)
    {
        let (main, ctor) = blocks.into_iter().fold(
            (BTreeMap::new(), BTreeMap::new()),
            |(mut main, mut ctor), (block_id, mut block)| {
                if block_id >= code_copy.old_offset.into() {
                    block.move_back(code_copy.old_offset);
                    main.insert((block_id.0 - code_copy.old_offset).into(), block);
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

fn find_code_copy(
    mut executor: Executor,
    block: BlockId,
    blocks: &BTreeMap<BlockId, InstructionBlock>,
) -> Option<CodeCopy> {
    let block = blocks.get(&block)?;
    let parent = executor.parent();
    let execution = executor.exec(block);
    match execution.code_copy(parent) {
        None => execution
            .last_jump(parent)?
            .jumps()
            .iter()
            .find_map(|jmp| find_code_copy(executor.clone(), *jmp, blocks)),
        Some(code_copy) => Some(code_copy),
    }
}
