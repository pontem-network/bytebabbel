use eth2move::evm::parse_program;
use eth2move::mv::mvir::MvModule;
use move_core_types::account_address::AccountAddress;

mod consts;

pub fn make_move_module(name: &str, eth: &str, abi: &str) -> Vec<u8> {
    let mut split = name.split("::");

    let addr = AccountAddress::from_hex_literal(split.next().unwrap()).unwrap();
    let name = split.next().unwrap();
    let program = parse_program(name, eth, abi).unwrap();
    let module = MvModule::from_evm_program(addr, program).unwrap();
    let compiled_module = module.make_move_module().unwrap();
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode).unwrap();
    bytecode
}
