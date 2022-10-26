use move_core_types::account_address::AccountAddress;

use eth::{
    compile::{build_sol, Evm},
    Flags,
};
use move_executor::{solidity::FromSolidity, MoveExecutor, MoveExecutorInstance};
use test_infra::init_log;

use crate::testssol::make_move_module;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_empty_constructor() {
    init_log();

    let mut vm = MoveExecutor::from_sol(
        "sol/constructors/empty.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

    vm.run("0x42::empty::constructor", "0x42", None).unwrap();
    let res = vm
        .run("0x42::empty::get_val", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(42)", res);
}

#[test]
pub fn test_constructor_with_data() {
    init_log();

    let evm = build_sol("sol/constructors/with_data.sol").unwrap();

    test(evm.contract(), "1000, true", 1000);
    test(evm.contract(), "1000, false", 42);

    fn test(evm: &Evm, init_args: &str, val: u128) {
        let bytecode = make_move_module(
            &format!("0x42::{}", evm.name()),
            evm.bin(),
            init_args,
            evm.abi(),
            Flags::default(),
        )
        .unwrap();
        let mut vm = MoveExecutor::new(
            serde_json::from_str(evm.abi()).unwrap(),
            Flags::default(),
            MoveExecutorInstance::Aptos,
        );
        vm.deploy("0x42", bytecode).unwrap();

        vm.run("0x42::with_data::constructor", "0x42", None)
            .unwrap();
        let res = vm
            .run("0x42::with_data::get_val", "0x42", Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!(format!("Uint({})", val), res);
    }
}

#[test]
pub fn test_store() {
    init_log();

    let mut vm = MoveExecutor::from_sol(
        "sol/store/load_store.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

    vm.run("0x42::load_store::constructor", "0x42", None)
        .unwrap();

    let a = rand::random::<u128>();
    let b = rand::random::<u128>();
    let c = rand::random::<bool>();
    let f = rand::random::<bool>();

    vm.run(
        "0x42::load_store::set_all",
        "0x42",
        Some(&format!("{},{},{},{}", a, b, c, f)),
    )
    .unwrap();

    let res = vm
        .run("0x42::load_store::get_all", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!(
        format!("Uint({}), Uint({}), Bool({}), Bool({})", a, b, c, f),
        res
    );

    let a = rand::random::<u128>();
    let b = rand::random::<u128>();
    let c = rand::random::<bool>();
    let f = rand::random::<bool>();

    vm.run("0x42::load_store::set_a", "0x42", Some(&format!("{}", a)))
        .unwrap();
    vm.run("0x42::load_store::set_b", "0x42", Some(&format!("{}", b)))
        .unwrap();
    vm.run("0x42::load_store::set_c", "0x42", Some(&format!("{}", c)))
        .unwrap();
    vm.run("0x42::load_store::set_f", "0x42", Some(&format!("{}", f)))
        .unwrap();

    let actual_a = vm
        .run("0x42::load_store::get_a", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!(format!("Uint({})", a), actual_a);

    let actual_b = vm
        .run("0x42::load_store::get_b", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!(format!("Uint({})", b), actual_b);

    let actual_c = vm
        .run("0x42::load_store::get_c", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!(format!("Bool({})", c), actual_c);

    let actual_f = vm
        .run("0x42::load_store::get_flag", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!(format!("Bool({})", f), actual_f);
}

#[test]
pub fn test_bool_store() {
    init_log();

    let mut vm = MoveExecutor::from_sol(
        "sol/store/bool_store.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

    vm.run("0x42::bool_store::constructor", "0x42", None)
        .unwrap();

    let actual_f = vm
        .run("0x42::bool_store::load", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!("Bool(false)", actual_f);

    vm.run("0x42::bool_store::store", "0x42", Some("true"))
        .unwrap();

    let actual_f = vm
        .run("0x42::bool_store::load", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!("Bool(true)", actual_f);

    vm.run("0x42::bool_store::store", "0x42", Some("false"))
        .unwrap();

    let actual_f = vm
        .run("0x42::bool_store::load", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!(format!("Bool({})", false), actual_f);
}
