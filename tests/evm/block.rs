use eth2move::evm::block::basic_blocks;
use eth2move::evm::{parse_bytecode, statement};

#[test]
pub fn test_block() {
    let mut blocks =
        basic_blocks(parse_bytecode(include_str!("../assets/bin/APlusB.bin")).unwrap());
    let block = statement::mark_stack(blocks.remove(0));
    println!("{:?}", block);
    println!("{:?}", block.last_jump());
}
