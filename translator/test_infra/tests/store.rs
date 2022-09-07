use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::revm::REvm;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use move_core_types::value::MoveValue;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_store() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/store/load_store.sol")).unwrap();
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);

    vm.run("0x1::load_store::constructor", "0x1").unwrap();
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
    vm.run("0x1::bool_store::constructor", "0x1").unwrap();
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
    println!("{:?}", evm.bin());
    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    vm.run("0x1::empty::constructor", "0x1").unwrap();
    let actual_f = vm.run("0x1::empty::get_val", "").unwrap().returns.remove(0);
    assert_eq!(
        MoveValue::U128(42),
        MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
    );
}

#[test]
pub fn constractor_with_data() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/constructors/with_data.sol")).unwrap();
    println!("-+- {:?}", evm.bin());

    let evm_1 = REvm::try_from(hex::decode(evm.bin()).unwrap()).unwrap();
    dbg!(evm_1.run_tx(vec![0x608060405234801561001057600080fd5b506040516101cb3803806101cb8339818101604052810190610032919061007e565b8015610044578160008190555061004d565b602a6000819055505b5050610107565b600081519050610063816100d9565b92915050565b600081519050610078816100f0565b92915050565b60008060408385031215610095576100946100d4565b5b60006100a385828601610069565b92505060206100b485828601610054565b9150509250929050565b60008115159050919050565b6000819050919050565b600080fd5b6100e2816100be565b81146100ed57600080fd5b50565b6100f9816100ca565b811461010457600080fd5b50565b60b6806101156000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c806319bdf84d14602d575b600080fd5b60336047565b604051603e9190605d565b60405180910390f35b60008054905090565b6057816076565b82525050565b6000602082019050607060008301846050565b92915050565b600081905091905056fea2646970667358221220db04a448aa5406a7168c21946600bdb8640adee82f058fc36a01a6b8a488c4ae64736f6c63430008070033000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000001]).unwrap());

    let bytecode = make_move_module(&format!("0x1::{}", evm.name()), evm.bin(), evm.abi()).unwrap();
    fn test(bytecode: Vec<u8>, val: u128, cnd: bool, expected: u128) {
        let mut vm = MoveExecutor::new();
        vm.deploy("0x1", bytecode);
        vm.run(
            "0x1::with_data::constructor",
            &format!("0x1, {}, {}", val, cnd),
        )
        .unwrap();
        // let actual_f = vm
        //     .run("0x1::with_data::get_val", "")
        //     .unwrap()
        //     .returns
        //     .remove(0);
        // assert_eq!(
        //     MoveValue::U128(expected),
        //     MoveValue::simple_deserialize(&actual_f.0, &actual_f.1).unwrap()
        // );
    }
    test(bytecode.clone(), 1000, false, 42);
    test(bytecode.clone(), 1000, true, 1000);
}

/*
    mem[64] = 128;
    let var_3 =  mem[64];
    let var_4 =  0;
    let var_5 =  0 + 128;
    let var_6 =  64;
    mem[64] = 128;
    let var_7 =  128 + 0;
    let var_8 =  64;
    let var_9 =  128 - 128;
    let var_10 =  0 <! 64; true
    let var_11 =  q == 0;
    if false {
       let var_12 =  0;
       let var_13 =  128 + var_12;
       let var_14 =  mem[var_13];
       let var_15 =  32;
       let var_16 =  128 + var_15;
       let var_17 =  mem[var_16];
       let var_18 =  var_17 == 0;
       let var_19 =  var_18 == 0;
       let var_20 =  var_17 == var_19;
       if var_20 {
           let var_21 =  var_17 == 0;
           if var_21 {
               let var_22 =  42;
               let var_23 =  0;
               store[var_23] = var_22;
               let var_24 =  182;
               let var_25 =  0;
               return [var_25; var_24];
           } else {
               let var_26 =  0;
               store[var_26] = var_14;
               let var_27 =  182;
               let var_28 =  0;
               return [var_28; var_27];
           }
       } else {
           abort!(255);
       }
    } else {
       abort!(255);
    }
*/
