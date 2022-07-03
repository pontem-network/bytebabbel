use eth2move::evm::parse_program;

#[test]
pub fn test_block() {
    let program = parse_program(
        "APlusB",
        include_str!("../assets/bin/APlusB.bin"),
        include_str!("../assets/bin/APlusB.abi"),
    )
    .unwrap();
    println!("{:?}", program);
}
