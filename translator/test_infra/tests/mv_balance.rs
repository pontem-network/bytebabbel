#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use ethabi::Contract;

use eth::Flags;
use move_executor::{MoveExecutor, MoveExecutorInstance};
use test_infra::init_log;

fn me() -> Result<MoveExecutor> {
    let path = PathBuf::from("./resources/mv/build/test_helper/bytecode_modules/helper.mv")
        .canonicalize()?;
    let bytecode = fs::read(path)?;
    let abi: Contract = serde_json::from_str("[]")?;

    let mut vm = MoveExecutor::new(abi, Flags::native_interface(), MoveExecutorInstance::Aptos);
    vm.deploy("0x1", bytecode)?;

    Ok(vm)
}

#[test]
pub fn test_balance() {
    init_log();

    let mut vm = me().unwrap();

    // Topping up the balance on account 0x42
    vm.run("0x1::helper::x42_1_000_000", "0x1", None).unwrap();
    // Getting a balance
    let result = vm.run("0x1::helper::balance", "0x42", None).unwrap();
    assert_eq!("Uint(1000000)", result.to_result_str());
}

#[test]
pub fn test_blocks() {
    init_log();

    let mut vm = me().unwrap();

    vm.run("0x1::helper::genesis_inic", "0x1", None).unwrap();
    vm.run("0x1::helper::fake_block", "0x0", None).unwrap();

    let height = vm
        .run_native("0x1::helper::block_height", "0x1", None)
        .unwrap()
        .to_result_str();
    assert_eq!("Uint(3)", &height);
}
