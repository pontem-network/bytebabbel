use crate::cases::make_move_module;
use crate::common::executor::MoveExecutor;
use crate::log_init;
use move_core_types::value::MoveValue;

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

    let props = [(
        "0x1::Params::minusmultiply_params",
        "8,2,3",
        [MoveValue::U128(2)],
    )];

    for (fn_name, params, exp_res) in props {
        let res = vm.run(fn_name, params).unwrap();
        for ((actual_val, actual_tp), res) in res.returns.iter().zip(exp_res) {
            let actual_res = MoveValue::simple_deserialize(&actual_val, &actual_tp).unwrap();
            println!("{}({}) => {}|{:?}", fn_name, params, actual_res, res);
            assert_eq!(actual_res, res, "Function {}", fn_name);
        }
    }
}

/*
0: CopyLoc(1) => [2]
1: LdU128(0) => [2, 0]
2: Eq => [false]
3: Not => [true]
4: CopyLoc(2) => [true, 3]
5: LdU128(340282366920938463463374607431768211455) => [true, 2, 340282366920938463463374607431768211455]
6: CopyLoc(1) => [true, 2, 340282366920938463463374607431768211455, 2]
Store stack: Stack { inner: [true, 2, 340282366920938463463374607431768211455, 2] }
7: StLoc(3) => [true, 2, 340282366920938463463374607431768211455]
8: StLoc(4) => [true, 2]
9: StLoc(5) => [true]
10: StLoc(6) => []
11: CopyLoc(4) => [340282366920938463463374607431768211455]
12: LdU128(0) => [340282366920938463463374607431768211455, 0]
13: Eq => [false]
14: BrTrue(19) => []
15: MoveLoc(3) => [2]
16: MoveLoc(4) => [2, 340282366920938463463374607431768211455]
17: Div => [170282366920938463463374607431768211455]
18: StLoc(4) => []
19: MoveLoc(6) => [true]
20: MoveLoc(5) => [true, 2]
21: MoveLoc(4) => [true, 2, 170282366920938463463374607431768211455]
22: Gt => [true, true]   --- l
23: StLoc(6) => [true]
24: StLoc(7) => []
25: MoveLoc(6) => [true]
26: BrTrue(30) => []
27: LdU128(0) => [0]
28: StLoc(4) => []
29: Branch(32) => []
30: LdU128(1) => [1]
31: StLoc(4) => []
32: MoveLoc(7) => [true]
33: MoveLoc(4) => [true, 170282366920938463463374607431768211455]
34: StLoc(4) => [true]
35: StLoc(7) => []
36: MoveLoc(7) => [true]
37: BrTrue(41) => []
38: LdU128(0) => [0]
39: StLoc(5) => []
40: Branch(43) => []
41: LdU128(1) => [U128]
42: StLoc(5) => []
43: MoveLoc(5) => [0]
44: MoveLoc(4) => [0, 170282366920938463463374607431768211455]
45: BitAnd => [U128]
46: LdU128(0) => [U128, U128]
47: Eq => [Bool]
48: BrTrue(0) => []
49: LdU64(0) => [U64]
50: Abort => []
51: CopyLoc(0) => [U128]
52: CopyLoc(1) => [U128, U128]
53: CopyLoc(2) => [U128, U128, U128]
54: Mul => [U128, U128]
55: Lt => [Bool]
56: Not => [Bool]
57: BrTrue(0) => []
58: LdU64(0) => [U64]
59: Abort => []
60: CopyLoc(0) => [U128]
61: CopyLoc(1) => [U128, U128]
62: CopyLoc(2) => [U128, U128, U128]
63: Mul => [U128, U128]
64: Sub => [U128]
65: StLoc(4) => []
66: MoveLoc(4) => [U128]
67: Ret => []
 */
