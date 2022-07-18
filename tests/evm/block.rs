use crate::log_init;
use eth2move::evm::parse_program;

#[test]
pub fn test_block() {
    log_init();

    let program = parse_program(
        "APlusB",
        include_str!("../assets/bin/ConstFn.bin"),
        include_str!("../assets/bin/ConstFn.abi"),
    )
    .unwrap();
    println!("{:?}", program);
}
