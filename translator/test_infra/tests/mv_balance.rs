#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use ethabi::Contract;

use eth::Flags;
use test_infra::init_log;

use crate::testssol::env::executor::MoveExecutor;

#[allow(dead_code)]
mod testssol;

const TEST_HELPER_MOVE: &str = "./resources/mv/build/test_helper/bytecode_modules/balance.mv";

#[test]
pub fn test_balance() {
    init_log();

    let path = PathBuf::from(TEST_HELPER_MOVE).canonicalize().unwrap();

    let bytecode = fs::read(&path).unwrap();
    let abi: Contract = serde_json::from_str("[]").unwrap();

    let mut vm = MoveExecutor::new(abi, Flags::native_interface());
    vm.deploy("0x1", bytecode);
    vm.run("0x1::balance::x42_1_000_000", "0x1", None).unwrap();

    let result = vm.run("0x1::balance::balance", "0x42", None).unwrap();
    assert_eq!("Uint(1000000)", result.to_result_str());
}
