use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct ConvertFlags {
    /// Input params of native type
    #[clap(long)]
    pub native_input: bool,

    /// Output value of native type
    #[clap(long)]
    pub native_output: bool,

    /// Hide all output during execution
    #[clap(long)]
    pub hide_output: bool,

    /// Use u128 instead of u256
    #[clap(long)]
    pub u128_io: bool,

    /// Generate an interface project
    #[clap(long, short = 'i')]
    pub interface_package: bool,

    /// Save solidity abi
    #[clap(long)]
    pub save_abi: bool,
}

impl ConvertFlags {
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
