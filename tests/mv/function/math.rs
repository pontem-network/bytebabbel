use crate::common::executor::MoveExecutor;
use eth2move::mv::function::code::intrinsic::math::u256_math::U256Math;
use eth2move::mv::function::code::intrinsic::math::Cast;
use eth2move::mv::function::code::writer::CodeWriter;
use eth2move::mv::function::MvFunction;
use eth2move::mv::mvir::MvModule;
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::file_format::{
    Bytecode, Signature, SignatureIndex, SignatureToken, Visibility,
};
use move_binary_format::CompiledModule;
use move_bytecode_source_map::mapping::SourceMapping;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_core_types::value::MoveValue;
use move_disassembler::disassembler::{Disassembler, DisassemblerOptions};
use move_ir_types::location::Spanned;

pub fn make_module(
    name: &str,
    input: Vec<SignatureToken>,
    output: Vec<SignatureToken>,
    signature: Vec<SignatureToken>,
    code: CodeWriter,
) -> Vec<u8> {
    let code = code.freeze();
    let function = MvFunction {
        name: Identifier::new(name).unwrap(),
        visibility: Visibility::Public,
        input: Signature(input),
        output: Signature(output),
        locals: code.locals,
        code: code.code,
    };
    let mut module: CompiledModule = MvModule {
        address: CORE_CODE_ADDRESS,
        name: Identifier::new("TestModule").unwrap(),
        funcs: vec![function],
    }
    .make_move_module()
    .unwrap();
    module.signatures.push(Signature(signature));

    let mut buff = Vec::new();
    module.serialize(&mut buff).unwrap();
    println!("{:?}", module);
    let source_mapping = SourceMapping::new_from_view(
        BinaryIndexedView::Module(&module),
        Spanned::unsafe_no_loc(()).loc,
    )
    .unwrap();

    let disassembler = Disassembler::new(source_mapping, DisassemblerOptions::new());
    let dissassemble_string = disassembler.disassemble().unwrap();
    println!("{}", dissassemble_string);

    buff
}

#[test]
pub fn test_u256_math_cast() {
    let math = U256Math {
        vec_sig_index: SignatureIndex(3),
    };

    let mut code = CodeWriter::new(1, true);

    code.push(Bytecode::CopyLoc(0));
    math.cast_from_u128(&mut code);
    math.cast_to_u128(&mut code);
    code.push(Bytecode::Ret);

    let module = make_module(
        "u256_cast",
        vec![SignatureToken::U128],
        vec![SignatureToken::U128],
        vec![SignatureToken::U64],
        code,
    );
    let mut executor = MoveExecutor::new();
    executor.deploy("0x1", module);

    fn test(exec: &mut MoveExecutor, expected: u128) {
        let res = exec
            .run("0x1::TestModule::u256_cast", &expected.to_string())
            .returns;
        let (val, tp) = &res[0];
        if let MoveValue::U128(val) = MoveValue::simple_deserialize(val, tp).unwrap() {
            assert_eq!(val, expected);
        } else {
            panic!("Invalid return type");
        }
    }

    test(&mut executor, 0);
    test(&mut executor, u128::MAX);

    for _ in 0..1000 {
        test(&mut executor, rand::random());
    }
}
