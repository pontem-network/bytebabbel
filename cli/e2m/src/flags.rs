use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct DeployFlags {
    /// Deploying the module in aptos node
    #[clap(long = "deploy", short = 'd', value_parser)]
    #[cfg(feature = "deploy")]
    pub deploy: bool,
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
    #[clap(long = "interface_package", short = 'i', default_value = "false")]
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
