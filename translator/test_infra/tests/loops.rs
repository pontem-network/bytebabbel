use crate::testssol::make_move_module;
use test_infra::executor::MoveExecutor;
use test_infra::sol::build_sol;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_loops() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/fn/fn.sol")).unwrap();
    println!("{:?}", evm.bin());
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    dbg!(vm.run("0x1::Fn::fn_const_return_bool", "").unwrap());
}
