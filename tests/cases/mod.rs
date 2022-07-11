use eth2move::evm::parse_program;
use eth2move::mv::mvir::MvModule;
use move_binary_format::binary_views::BinaryIndexedView;
use move_bytecode_source_map::mapping::SourceMapping;
use move_core_types::account_address::AccountAddress;
use move_disassembler::disassembler::{Disassembler, DisassemblerOptions};
use move_ir_types::location::Spanned;

mod consts;
mod math;
mod params;

pub fn make_move_module(name: &str, eth: &str, abi: &str, trace: bool) -> Vec<u8> {
    let mut split = name.split("::");

    let addr = AccountAddress::from_hex_literal(split.next().unwrap()).unwrap();
    let name = split.next().unwrap();
    let program = parse_program(name, eth, abi, trace).unwrap();
    let module = MvModule::from_evm_program(addr, program).unwrap();
    let compiled_module = module.make_move_module().unwrap();
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode).unwrap();

    let source_mapping = SourceMapping::new_from_view(
        BinaryIndexedView::Module(&compiled_module),
        Spanned::unsafe_no_loc(()).loc,
    )
    .unwrap();
    let disassembler = Disassembler::new(source_mapping, DisassemblerOptions::new());
    let dissassemble_string = disassembler.disassemble().unwrap();
    println!("{}", dissassemble_string);
    bytecode
}