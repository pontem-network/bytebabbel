use anyhow::Result;

// # local
use eth::Flags;
use move_executor::{self as me, solidity::FromSolidity, MoveExecutor};

const PROFILE_NAME: &str = "local";
const PATH_CONTRACT: &str = "./translator/test_infra/sol/demo/address_support.sol";

fn main() -> Result<()> {
    test_infra::init_log();

    println!("Profile name: {PROFILE_NAME}");
    let signer_address = me::profile::load_profile(PROFILE_NAME)?.account.unwrap();
    let signer_address_hex = signer_address.to_hex_literal();
    println!("Address: {signer_address_hex}");

    println!("Convert {PATH_CONTRACT}");
    let mut vm = MoveExecutor::from_sol(PATH_CONTRACT, signer_address, "", Flags::default())?;

    println!("VM RUN: {signer_address_hex}::AddressSupport::constructor({signer_address_hex})");
    vm.run(
        &format!("{signer_address_hex}::AddressSupport::constructor"),
        &signer_address_hex,
        None,
    )
    .unwrap();

    println!("VM RUN: {signer_address_hex}::AddressSupport::is_owner({signer_address_hex})");
    let res = vm
        .run(
            &format!("{signer_address_hex}::AddressSupport::is_owner"),
            &signer_address_hex,
            Some(""),
        )?
        .to_result_str();
    assert_eq!("Bool(true)", res);

    println!("VM RUN: {signer_address_hex}::AddressSupport::is_owner(0x44)");
    let res = vm
        .run(
            &format!("{signer_address_hex}::AddressSupport::is_owner"),
            "0x44",
            Some(""),
        )?
        .to_result_str();
    assert_eq!("Bool(false)", res);

    Ok(())
}
