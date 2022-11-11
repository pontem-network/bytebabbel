use std::fs;

use anyhow::{anyhow, Result};
use framework::natives::code::{
    ModuleMetadata, MoveOption, PackageDep, PackageMetadata, UpgradePolicy,
};
use framework::zip_metadata_str;
use move_binary_format::access::ModuleAccess;

use crate::convert::ResultConvert;
use crate::profile::ProfileValue;
use crate::{wait, CmdConvert};

const TEMPLATE_MOVE_TOML: &str = include_str!("../../resources/template_move.toml");

impl CmdConvert {
    /// Publish in aptos node
    /// Access keys are taken from profiles (back.aptos/config.yaml).
    pub fn publish(&self, result_convert: &ResultConvert) -> Result<String> {
        use clap::Parser;

        let profile = self
            .profile_or_address
            .clone()
            .unwrap_or(ProfileValue::default().map_err(|_| {
                anyhow!(
                    "For deploy, you need to specify the profile name. \n\n\
                    Example: \n\
                    $ e2m <path/to/file.sol> --profile <NameProfile>\n\n\
                    Create profile default: \n\
                    $ aptos init\n\n\
                    Create profile with name:\n\
                    $ aptos init --profile <NameProfile>"
                )
            })?);
        let profile_name = profile.name_profile()?;

        let txn_options: aptos::common::types::TransactionOptions =
            aptos::common::types::TransactionOptions::try_parse_from([
                "subcommand",
                "--profile",
                profile_name,
                "--max-gas",
                &self.transaction_flags.max_gas.to_string(),
                "--assume-yes",
            ])
            .map_err(|err| anyhow!("Invalid profile parameter. {err}"))?;

        let binarycode = fs::read(&result_convert.binary_code_path)?;

        // Send the compiled module and metadata using the code::publish_package_txn.
        let metadata = gen_meta(result_convert, &binarycode)?;
        let compiled_units = vec![binarycode];

        let payload = cached_packages::aptos_stdlib::code_publish_package_txn(
            bcs::to_bytes(&metadata).expect("PackageMetadata has BCS"),
            compiled_units,
        );

        let size = bcs::serialized_size(&payload)?;
        println!("package size {} bytes", size);

        let fut = txn_options.submit_transaction(payload);
        let result = wait(fut).map(aptos::common::types::TransactionSummary::from)?;

        Ok(serde_json::to_string_pretty(&serde_json::to_value(
            result,
        )?)?)
    }
}

fn gen_meta(result_convert: &ResultConvert, binarycode: &[u8]) -> Result<PackageMetadata> {
    let module_name = module_name(binarycode)?;

    let move_toml_string = TEMPLATE_MOVE_TOML
        .replace("###NAME###", &module_name)
        .replace("###ADDRESS###", &result_convert.address.to_string());
    let manifest = zip_metadata_str(&move_toml_string)?;

    let modules = vec![ModuleMetadata {
        name: module_name.clone(),
        source: Vec::new(),
        source_map: Vec::new(),
        extension: MoveOption::default(),
    }];

    let deps = ["AptosFramework", "AptosStdlib", "MoveStdlib"]
        .into_iter()
        .map(|name| {
            Ok(PackageDep {
                account: move_core_types::account_address::AccountAddress::from_hex_literal("0x1")?,
                package_name: name.to_string(),
            })
        })
        .collect::<Result<Vec<PackageDep>>>()?;
    let source_digest = source_digest(binarycode)?.to_uppercase();

    Ok(PackageMetadata {
        name: module_name,
        upgrade_policy: UpgradePolicy::compat(),
        upgrade_number: 0,
        source_digest,
        manifest,
        modules,
        deps,
        extension: MoveOption::default(),
    })
}

fn module_name(binarycode: &[u8]) -> Result<String> {
    use move_binary_format::CompiledModule;

    let compiled_module = CompiledModule::deserialize(binarycode)?;

    Ok(compiled_module
        .identifier_at(compiled_module.self_handle().name)
        .to_string())
}

fn source_digest(binarycode: &[u8]) -> Result<String> {
    use sha2::{Digest, Sha256};

    // create a Sha256 object
    let mut hasher = Sha256::new();
    hasher.update(binarycode);
    let result = hasher.finalize().to_vec();
    Ok(hex::encode(result))
}
