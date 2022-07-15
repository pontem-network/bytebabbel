use eth2move::evm::parse_program;
use eth2move::mv::mvir::MvModule;
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::CompiledModule;
use move_bytecode_source_map::mapping::SourceMapping;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_disassembler::disassembler::{Disassembler, DisassemblerOptions};
use move_ir_types::location::Spanned;

mod function;

#[test]
pub fn test_move_translator() {
    let program = parse_program(
        "ConstFn",
        include_str!("../assets/bin/ConstFn.bin"),
        include_str!("../assets/bin/ConstFn.abi"),
        true,
    )
    .unwrap();
    println!("{:?}", program);

    let module = MvModule::from_evm_program(CORE_CODE_ADDRESS, program).unwrap();

    let compiled_module = module.make_move_module().unwrap();
    println!("actual {:?}", compiled_module);
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode).unwrap();

    let source_mapping = SourceMapping::new_from_view(
        BinaryIndexedView::Module(&compiled_module),
        Spanned::unsafe_no_loc(()).loc,
    )
    .unwrap();

    let expected = CompiledModule::deserialize(include_bytes!(
        "../assets/move/build/move/bytecode_modules/ConstFn.mv"
    ))
    .unwrap();
    println!();
    println!("expected {:?}", expected);

    let disassembler = Disassembler::new(source_mapping, DisassemblerOptions::new());
    let dissassemble_string = disassembler.disassemble().unwrap();
    println!("{}", dissassemble_string);
}
