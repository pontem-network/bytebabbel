use move_core_types::account_address::AccountAddress;

use eth::Flags;
use move_executor::{solidity::FromSolidity, MoveExecutor};
use test_infra::init_log;

#[test]
pub fn test_address_support() {
    init_log();

    let mut vm = MoveExecutor::from_sol(
        "sol/demo/address_support.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

    vm.run("0x42::AddressSupport::constructor", "0x42", None)
        .unwrap();

    let res = vm
        .run("0x42::AddressSupport::is_owner", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!("Bool(true)", res);

    let res = vm
        .run("0x42::AddressSupport::is_owner", "0x44", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!("Bool(false)", res);
}
