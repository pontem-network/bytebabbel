use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::make_move_module;
use eth::compile::build_sol;
use eth::Flags;
use test_infra::init_log;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_for_loop() {
    init_log();

    let evm = build_sol(include_bytes!("../sol/loop/for.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.bin(),
        "",
        evm.abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(serde_json::from_str(evm.abi()).unwrap(), Flags::default());
    vm.deploy("0x42", bytecode);

    vm.run("0x42::ForLoop::constructor", "0x42", None).unwrap();
    let res = vm
        .run("0x42::ForLoop::sum", "0x42", Some("9, 9"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(81)", res);
}
