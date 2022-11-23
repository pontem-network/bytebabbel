/// !!! IMPORTANT
/// Don't forget to raise the local node before starting
///
/// $ ```aptos node run-local-testnet --with-faucet --force-restart --assume-yes```
///
use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use crate::{
    aptos_add_coins, aptos_init_local_profile, aptos_profile_name_and_account_address,
    aptos_publish_package, checking_the_file_structure, e2m, run_cli, StrLastLine,
};
use test_infra::color::font_blue;

/// Module will support "move" types
#[test]
#[ignore]
fn test_native_types() {
    // After the test is completed, it will be deleted
    let tmp_project_folder = tempdir().unwrap();

    println!("Creating a `default` profile ");
    aptos_init_local_profile(&tmp_project_folder, "default").unwrap();
    aptos_add_coins(&tmp_project_folder, "default").unwrap();

    println!("Creating a `demo` profile ");
    aptos_init_local_profile(&tmp_project_folder, "demo").unwrap();
    aptos_add_coins(&tmp_project_folder, "demo").unwrap();

    let profiles = aptos_profile_name_and_account_address(tmp_project_folder.as_ref()).unwrap();
    let default_address_hex = profiles.get("default").unwrap().to_hex_literal();
    let demo_address_hex = profiles.get("demo").unwrap().to_hex_literal();

    // Converting `sol` file with native types and publishing to node
    e2m(
        &[
            "convert",
            "../../examples/users.sol",
            "-o",
            "i_users_native",
            "--module",
            "UsersNative",
            "-a",
            "self",
            "--native-input",
            "--native-output",
            "--max-gas",
            "25000",
            "-d",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    checking_the_file_structure(
        &tmp_project_folder.as_ref().join("i_users_native"),
        "UsersNative",
    );

    // Preparing a script using the interface
    let sc_users_dir = tmp_project_folder.as_ref().join("sc_users_native");
    copy_dir::copy_dir("../../examples/sc_users_native", &sc_users_dir).unwrap();
    let sc_users_move_toml = sc_users_dir.join("Move.toml");
    let mut move_toml_str = fs::read_to_string(&sc_users_move_toml).unwrap();
    move_toml_str = move_toml_str.replace(r#""_""#, &format!(r#""{default_address_hex}""#));
    fs::write(&sc_users_move_toml, move_toml_str).unwrap();

    // Publishing a script to node
    aptos_publish_package(&sc_users_dir).unwrap();

    // ### Calling the module constructor.
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::constructor",
            "--max-gas",
            "5000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // ### Adding a "demo" account
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::create_user",
            "--max-gas",
            "25000",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // ### ID verification
    // --profile default
    println!("an {} is expected", font_blue("error"));
    assert!(e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::is_id",
            "--args",
            "u128:1",
            "--max-gas",
            "15000",
        ],
        &tmp_project_folder,
    )
    .is_err());

    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::is_id",
            "--args",
            "u128:1",
            "--native",
            "--max-gas",
            "15000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    run_cli(
        "aptos",
        &[
            "move",
            "run",
            "--function-id",
            "default::ScUsersNative::is_id",
            "--args",
            "u128:1",
            "--max-gas",
            "15000",
            "--assume-yes",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // --profile demo
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::is_id",
            "--args",
            "u128:2",
            "--native",
            "--max-gas",
            "15000",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    run_cli(
        "aptos",
        &[
            "move",
            "run",
            "--function-id",
            "default::ScUsersNative::is_id",
            "--args",
            "u128:2",
            "--max-gas",
            "15000",
            "--assume-yes",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // ### Checking whether this account is the owner:
    // --profile default
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::is_owner",
            "--max-gas",
            "15000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    run_cli(
        "aptos",
        &[
            "move",
            "run",
            "--function-id",
            "default::ScUsersNative::is_owner",
            "--max-gas",
            "15000",
            "--assume-yes",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // --profile demo
    println!("an {} is expected", font_blue("error"));
    assert!(e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::is_owner",
            "--max-gas",
            "15000",
            "--profile",
            "demo"
        ],
        &tmp_project_folder,
    )
    .is_err());

    println!("an {} is expected", font_blue("error"));
    assert!(run_cli(
        "aptos",
        &[
            "move",
            "run",
            "--function-id",
            "default::ScUsersNative::is_owner",
            "--max-gas",
            "15000",
            "--assume-yes",
            "--profile",
            "demo"
        ],
        &tmp_project_folder,
    )
    .is_err());

    // #### Checking the balance
    // --profile default
    e2m_script_native_check_balance(10000000000000000000000000000, None, &tmp_project_folder)
        .unwrap();
    aptos_script_check_balance(10000000000000000000000000000, None, &tmp_project_folder).unwrap();

    // --profile demo
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::is_empty_balance",
            "--max-gas",
            "15000",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    run_cli(
        "aptos",
        &[
            "move",
            "run",
            "--function-id",
            "default::ScUsersNative::is_empty_balance",
            "--max-gas",
            "15000",
            "--assume-yes",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // #### Transfer
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::transfer",
            "--args",
            "address:demo",
            "u128:200",
            "--native",
            "--max-gas",
            "25000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersNative::transfer",
            "--args",
            &format!("address:{demo_address_hex}"),
            "u128:200",
            "--native",
            "--max-gas",
            "25000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    run_cli(
        "aptos",
        &[
            "move",
            "run",
            "--function-id",
            "default::ScUsersNative::transfer",
            "--args",
            "address:demo",
            "u128:200",
            "--max-gas",
            "25000",
            "--assume-yes",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // #### Checking the transfer
    // --profile default
    e2m_script_native_check_balance(9999999999999999999999999400, None, &tmp_project_folder)
        .unwrap();
    aptos_script_check_balance(9999999999999999999999999400, None, &tmp_project_folder).unwrap();

    // --profile demo
    e2m_script_native_check_balance(600, Some("demo"), &tmp_project_folder).unwrap();
    aptos_script_check_balance(600, Some("demo"), &tmp_project_folder).unwrap();

    // local call: native
    let native_abi = tmp_project_folder
        .as_ref()
        .join("i_users_native/UsersNative.abi")
        .to_string_lossy()
        .to_string();

    // dir with abi
    let output = e2m(
        &[
            "call",
            "--function-id",
            "default::UsersNative::get_id",
            "--path",
            tmp_project_folder
                .as_ref()
                .join("i_users_native")
                .to_string_lossy()
                .as_ref(),
            "--how",
            "local",
            "--native",
        ],
        &tmp_project_folder,
    )
    .unwrap()
    .last_line();
    assert_eq!("Uint(1)", &output);

    // abi
    let output = e2m(
        &[
            "call",
            "--function-id",
            "default::UsersNative::get_id",
            "--path",
            &native_abi,
            "--how",
            "local",
            "--native",
        ],
        &tmp_project_folder,
    )
    .unwrap()
    .last_line();
    assert_eq!("Uint(1)", &output);

    let output = e2m(
        &[
            "call",
            "--function-id",
            "default::UsersNative::get_id",
            "--path",
            &native_abi,
            "--how",
            "local",
            "--native",
            "--init-args",
            "default",
        ],
        &tmp_project_folder,
    )
    .unwrap()
    .last_line();
    assert_eq!("Uint(1)", &output);

    // sol
    let output = e2m(
        &[
            "call",
            "--function-id",
            "default::Users::get_id",
            "--path",
            "../../examples/users.sol",
            "--how",
            "local-source",
            "--native",
            "--init-args",
            "self",
        ],
        &tmp_project_folder,
    )
    .unwrap()
    .last_line();
    assert_eq!("Uint(1)", &output);
}

/// Module will support "move" types
#[test]
#[ignore]
fn test_ethereum_types() {
    // After the test is completed, it will be deleted
    let tmp_project_folder = tempdir().unwrap();

    println!("Creating a `default` profile ");
    aptos_init_local_profile(&tmp_project_folder, "default").unwrap();
    aptos_add_coins(&tmp_project_folder, "default").unwrap();

    println!("Creating a `demo` profile ");
    aptos_init_local_profile(&tmp_project_folder, "demo").unwrap();
    aptos_add_coins(&tmp_project_folder, "demo").unwrap();

    let profiles = aptos_profile_name_and_account_address(tmp_project_folder.as_ref()).unwrap();
    let default_address_hex = profiles.get("default").unwrap().to_hex_literal();
    let demo_address_hex = profiles.get("demo").unwrap().to_hex_literal();

    // Converting `sol` file with ethereum types and publishing to node
    e2m(
        &[
            "convert",
            "../../examples/users.sol",
            "-o",
            "i_users_ethtypes",
            "--module",
            "UsersEth",
            "-a",
            "self",
            "--max-gas",
            "25000",
            "-d",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    checking_the_file_structure(
        &tmp_project_folder.as_ref().join("i_users_ethtypes"),
        "UsersEth",
    );

    // Preparing a script using the interface
    let sc_users_dir = tmp_project_folder.as_ref().join("sc_users_ethtypes");
    copy_dir::copy_dir("../../examples/sc_users_ethtypes", &sc_users_dir).unwrap();
    let sc_users_move_toml = sc_users_dir.join("Move.toml");
    let mut move_toml_str = fs::read_to_string(&sc_users_move_toml).unwrap();
    move_toml_str = move_toml_str.replace(r#""_""#, &format!(r#""{default_address_hex}""#));
    fs::write(&sc_users_move_toml, move_toml_str).unwrap();

    // Publishing a script to node
    aptos_publish_package(&sc_users_dir).unwrap();

    // ### Calling the module constructor.
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::constructor",
            "--max-gas",
            "5000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // ### Adding a "demo" account
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::create_user",
            "--max-gas",
            "25000",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // ### ID verification
    // --profile default
    println!("an {} is expected", font_blue("error"));
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::is_id",
            "--args",
            "u128:1",
            "--max-gas",
            "15000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::is_id",
            "--args",
            "u128:1",
            "--max-gas",
            "15000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // --profile demo
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::is_id",
            "--args",
            "u128:2",
            "--max-gas",
            "15000",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // ### Checking whether this account is the owner:
    // --profile default
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::is_owner",
            "--max-gas",
            "15000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // --profile demo
    println!("an {} is expected", font_blue("error"));
    assert!(e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::is_owner",
            "--max-gas",
            "15000",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .is_err());

    // #### Checking the balance
    // --profile default
    e2m_script_check_balance(10000000000000000000000000000, None, &tmp_project_folder).unwrap();

    // --profile demo
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::is_empty_balance",
            "--max-gas",
            "15000",
            "--profile",
            "demo",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // #### Transfer
    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::transfer",
            "--args",
            "address:demo",
            "u128:200",
            "--max-gas",
            "25000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    e2m(
        &[
            "call",
            "--function-id",
            "default::ScUsersEth::transfer",
            "--args",
            &format!("address:{demo_address_hex}"),
            "u128:200",
            "--max-gas",
            "25000",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    // #### Checking the transfer
    // --profile default
    e2m_script_check_balance(9999999999999999999999999600, None, &tmp_project_folder).unwrap();

    // --profile demo
    e2m_script_check_balance(400, Some("demo"), &tmp_project_folder).unwrap();

    // local call: native
    let abi_path = tmp_project_folder
        .as_ref()
        .join("i_users_ethtypes/UsersEth.abi")
        .to_string_lossy()
        .to_string();

    // dir with abi
    let output = e2m(
        &[
            "call",
            "--function-id",
            "default::UsersEth::get_id",
            "--path",
            tmp_project_folder
                .as_ref()
                .join("i_users_ethtypes")
                .to_string_lossy()
                .as_ref(),
            "--how",
            "local",
        ],
        &tmp_project_folder,
    )
    .unwrap()
    .last_line();
    assert_eq!("Uint(1)", &output);

    // abi
    let output = e2m(
        &[
            "call",
            "--function-id",
            "default::UsersEth::get_id",
            "--path",
            &abi_path,
            "--how",
            "local",
        ],
        &tmp_project_folder,
    )
    .unwrap()
    .last_line();
    assert_eq!("Uint(1)", &output);

    let output = e2m(
        &[
            "call",
            "--function-id",
            "default::UsersEth::get_id",
            "--path",
            &abi_path,
            "--how",
            "local",
            "--init-args",
            "default",
        ],
        &tmp_project_folder,
    )
    .unwrap()
    .last_line();
    assert_eq!("Uint(1)", &output);

    // view resources
    e2m(
        &[
            "resources",
            "--query",
            "events",
            "--resource-path",
            "default::UsersEth::Persist::events",
        ],
        &tmp_project_folder,
    )
    .unwrap();

    let output = e2m(
        &[
            "resources",
            "--query",
            "events",
            "--resource-path",
            "default::UsersEth::Persist::events",
            "--abi",
            &abi_path,
        ],
        &tmp_project_folder,
    )
    .unwrap();
    assert!(output.contains(&format!(
        r#""from": "Address({default_address_hex})",
        "to": "Address({default_address_hex})",
        "amount": "Uint(200)""#
    )));

    let output = e2m(
        &[
            "resources",
            "--query",
            "events",
            "--resource-path",
            "default::UsersEth::Persist::events",
            "--decode-types",
            "data:address,address,u256 topics:bytes",
        ],
        &tmp_project_folder,
    )
    .unwrap();
    assert!(output.contains(&format!(
        r#""data": "[Address({default_address_hex}), Address({default_address_hex}), Uint(200)]""#
    )));

    todo!()
}

fn e2m_script_native_check_balance<P: AsRef<Path>>(
    coins: u128,
    profile: Option<&str>,
    tmp_project_folder: P,
) -> Result<String> {
    let coins_arg = format!("u128:{coins}");
    let mut args = vec![
        "call",
        "--function-id",
        "default::ScUsersNative::check_balance",
        "--args",
        &coins_arg,
        "--native",
        "--max-gas",
        "15000",
    ];

    if let Some(profile_name) = profile {
        args.extend(["--profile", profile_name]);
    }
    e2m(&args, tmp_project_folder)
}

fn e2m_script_check_balance<P: AsRef<Path>>(
    coins: u128,
    profile: Option<&str>,
    tmp_project_folder: P,
) -> Result<String> {
    let coins_arg = format!("u128:{coins}");
    let mut args = vec![
        "call",
        "--function-id",
        "default::ScUsersEth::check_balance",
        "--args",
        &coins_arg,
        "--max-gas",
        "15000",
    ];

    if let Some(profile_name) = profile {
        args.extend(["--profile", profile_name]);
    }
    e2m(&args, tmp_project_folder)
}

fn aptos_script_check_balance<P: AsRef<Path>>(
    coins: u128,
    profile: Option<&str>,
    tmp_project_folder: P,
) -> Result<String> {
    let coins_arg = format!("u128:{coins}");
    let mut args = vec![
        "move",
        "run",
        "--function-id",
        "default::ScUsersNative::check_balance",
        "--args",
        &coins_arg,
        "--max-gas",
        "15000",
        "--assume-yes",
    ];
    if let Some(profile_name) = profile {
        args.extend(["--profile", profile_name]);
    }
    run_cli("aptos", &args, &tmp_project_folder)
}
