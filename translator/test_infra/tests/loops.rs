use crate::testssol::make_move_module;
use move_core_types::value::MoveValue;
use test_infra::executor::MoveExecutor;
use test_infra::sol::build_sol;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_loops() {
    //const loop
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/ignore_loops.sol")).unwrap();
    println!("{:?}", evm.bin());
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    println!("run");

    let res = vm.run("0x1::Loop::for_loop", "2").unwrap();
    for (val, tp) in res.returns.iter() {
        println!("{:?}", MoveValue::simple_deserialize(val, tp));
    }

    // for loop
}
