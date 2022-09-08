use crate::testssol::convert::ResultToString;
use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::{build_sol, Evm};
use crate::testssol::make_move_module;
use eth::abi::entries::AbiEntries;
use eth::Flags;
use move_core_types::value::MoveValue;
use test_infra::init_log;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_empty_constructor() {
    init_log();

    let evm = build_sol(include_bytes!("../sol/constructors/empty.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.bin(),
        "",
        evm.abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap(), Flags::default());
    vm.deploy("0x42", bytecode);
    vm.run("0x42::empty::constructor", "0x42", None).unwrap();
    let res = vm
        .run("0x42::empty::get_val", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(42)", res);
}

#[test]
pub fn test_constructor_with_data() {
    init_log();

    let evm = build_sol(include_bytes!("../sol/constructors/with_data.sol")).unwrap();

    test(&evm, "1000, true", 1000);
    test(&evm, "1000, false", 42);

    fn test(evm: &Evm, init_args: &str, val: u128) {
        let bytecode = make_move_module(
            &format!("0x42::{}", evm.name()),
            evm.bin(),
            init_args,
            evm.abi(),
            Flags::default(),
        )
        .unwrap();
        let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap(), Flags::default());
        vm.deploy("0x42", bytecode);
        vm.run("0x42::with_data::constructor", "0x42", None)
            .unwrap();
        let res = vm
            .run("0x42::with_data::get_val", "0x42", Some(""))
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!(format!("({})", val), res);
    }
}

#[test]
pub fn test_store() {
    init_log();

    let evm = build_sol(include_bytes!("../sol/store/load_store.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.bin(),
        "",
        evm.abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap(), Flags::default());
    vm.deploy("0x42", bytecode);

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
        .returns;
    assert_eq!(format!("({}, {}, {}, {})", a, b, c, f), res.to_result_str());

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
        .returns
        .to_result_str();
    assert_eq!(format!("({})", a), actual_a);

    let actual_b = vm
        .run("0x42::load_store::get_b", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!(format!("({})", b), actual_b);

    let actual_c = vm
        .run("0x42::load_store::get_c", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!(format!("({})", c), actual_c);

    let actual_f = vm
        .run("0x42::load_store::get_flag", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!(format!("({})", f), actual_f);
}

#[test]
pub fn test_bool_store() {
    init_log();

    let evm = build_sol(include_bytes!("../sol/store/bool_store.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.bin(),
        "",
        evm.abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap(), Flags::default());
    vm.deploy("0x42", bytecode);
    vm.run("0x42::bool_store::constructor", "0x42", None)
        .unwrap();

    let actual_f = vm
        .run("0x42::bool_store::load", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!(format!("({})", false), actual_f);

    vm.run("0x42::bool_store::store", "0x42", Some("true"))
        .unwrap();

    let actual_f = vm
        .run("0x42::bool_store::load", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!(format!("({})", true), actual_f);

    vm.run("0x42::bool_store::store", "0x42", Some("false"))
        .unwrap();

    let actual_f = vm
        .run("0x42::bool_store::load", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!(format!("({})", false), actual_f);
}
