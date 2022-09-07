use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use move_core_types::value::MoveValue;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_demo() {
    let evm = build_sol(include_bytes!("../sol/demo/user_store.sol")).unwrap();
    println!("evm: {:?}", evm);
    // let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    // let mut vm = MoveExecutor::new();
    // vm.deploy("0x1", bytecode);
}
