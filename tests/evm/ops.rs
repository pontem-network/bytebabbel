use eth2move::evm::bytecode::instruction::Instruction;
use eth2move::evm::bytecode::ops::InstructionIter;
use eth2move::evm::{parse_bytecode, OpCode};

#[test]
fn test_iter() {
    let program = "0x608040526002610100";
    let bytecode =
        InstructionIter::new(parse_bytecode(program).unwrap()).collect::<Vec<Instruction>>();

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
