use std::fs;

use anyhow::Result;
use ethabi::Contract;

use eth::Flags;
use move_executor::{self as me, load::LoadRemoteData, MoveExecutorInstance};

const PROFILE_DEFAULT: &str = "local";

fn main() -> Result<()> {
    test_infra::init_log();

    let profile_default = me::profile::load_profile(PROFILE_DEFAULT)?;
    let signer_address = profile_default.account.unwrap();
    let signer_address_hex = signer_address.to_hex_literal();

    let abi: Contract =
        serde_json::from_str(&fs::read_to_string("./AddressSupport/AddressSupport.abi")?)?;

    let flags = Flags::default();
    let mut vm = me::MoveExecutor::new(abi, flags, MoveExecutorInstance::Aptos);
    vm.load_modules(&profile_default, &signer_address).unwrap();

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
