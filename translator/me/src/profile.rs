use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use aptos_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use move_core_types::account_address::AccountAddress;

const CONFIG_FILE: &str = ".aptos/config.yaml";

pub fn load_configs() -> Result<CliConfig> {
    let root_dir = PathBuf::from(".").canonicalize()?;
    let mut current_dir = root_dir.as_path();
    let path_config = loop {
        let path_config = current_dir.join(CONFIG_FILE);
        if path_config.exists() {
            break path_config;
        }
        match current_dir.parent() {
            None => bail!(
                "Profile configurations not found. Create profiles using the `aptos init` command"
            ),
            Some(parent_dir) => current_dir = parent_dir,
        }
    };

    let profile_configurations: CliConfig =
        serde_yaml::from_str(&fs::read_to_string(path_config)?)?;

    Ok(profile_configurations)
}

pub fn load_profile(profile_name: &str) -> Result<ProfileConfig> {
    load_configs()?
        .profiles
        .and_then(|mut profiles| profiles.remove(profile_name))
        .ok_or_else(|| anyhow!("Profile {profile_name} not found"))
}

pub fn profile_to_address(profile: &ProfileConfig) -> Result<AccountAddress> {
    profile
        .account
        .ok_or_else(|| anyhow!("The address in the profile is not specified"))
}

/// Config saved to `.aptos/config.yaml`
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CliConfig {
    /// Map of profile configs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<BTreeMap<String, ProfileConfig>>,
}

/// An individual profile
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    /// Private key for commands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<Ed25519PrivateKey>,
    /// Public key for commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<Ed25519PublicKey>,
    /// Account for commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountAddress>,
    /// URL for the Aptos rest endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_url: Option<String>,
    /// URL for the Faucet endpoint (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faucet_url: Option<String>,
}

/// A simplified list of all networks supported by the CLI
///
/// Any command using this, will be simpler to setup as profiles
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Network {
    Mainnet,
    Testnet,
    Devnet,
    Local,
    Custom,
}
