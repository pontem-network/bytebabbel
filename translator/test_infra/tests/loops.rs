use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use move_core_types::value::MoveValue;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_loops() {
    //const loop
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/types/simple.sol")).unwrap();
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    println!("run");

    vm.run("0x1::Simple::init_store", "0x1").unwrap();

    let res = vm.run("0x1::Simple::f_bool", "true").unwrap();
    for (val, tp) in res.returns.iter() {
        println!("{:?}", MoveValue::simple_deserialize(val, tp));
    }
}
