use std::fs;
use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use move_core_types::account_address::AccountAddress;
use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileValue {
    Address(AccountAddress),
    Profile(ProfileConfig),
}

impl ProfileValue {
    pub fn to_address(&self) -> Result<AccountAddress> {
        let address = match self {
            ProfileValue::Address(address) => *address,
            ProfileValue::Profile(profile_name) => profile_name.address,
        };
        Ok(address)
    }

    pub fn name_profile(&self) -> Result<&String> {
        match self {
            ProfileValue::Address(..) => {
                anyhow::bail!("The address was transmitted. The profile name was expected.")
            }
            ProfileValue::Profile(profile_name) => Ok(&profile_name.name),
        }
    }
}

impl FromStr for ProfileValue {
    type Err = Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        if value.starts_with("0x") {
            Ok(ProfileValue::Address(AccountAddress::from_hex_literal(
                value,
            )?))
        } else {
            Ok(ProfileValue::Profile(ProfileConfig::load(value)?))
        }
    }
}

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
