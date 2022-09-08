use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use move_core_types::value::MoveValue;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_address_support() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/demo/address_support.sol")).unwrap();
    let bytecode =
        make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), "", evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    vm.run("0x1::AddressSupport::constructor", "0x1").unwrap();
    let res = vm
        .run("0x1::AddressSupport::is_owner", "0x1")
        .unwrap()
        .returns;
    assert_eq!(
        MoveValue::Bool(true),
        MoveValue::simple_deserialize(&res[0].0, &res[0].1).unwrap()
    );

    let res = vm
        .run("0x1::AddressSupport::is_owner", "0x42")
        .unwrap()
        .returns;
    assert_eq!(
        MoveValue::Bool(false),
        MoveValue::simple_deserialize(&res[0].0, &res[0].1).unwrap()
    );
}
