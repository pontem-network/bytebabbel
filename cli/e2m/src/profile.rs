use std::fs;

use anyhow::{anyhow, Result};
use move_core_types::account_address::AccountAddress;
use serde_yaml::Value;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProfileConfig {
    pub name: String,
    pub address: AccountAddress,
}

impl ProfileConfig {
    pub fn load(profile_name: &str) -> Result<ProfileConfig> {
        fs::read_to_string("./.aptos/config.yaml")
            .map_err(|_| anyhow!("No profiles found. To create a profile, use \"$ aptos init\""))
            .and_then(|yaml_string| {
                serde_yaml::from_str(&yaml_string)
                    .map_err(|err| anyhow!("Invalid profile format. Error: {err:?}"))
            })
            .and_then(|yaml: Value| {
                let account: String = yaml
                    .get("profiles")
                    .and_then(|profiles| profiles.get(profile_name))
                    .and_then(|profile| profile.get("account"))
                    .and_then(|address| address.as_str().map(String::from))
                    .ok_or_else(|| anyhow!("Account not found"))?;
                let address = AccountAddress::from_hex_literal(&format!("0x{account}"))?;

                Ok(ProfileConfig {
                    name: profile_name.to_string(),
                    address,
                })
            })
    }
}
