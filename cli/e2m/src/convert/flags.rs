use anyhow::{anyhow, Result};
use clap::Args;

#[cfg(feature = "deploy")]
#[derive(Args, Debug)]
pub struct DeployFlags {
    /// Publishes the modules in a Move package to the Aptos blockchain
    #[clap(long = "deploy", short = 'd', value_parser)]
    pub deploy: bool,

    /// Maximum amount of gas units to be used to send this transaction
    ///
    /// The maximum amount of gas units willing to pay for the transaction. This is the (max gas in coins / gas unit price).
    ///
    /// For example if I wanted to pay a maximum of 100 coins, I may have the max gas set to 100 if the gas unit price is 1.  
    /// If I want it to have a gas unit price of 2, the max gas would need to be 50 to still only have a maximum price of 100 coins.
    ///
    /// Without a value, it will determine the price based on simulating the current transaction
    #[clap(long = "max-gas", default_value = "5000", value_parser)]
    pub max_gas: u32,
}

#[derive(Args, Debug)]
pub struct TranslationFlags {
    /// Input params of native type
    #[clap(long = "native_input")]
    pub native_input: bool,

    /// Output value of native type
    #[clap(long = "native_output")]
    pub native_output: bool,

    /// Hide all output during execution
    #[clap(long = "hide_output")]
    pub hide_output: bool,

    /// Use u128 instead of u256
    #[clap(long = "u128_io")]
    pub u128_io: bool,

    /// Generate an interface project
    #[clap(long = "interface_package", short = 'i')]
    pub interface_package: bool,
}

impl TranslationFlags {
    pub fn check(&self) -> Result<()> {
        if self.native_output && self.hide_output {
            Err(anyhow!("Wrong set of flags: native_output & hide_output"))
        } else if self.u128_io && !(self.native_input || self.native_output) {
            Err(anyhow!(
                "Wrong set of flags: u128_io must use with native_input or native_output"
            ))
        } else {
            Ok(())
        }
    }
}
