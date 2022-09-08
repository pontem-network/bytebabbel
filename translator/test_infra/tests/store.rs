use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::{build_sol, Evm};
use crate::testssol::make_move_module;
use move_core_types::value::MoveValue;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_empty_constructor() {
    let evm = build_sol(include_bytes!("../sol/constructors/empty.sol")).unwrap();
    let bytecode =
        make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), "", evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    vm.run("0x1::empty::constructor", "0x1").unwrap();
    let res = vm.run("0x1::empty::get_val", "0x1").unwrap().returns;
    assert_eq!(
        MoveValue::U128(42),
        MoveValue::simple_deserialize(&res[0].0, &res[0].1).unwrap()
    );
}

#[test]
pub fn test_constructor_with_data() {
    let evm = build_sol(include_bytes!("../sol/constructors/with_data.sol")).unwrap();

    test(&evm, "1000, true", 1000);
    test(&evm, "1000, false", 42);

    fn test(evm: &Evm, init_args: &str, val: u128) {
        let bytecode = make_move_module(
            &format!("0x1::{}", evm.name()),
            evm.bin(),
            init_args,
            evm.abi(),
        )
        .unwrap();
        let mut vm = MoveExecutor::new();
        vm.deploy("0x1", bytecode);
        vm.run("0x1::with_data::constructor", "0x1").unwrap();
        let res = vm.run("0x1::with_data::get_val", "0x1").unwrap().returns;
        assert_eq!(
            MoveValue::U128(val),
            MoveValue::simple_deserialize(&res[0].0, &res[0].1).unwrap()
        );
    }
}

#[test]
pub fn test_store() {
    let evm = build_sol(include_bytes!("../sol/store/load_store.sol")).unwrap();
    let bytecode =
        make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), "", evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);

    vm.run("0x1::load_store::constructor", "0x1").unwrap();
    let a = rand::random::<u128>();
    let b = rand::random::<u128>();
    let c = rand::random::<bool>();
    let f = rand::random::<bool>();
    vm.run(
        "0x1::load_store::set_all",
        &format!("0x1,{},{},{},{}", a, b, c, f),
    )
    .unwrap();

    let res = vm.run("0x1::load_store::get_all", "0x1").unwrap().returns;

    assert_eq!(
        MoveValue::U128(a),
        MoveValue::simple_deserialize(&res[0].0, &res[0].1).unwrap()
    );
    assert_eq!(
        MoveValue::U128(b),
        MoveValue::simple_deserialize(&res[1].0, &res[1].1).unwrap()
    );
    assert_eq!(
        MoveValue::Bool(c),
        MoveValue::simple_deserialize(&res[2].0, &res[3].1).unwrap()
    );
    assert_eq!(
        MoveValue::Bool(f),
        MoveValue::simple_deserialize(&res[3].0, &res[3].1).unwrap()
    );

    let a = rand::random::<u128>();
    let b = rand::random::<u128>();
    let c = rand::random::<bool>();
    let f = rand::random::<bool>();

    vm.run("0x1::load_store::set_a", &format!("0x1, {}", a))
        .unwrap();
    vm.run("0x1::load_store::set_b", &format!("0x1, {}", b))
        .unwrap();
    vm.run("0x1::load_store::set_c", &format!("0x1, {}", c))
        .unwrap();
    vm.run("0x1::load_store::set_f", &format!("0x1, {}", f))
        .unwrap();

    let actual_a = vm
        .run("0x1::load_store::get_a", "0x1")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::U128(a),
        MoveValue::simple_deserialize(&actual_a.0, &actual_a.1).unwrap()
    );

    let actual_b = vm
        .run("0x1::load_store::get_b", "0x1")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::U128(b),
        MoveValue::simple_deserialize(&actual_b.0, &actual_b.1).unwrap()
    );

    let actual_c = vm
        .run("0x1::load_store::get_c", "0x1")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(c),
        MoveValue::simple_deserialize(&actual_c.0, &actual_c.1).unwrap()
    );

    let actual_f = vm
        .run("0x1::load_store::get_flag", "0x1")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(f),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );
}

#[test]
pub fn test_bool_store() {
    let evm = build_sol(include_bytes!("../sol/store/bool_store.sol")).unwrap();
    let bytecode =
        make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), "", evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    vm.run("0x1::bool_store::constructor", "0x1").unwrap();
    let actual_f = vm
        .run("0x1::bool_store::load", "0x1")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(false),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );

    vm.run("0x1::bool_store::store", "0x1,true").unwrap();
    let actual_f = vm
        .run("0x1::bool_store::load", "0x1")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(true),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );

    vm.run("0x1::bool_store::store", "0x1,false").unwrap();
    let actual_f = vm
        .run("0x1::bool_store::load", "0x1")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(false),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );
}
