use eth2move::evm::block::basic_blocks;
use eth2move::evm::parse_bytecode;

#[test]
pub fn test_block() {
    let blocks = basic_blocks(parse_bytecode(include_str!("../assets/bin/APlusB.bin")).unwrap());
    println!("{:?}", blocks);
}
