use crate::cases::make_move_module;
use crate::common::executor::MoveExecutor;
use crate::log_init;

#[test]
pub fn plus_fn_tests() {
    log_init();

    let bytecode = make_move_module(
        "0x1::Params",
        include_str!("../assets/bin/Parameters.bin"),
        include_str!("../assets/bin/Parameters.abi"),
    );

    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);

    // let props = [
    //   //  ("0x1::Params::test_3", "0, 1, 2", [MoveValue::U128(1)]),
    //     // ("0x1::Params::plus", "13,90", [MoveValue::U128(13 + 90)]),
    //     // ("0x1::Params::is_zero", "13", [MoveValue::Bool(false)]),
    //     // ("0x1::Params::is_zero", "0", [MoveValue::Bool(true)]),
    //     // ("0x1::Params::a_or_b", "10, 15, true", [MoveValue::U128(10)]),
    //     // (
    //     //     "0x1::Params::a_or_b",
    //     //     "10, 15, false",
    //     //     [MoveValue::U128(15)],
    //     // ),
    // ];
    //
    // for (fn_name, params, exp_res) in props {
    //     let res = vm.run(fn_name, params).unwrap();
    //     for ((actual_val, actual_tp), res) in res.returns.iter().zip(exp_res) {
    //         let actual_res = MoveValue::simple_deserialize(&actual_val, &actual_tp).unwrap();
    //         println!("{}({}) => {}|{:?}", fn_name, params, actual_res, res);
    //         assert_eq!(actual_res, res, "Function {}", fn_name);
    //     }
    // }
}
