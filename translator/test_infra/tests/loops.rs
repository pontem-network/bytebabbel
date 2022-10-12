use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::{make_move_module, sol_path};
use eth::compile::build_sol;
use eth::Flags;
use test_infra::init_log;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_loop() {
    init_log();
    let evm = build_sol(sol_path().join("loop/for.sol")).unwrap();
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

    vm.run("0x42::ForLoop::constructor", "0x42", None).unwrap();
    let res = vm
        .run("0x42::ForLoop::for_loop", "0x42", Some("10,10"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(100), Uint(9999999999999990)", res);

    let res = vm
        .run("0x42::ForLoop::for_static", "0x42", Some("10,10"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(10)", res);
}

#[test]
pub fn test_while() {
    init_log();
    let evm = build_sol(sol_path().join("loop/while.sol")).unwrap();
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

    vm.run("0x42::WhileLoop::constructor", "0x42", None)
        .unwrap();

    let res = vm
        .run("0x42::WhileLoop::sum", "0x42", Some("10, 10"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(100)", res);
}

#[test]
pub fn test_do_while() {
    init_log();
    let evm = build_sol(sol_path().join("loop/dowhile.sol")).unwrap();
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

    vm.run("0x42::WhileLoop::constructor", "0x42", None)
        .unwrap();

    let res = vm
        .run("0x42::WhileLoop::sum", "0x42", Some("10, 10"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(100)", res);
}

#[test]
pub fn using() {
    init_log();
    let evm = build_sol(sol_path().join("uniswap/9_using.sol")).unwrap();
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

    vm.run("0x42::Using::constructor", "0x42", None).unwrap();

    vm.run("0x42::Using::add", "0x42", Some("1,2")).unwrap();
}
