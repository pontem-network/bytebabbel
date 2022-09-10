use crate::testssol::env::sol::build_sol;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_demo() {
    let _evm = build_sol(include_bytes!("../sol/demo/user_store.sol")).unwrap();
    // let bytecode = make_move_module(&format!("0x42::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    // let mut vm = MoveExecutor::new();
    // vm.deploy("0x42", bytecode);
}
