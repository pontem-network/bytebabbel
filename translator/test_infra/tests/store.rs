use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use move_core_types::value::MoveValue;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_store() {
    let evm = build_sol(include_bytes!("../sol/store/load_store.sol")).unwrap();
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);

    vm.run("0x1::load_store::init_store", "0x1").unwrap();
    let a = rand::random::<u128>();
    let b = rand::random::<u128>();
    let c = rand::random::<bool>();
    let f = rand::random::<bool>();
    vm.run(
        "0x1::load_store::set_all",
        &format!("{},{},{},{}", a, b, c, f),
    )
    .unwrap();

    let res = vm.run("0x1::load_store::get_all", "").unwrap().returns;

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

    vm.run("0x1::load_store::set_a", &a.to_string()).unwrap();
    vm.run("0x1::load_store::set_b", &b.to_string()).unwrap();
    vm.run("0x1::load_store::set_c", &c.to_string()).unwrap();
    vm.run("0x1::load_store::set_f", &f.to_string()).unwrap();

    let actual_a = vm
        .run("0x1::load_store::get_a", "")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::U128(a),
        MoveValue::simple_deserialize(&actual_a.0, &actual_a.1).unwrap()
    );

    let actual_b = vm
        .run("0x1::load_store::get_b", "")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::U128(b),
        MoveValue::simple_deserialize(&actual_b.0, &actual_b.1).unwrap()
    );

    let actual_c = vm
        .run("0x1::load_store::get_c", "")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(c),
        MoveValue::simple_deserialize(&actual_c.0, &actual_c.1).unwrap()
    );

    let actual_f = vm
        .run("0x1::load_store::get_flag", "")
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
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    vm.run("0x1::bool_store::init_store", "0x1").unwrap();
    let actual_f = vm
        .run("0x1::bool_store::load", "")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(false),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );

    vm.run("0x1::bool_store::store", "true").unwrap();
    let actual_f = vm
        .run("0x1::bool_store::load", "")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(true),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );

    vm.run("0x1::bool_store::store", "false").unwrap();
    let actual_f = vm
        .run("0x1::bool_store::load", "")
        .unwrap()
        .returns
        .remove(0);
    assert_eq!(
        MoveValue::Bool(false),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );
}

#[test]
pub fn empty_constractor() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/constructors/empty.sol")).unwrap();
    dbg!(&evm.bin());
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
}

#[test]
pub fn constractor_with_data() {}
