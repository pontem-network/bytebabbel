use crate::testssol::convert::ResultToString;
use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use eth::abi::entries::AbiEntries;
use eth::Flags;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_address_support() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/demo/address_support.sol")).unwrap();

    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.bin(),
        "",
        evm.abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap());
    vm.deploy("0x42", bytecode);
    vm.run("0x42::AddressSupport::constructor", "0x42", None)
        .unwrap();
    let res = vm
        .run("0x42::AddressSupport::is_owner", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(true)", res);

    let res = vm
        .run("0x42::AddressSupport::is_owner", "0x44", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(false)", res);
}
