use crate::testssol::convert::ResultToString;
use crate::testssol::env::executor::MoveExecutor;
use crate::testssol::env::sol::build_sol;
use crate::testssol::make_move_module;
use eth::abi::entries::AbiEntries;
use eth::Flags;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};

#[allow(dead_code)]
mod testssol;

#[test]
pub fn test_for_users() {
    fn test(flags: Flags) {
        let evm = build_sol(include_bytes!("../sol/demo/users.sol")).unwrap();
        let bytecode = make_move_module(
            &format!(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::{}",
                evm.name()
            ),
            evm.bin(),
            "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0",
            evm.abi(),
            flags,
        )
        .unwrap();
        let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap(), flags);
        vm.deploy(
            "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
            bytecode,
        );

        vm.run(
            "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::constructor",
            "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
            None,
        )
            .unwrap();
        let res = vm
            .run("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::create_user", "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00", Some(""))
            .unwrap();
        let new_user_event = &res.events[0];

        let mut guid = vec![4, 0, 0, 0, 0, 0, 0, 0];
        guid.extend(
            AccountAddress::from_hex_literal(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
            )
            .unwrap()
            .as_slice(),
        );
        assert_eq!(guid, new_user_event.0);
        assert_eq!(0, new_user_event.1);
        assert_eq!(
            TypeTag::Struct(StructTag {
                address: AccountAddress::from_hex_literal(
                    "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00"
                )
                .unwrap(),
                module: Identifier::new("Users").unwrap(),
                name: Identifier::new("Event").unwrap(),
                type_params: vec![]
            }),
            new_user_event.2
        );

        let res = vm
            .run(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::get_id",
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
                Some(""),
            )
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!(res, "(2)");

        let res = vm
            .run(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::get_id",
                "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0",
                Some(""),
            )
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!(res, "(1)");

        let res = vm
            .run(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::is_owner",
                "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0",
                Some(""),
            )
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!("(true)", res);

        let res = vm
            .run(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::is_owner",
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
                Some(""),
            )
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!("(false)", res);

        let res = vm
            .run("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::get_balance", "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0", Some(""))
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!("(10000000000000000000000000000)", res);

        let res = vm
            .run("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::get_balance", "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00", Some(""))
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!("(0)", res);

        let res = vm
            .run(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::transfer",
                "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0",
                Some("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00, 1000"),
            )
            .unwrap();
        let new_user_event = &res.events[0];
        let mut guid = vec![4, 0, 0, 0, 0, 0, 0, 0];
        guid.extend(
            AccountAddress::from_hex_literal(
                "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
            )
            .unwrap()
            .as_slice(),
        );
        assert_eq!(guid, new_user_event.0);
        assert_eq!(1, new_user_event.1);
        assert_eq!(
            TypeTag::Struct(StructTag {
                address: AccountAddress::from_hex_literal(
                    "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00"
                )
                .unwrap(),
                module: Identifier::new("Users").unwrap(),
                name: Identifier::new("Event").unwrap(),
                type_params: vec![]
            }),
            new_user_event.2
        );

        let res = vm
            .run("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::get_balance", "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0", Some(""))
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!("(9999999999999999999999999000)", res);

        let res = vm
            .run("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::get_balance", "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00", Some(""))
            .unwrap()
            .returns
            .to_result_str();
        assert_eq!("(1000)", res);

        let res = vm.run(
            "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::transfer",
            "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
            Some("0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0, 1001"),
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
        package_interface: false,
    });
    test_for_users_with_hidden_result();
}

pub fn test_for_users_with_hidden_result() {
    let flags = Flags {
        native_input: false,
        native_output: false,
        hidden_output: true,
        u128_io: false,
        package_interface: false,
    };
    let evm = build_sol(include_bytes!("../sol/demo/users.sol")).unwrap();
    let bytecode = make_move_module(
        &format!(
            "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::{}",
            evm.name()
        ),
        evm.bin(),
        "0x61508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699ed0",
        evm.abi(),
        flags,
    )
    .unwrap();
    let mut vm = MoveExecutor::new(AbiEntries::try_from(evm.abi()).unwrap(), flags);
    vm.deploy(
        "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
        bytecode,
    );

    vm.run(
        "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::constructor",
        "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00",
        None,
    )
    .unwrap();

    let res = vm
        .run("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::create_user", "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00", Some(""))
        .unwrap();
    assert_eq!(0, res.returns.len());

    let res = vm
        .run("0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00::Users::get_balance", "0x00508c3c7d491d5911f81d90f80f064eda2a44e25db349bfc0e6d3f023699e00", Some(""))
        .unwrap()
        .returns;
    assert_eq!(0, res.len());
}
