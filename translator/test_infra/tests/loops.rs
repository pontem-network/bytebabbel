use test_infra::sol::build_sol;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_loops() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/loops.sol")).unwrap();
    println!("{:?}", evm);
    // let bytecode = make_move_module(
    //     "0x1::Loop",
    //     include_str!("../bin/loop.bin"),
    //     include_str!("../bin/loop.abi"),
    // );
    // let mut vm = MoveExecutor::new();
    // vm.deploy("0x1", bytecode);
    // vm.run("0x1::Loop::for_loop", "10").unwrap();
}
