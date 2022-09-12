use crate::testssol::convert::ResultToString;
use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use eth::abi::entries::AbiEntries;
use eth::Flags;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_for_loop() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/loop/ignore_for.sol")).unwrap();
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

    vm.run("0x42::ForLoop::constructor", "0x42", None).unwrap();
    let res = vm
        .run("0x42::ForLoop::sum", "0x42", Some("9, 9"))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(81)", res);
}
