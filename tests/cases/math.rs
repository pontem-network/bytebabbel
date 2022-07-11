use crate::cases::make_move_module;
use crate::common::executor::MoveExecutor;
use move_core_types::value::MoveValue;

#[test]
pub fn math_fn_tests() {
    let bytecode = make_move_module(
        "0x1::MathFn",
        include_str!("../assets/bin/MathFn.bin"),
        include_str!("../assets/bin/MathFn.abi"),
        true,
    );
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);

    let props = [
        ("0x1::MathFn::summation_3", "", [MoveValue::U128(3)]),
        ("0x1::MathFn::summation_15", "", [MoveValue::U128(15)]),
        ("0x1::MathFn::summation_30", "", [MoveValue::U128(30)]),
        ("0x1::MathFn::subtraction_100", "", [MoveValue::U128(100)]),
        ("0x1::MathFn::subtraction_50", "", [MoveValue::U128(50)]),
        ("0x1::MathFn::subsum_106", "", [MoveValue::U128(106)]),
        ("0x1::MathFn::subsum_40", "", [MoveValue::U128(40)]),
    ];

    for (fn_name, params, exp_res) in props {
        let res = vm.run(fn_name, params);
        for ((actual_val, actual_tp), res) in res.returns.iter().zip(exp_res) {
            let actual_res = MoveValue::simple_deserialize(&actual_val, &actual_tp).unwrap();
            println!("{}({}) => {}|{:?}", fn_name, params, actual_res, res);
            assert_eq!(actual_res, res, "Function {}", fn_name);
        }
    }
}

#[test]
pub fn mult_fn_tests() {
    let bytecode = make_move_module(
        "0x1::MultFn",
        include_str!("../assets/bin/MultFn.bin"),
        include_str!("../assets/bin/MultFn.abi"),
        true,
    );
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);

    let props = [
        ("0x1::MultFn::multiplication_6", "", [MoveValue::U128(6)]),
        ("0x1::MultFn::multiplication_18", "", [MoveValue::U128(18)]),
        ("0x1::MultFn::multiplication_36", "", [MoveValue::U128(36)]),
        (
            "0x1::MultFn::multiplication_3888",
            "",
            [MoveValue::U128(3888)],
        ),
    ];

    for (fn_name, params, exp_res) in props {
        let res = vm.run(fn_name, params);
        for ((actual_val, actual_tp), res) in res.returns.iter().zip(exp_res) {
            let actual_res = MoveValue::simple_deserialize(&actual_val, &actual_tp).unwrap();
            println!("{}({}) => {}|{:?}", fn_name, params, actual_res, res);
            assert_eq!(actual_res, res, "Function {}", fn_name);
        }
    }
}
