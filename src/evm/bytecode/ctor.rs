use crate::evm::bytecode::block::InstructionBlock;
use crate::evm::bytecode::executor::{BlockId, CodeCopy, Executor};
use crate::evm::bytecode::loc::Move;
use crate::evm::OpCode;
use std::collections::BTreeMap;

pub fn split(
    blocks: BTreeMap<BlockId, InstructionBlock>,
) -> (
    BTreeMap<BlockId, InstructionBlock>,
    Option<BTreeMap<BlockId, InstructionBlock>>,
) {
    let code_reallocation = blocks
        .iter()
        .any(|(_, block)| block.iter().any(|i| i.1 == OpCode::CodeCopy));

    if !code_reallocation {
        return (blocks, None);
    }

    if let Some(code_copy) = find_code_copy(Executor::with_parent(0.into()), 0.into(), &blocks) {
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
    let parent = executor.parent().clone();
    let execution = executor.exec(block);
    match execution.code_copy(&parent) {
        None => execution
            .last_jump(&parent)?
            .jumps()
            .iter()
            .find_map(|jmp| find_code_copy(executor.clone(), *jmp, blocks)),
        Some(code_copy) => Some(code_copy),
    }
}
