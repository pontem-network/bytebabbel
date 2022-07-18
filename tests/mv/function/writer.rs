use crate::common::generator::rand_opcode;
use crate::log_init;
use eth2move::mv::function::code::writer::CodeWriter;
use move_binary_format::file_format::{Bytecode, SignatureToken};
use rand::random;

#[test]
pub fn test_write_code() {
    log_init();

    let mut writer = CodeWriter::new(10);
    let mut code = Vec::new();
    for i in 0..1000 {
        if i % 2 == 0 {
            let opcode = rand_opcode();
            writer.push(opcode.clone());
            code.push(opcode);
        } else {
            let ext = (0..random::<usize>() % 50)
                .map(|_| rand_opcode())
                .collect::<Vec<_>>();
            writer.extend(ext.clone());
            code.extend(ext);
        }
    }

    assert_eq!(writer.freeze().code, code);
}

#[test]
pub fn test_locals() {
    log_init();

    let mut writer = CodeWriter::new(10);
    writer.push(Bytecode::Pop);
    writer.push(Bytecode::Nop);
    assert_eq!(writer.borrow_local(SignatureToken::Address), 10);

    writer.push(Bytecode::Add);
    writer.push(Bytecode::Eq);
    assert_eq!(writer.borrow_local(SignatureToken::U64), 11);
    assert_eq!(writer.borrow_local(SignatureToken::U128), 12);

    assert_eq!(writer.borrow_local(SignatureToken::Address), 13);
    assert_eq!(writer.borrow_local(SignatureToken::U64), 14);
    assert_eq!(writer.borrow_local(SignatureToken::U128), 15);
    assert_eq!(writer.borrow_local(SignatureToken::Signer), 16);

    writer.release_local(13);
    assert_eq!(writer.borrow_local(SignatureToken::U128), 17);
    assert_eq!(writer.borrow_local(SignatureToken::Address), 13);
    assert_eq!(writer.borrow_local(SignatureToken::Address), 18);

    writer.push(Bytecode::Or);
    writer.push(Bytecode::ReadRef);
    writer.release_local(10);
    writer.release_local(11);
    writer.release_local(12);

    assert_eq!(writer.borrow_local(SignatureToken::Address), 10);
    assert_eq!(writer.borrow_local(SignatureToken::U64), 11);
    assert_eq!(writer.borrow_local(SignatureToken::U128), 12);

    let code = writer.freeze();
    assert_eq!(
        code.code,
        vec![
            Bytecode::Pop,
            Bytecode::Nop,
            Bytecode::Add,
            Bytecode::Eq,
            Bytecode::Or,
            Bytecode::ReadRef
        ]
    );
    assert_eq!(
        code.locals,
        vec![
            SignatureToken::Address,
            SignatureToken::U64,
            SignatureToken::U128,
            SignatureToken::Address,
            SignatureToken::U64,
            SignatureToken::U128,
            SignatureToken::Signer,
            SignatureToken::U128,
            SignatureToken::Address,
        ]
    );
}
