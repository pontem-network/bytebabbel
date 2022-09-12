use crate::testssol::convert::ResultToString;
use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use eth::abi::entries::AbiEntries;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_for_users() {
    env_logger::init();
    let evm = build_sol(include_bytes!("../sol/demo/users.sol")).unwrap();
    let bytecode = make_move_module(
        &format!("0x42::{}", evm.name()),
        evm.bin(),
        "0x13",
        evm.abi(),
    )
    .unwrap();
    let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap());
    vm.deploy("0x42", bytecode);

    vm.run("0x42::Users::constructor", "0x42", None).unwrap();
    let res = vm
        .run("0x42::Users::create_user", "0x42", Some(""))
        .unwrap();
    let new_user_event = &res.events[0];

    let mut guid = vec![4, 0, 0, 0, 0, 0, 0, 0];
    guid.extend(AccountAddress::from_hex_literal("0x42").unwrap().as_slice());
    assert_eq!(guid, new_user_event.0);
    assert_eq!(0, new_user_event.1);
    assert_eq!(
        TypeTag::Struct(StructTag {
            address: AccountAddress::from_hex_literal("0x42").unwrap(),
            module: Identifier::new("Users").unwrap(),
            name: Identifier::new("Event").unwrap(),
            type_params: vec![]
        }),
        new_user_event.2
    );

    let res = vm
        .run("0x42::Users::get_id", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(2)", res);

    let res = vm
        .run("0x42::Users::get_id", "0x13", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(1)", res);

    let res = vm
        .run("0x42::Users::is_owner", "0x13", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(true)", res);

    let res = vm
        .run("0x42::Users::is_owner", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(false)", res);

    let res = vm
        .run("0x42::Users::get_balance", "0x13", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(10000000000000000000000000000)", res);

    let res = vm
        .run("0x42::Users::get_balance", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(0)", res);

    let res = vm
        .run("0x42::Users::transfer", "0x13", Some("0x42, 1000"))
        .unwrap();
    let new_user_event = &res.events[0];
    let mut guid = vec![4, 0, 0, 0, 0, 0, 0, 0];
    guid.extend(AccountAddress::from_hex_literal("0x42").unwrap().as_slice());
    assert_eq!(guid, new_user_event.0);
    assert_eq!(1, new_user_event.1);
    assert_eq!(
        TypeTag::Struct(StructTag {
            address: AccountAddress::from_hex_literal("0x42").unwrap(),
            module: Identifier::new("Users").unwrap(),
            name: Identifier::new("Event").unwrap(),
            type_params: vec![]
        }),
        new_user_event.2
    );

    let res = vm
        .run("0x42::Users::get_balance", "0x13", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(9999999999999999999999999000)", res);

    let res = vm
        .run("0x42::Users::get_balance", "0x42", Some(""))
        .unwrap()
        .returns
        .to_result_str();
    assert_eq!("(1000)", res);

    let res = vm.run("0x42::Users::transfer", "0x42", Some("0x13, 1001"));
    assert!(res.is_err());
}
