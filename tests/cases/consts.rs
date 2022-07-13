use crate::cases::make_move_module;
use crate::common::executor::MoveExecutor;
use move_core_types::value::MoveValue;

#[test]
pub fn consts_fn_tests() {
    let bytecode = make_move_module(
        "0x1::ConstFn",
        include_str!("../assets/bin/ConstFn.bin"),
        include_str!("../assets/bin/ConstFn.abi"),
        true,
    );
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);

    let props = [
        ("0x1::ConstFn::const_fn_10", "", [MoveValue::U128(10)]),
        (
            "0x1::ConstFn::const_fn_426574676453456",
            "",
            [MoveValue::U128(426574676453456)],
        ),
        ("0x1::ConstFn::const_fn_true", "", [MoveValue::Bool(true)]),
        (
            "0x1::ConstFn::const_fn_90_plus_54",
            "",
            [MoveValue::U128(90 + 54)],
        ),
        (
            "0x1::ConstFn::const_fn_true_1",
            "",
            [MoveValue::Bool(false)],
        ),
    ];

    for (fn_name, params, exp_res) in props {
        let res = vm.run(fn_name, params).unwrap();
        for ((actual_val, actual_tp), res) in res.returns.iter().zip(exp_res) {
            let actual_res = MoveValue::simple_deserialize(&actual_val, &actual_tp).unwrap();
            println!("{}({}) => {}|{:?}", fn_name, params, actual_res, res);
            assert_eq!(actual_res, res, "Function {}", fn_name);
        }
    }
}
