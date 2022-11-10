use move_core_types::account_address::AccountAddress;

use eth::Flags;
use move_executor::{solidity::FromSolidity, MoveExecutor};
use test_infra::init_log;

#[test]
pub fn test_loop() {
    init_log();

    let mut vm = MoveExecutor::from_sol(
        "sol/loop/for.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

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

    let mut vm = MoveExecutor::from_sol(
        "sol/loop/while.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

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

    let mut vm = MoveExecutor::from_sol(
        "sol/loop/dowhile.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

    vm.run("0x42::WhileLoop::constructor", "0x42", None)
        .unwrap();

    let res = vm
        .run("0x42::WhileLoop::sum", "0x42", Some("10, 10"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(100)", res);
}
