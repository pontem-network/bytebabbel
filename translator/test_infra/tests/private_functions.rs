use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::{make_move_module, sol_path};
use eth::compile::build_sol;
use eth::Flags;
use test_infra::init_log;

mod testssol;

#[test]
pub fn test_private_functions() {
    init_log();
    let evm = build_sol(sol_path().join("private_functions.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.contract().bin(),
        "",
        evm.contract().abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(
        serde_json::from_str(evm.contract().abi()).unwrap(),
        Flags::default(),
    );
    vm.deploy("0x42", bytecode);

    vm.run("0x42::private_functions::constructor", "0x42", None)
        .unwrap();

    // let res = vm
    //     .run("0x42::PrivateFunctions::test", "0x42", None)
    //     .unwrap()
    //     .to_result_str();
    // assert_eq!("Uint(10)", res);
}
