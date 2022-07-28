use crate::testssol::make_move_module;
use test_infra::executor::MoveExecutor;

mod testssol;

#[test]
pub fn test_loops() {
    let bytecode = make_move_module(
        "0x1::Loop",
        include_str!("../bin/loop.bin"),
        include_str!("../bin/loop.abi"),
    );
    let mut vm = MoveExecutor::new();
    vm.deploy("0x1", bytecode);
    vm.run("0x1::Loop::for_loop", "10").unwrap();
}
