use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::{make_move_module, sol_path};
use eth::compile::build_sol;
use eth::Flags;
use test_infra::init_log;

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_template_crop() {
    init_log();
    let evm = build_sol(sol_path().join("mod/add.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.contract().bin(),
        "",
        evm.contract().abi(),
        Flags::default(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(
        serde_json::from_str(evm.contract().abi()).unwrap(),
        Flags::default(),
    );
    vm.deploy("0x42", bytecode);

    vm.run("0x42::AddMod::constructor", "0x42", None).unwrap();

    let res = vm
        .run("0x42::AddMod::add_mod_u256", "0x42", Some("25, 10, 10"))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(5)", res);

    let res = vm
        .run("0x42::AddMod::add_mod_u256_max", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(202)", res);
}
