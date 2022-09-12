use anyhow::{Error, Result};
use clap::Parser;
use move_core_types::account_address::AccountAddress;
use std::path::PathBuf;
use std::str::FromStr;

pub mod convert;
#[cfg(feature = "deploy")]
pub mod deploy;
pub mod profile;

use crate::profile::ProfileConfig;

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

    #[clap(long = "args", short = 'a', default_value = "")]
    init_args: String,

    /// deploying the module in aptos node
    #[clap(long = "deploy", short = 'd', value_parser)]
    #[cfg(feature = "deploy")]
    deploy: bool,
}

impl Args {
    pub fn execute(&self) -> Result<String> {
        let result = self.convert()?;

        #[cfg(feature = "deploy")]
        if self.deploy {
            return self.publish(&result);
        }

        Ok(format!(
            "{}\n{}",
            result.mv_path.to_string_lossy(),
            result.move_path.to_string_lossy()
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ProfileValue {
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

    #[cfg(feature = "deploy")]
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
