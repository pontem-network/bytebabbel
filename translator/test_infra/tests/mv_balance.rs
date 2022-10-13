#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use ethabi::Contract;

use eth::Flags;
use test_infra::init_log;

use crate::testssol::env::executor::MoveExecutor;

#[allow(dead_code)]
mod testssol;

const HELPER_MV: &str = "./resources/mv/build/test_helper/bytecode_modules/helper.mv";

#[test]
pub fn test_balance() {
    init_log();

    let path = PathBuf::from(HELPER_MV).canonicalize().unwrap();

    let bytecode = fs::read(&path).unwrap();
    let abi: Contract = serde_json::from_str("[]").unwrap();

    let mut vm = MoveExecutor::new(abi, Flags::native_interface());

    vm.deploy("0x1", bytecode);

    // Topping up the balance on account 0x42
    vm.run("0x1::helper::x42_1_000_000", "0x1", None).unwrap();
    // Getting a balance
    let result = vm.run("0x1::helper::balance", "0x42", None).unwrap();
    assert_eq!("Uint(1000000)", result.to_result_str());
}

#[test]
pub fn test_blocks() {
    init_log();

    let path = PathBuf::from(HELPER_MV).canonicalize().unwrap();

    let bytecode = fs::read(&path).unwrap();
    let abi: Contract = serde_json::from_str("[]").unwrap();

    let mut vm = MoveExecutor::new(abi, Flags::native_interface());

    vm.deploy("0x1", bytecode);

    vm.run("0x1::helper::genesis_inic", "0x1", None).unwrap();

    let height = fake_block(&mut vm, &["0x990", "0x991"]).unwrap();
    assert_eq!("Uint(1)", &height);

    let height = fake_block(&mut vm, &["0x992", "0x993"]).unwrap();
    assert_eq!("Uint(3)", &height);
}

fn fake_block(vm: &mut MoveExecutor, addresses: &[&str]) -> Result<String> {
    addresses
        .iter()
        .map(|addr| vm.run("0x1::helper::fake_block", "0x0", Some(addr)))
        .collect::<Result<Vec<_>>>()?;
    blocks_height(vm)
}

#[inline]
fn blocks_height(vm: &mut MoveExecutor) -> Result<String> {
    Ok(vm
        .run("0x1::helper::block_height", "0x1", None)?
        .to_result_str())
}
