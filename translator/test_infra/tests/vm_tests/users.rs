use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};

use eth::compile::build_sol;
use eth::Flags;
use move_executor::{solidity::FromSolidity, MoveExecutor, MoveExecutorInstance};
use test_infra::init_log;

use crate::testssol::make_move_module;

const ALICE: &str = "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00";
const BOB: &str = "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0";

#[test]
pub fn test_for_users() {
    init_log();

    fn test(flags: Flags) {
        let mut vm = MoveExecutor::from_sol(
            "../../examples/users.sol",
            AccountAddress::from_hex_literal(ALICE).unwrap(),
            BOB,
            flags,
        )
        .unwrap();

        vm.run(&format!("{ALICE}::Users::constructor"), ALICE, None)
            .unwrap();

        let res = vm
            .run(&format!("{ALICE}::Users::create_user"), ALICE, Some(""))
            .unwrap();
        let new_user_event = &res.events[0];

        let mut guid = vec![4, 0, 0, 0, 0, 0, 0, 0];
        guid.extend(AccountAddress::from_hex_literal(ALICE).unwrap().as_slice());
        assert_eq!(guid, new_user_event.0);
        assert_eq!(0, new_user_event.1);
        assert_eq!(
            TypeTag::Struct(Box::new(StructTag {
                address: AccountAddress::from_hex_literal(ALICE).unwrap(),
                module: Identifier::new("Users").unwrap(),
                name: Identifier::new("Event").unwrap(),
                type_params: vec![],
            })),
            new_user_event.2
        );

        let res = vm
            .run(&format!("{ALICE}::Users::get_id"), ALICE, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!(res, "Uint(2)");

        let res = vm
            .run(&format!("{ALICE}::Users::get_id"), BOB, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!(res, "Uint(1)");

        let res = vm
            .run(&format!("{ALICE}::Users::is_owner"), BOB, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!("Bool(true)", res);

        let res = vm
            .run(&format!("{ALICE}::Users::is_owner"), ALICE, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!("Bool(false)", res);

        let res = vm
            .run(&format!("{ALICE}::Users::get_balance"), BOB, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!("Uint(10000000000000000000000000000)", res);

        let res = vm
            .run(&format!("{ALICE}::Users::get_balance"), ALICE, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!("Uint(0)", res);

        let res = vm
            .run(
                &format!("{ALICE}::Users::transfer"),
                BOB,
                Some(&format!("{ALICE}, 1000")),
            )
            .unwrap();
        let new_user_event = &res.events[0];
        let mut guid = vec![4, 0, 0, 0, 0, 0, 0, 0];
        guid.extend(AccountAddress::from_hex_literal(ALICE).unwrap().as_slice());
        assert_eq!(guid, new_user_event.0);
        assert_eq!(1, new_user_event.1);
        assert_eq!(
            TypeTag::Struct(Box::new(StructTag {
                address: AccountAddress::from_hex_literal(ALICE).unwrap(),
                module: Identifier::new("Users").unwrap(),
                name: Identifier::new("Event").unwrap(),
                type_params: vec![],
            })),
            new_user_event.2
        );

        let res = vm
            .run(&format!("{ALICE}::Users::get_balance"), BOB, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!("Uint(9999999999999999999999999000)", res);

        let res = vm
            .run(&format!("{ALICE}::Users::get_balance"), ALICE, Some(""))
            .unwrap()
            .to_result_str();
        assert_eq!("Uint(1000)", res);

        let res = vm.run(&format!("{ALICE}::Users::create_user"), ALICE, Some(""));
        assert!(res.is_err());

        let res = vm.run(
            &format!("{ALICE}::Users::transfer"),
            ALICE,
            Some(&format!("{BOB}, 1001")),
        );
        assert!(res.is_err());
    }

    test(Flags::default());
    test(Flags::native_interface());
    test(Flags {
        native_input: true,
        native_output: true,
        hidden_output: false,
        u128_io: true,
    });
    test_for_users_with_hidden_result();
}

pub fn test_for_users_with_hidden_result() {
    let flags = Flags {
        native_input: false,
        native_output: false,
        hidden_output: true,
        u128_io: false,
    };
    let evm = build_sol("../../examples/users.sol").unwrap();
    let bytecode = make_move_module(
        &format!("{ALICE}::{}", evm.name()),
        evm.contract().bin(),
        BOB,
        evm.contract().abi(),
        flags,
    )
    .unwrap();
    let mut vm = MoveExecutor::new(evm.abi().unwrap(), flags, MoveExecutorInstance::Aptos);
    vm.deploy(ALICE, bytecode).unwrap();

    vm.run(&format!("{ALICE}::Users::constructor"), ALICE, None)
        .unwrap();

    let res = vm
        .run(&format!("{ALICE}::Users::create_user"), ALICE, Some(""))
        .unwrap();
    assert_eq!(0, res.returns.len());

    let res = vm
        .run(&format!("{ALICE}::Users::get_balance"), ALICE, Some(""))
        .unwrap()
        .returns;
    assert_eq!(0, res.len());
}
