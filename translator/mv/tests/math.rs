use evm::abi::Abi;
use evm::bytecode::executor::execution::FunctionFlow;
use evm::bytecode::executor::stack::{Frame, StackFrame};
use evm::program::Program;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_core_types::value::MoveValue;
use mv::function::code::intrinsic::math::u128_model::U128MathModel;
use mv::mvir::MvModule;
use std::collections::HashMap;
use test_infra::executor::MoveExecutor;

pub fn make_module(flow: FunctionFlow) -> Vec<u8> {
    let mut graph = HashMap::default();
    let abi = Abi::try_from(
        "[
        {
            \"inputs\": [
            {
                \"internalType\": \"uint256\",
                \"name\": \"a\",
                \"type\": \"uint256\"
            },
            {
                \"internalType\": \"uint256\",
                \"name\": \"b\",
                \"type\": \"uint256\"
            }
            ],
            \"name\": \"fun_1\",
            \"outputs\": [
            {
                \"internalType\": \"uint256\",
                \"name\": \"\",
                \"type\": \"uint256\"
            }
            ],
            \"stateMutability\": \"pure\",
            \"type\": \"function\"
        }
    ]",
    )
    .unwrap();
    let hash = abi.fun_hashes().next().unwrap();

    graph.insert(hash, flow);
    let program = Program::new("TestMod", graph, None, abi).unwrap();
    let module =
        MvModule::from_evm_program(CORE_CODE_ADDRESS, U128MathModel::default(), program).unwrap();

    let compiled_module = module.make_move_module().unwrap();
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode).unwrap();
    bytecode
}

#[test]
pub fn test_u256_math_cast() {
    let mut flow = FunctionFlow::default();
    let var = flow.calc_var(StackFrame::new(Frame::Param(0)));
    flow.set_result(var);
    let module = make_module(flow);

    let mut executor = MoveExecutor::new();
    executor.deploy("0x1", module);

    fn test(exec: &mut MoveExecutor, expected: u128) {
        let res = exec
            .run(
                "0x1::TestMod::fun_1",
                &format!("{}, 0", expected.to_string()),
            )
            .unwrap()
            .returns;
        let (val, tp) = &res[0];
        if let MoveValue::U128(val) = MoveValue::simple_deserialize(val, tp).unwrap() {
            assert_eq!(val, expected);
        } else {
            panic!("Invalid return type");
        }
    }

    test(&mut executor, 0);
    test(&mut executor, u128::MAX);

    for _ in 0..1000 {
        test(&mut executor, rand::random());
    }
}