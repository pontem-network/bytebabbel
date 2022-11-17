use std::str::FromStr;

use anyhow::{anyhow, Error, Result};

use aptos::common::types::{CliConfig, ConfigSearchMode};
use move_core_types::account_address::AccountAddress;

/// Converting a profile name to an address
pub(crate) fn profile_to_address(name_or_address: &str) -> Result<AccountAddress> {
    if name_or_address.starts_with("0x") {
        return Ok(AccountAddress::from_hex_literal(name_or_address)?);
    }

    let profile = CliConfig::load_profile(Some(name_or_address), ConfigSearchMode::CurrentDir)?
        .ok_or_else(|| anyhow!("Profile {name_or_address} not found"))?;
    profile
        .account
        .ok_or_else(|| anyhow!("The address is not specified in the profile {name_or_address}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileValue {
    Address(AccountAddress),
    Profile {
        name: String,
        address: AccountAddress,
    },
}

impl ProfileValue {
    pub fn to_address(&self) -> Result<AccountAddress> {
        let address = match self {
            ProfileValue::Address(address) => *address,
            ProfileValue::Profile { address, .. } => *address,
        };
        Ok(address)
    }

    pub fn name_profile(&self) -> Result<&String> {
        match self {
            ProfileValue::Address(..) => {
                anyhow::bail!("The address was transmitted. The profile name was expected.")
            }
            ProfileValue::Profile { name, .. } => Ok(name),
        }
    }

    pub fn default_profile() -> Result<ProfileValue> {
        log::debug!("ProfileValue::default");
        ProfileValue::from_str("default")
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
            Ok(ProfileValue::Profile {
                name: value.to_string(),
                address: profile_to_address(value)?,
            })
        }
    }
}
