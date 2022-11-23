use move_core_types::account_address::AccountAddress;

use eth::Flags;
use move_executor::{solidity::FromSolidity, MoveExecutor};
use test_infra::init_log;

#[test]
pub fn test_strings() {
    init_log();

    let mut vm = MoveExecutor::from_sol(
        "sol/strings.sol",
        AccountAddress::from_hex_literal("0x42").unwrap(),
        "",
        Flags::default(),
    )
    .unwrap();

    vm.run("0x42::Strings::constructor", "0x42", None).unwrap();

    let res = vm
        .run("0x42::Strings::small_const_str", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!("String(\"hello\")", res);

    let res = vm
        .run("0x42::Strings::large_const_str", "0x42", Some(""))
        .unwrap()
        .to_result_str();
    assert_eq!(
        "String(\"This is the large string that we are testing. And it is bigger than 32 bytes.\")",
        res
    );
}
