use eth2move::evm::instruction::Instruction;
use eth2move::evm::{parse_bytecode, OpCode};

#[test]
fn test_iter() {
    let program = "0x608040526002610100";
    let bytecode = parse_bytecode(program)
        .unwrap()
        .collect::<Vec<Instruction>>();

    assert_eq!(
        bytecode,
        vec![
            Instruction::new(0, OpCode::Push(vec![0x80])),
            Instruction::new(2, OpCode::Blockhash),
            Instruction::new(3, OpCode::MStore),
            Instruction::new(4, OpCode::Push(vec![0x2])),
            Instruction::new(6, OpCode::Push(vec!(0x1, 0x00))),
        ]
    );
}

#[test]
fn test_iter_a_plus_b() {
    let res = parse_bytecode(include_str!("../assets/bin/APlusB.bin"))
        .unwrap()
        .collect::<Vec<Instruction>>();
}
