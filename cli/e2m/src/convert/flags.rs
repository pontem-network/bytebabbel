use anyhow::{anyhow, Result};
use clap::Args;
use translator::Flags;

#[derive(Args, Debug, Copy, Clone)]
pub struct ConvertFlags {
    /// Input params of native type
    #[clap(long)]
    pub native_input: bool,

    /// Output value of native type
    #[clap(long)]
    pub native_output: bool,

    /// Hide all output during execution
    #[clap(long)]
    pub hidden_output: bool,

    /// Use u128 instead of u256
    #[clap(long)]
    pub u128_io: bool,
}

impl ConvertFlags {
    pub fn check(&self) -> Result<()> {
        if self.native_output && self.hidden_output {
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

impl From<ConvertFlags> for Flags {
    fn from(fl: ConvertFlags) -> Self {
        Flags {
            native_input: fl.native_input,
            native_output: fl.native_output,
            hidden_output: fl.hidden_output,
            u128_io: fl.u128_io,
        }
    }
}
