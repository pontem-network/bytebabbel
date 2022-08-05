use anyhow::{anyhow, Error, Result};
use aptos::common::types::ProfileConfig;
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use std::path::PathBuf;
use std::str::FromStr;

pub mod convert;
pub mod deploy;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Path to the file. Specify the path to sol file or abi|bin.
    #[clap(value_parser)]
    path: PathBuf,

    /// Where to save the converted move binary code
    #[clap(short, long = "output", display_order = 3, value_parser)]
    output_path: Option<PathBuf>,

    /// The name of the Move module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 4, value_parser)]
    move_module_name: Option<String>,

    /// Profile name or address. The address must start with "0x". Needed for the module address
    #[clap(
        long = "profile",
        display_order = 5,
        short = 'p',
        default_value = "default",
        value_parser
    )]
    profile_or_address: ProfileValue,

    /// Math backend.
    #[clap(long = "math", short = 'm', default_value = "u128", value_parser)]
    math_backend: String,

    /// Math backend.
    #[clap(long = "deploy", short = 'd', value_parser)]
    deploy: bool,
}

impl Args {
    pub fn execute(&self) -> Result<String> {
        let output_path = self.convert()?;

        if self.deploy {
            return self.publish(&output_path);
        }

        Ok(output_path.to_string_lossy().to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ProfileValue {
    Address(AccountAddress),
    Profile(String),
}

impl ProfileValue {
    pub fn to_address(&self) -> Result<AccountAddress> {
        let address = match self {
            ProfileValue::Address(address) => address.clone(),
            ProfileValue::Profile(profile_name) => {
                let aptos_account = ProfileValue::profile(profile_name)?.account
                .ok_or_else(|| {
                    anyhow!(
                        "The address is not specified in the profile {profile_name:?}. Try to recreate the profile"
                    )
                })?;
                AccountAddress::from_bytes(&aptos_account.into_bytes())?
            }
        };
        Ok(address)
    }

    pub fn is_address(&self) -> bool {
        matches!(self, ProfileValue::Address(..))
    }

    fn profile(profile_name: &str) -> Result<ProfileConfig> {
        aptos::common::types::CliConfig::load_profile(profile_name)?.ok_or_else(|| {
            anyhow!(
                "Profile {:?} not found. To create a profile, use: $ aptos init --profile <NAME>",
                &profile_name
            )
        })
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
            ProfileValue::profile(value)?;

            Ok(ProfileValue::Profile(value.to_string()))
        }
    }
}

fn main() {
    env_logger::init();

    match Args::parse().execute() {
        Ok(result) => {
            println!("{result}");
        }
        Err(err) => {
            println!("Error: {err:?}");
        }
    }
}
